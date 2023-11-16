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

//! This pallet is a core component of the Decentralized Identity Provider
//! protocol. It enables a Substrate-based chain (provider) to bridge the
//! identities of its users to other connected chains (consumers) trustlessly. A
//! consumer chain is *connected* to a provider if there is a way for the
//! consumer chain to verify state proofs about parts of the state of the
//! provider chain.

//! The pallet is agnostic over the chain-specific definition of *identity*, and
//! delegates the definition of it to the provider chain's runtime.

//! What the pallet stores are *identity commitments*, which are opaque byte
//! blobs put in the pallet storage and on which the cross-chain identity
//! bridging protocol can be built. As for identities, the definition of an
//! identity commitment must be provided by the runtime and is therefore
//! provider-specific. Naturally, this definition must be made available to
//! consumers willing to integrate the identities living on the provider chain.

//! Because providers and consumers evolve at different speeds, identity
//! commitments are versioned. This allows the provider chain to upgrade to a
//! newer commitment scheme, while still giving its users the possibility to use
//! the old version, if the chains on which they want to use their identity does
//! not yet support the new scheme.

//! Identity commitments can be replaced (e.g., if something in the identity
//! info changes), or removed altogether by the identity subject. After removal,
//! the identity becomes unusable cross-chain, although it will still continue
//! to exist on the provider chain and will be usable for local operations.

#![cfg_attr(not(feature = "std"), no_std)]

// !!! When Rust docs changes here, make sure to update the crate README.md file
// as well.
// !!!

pub mod traits;

