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

//! CTYPE: Handles CTYPEs on chain,
//! adding CTYPEs.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

/// Test module for CTYPEs
#[cfg(test)]
mod tests;

#[cfg(any(feature = "mock", test))]
pub mod mock;

pub mod default_weights;
pub use default_weights::WeightInfo;

use codec::{Decode, Encode};
use frame_support::ensure;
use sp_std::fmt::Debug;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// The type of a CTYPE hash.
	pub type CtypeHash<T> = <T as frame_system::Config>::Hash;

	/// The type of a CTYPE creator.
	pub type CtypeCreator<T> = did::DidIdentifier<T>;

	#[pallet::config]
	pub trait Config: frame_system::Config + did::Config {
		type EnsureOrigin: EnsureOrigin<Success = CtypeCreator<Self>, <Self as frame_system::Config>::Origin>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// CTYPEs stored on chain.
	///
	/// It maps from a CTYPE hash to its creator.
	#[pallet::storage]
	#[pallet::getter(fn ctypes)]
	pub type Ctypes<T> =
		StorageMap<_, Blake2_128Concat, CtypeHash<T>, CtypeCreator<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new CTYPE has been created.
		/// \[creator DID, CTYPE hash\]
		CTypeCreated(CtypeCreator<T>, CtypeHash<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// There is no CTYPE with the given hash.
		CTypeNotFound,
		/// The CTYPE already exists.
		CTypeAlreadyExists,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new CTYPE.
		///
		/// * origin: the identifier of the CTYPE creator
		/// * hash: the CTYPE hash. It has to be unique.
		#[pallet::weight(10)]
		pub fn add(
			origin: OriginFor<T>,
			hash: CtypeHash<T>
		) -> DispatchResultWithPostInfo {
			let creator = T::EnsureOrigin::ensure_origin(origin)?;

			ensure!(
				!<Ctypes<T>>::contains_key(&hash),
				Error::<T>::CTypeAlreadyExists
			);

			log::debug!("insert CTYPE");
			<Ctypes<T>>::insert(&hash, creator.clone());

			Self::deposit_event(Event::CTypeCreated(creator, hash));

			Ok(None.into())
		}
	}
}
