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

//! # CType Pallet
//!
//! A simple pallet which enables users to store their CType hash (blake2b as
//! hex string) on chain and associate it with their account id.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ### Terminology
//!
//! - **CType:**: CTypes are claim types. In everyday language, they are
//!   standardised structures for credentials. For example, a company may need a
//!   standard identification credential to identify workers that includes their
//!   full name, date of birth, access level and id number. Each of these are
//!   referred to as an attribute of a credential.
//!
//! ## Assumptions
//!
//! - The CType hash was created using our KILT JS-SDK.
//! - The underlying CType includes only the following required fields for the
//!   JSON-Schema we use in the SDK: Identifier, KILT specific JSON-Schema,
//!   Title and Properties.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod ctype_entry;
pub mod default_weights;

#[cfg(any(feature = "mock", test))]
pub mod mock;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

/// Test module for CTypes
#[cfg(test)]
mod tests;

pub use crate::{default_weights::WeightInfo, pallet::*};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::traits::Hash,
		traits::{Currency, ExistenceRequirement, OnUnbalanced, StorageVersion, WithdrawReasons},
	};
	use frame_system::pallet_prelude::*;
	use kilt_support::traits::CallSources;
	use sp_runtime::{traits::Saturating, SaturatedConversion};
	use sp_std::vec::Vec;

	use crate::ctype_entry::CtypeEntry;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

	/// Type of a CType hash.
	pub type CtypeHashOf<T> = <T as frame_system::Config>::Hash;

	pub type CtypeEntryOf<T> = CtypeEntry<<T as Config>::CtypeCreatorId, BlockNumberFor<T>>;

	/// Type of a CType creator.
	pub type CtypeCreatorOf<T> = <T as Config>::CtypeCreatorId;

	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::NegativeImbalance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type EnsureOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin, Success = Self::OriginSuccess>;
		type OverarchingOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;
		type OriginSuccess: CallSources<AccountIdOf<Self>, CtypeCreatorOf<Self>>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: Currency<AccountIdOf<Self>>;
		type WeightInfo: WeightInfo;
		type CtypeCreatorId: Parameter + MaxEncodedLen;
		type Fee: Get<BalanceOf<Self>>;
		type FeeCollector: OnUnbalanced<NegativeImbalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// CTypes stored on chain.
	///
	/// It maps from a CType hash to its creator and block number in which it
	/// was created.
	#[pallet::storage]
	#[pallet::getter(fn ctypes)]
	pub type Ctypes<T> = StorageMap<_, Blake2_128Concat, CtypeHashOf<T>, CtypeEntryOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new CType has been created.
		/// \[creator identifier, CType hash\]
		CTypeCreated(CtypeCreatorOf<T>, CtypeHashOf<T>),
		/// Information about a CType has been updated.
		/// \[CType hash\]
		CTypeUpdated(CtypeHashOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// There is no CType with the given hash.
		NotFound,
		/// The CType already exists.
		AlreadyExists,
		/// The paying account was unable to pay the fees for creating a ctype.
		UnableToPayFees,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new CType from the given unique CType hash and associates
		/// it with its creator.
		///
		/// A CType with the same hash must not be stored on chain.
		///
		/// Emits `CTypeCreated`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: Ctypes, Balance
		/// - Writes: Ctypes, Balance
		/// # </weight>
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::add(ctype.len().saturated_into()))]
		pub fn add(origin: OriginFor<T>, ctype: Vec<u8>) -> DispatchResult {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;
			let creator = source.subject();
			let payer = source.sender();

			// Check the free balance before we do any heavy work (e.g. calculate the ctype
			// hash)
			let balance = <T::Currency as Currency<AccountIdOf<T>>>::free_balance(&payer);
			<T::Currency as Currency<AccountIdOf<T>>>::ensure_can_withdraw(
				&payer,
				T::Fee::get(),
				WithdrawReasons::FEE,
				balance.saturating_sub(T::Fee::get()),
			)?;

			let hash = <T as frame_system::Config>::Hashing::hash(&ctype[..]);

			ensure!(!Ctypes::<T>::contains_key(hash), Error::<T>::AlreadyExists);

			// Collect the fees. This should not fail since we checked the free balance in
			// the beginning.
			let imbalance = <T::Currency as Currency<AccountIdOf<T>>>::withdraw(
				&payer,
				T::Fee::get(),
				WithdrawReasons::FEE,
				ExistenceRequirement::AllowDeath,
			)
			.map_err(|_| Error::<T>::UnableToPayFees)?;

			T::FeeCollector::on_unbalanced(imbalance);
			log::debug!("Creating CType with hash {:?} and creator {:?}", hash, creator);
			Ctypes::<T>::insert(
				hash,
				CtypeEntryOf::<T> {
					creator: creator.clone(),
					created_at: frame_system::Pallet::<T>::block_number(),
				},
			);

			Self::deposit_event(Event::CTypeCreated(creator, hash));

			Ok(())
		}

		/// Set the creation block number for a given CType, if found.
		///
		/// Emits `CTypeUpdated`.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_block_number())]
		pub fn set_block_number(
			origin: OriginFor<T>,
			ctype_hash: CtypeHashOf<T>,
			block_number: BlockNumberFor<T>,
		) -> DispatchResult {
			T::OverarchingOrigin::ensure_origin(origin)?;
			Ctypes::<T>::try_mutate(ctype_hash, |ctype_entry| {
				if let Some(ctype_entry) = ctype_entry {
					ctype_entry.created_at = block_number;
					Ok(())
				} else {
					Err(Error::<T>::NotFound)
				}
			})?;

			Self::deposit_event(Event::CTypeUpdated(ctype_hash));

			Ok(())
		}
	}
}
