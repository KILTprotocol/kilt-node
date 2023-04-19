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

mod origin;
pub mod proof;
pub mod traits;

pub use crate::{origin::*, pallet::*};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use cumulus_pallet_xcm::ensure_sibling_para;
	use frame_support::{dispatch::Dispatchable, pallet_prelude::*, Twox64Concat};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::MaxEncodedLen;
	use sp_std::boxed::Box;

	use dip_support::{latest::IdentityProofAction, VersionedIdentityProofAction};

	use crate::{proof::ProofEntry, traits::DipCallProofVerifier};

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	pub type ProofEntryOf<T> = ProofEntry<<T as Config>::ProofDigest, <T as Config>::IdentityDetails>;

	// TODO: Store also additional details received by the provider.
	#[pallet::storage]
	#[pallet::getter(fn identity_proofs)]
	pub(crate) type IdentityProofs<T> = StorageMap<_, Twox64Concat, <T as Config>::Identifier, ProofEntryOf<T>>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type CallProof: Parameter;
		type CallProofVerifier: DipCallProofVerifier<
			<Self as Config>::RuntimeCall,
			Proof = Self::CallProof,
			ProofEntry = ProofEntryOf<Self>,
			Submitter = <Self as frame_system::Config>::AccountId,
		>;
		type IdentityDetails: Parameter + MaxEncodedLen + Default;
		type Identifier: Parameter + MaxEncodedLen;
		type ProofDigest: Parameter + MaxEncodedLen;
		type RuntimeCall: Parameter + Dispatchable<RuntimeOrigin = <Self as Config>::RuntimeOrigin>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type RuntimeOrigin: From<Origin<Self>>
			+ From<<Self as frame_system::Config>::RuntimeOrigin>
			+ Into<Result<cumulus_pallet_xcm::Origin, <Self as Config>::RuntimeOrigin>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		IdentityInfoDeleted(T::Identifier),
		IdentityInfoUpdated(T::Identifier, T::ProofDigest),
	}

	#[pallet::error]
	pub enum Error<T> {
		BadOrigin,
		Dispatch,
		IdentityNotFound,
		InvalidProof,
		UnsupportedVersion,
	}

	// The new origin other pallets can use.
	#[pallet::origin]
	pub type Origin<T> = DipOrigin<
		<T as Config>::Identifier,
		<T as frame_system::Config>::AccountId,
		<<T as Config>::CallProofVerifier as DipCallProofVerifier<<T as Config>::RuntimeCall>>::Success,
	>;

	// TODO: Benchmarking
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn process_identity_action(
			origin: OriginFor<T>,
			action: VersionedIdentityProofAction<T::Identifier, T::ProofDigest>,
		) -> DispatchResult {
			ensure_sibling_para(<T as Config>::RuntimeOrigin::from(origin))?;

			let event = match action {
				VersionedIdentityProofAction::V1(IdentityProofAction::Updated(identifier, proof, _)) => {
					IdentityProofs::<T>::mutate(&identifier, |entry| {
						*entry = Some(ProofEntry::from_digest(proof.clone()))
					});
					Ok::<_, Error<T>>(Event::<T>::IdentityInfoUpdated(identifier, proof))
				}
				VersionedIdentityProofAction::V1(IdentityProofAction::Deleted(identifier)) => {
					IdentityProofs::<T>::remove(&identifier);
					Ok::<_, Error<T>>(Event::<T>::IdentityInfoDeleted(identifier))
				}
				_ => Err(Error::<T>::UnsupportedVersion),
			}?;

			Self::deposit_event(event);

			Ok(())
		}

		// TODO: Replace with a SignedExtra.
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn dispatch_as(
			origin: OriginFor<T>,
			identifier: T::Identifier,
			call: Box<<T as Config>::RuntimeCall>,
			proof: T::CallProof,
		) -> DispatchResult {
			let submitter = ensure_signed(origin)?;
			let proof_entry = IdentityProofs::<T>::get(&identifier).ok_or(Error::<T>::IdentityNotFound)?;
			let proof_verification_result =
				T::CallProofVerifier::verify_pre_dispatch(&*call, &submitter, &proof_entry, proof)
					.map_err(|_| Error::<T>::BadOrigin)?;
			// TODO: Proper DID signature verification (and cross-chain replay protection)
			let did_origin = DipOrigin {
				identifier,
				account_address: submitter,
				details: proof_verification_result,
			};
			// TODO: Use dispatch info for weight calculation
			let _ = call.dispatch(did_origin.into()).map_err(|_| Error::<T>::Dispatch)?;
			Ok(())
		}
	}
}
