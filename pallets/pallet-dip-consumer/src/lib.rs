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

pub use crate::{origin::*, pallet::*, traits::SuccessfulProofVerifier};

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;

	use frame_support::{dispatch::Dispatchable, pallet_prelude::*, traits::Contains, Twox64Concat};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::{FullCodec, MaxEncodedLen};
	use scale_info::TypeInfo;
	use sp_std::boxed::Box;

	use crate::traits::IdentityProofVerifier;

	pub type IdentityProofOf<T> = <<T as Config>::ProofVerifier as IdentityProofVerifier<T>>::Proof;
	pub type RuntimeCallOf<T> = <T as Config>::RuntimeCall;
	pub type VerificationResultOf<T> = <<T as Config>::ProofVerifier as IdentityProofVerifier<T>>::VerificationResult;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::storage]
	#[pallet::getter(fn identity_proofs)]
	pub(crate) type IdentityEntries<T> =
		StorageMap<_, Twox64Concat, <T as Config>::Identifier, <T as Config>::LocalIdentityInfo>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Preliminary filter to filter out calls before doing any heavier
		/// computations.
		type DipCallOriginFilter: Contains<RuntimeCallOf<Self>>;
		/// The origin check for the `dispatch_as` call.
		type DispatchOriginCheck: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin, Success = Self::AccountId>;
		/// The identifier of a subject, e.g., a DID.
		type Identifier: Parameter + MaxEncodedLen;
		/// The details stored in this pallet associated with any given subject.
		type LocalIdentityInfo: FullCodec + TypeInfo + MaxEncodedLen;
		/// The logic of the proof verifier, called upon each execution of the
		/// `dispatch_as` extrinsic.
		type ProofVerifier: IdentityProofVerifier<Self>;
		type RuntimeCall: Parameter + Dispatchable<RuntimeOrigin = <Self as Config>::RuntimeOrigin>;
		type RuntimeOrigin: From<Origin<Self>> + From<<Self as frame_system::Config>::RuntimeOrigin>;
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		/// The identity proof provided could not be successfully verified.
		InvalidProof(u16),
		/// The specified call is filtered by the DIP call origin filter.
		Filtered,
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
			proof: IdentityProofOf<T>,
			call: Box<RuntimeCallOf<T>>,
		) -> DispatchResult {
			let submitter = T::DispatchOriginCheck::ensure_origin(origin)?;
			ensure!(T::DipCallOriginFilter::contains(&*call), Error::<T>::Filtered);
			let mut identity_entry = IdentityEntries::<T>::get(&identifier);
			let proof_verification_result = T::ProofVerifier::verify_proof_for_call_against_details(
				&*call,
				&identifier,
				&submitter,
				&mut identity_entry,
				proof,
			)
			.map_err(|e| Error::<T>::InvalidProof(e.into()))?;
			IdentityEntries::<T>::mutate(&identifier, |entry| *entry = identity_entry);
			let did_origin = DipOrigin {
				identifier,
				account_address: submitter,
				details: proof_verification_result,
			};
			// TODO: Use dispatch info for weight calculation
			let _ = call.dispatch(did_origin.into()).map_err(|e| e.error)?;
			Ok(())
		}
	}
}
