// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

// The KILT Blockchain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The KILT Blockchain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// If you feel like getting in touch with us, you can do so at info@botlabs.org

//! Pallet to store namespaced deposits for the configured `Currency`. It allows
//! the original payer of a deposit to claim it back, triggering a hook to
//! optionally perform related actions somewhere else in the runtime.
//! Each deposit is identified by a namespace and a key. There cannot be two
//! equal keys under the same namespace, but the same key can be present under
//! different namespaces.

#![cfg_attr(not(feature = "std"), no_std)]

mod default_weights;
mod deposit;
pub mod traits;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use crate::{
	default_weights::WeightInfo, deposit::FixedDepositCollectorViaDepositsPallet, pallet::*,
	traits::NoopDepositStorageHooks,
};

#[frame_support::pallet]
pub mod pallet {
	use crate::{
		default_weights::WeightInfo,
		deposit::{free_deposit, reserve_deposit, DepositEntry},
		traits::DepositStorageHooks,
	};

	use super::*;

	use frame_support::{
		pallet_prelude::*,
		traits::{
			fungible::{hold::Mutate, Inspect},
			EnsureOrigin,
		},
	};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::FullCodec;
	use scale_info::TypeInfo;
	use sp_runtime::DispatchError;
	use sp_std::fmt::Debug;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type BalanceOf<T> = <<T as Config>::Currency as Inspect<AccountIdOf<T>>>::Balance;
	pub type DepositKeyOf<T> = BoundedVec<u8, <T as Config>::MaxKeyLength>;
	pub type DepositEntryOf<T> = DepositEntry<AccountIdOf<T>, BalanceOf<T>, <T as Config>::RuntimeHoldReason>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The maximum length of keys.
		#[pallet::constant]
		type MaxKeyLength: Get<u32>;

		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHooks: crate::traits::BenchmarkHooks<Self>;
		/// The origin check, returning an `AccountId` upon completion, for who
		/// can reclaim a deposit.
		type CheckOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
		/// The currency from which deposits are to be taken.
		type Currency: Mutate<Self::AccountId, Reason = Self::RuntimeHoldReason>;
		/// Additional logic to execute whenever a new deposit a created or a
		/// deposit is released.
		type DepositHooks: DepositStorageHooks<Self>;
		/// The type of a deposit namespace.
		type Namespace: Parameter + MaxEncodedLen;
		/// The aggregated `Event` type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The aggregated `HoldReason` type.
		type RuntimeHoldReason: From<HoldReason> + Clone + PartialEq + Debug + FullCodec + MaxEncodedLen + TypeInfo;
		type WeightInfo: WeightInfo;
	}

	/// The hold reasons for deposits taken by the pallet.
	#[pallet::composite_enum]
	pub enum HoldReason {
		Deposit,
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The deposit with the provided key was not found within the specified
		/// namespace.
		DepositNotFound,
		/// A deposit with the provided key already exists within the specified
		/// namespace.
		DepositExisting,
		/// The origin was not authorized to perform the operation on the
		/// specified deposit entry.
		Unauthorized,
		/// The external hook failed.
		Hook(u16),
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new deposit has been reserved and stored.
		DepositAdded {
			/// The deposit namespace.
			namespace: T::Namespace,
			/// The deposit key.
			key: DepositKeyOf<T>,
			/// The deposit details.
			deposit_entry: DepositEntryOf<T>,
		},
		/// A deposit has been released and deleted from storage.
		DepositReclaimed {
			/// The deposit namespace.
			namespace: T::Namespace,
			/// The deposit key.
			key: DepositKeyOf<T>,
			/// The deposit details.
			deposit_entry: DepositEntryOf<T>,
		},
	}

	/// Storage of all deposits. Its first key is a namespace, and the second
	/// one the deposit key. Its value includes the details associated to a
	/// deposit instance.
	#[pallet::storage]
	#[pallet::getter(fn deposits)]
	pub(crate) type Deposits<T> =
		StorageDoubleMap<_, Twox64Concat, <T as Config>::Namespace, Twox64Concat, DepositKeyOf<T>, DepositEntryOf<T>>;

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Reclaim a deposit that was previously taken. If there is no deposit
		/// with the given key under the given namespace, it returns an error.
		/// If a deposit exists, the deposit hooks are invoked after the deposit
		/// has been removed from the pallet storage.
		#[pallet::call_index(0)]
		#[pallet::weight({
			<T as Config>::WeightInfo::reclaim_deposit()
		})]
		pub fn reclaim_deposit(origin: OriginFor<T>, namespace: T::Namespace, key: DepositKeyOf<T>) -> DispatchResult {
			let dispatcher = T::CheckOrigin::ensure_origin(origin)?;

			let deposit = Self::remove_deposit(&namespace, &key, Some(&dispatcher))?;
			T::DepositHooks::on_deposit_reclaimed(&namespace, &key, deposit).map_err(|e| Error::<T>::Hook(e.into()))?;
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Add a deposit identified by the given key under the given namespace.
		/// If there is already a deposit entry for the same key under the same
		/// namespace, it returns an error. It also returns an error if the
		/// deposit cannot be reserved on the pallet's `Currency`.
		pub fn add_deposit(namespace: T::Namespace, key: DepositKeyOf<T>, entry: DepositEntryOf<T>) -> DispatchResult {
			Deposits::<T>::try_mutate(&namespace, &key, |deposit_entry| match deposit_entry {
				Some(_) => Err(DispatchError::from(Error::<T>::DepositExisting)),
				None => {
					reserve_deposit::<AccountIdOf<T>, T::Currency>(
						entry.deposit.owner.clone(),
						entry.deposit.amount,
						&entry.reason,
					)?;
					Self::deposit_event(Event::<T>::DepositAdded {
						namespace: namespace.clone(),
						key: key.clone(),
						deposit_entry: entry.clone(),
					});
					*deposit_entry = Some(entry);
					Ok(())
				}
			})?;
			Ok(())
		}

		/// Remove and release a deposit identified by the given key under the
		/// given namespace. If there is no deposit with under the provided
		/// namespace with the provided key, it returns an error. It also
		/// returns an error if the deposit cannot be released on the pallet's
		/// `Currency`.
		pub fn remove_deposit(
			namespace: &T::Namespace,
			key: &DepositKeyOf<T>,
			expected_owner: Option<&AccountIdOf<T>>,
		) -> Result<DepositEntryOf<T>, DispatchError> {
			let existing_entry = Deposits::<T>::take(namespace, key).ok_or(Error::<T>::DepositNotFound)?;
			if let Some(expected_owner) = expected_owner {
				ensure!(
					existing_entry.deposit.owner == *expected_owner,
					Error::<T>::Unauthorized
				);
			}
			free_deposit::<AccountIdOf<T>, T::Currency>(&existing_entry.deposit, &existing_entry.reason)?;
			Self::deposit_event(Event::<T>::DepositReclaimed {
				namespace: namespace.clone(),
				key: key.clone(),
				deposit_entry: existing_entry.clone(),
			});
			Ok(existing_entry)
		}
	}
}
