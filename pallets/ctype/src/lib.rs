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

//! The CType pallet registers CTypes on chain. Only the hash of the CType and
//! the owner are stored. CTypes cannot be removed once they where added to the
//! chain.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod benchmarking;

#[cfg(any(feature = "mock", test))]
pub mod mock;

/// Test module for CTYPEs
#[cfg(test)]
mod tests;

pub mod default_weights;

pub use default_weights::WeightInfo;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	use crate::WeightInfo;

	/// Type of a CTYPE hash.
	pub type CtypeHashOf<T> = <T as frame_system::Config>::Hash;

	/// Type of a CTYPE creator.
	pub type CtypeCreatorOf<T> = <T as Config>::CtypeCreatorId;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type EnsureOrigin: EnsureOrigin<Success = CtypeCreatorOf<Self>, <Self as frame_system::Config>::Origin>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		type CtypeCreatorId: Parameter;
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
	pub type Ctypes<T> = StorageMap<_, Blake2_128Concat, CtypeHashOf<T>, CtypeCreatorOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new CTYPE has been created.
		/// \[creator identifier, CTYPE hash\]
		CtypeCreated(CtypeCreatorOf<T>, CtypeHashOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// There is no CTYPE with the given hash.
		CtypeNotFound,
		/// The CTYPE already exists.
		CtypeAlreadyExists,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new CTYPE and associates it with its creator.
		///
		/// * origin: the identifier of the CTYPE creator
		/// * hash: the CTYPE hash. It has to be unique.
		#[pallet::weight(0)]
		pub fn add(origin: OriginFor<T>, hash: CtypeHashOf<T>) -> DispatchResultWithPostInfo {
			let creator = <T as Config>::EnsureOrigin::ensure_origin(origin)?;

			ensure!(!<Ctypes<T>>::contains_key(&hash), Error::<T>::CtypeAlreadyExists);

			log::debug!("Creating CTYPE with hash {:?} and creator {:?}", &hash, &creator);
			<Ctypes<T>>::insert(&hash, creator.clone());

			Self::deposit_event(Event::CtypeCreated(creator, hash));

			Ok(None.into())
		}
	}
}
