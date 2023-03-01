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

	use crate::traits::{IdentityProofAction, IdentityProofDispatcher, IdentityProofGenerator, IdentityProvider};

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::fmt::Debug;
	use xcm::v3::{MultiAsset, MultiLocation};

	pub type IdentityProofActionOf<T> = IdentityProofAction<<T as Config>::Identifier, <T as Config>::ProofOutput>;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0); // No need to write a migration to store it.

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Identifier: Parameter;
		type Identity;
		type ProofOutput: Clone + Eq + Debug;
		type IdentityProofGenerator: IdentityProofGenerator<Self::Identifier, Self::Identity, Self::ProofOutput>;
		type IdentityProofDispatcher: IdentityProofDispatcher<Self::Identifier, Self::AccountId, Self::ProofOutput>;
		type IdentityProvider: IdentityProvider<Self::Identifier, Self::Identity>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		IdentityInfoDispatched(IdentityProofActionOf<T>, Box<MultiLocation>),
	}

	#[pallet::error]
	pub enum Error<T> {
		IdentityNotFound,
		Dispatch,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn commit_identity(
			origin: OriginFor<T>,
			identifier: T::Identifier,
			asset: Box<MultiAsset>,
			destination: Box<MultiLocation>,
		) -> DispatchResult {
			println!("dip_sender::commit_identity 1");
			let dispatcher = ensure_signed(origin)?;
			println!("dip_sender::commit_identity 2");

			let action = match T::IdentityProvider::retrieve(&identifier) {
				Ok(Some(identity)) => {
					let identity_proof = T::IdentityProofGenerator::generate_proof(&identifier, &identity)?;
					Ok(IdentityProofAction::Updated(identifier, identity_proof))
				}
				Ok(None) => Ok(IdentityProofAction::Deleted(identifier)),
				_ => Err(Error::<T>::IdentityNotFound),
			}?;
			println!("dip_sender::commit_identity 3");

			//TODO: Proper error handling
			T::IdentityProofDispatcher::dispatch(action.clone(), dispatcher, *asset, *destination)
				.map_err(|_| Error::<T>::Dispatch)?;
			Self::deposit_event(Event::IdentityInfoDispatched(action, destination));
			println!("dip_sender::commit_identity 5");
			Ok(())
		}
	}
}