pub use crate::{
	pallet::*,
	traits::{DefaultIdentityCommitmentGenerator, DefaultIdentityProvider, NoopHooks},
};

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;

	use frame_support::{pallet_prelude::*, traits::EnsureOriginWithArg};
	use frame_system::pallet_prelude::*;

	use crate::traits::{IdentityCommitmentGenerator, IdentityProvider, ProviderHooks, SubmitterInfo};

	pub type IdentityCommitmentOf<T> =
		<<T as Config>::IdentityCommitmentGenerator as IdentityCommitmentGenerator<T>>::Output;
	pub type IdentityProviderOf<T> = <T as Config>::IdentityProvider;
	pub type IdentityOf<T> = <<T as Config>::IdentityProvider as IdentityProvider<T>>::Success;
	pub type IdentityCommitmentVersion = u16;

	pub const LATEST_COMMITMENT_VERSION: IdentityCommitmentVersion = 0;
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The check ensuring a given runtime origin is allowed to generate and
		/// remove identity commitments.
		type CommitOriginCheck: EnsureOriginWithArg<Self::RuntimeOrigin, Self::Identifier, Success = Self::CommitOrigin>;
		/// The resulting origin if `CommitOriginCheck` returns with errors. The
		/// origin is not required to be an `AccountId`, but must include
		/// information about the `AccountId` of the tx submitter.
		type CommitOrigin: SubmitterInfo<Submitter = Self::AccountId>;
		/// The type of an identifier used to retrieve identity information
		/// about a subject.
		type Identifier: Parameter + MaxEncodedLen;
		/// The type responsible for generating identity commitments, given the
		/// identity information associated to a given `Identifier`.
		type IdentityCommitmentGenerator: IdentityCommitmentGenerator<Self>;
		/// The type responsible for retrieving the information associated to a
		/// subject given their identifier. The information can potentially be
		/// retrieved from any source, using a combination of on-chain and
		/// off-chain solutions.
		type IdentityProvider: IdentityProvider<Self>;
		/// Customizable external logic to handle events in which a new identity
		/// commitment is generated or removed.
		type ProviderHooks: ProviderHooks<Self>;
		/// The aggregate `Event` type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	/// The pallet contains a single storage element, the `IdentityCommitments`
	/// double map. Its first key is the `Identifier` of subjects, while the
	/// second key is the commitment version. The values are identity
	/// commitments.
	#[pallet::storage]
	#[pallet::getter(fn identity_commitments)]
	pub type IdentityCommitments<T> = StorageDoubleMap<
		_,
		Twox64Concat,
		<T as Config>::Identifier,
		Twox64Concat,
		IdentityCommitmentVersion,
		IdentityCommitmentOf<T>,
	>;

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new commitment has been stored.
		VersionedIdentityCommitted {
			/// The identifier of the identity committed.
			identifier: T::Identifier,
			/// The value of the commitment.
			commitment: IdentityCommitmentOf<T>,
			/// The version of the commitment.
			version: IdentityCommitmentVersion,
		},
		/// A commitment has been deleted.
		VersionedIdentityDeleted {
			/// The identifier of the identity committed.
			identifier: T::Identifier,
			/// The version of the commitment.
			version: IdentityCommitmentVersion,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The specified commitment cannot be found.
		CommitmentNotFound,
		/// Error when retrieving the identity details of the provided subject.
		IdentityProvider(u16),
		/// Error when generating a commitment for the retrieved identity.
		IdentityCommitmentGenerator(u16),
		/// Error inside the external hook logic.
		Hook(u16),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Generate a new versioned commitment for the subject identified by
		/// the provided `Identifier`. If an old commitment for the same version
		/// is present, it is overridden. Hooks are called before the new
		/// commitment is stored, and optionally before the old one is replaced.
		#[pallet::call_index(0)]
		// TODO: Update weight
		#[pallet::weight(0)]
		pub fn commit_identity(
			origin: OriginFor<T>,
			identifier: T::Identifier,
			version: Option<IdentityCommitmentVersion>,
		) -> DispatchResult {
			let dispatcher = T::CommitOriginCheck::ensure_origin(origin, &identifier)
				.map(|e: <T as Config>::CommitOrigin| e.submitter())?;

			let commitment_version = version.unwrap_or(LATEST_COMMITMENT_VERSION);
			let identity = T::IdentityProvider::retrieve(&identifier)
				.map_err(|error| Error::<T>::IdentityProvider(error.into()))?;
			let commitment =
				T::IdentityCommitmentGenerator::generate_commitment(&identifier, &identity, commitment_version)
					.map_err(|error| Error::<T>::IdentityCommitmentGenerator(error.into()))?;

			IdentityCommitments::<T>::try_mutate(&identifier, commitment_version, |commitment_entry| {
				if let Some(old_commitment) = commitment_entry {
					T::ProviderHooks::on_commitment_removed(
						&identifier,
						&dispatcher,
						old_commitment,
						commitment_version,
					)
					.map_err(|e| Error::<T>::Hook(e.into()))?;
					Self::deposit_event(Event::<T>::VersionedIdentityDeleted {
						identifier: identifier.clone(),
						version: commitment_version,
					});
				}
				T::ProviderHooks::on_identity_committed(&identifier, &dispatcher, &commitment, commitment_version)
					.map_err(|e| Error::<T>::Hook(e.into()))?;
				*commitment_entry = Some(commitment.clone());
				Self::deposit_event(Event::<T>::VersionedIdentityCommitted {
					identifier: identifier.clone(),
					commitment,
					version: commitment_version,
				});
				Ok::<_, Error<T>>(())
			})?;
			Ok(())
		}

		/// Delete an identity commitment of a specific version for a specific
		/// `Identifier`. If a commitment of the provided version does not exist
		/// for the given `Identifier`, an error is returned. Hooks are called
		/// after the commitment has been removed.
		#[pallet::call_index(1)]
		// TODO: Update weight
		#[pallet::weight(0)]
		pub fn delete_identity_commitment(
			origin: OriginFor<T>,
			identifier: T::Identifier,
			version: Option<IdentityCommitmentVersion>,
		) -> DispatchResult {
			let dispatcher = T::CommitOriginCheck::ensure_origin(origin, &identifier)
				.map(|e: <T as Config>::CommitOrigin| e.submitter())?;

			let commitment_version = version.unwrap_or(LATEST_COMMITMENT_VERSION);
			let commitment = Self::delete_identity_commitment_storage_entry(&identifier, commitment_version)?;
			T::ProviderHooks::on_commitment_removed(&identifier, &dispatcher, &commitment, commitment_version)
				.map_err(|e| Error::<T>::Hook(e.into()))?;
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn delete_identity_commitment_storage_entry(
			identifier: &T::Identifier,
			version: IdentityCommitmentVersion,
		) -> Result<IdentityCommitmentOf<T>, DispatchError> {
			let commitment =
				IdentityCommitments::<T>::take(identifier, version).ok_or(Error::<T>::CommitmentNotFound)?;
			Self::deposit_event(Event::<T>::VersionedIdentityDeleted {
				identifier: identifier.clone(),
				version,
			});
			Ok(commitment)
		}
	}
}
