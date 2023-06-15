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

pub use crate::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::{pallet_prelude::*, traits::EnsureOrigin, weights::Weight};
	use frame_system::pallet_prelude::*;
	use sp_std::{boxed::Box, fmt::Debug};
	use xcm::{latest::prelude::*, VersionedMultiAsset, VersionedMultiLocation};

	use dip_support::IdentityDetailsAction;

	use crate::traits::{IdentityProofDispatcher, IdentityProofGenerator, IdentityProvider, SubmitterInfo, TxBuilder};

	pub type IdentityOf<T> = <<T as Config>::IdentityProvider as IdentityProvider<<T as Config>::Identifier>>::Success;
	pub type IdentityProofActionOf<T> = IdentityDetailsAction<<T as Config>::Identifier, <T as Config>::ProofOutput>;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type CommitOriginCheck: EnsureOrigin<Self::RuntimeOrigin, Success = Self::CommitOrigin>;
		type CommitOrigin: SubmitterInfo<Submitter = Self::AccountId>;
		type Identifier: Parameter;
		type IdentityProofGenerator: IdentityProofGenerator<
			Self::Identifier,
			IdentityOf<Self>,
			Output = Self::ProofOutput,
		>;
		type IdentityProofDispatcher: IdentityProofDispatcher<Self::Identifier, Self::ProofOutput, Self::AccountId, ()>;
		type IdentityProvider: IdentityProvider<Self::Identifier>;
		type ProofOutput: Clone + Eq + Debug;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type TxBuilder: TxBuilder<Self::Identifier, Self::ProofOutput, ()>;
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
		BadVersion,
		Dispatch,
		IdentityNotFound,
		IdentityProofGeneration,
		Predispatch,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		// TODO: Update weight
		#[pallet::weight(0)]
		pub fn commit_identity(
			origin: OriginFor<T>,
			identifier: T::Identifier,
			destination: Box<VersionedMultiLocation>,
			asset: Box<VersionedMultiAsset>,
			weight: Weight,
		) -> DispatchResult {
			let dispatcher = T::CommitOriginCheck::ensure_origin(origin).map(|e| e.submitter())?;

			let destination: MultiLocation = (*destination).try_into().map_err(|_| Error::<T>::BadVersion)?;
			let action: IdentityProofActionOf<T> = match T::IdentityProvider::retrieve(&identifier) {
				Ok(Some(identity)) => {
					let identity_proof = T::IdentityProofGenerator::generate_commitment(&identifier, &identity)
						.map_err(|_| Error::<T>::IdentityProofGeneration)?;
					Ok(IdentityDetailsAction::Updated(identifier, identity_proof, ()))
				}
				Ok(None) => Ok(IdentityDetailsAction::Deleted(identifier)),
				Err(_) => Err(Error::<T>::IdentityNotFound),
			}?;
			// TODO: Add correct version creation based on lookup (?)

			let asset: MultiAsset = (*asset).try_into().map_err(|_| Error::<T>::BadVersion)?;

			let (ticket, _) = T::IdentityProofDispatcher::pre_dispatch::<T::TxBuilder>(
				action.clone(),
				dispatcher,
				asset,
				weight,
				destination,
			)
			.map_err(|_| Error::<T>::Predispatch)?;

			// TODO: Use returned asset of `pre_dispatch` to charge the tx submitter for the
			// fee.
			T::IdentityProofDispatcher::dispatch(ticket).map_err(|_| Error::<T>::Dispatch)?;

			Self::deposit_event(Event::IdentityInfoDispatched(action, Box::new(destination)));
			Ok(())
		}
	}
}
