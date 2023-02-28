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

// TODO: Pallet description

#![cfg_attr(not(feature = "std"), no_std)]

pub mod traits;

#[cfg(tests)]
mod tests;

pub use crate::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use crate::traits::IdentityProofAction;

	use frame_support::{pallet_prelude::*, traits::EnsureOrigin, Twox64Concat};
	use frame_system::pallet_prelude::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::storage]
	#[pallet::getter(fn identity_proofs)]
	pub(crate) type IdentityProofs<T> = StorageMap<_, Twox64Concat, <T as Config>::Identifier, <T as Config>::Proof>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Identifier: Parameter + MaxEncodedLen;
		type Proof: Parameter + MaxEncodedLen;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type EnsureSourceXcmOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		IdentityInfoUpdated(T::Identifier, T::Proof),
		IdentityInfoDeleted(T::Identifier),
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn process_identity_action(
			origin: OriginFor<T>,
			action: IdentityProofAction<T::Identifier, T::Proof>,
		) -> DispatchResult {
			T::EnsureSourceXcmOrigin::ensure_origin(origin)?;

			let event = match action {
				IdentityProofAction::Updated(identifier, proof) => {
					IdentityProofs::<T>::mutate(&identifier, |entry| *entry = Some(proof.clone()));
					Event::<T>::IdentityInfoUpdated(identifier, proof)
				}
				IdentityProofAction::Deleted(identifier) => {
					IdentityProofs::<T>::remove(&identifier);
					Event::<T>::IdentityInfoDeleted(identifier)
				}
			};

			Self::deposit_event(event);
			Ok(())
		}
	}
}
