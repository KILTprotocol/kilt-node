// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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
//! ## Interface
//!
//! ### Dispatchable Functions
//! - `add` - Create a new CType from a given unique CType hash and associate it
//!   with the origin of the call.
//!
//! ## Assumptions
//!
//! - The CType hash was created using our KILT JS-SDK.
//! - The underlying CType includes only the following required fields for the
//!   JSON-Schema we use in the SDK: Identifier, KILT specific JSON-Schema,
//!   Title and Properties.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod default_weights;

#[cfg(any(feature = "mock", test))]
pub mod mock;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

/// Test module for CTypes
#[cfg(test)]
mod tests;

pub use crate::{default_weights::WeightInfo, pallet::*};

pub trait PayFee<AccountId> {
	/// Make sure that the fee can be paid.
	///
	/// This function must not change any storage value since the ctype might
	/// not get created.
	fn secure_fee(payer: AccountId, ctype_size: usize) -> Result<(), ()>;

	/// Actually pay the fee.
	fn pay_fee(payer: AccountId, ctype_size: usize);
}

impl<AccountId> PayFee<AccountId> for () {
	fn secure_fee(_payer: AccountId, _ctype_size: usize) -> Result<(), ()> {
		Ok(())
	}

	fn pay_fee(_payer: AccountId, _ctype_size: usize) {}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, sp_runtime::traits::Hash};
	use frame_system::pallet_prelude::*;
	use kilt_support::traits::CallSources;
	use sp_std::vec::Vec;

	/// Type of a CType hash.
	pub type CtypeHashOf<T> = <T as frame_system::Config>::Hash;

	/// Type of a CType creator.
	pub type CtypeCreatorOf<T> = <T as Config>::CtypeCreatorId;

	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type EnsureOrigin: EnsureOrigin<Success = Self::OriginSuccess, <Self as frame_system::Config>::Origin>;
		type OriginSuccess: CallSources<AccountIdOf<Self>, CtypeCreatorOf<Self>>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
		type CtypeCreatorId: Parameter + Default;
		type FeeHandler: PayFee<AccountIdOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// CTypes stored on chain.
	///
	/// It maps from a CType hash to its creator.
	#[pallet::storage]
	#[pallet::getter(fn ctypes)]
	pub type Ctypes<T> = StorageMap<_, Blake2_128Concat, CtypeHashOf<T>, CtypeCreatorOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new CType has been created.
		/// \[creator identifier, CType hash\]
		CTypeCreated(CtypeCreatorOf<T>, CtypeHashOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// There is no CType with the given hash.
		CTypeNotFound,
		/// The CType already exists.
		CTypeAlreadyExists,
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
		/// - Reads: Ctypes
		/// - Writes: Ctypes
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::add())]
		pub fn add(origin: OriginFor<T>, ctype: Vec<u8>) -> DispatchResult {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;
			let creator = source.subject();
			let payer = source.sender();

			T::FeeHandler::secure_fee(payer.clone(), ctype.len()).map_err(|_| Error::<T>::UnableToPayFees)?;

			let hash = <T as frame_system::Config>::Hashing::hash(&ctype[..]);

			ensure!(!<Ctypes<T>>::contains_key(&hash), Error::<T>::CTypeAlreadyExists);

			log::debug!("Creating CType with hash {:?} and creator {:?}", hash, creator);
			<Ctypes<T>>::insert(&hash, creator.clone());
			T::FeeHandler::pay_fee(payer, ctype.len());

			Self::deposit_event(Event::CTypeCreated(creator, hash));

			Ok(())
		}
	}
}
