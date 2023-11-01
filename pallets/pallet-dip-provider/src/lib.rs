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

	use crate::traits::{IdentityCommitmentGenerator, IdentityProvider, ProviderHooks, SubmitterInfo};

	pub type IdentityOf<T> = <<T as Config>::IdentityProvider as IdentityProvider<<T as Config>::Identifier>>::Success;
	pub type IdentityCommitmentVersion = u16;

	pub const LATEST_COMMITMENT_VERSION: IdentityCommitmentVersion = 0;
	pub const MAX_COMMITMENTS_PER_IDENTITY: u16 = LATEST_COMMITMENT_VERSION + 1;
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::composite_enum]
	pub enum HoldReason {
		Deposit,
	}

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
		type ProviderHooks: ProviderHooks<
			Identifier = Self::Identifier,
			Submitter = Self::AccountId,
			IdentityCommitment = Self::IdentityCommitment,
		>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn identity_commitments)]
	pub type IdentityCommitments<T> = StorageDoubleMap<
		_,
		Twox64Concat,
		<T as Config>::Identifier,
		Twox64Concat,
		IdentityCommitmentVersion,
		<T as Config>::IdentityCommitment,
	>;

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		IdentityCommitted {
			identifier: T::Identifier,
			commitment: T::IdentityCommitment,
			version: IdentityCommitmentVersion,
		},
		VersionedIdentityDeleted {
			identifier: T::Identifier,
			version: IdentityCommitmentVersion,
		},
		IdentityDeleted {
			identifier: T::Identifier,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		IdentityNotFound,
		LimitTooLow,
		IdentityProvider(u16),
		IdentityCommitmentGenerator(u16),
		HookError(u16),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		// TODO: Update weight
		#[pallet::weight(0)]
		pub fn commit_identity(
			origin: OriginFor<T>,
			identifier: T::Identifier,
			version: Option<IdentityCommitmentVersion>,
		) -> DispatchResult {
			let dispatcher =
				T::CommitOriginCheck::ensure_origin(origin).map(|e: <T as Config>::CommitOrigin| e.submitter())?;

			let commitment_version = version.unwrap_or(LATEST_COMMITMENT_VERSION);
			let commitment = match T::IdentityProvider::retrieve(&identifier) {
				Ok(None) => Err(Error::<T>::IdentityNotFound),
				Err(error) => Err(Error::<T>::IdentityProvider(error.into())),
				Ok(Some(identity)) => {
					T::IdentityCommitmentGenerator::generate_commitment(&identifier, &identity, commitment_version)
						.map_err(|error| Error::<T>::IdentityCommitmentGenerator(error.into()))
				}
			}?;

			IdentityCommitments::<T>::try_mutate(&identifier, commitment_version, |commitment_entry| {
				if let Some(old_commitment) = commitment_entry {
					T::ProviderHooks::on_commitment_removed(
						&identifier,
						&dispatcher,
						old_commitment,
						commitment_version,
					)
					.map_err(|e| Error::<T>::HookError(e.into()))?;
					Self::deposit_event(Event::<T>::VersionedIdentityDeleted {
						identifier: identifier.clone(),
						version: commitment_version,
					});
				}
				T::ProviderHooks::on_identity_committed(&identifier, &dispatcher, &commitment, commitment_version)
					.map_err(|e| Error::<T>::HookError(e.into()))?;
				*commitment_entry = Some(commitment.clone());
				Self::deposit_event(Event::<T>::IdentityCommitted {
					identifier: identifier.clone(),
					commitment,
					version: commitment_version,
				});
				Ok::<_, Error<T>>(())
			})?;
			Ok(())
		}

		#[pallet::call_index(1)]
		// TODO: Update weight
		#[pallet::weight(0)]
		pub fn delete_identity_commitment(
			origin: OriginFor<T>,
			identifier: T::Identifier,
			version: Option<IdentityCommitmentVersion>,
		) -> DispatchResult {
			let dispatcher =
				T::CommitOriginCheck::ensure_origin(origin).map(|e: <T as Config>::CommitOrigin| e.submitter())?;

			let commitment_version = version.unwrap_or(LATEST_COMMITMENT_VERSION);
			IdentityCommitments::<T>::try_mutate(&identifier, commitment_version, |commitment_entry| {
				match commitment_entry {
					None => Err(Error::<T>::IdentityNotFound),
					Some(commitment) => {
						T::ProviderHooks::on_commitment_removed(
							&identifier,
							&dispatcher,
							commitment,
							commitment_version,
						)
						.map_err(|e| Error::<T>::HookError(e.into()))?;
						*commitment_entry = None;
						Self::deposit_event(Event::<T>::VersionedIdentityDeleted {
							identifier: identifier.clone(),
							version: commitment_version,
						});
						Ok(())
					}
				}
			})?;
			Ok(())
		}
	}
}
