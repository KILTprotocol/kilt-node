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

	use frame_support::{dispatch::Dispatchable, pallet_prelude::*, traits::Contains, Twox64Concat};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::{FullCodec, MaxEncodedLen};
	use scale_info::TypeInfo;
	use sp_core::H256;
	use sp_std::boxed::Box;

	use crate::traits::IdentityProofVerifier;

	pub type VerificationResultOf<T> = <<T as Config>::ProofVerifier as IdentityProofVerifier<
		<T as Config>::RuntimeCall,
		<T as Config>::Identifier,
	>>::VerificationResult;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::storage]
	#[pallet::getter(fn identity_proofs)]
	pub(crate) type IdentityEntries<T> =
		StorageMap<_, Twox64Concat, <T as Config>::Identifier, <T as Config>::LocalIdentityInfo>;

	#[pallet::storage]
	#[pallet::getter(fn latest_relay_roots)]
	pub(crate) type LatestRelayStateRoots<T: Config> = StorageValue<_, (H256, H256, bool), OptionQuery>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Preliminary filter to filter out calls before doing any heavier
		/// computations.
		type DipCallOriginFilter: Contains<<Self as Config>::RuntimeCall>;
		/// The identifier of a subject, e.g., a DID.
		type Identifier: Parameter + MaxEncodedLen;
		/// The proof users must provide to operate with their higher-level
		/// identity. Depending on the use cases, this proof can contain
		/// heterogeneous bits of information that the proof verifier will
		/// utilize. For instance, a proof could contain both a Merkle proof and
		/// a DID signature.
		type IdentityProof: Parameter;
		/// The details stored in this pallet associated with any given subject.
		type LocalIdentityInfo: FullCodec + TypeInfo + MaxEncodedLen;
		/// The logic of the proof verifier, called upon each execution of the
		/// `dispatch_as` extrinsic.
		type ProofVerifier: IdentityProofVerifier<
			<Self as Config>::RuntimeCall,
			Self::Identifier,
			Proof = Self::IdentityProof,
			IdentityDetails = Self::LocalIdentityInfo,
			Submitter = <Self as frame_system::Config>::AccountId,
		>;
		/// The overarching runtime call type.
		type RuntimeCall: Parameter + Dispatchable<RuntimeOrigin = <Self as Config>::RuntimeOrigin>;
		/// The overarching runtime origin type.
		type RuntimeOrigin: From<Origin<Self>> + From<<Self as frame_system::Config>::RuntimeOrigin>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		/// An identity with the provided identifier could not be found.
		IdentityNotFound,
		/// The identity proof provided could not be successfully verified.
		InvalidProof,
		/// The specified call could not be dispatched.
		Dispatch,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
	where
		T: cumulus_pallet_parachain_system::Config,
	{
		fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
			// Reserve weight to update the last relay state root
			<T as frame_system::Config>::DbWeight::get().writes(1)
		}
		fn on_finalize(_n: BlockNumberFor<T>) {
			// Called before the validation data is cleaned in the
			// parachain_system::on_finalize hook
			if let Some(new_validation_data) = cumulus_pallet_parachain_system::Pallet::<T>::validation_data() {
				// TODO: Add test cases
				let new_entry = match LatestRelayStateRoots::<T>::get() {
					Some((first, _, true)) => (first, new_validation_data.relay_parent_storage_root, false),
					Some((_, second, false)) => (new_validation_data.relay_parent_storage_root, second, true),
					None => (new_validation_data.relay_parent_storage_root, H256::default(), true),
				};
				LatestRelayStateRoots::<T>::set(Some(new_entry));
			}
		}
	}

	/// The origin this pallet creates after a user has provided a valid
	/// identity proof to dispatch other calls.
	#[pallet::origin]
	pub type Origin<T> =
		DipOrigin<<T as Config>::Identifier, <T as frame_system::Config>::AccountId, VerificationResultOf<T>>;

	// TODO: Benchmarking
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO: Replace with a SignedExtra.
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn dispatch_as(
			origin: OriginFor<T>,
			identifier: T::Identifier,
			proof: T::IdentityProof,
			call: Box<<T as Config>::RuntimeCall>,
		) -> DispatchResult {
			// TODO: Make origin check configurable, and require that it at least returns
			// the submitter's account.
			let submitter = ensure_signed(origin)?;
			// TODO: Proper error handling
			ensure!(T::DipCallOriginFilter::contains(&*call), Error::<T>::Dispatch);
			let mut identity_entry = IdentityEntries::<T>::get(&identifier);
			let proof_verification_result = T::ProofVerifier::verify_proof_for_call_against_details(
				&*call,
				&identifier,
				&submitter,
				&mut identity_entry,
				proof,
			);
			// Write the identity info to storage after it has optionally been updated by
			// the `ProofVerifier`, regardless of whether the proof has been verified or
			// not.
			IdentityEntries::<T>::mutate(&identifier, |entry| *entry = identity_entry);
			// Unwrap the result if `ok`.
			let proof_verification_result = proof_verification_result.map_err(|_| Error::<T>::InvalidProof)?;
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
