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

pub mod identity;
pub mod traits;

mod origin;

pub use crate::{origin::*, pallet::*};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use cumulus_pallet_xcm::ensure_sibling_para;
	use frame_support::{dispatch::Dispatchable, pallet_prelude::*, traits::Contains, Twox64Concat};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::MaxEncodedLen;
	use sp_std::boxed::Box;

	use dip_support::IdentityDetailsAction;

	use crate::{identity::IdentityDetails, traits::IdentityProofVerifier};

	pub type VerificationResultOf<T> = <<T as Config>::ProofVerifier as IdentityProofVerifier<
		<T as Config>::RuntimeCall,
		<T as Config>::Identifier,
	>>::VerificationResult;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	// TODO: Store also additional details received by the provider.
	#[pallet::storage]
	#[pallet::getter(fn identity_proofs)]
	pub(crate) type IdentityEntries<T> = StorageMap<
		_,
		Twox64Concat,
		<T as Config>::Identifier,
		IdentityDetails<<T as Config>::ProofDigest, <T as Config>::IdentityDetails>,
	>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Preliminary filter to filter out calls before doing any heavier
		/// computations.
		type DipCallOriginFilter: Contains<<Self as Config>::RuntimeCall>;
		/// The identifier of a subject, e.g., a DID.
		type Identifier: Parameter + MaxEncodedLen;
		/// The details stored in this pallet associated with any given subject.
		type IdentityDetails: Parameter + MaxEncodedLen + Default;
		/// The proof users must provide to operate with their higher-level
		/// identity. Depending on the use cases, this proof can contain
		/// heterogeneous bits of information that the proof verifier will
		/// utilize. For instance, a proof could contain both a Merkle proof and
		/// a DID signature.
		type Proof: Parameter;
		/// The type of the committed proof digest used as the basis for
		/// verifying identity proofs.
		type ProofDigest: Parameter + MaxEncodedLen;
		/// The logic of the proof verifier, called upon each execution of the
		/// `dispatch_as` extrinsic.
		type ProofVerifier: IdentityProofVerifier<
			<Self as Config>::RuntimeCall,
			Self::Identifier,
			Proof = Self::Proof,
			IdentityDetails = IdentityDetails<Self::ProofDigest, Self::IdentityDetails>,
			Submitter = <Self as frame_system::Config>::AccountId,
		>;
		/// The overarching runtime call type.
		type RuntimeCall: Parameter + Dispatchable<RuntimeOrigin = <Self as Config>::RuntimeOrigin>;
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The overarching runtime origin type.
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
		/// The identity information related to a given subject has been
		/// deleted.
		IdentityInfoDeleted(T::Identifier),
		/// The identity information related to a given subject has been updated
		/// to a new digest.
		IdentityInfoUpdated(T::Identifier, T::ProofDigest),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// An identity with the provided identifier could not be found.
		IdentityNotFound,
		/// The identity proof provided could not be successfully verified.
		InvalidProof,
		/// The specified call could not be dispatched.
		Dispatch,
	}

	/// The origin this pallet creates after a user has provided a valid
	/// identity proof to dispatch other calls.
	#[pallet::origin]
	pub type Origin<T> =
		DipOrigin<<T as Config>::Identifier, <T as frame_system::Config>::AccountId, VerificationResultOf<T>>;

	// TODO: Benchmarking
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn process_identity_action(
			origin: OriginFor<T>,
			action: IdentityDetailsAction<T::Identifier, T::ProofDigest>,
		) -> DispatchResult {
			ensure_sibling_para(<T as Config>::RuntimeOrigin::from(origin))?;

			let event = match action {
				IdentityDetailsAction::Updated(identifier, proof, _) => {
					IdentityEntries::<T>::mutate(
						&identifier,
						|entry: &mut Option<
							IdentityDetails<<T as Config>::ProofDigest, <T as Config>::IdentityDetails>,
						>| { *entry = Some(proof.clone().into()) },
					);
					Ok::<_, Error<T>>(Event::<T>::IdentityInfoUpdated(identifier, proof))
				}
				IdentityDetailsAction::Deleted(identifier) => {
					IdentityEntries::<T>::remove(&identifier);
					Ok::<_, Error<T>>(Event::<T>::IdentityInfoDeleted(identifier))
				}
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
			proof: T::Proof,
			call: Box<<T as Config>::RuntimeCall>,
		) -> DispatchResult {
			let submitter = ensure_signed(origin)?;
			// TODO: Proper error handling
			ensure!(T::DipCallOriginFilter::contains(&*call), Error::<T>::Dispatch);
			let mut identity_entry = IdentityEntries::<T>::get(&identifier).ok_or(Error::<T>::IdentityNotFound)?;
			let proof_verification_result = T::ProofVerifier::verify_proof_for_call_against_details(
				&*call,
				&identifier,
				&submitter,
				&mut identity_entry,
				&proof,
			)
			.map_err(|_| Error::<T>::InvalidProof)?;
			// Write the identity info to storage after it has optionally been updated by
			// the `ProofVerifier`.
			IdentityEntries::<T>::mutate(&identifier, |entry| *entry = Some(identity_entry));
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
