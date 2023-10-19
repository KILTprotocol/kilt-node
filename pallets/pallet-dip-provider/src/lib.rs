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

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;

	use frame_support::{pallet_prelude::*, traits::EnsureOrigin};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::FullCodec;
	use sp_std::fmt::Debug;

	use crate::traits::{IdentityCommitmentGenerator, IdentityProvider, SubmitterInfo};

	pub type IdentityOf<T> = <<T as Config>::IdentityProvider as IdentityProvider<<T as Config>::Identifier>>::Success;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type CommitOriginCheck: EnsureOrigin<Self::RuntimeOrigin, Success = Self::CommitOrigin>;
		type CommitOrigin: SubmitterInfo<Submitter = Self::AccountId>;
		type Identifier: Parameter + MaxEncodedLen;
		type IdentityCommitment: Clone + Eq + Debug + TypeInfo + FullCodec + MaxEncodedLen;
		type IdentityCommitmentGenerator: IdentityCommitmentGenerator<
			Self::Identifier,
			IdentityOf<Self>,
			Error = Self::IdentityCommitmentGeneratorError,
			Output = Self::IdentityCommitment,
		>;
		type IdentityCommitmentGeneratorError: Into<u16>;
		type IdentityProvider: IdentityProvider<Self::Identifier, Error = Self::IdentityProviderError>;
		type IdentityProviderError: Into<u16>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn identity_commitments)]
	pub type IdentityCommitments<T> =
		StorageMap<_, Twox64Concat, <T as Config>::Identifier, <T as Config>::IdentityCommitment>;

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		IdentityCommitted {
			identifier: T::Identifier,
			commitment: T::IdentityCommitment,
		},
		IdentityDeleted {
			identifier: T::Identifier,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		IdentityProvider { reason: u16 },
		IdentityCommitmentGenerator { reason: u16 },
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		// TODO: Update weight
		#[pallet::weight(0)]
		pub fn commit_identity(origin: OriginFor<T>, identifier: T::Identifier) -> DispatchResult {
			// TODO: use dispatcher to get deposit
			let _dispatcher =
				T::CommitOriginCheck::ensure_origin(origin).map(|e: <T as Config>::CommitOrigin| e.submitter())?;

			let identity_commitment: Option<T::IdentityCommitment> = match T::IdentityProvider::retrieve(&identifier) {
				Ok(Some(identity)) => T::IdentityCommitmentGenerator::generate_commitment(&identifier, &identity)
					.map(Some)
					.map_err(|error| Error::<T>::IdentityCommitmentGenerator { reason: error.into() }),
				Ok(None) => Ok(None),
				Err(error) => Err(Error::<T>::IdentityProvider { reason: error.into() }),
			}?;

			if let Some(commitment) = identity_commitment {
				// TODO: Take deposit (once 0.9.42 PR is merged into develop)
				IdentityCommitments::<T>::insert(&identifier, commitment.clone());
				Self::deposit_event(Event::<T>::IdentityCommitted { identifier, commitment });
			} else {
				// TODO: Release deposit (once 0.9.42 PR is merged into develop)
				IdentityCommitments::<T>::remove(&identifier);
				Self::deposit_event(Event::<T>::IdentityDeleted { identifier });
			}

			Ok(())
		}
		// TODO: Add extrinsic to remove commitment without requiring the identity to be
		// deleted.
	}
}
