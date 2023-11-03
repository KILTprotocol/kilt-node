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

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

mod deposit;

pub use deposit::StorageDepositCollectorViaDepositsPallet;
pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use crate::deposit::{reserve_deposit, DepositEntry};

	use super::*;

	use frame_support::{
		pallet_prelude::*,
		traits::{
			fungible::{Inspect, MutateHold},
			ConstU32, EnsureOrigin,
		},
	};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::FullCodec;
	use sp_std::fmt::Debug;

	use deposit::free_deposit;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);
	pub const MAX_NAMESPACE_LENGTH: u32 = 16;

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type BalanceOf<T> = <<T as Config>::Currency as Inspect<AccountIdOf<T>>>::Balance;
	pub type DepositKeyOf<T> = <T as frame_system::Config>::Hash;
	pub type DepositEntryOf<T> = DepositEntry<AccountIdOf<T>, BalanceOf<T>, <T as Config>::RuntimeHoldReason>;
	pub type Namespace = BoundedVec<u8, ConstU32<MAX_NAMESPACE_LENGTH>>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type CheckOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
		type Currency: MutateHold<Self::AccountId, Reason = Self::RuntimeHoldReason>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type RuntimeHoldReason: Clone + PartialEq + Debug + FullCodec + MaxEncodedLen + TypeInfo;
	}

	#[pallet::composite_enum]
	pub enum HoldReason {
		Deposit,
	}

	#[pallet::error]
	pub enum Error<T> {
		DepositNotFound,
		DepositExisting,
		Unauthorized,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		DepositAdded(DepositEntryOf<T>),
		DepositReclaimed(DepositEntryOf<T>),
	}

	// Double map (namespace, key) -> deposit
	#[pallet::storage]
	#[pallet::getter(fn deposits)]
	pub(crate) type Deposits<T> =
		StorageDoubleMap<_, Twox64Concat, Namespace, Twox64Concat, DepositKeyOf<T>, DepositEntryOf<T>>;

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		// TODO: Update weight
		#[pallet::weight(0)]
		pub fn reclaim_deposit(origin: OriginFor<T>, namespace: Namespace, key: DepositKeyOf<T>) -> DispatchResult {
			let dispatcher = T::CheckOrigin::ensure_origin(origin)?;

			Deposits::<T>::try_mutate(namespace, key, |deposit_entry| match deposit_entry {
				None => Err(DispatchError::from(Error::<T>::DepositNotFound)),
				Some(ref existing_deposit_entry) => {
					ensure!(
						existing_deposit_entry.deposit.owner == dispatcher,
						DispatchError::from(Error::<T>::Unauthorized)
					);

					free_deposit::<AccountIdOf<T>, T::Currency>(
						&existing_deposit_entry.deposit,
						&existing_deposit_entry.reason,
					)?;
					Self::deposit_event(Event::<T>::DepositReclaimed(existing_deposit_entry.clone()));
					*deposit_entry = None;
					Ok(())
				}
			})?;
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn add_deposit(namespace: Namespace, key: DepositKeyOf<T>, entry: DepositEntryOf<T>) -> DispatchResult {
			Deposits::<T>::try_mutate(namespace, key, |deposit_entry| match deposit_entry {
				Some(_) => Err(DispatchError::from(Error::<T>::DepositExisting)),
				None => {
					reserve_deposit::<AccountIdOf<T>, T::Currency>(
						entry.deposit.owner.clone(),
						entry.deposit.amount,
						&entry.reason,
					)?;
					Self::deposit_event(Event::<T>::DepositAdded(entry.clone()));
					*deposit_entry = Some(entry);
					Ok(())
				}
			})?;
			Ok(())
		}
	}
}
