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

pub mod default_weights;

#[cfg(any(feature = "mock", test))]
pub mod mock;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

/// Test module for CTYPEs
#[cfg(test)]
mod tests;

pub use crate::{default_weights::WeightInfo, pallet::*};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Type of a CTYPE hash.
	pub type CtypeHashOf<T> = <T as frame_system::Config>::Hash;

	/// Type of a CTYPE creator.
	pub type CtypeCreatorOf<T> = <T as Config>::CtypeCreatorId;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type CtypeCreatorId: Parameter + Default;
		type EnsureOrigin: EnsureOrigin<Success = CtypeCreatorOf<Self>, <Self as frame_system::Config>::Origin>;
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
	pub type Ctypes<T> = StorageMap<_, Blake2_128Concat, CtypeHashOf<T>, CtypeCreatorOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new CTYPE has been created.
		/// \[creator identifier, CTYPE hash\]
		CTypeCreated(CtypeCreatorOf<T>, CtypeHashOf<T>),
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
		/// Create a new CTYPE and associates it with its creator.
		///
		/// * origin: the identifier of the CTYPE creator
		/// * hash: the CTYPE hash. It has to be unique.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::add())]
		pub fn add(origin: OriginFor<T>, hash: CtypeHashOf<T>) -> DispatchResult {
			let creator = <T as Config>::EnsureOrigin::ensure_origin(origin)?;

			ensure!(!<Ctypes<T>>::contains_key(&hash), Error::<T>::CTypeAlreadyExists);

			log::debug!("Creating CTYPE with hash {:?} and creator {:?}", &hash, &creator);
			<Ctypes<T>>::insert(&hash, creator.clone());

			Self::deposit_event(Event::CTypeCreated(creator, hash));

			Ok(())
		}
	}
}
