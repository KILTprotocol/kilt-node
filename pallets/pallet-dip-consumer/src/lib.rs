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

#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

pub mod traits;

mod default_weights;

#[cfg(test)]
pub mod mock;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

mod origin;

pub use crate::{default_weights::WeightInfo, origin::*, pallet::*, traits::SuccessfulProofVerifier};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::{
		dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
		pallet_prelude::*,
		traits::{Contains, EnsureOriginWithArg},
		Twox64Concat,
	};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::{FullCodec, MaxEncodedLen};
	use scale_info::TypeInfo;
	use sp_std::boxed::Box;

	use crate::traits::IdentityProofVerifier;

	pub type IdentityProofOf<T> = <<T as Config>::ProofVerifier as IdentityProofVerifier<T>>::Proof;
	pub type RuntimeCallOf<T> = <T as Config>::RuntimeCall;
	pub type VerificationResultOf<T> = <<T as Config>::ProofVerifier as IdentityProofVerifier<T>>::VerificationResult;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// A preliminary filter that checks whether a provided `Call` accepts a
		/// DIP origin or not. If a call such as a system call does not accept a
		/// DIP origin, there is no need to verify the identity proof, hence the
		/// execution can bail out early. This does not guarantee that the
		/// dispatch call will succeed, but rather than it will mostly not fail
		/// with a `BadOrigin` error.
		type DipCallOriginFilter: Contains<RuntimeCallOf<Self>>;
		/// The origin check on the `dispatch_as` extrinsic to verify that the
		/// caller is authorized to call the extrinsic. If successful, the check
		/// must return a `AccountId` as defined by the consumer runtime.
		type DispatchOriginCheck: EnsureOriginWithArg<
			<Self as frame_system::Config>::RuntimeOrigin,
			Self::Identifier,
			Success = Self::AccountId,
		>;
		/// The type of a subject identifier. This must match the definition of
		/// `Identifier` the identity provider has defined in their deployment
		/// of the provider pallet.
		type Identifier: Parameter + MaxEncodedLen;
		/// Any additional information that must be available only to the
		/// provider runtime that is required to provide additional context when
		/// verifying a cross-chain identity proof.
		type LocalIdentityInfo: FullCodec + TypeInfo + MaxEncodedLen;
		/// The core component of this pallet. It takes care of validating an
		/// identity proof and optionally update any `LocalIdentityInfo`. It
		/// also defines, via its associated type, the structure of the identity
		/// proof that must be passed to the `dispatch_as` extrinsic. Although
		/// not directly, the proof structure depends on the information that
		/// goes into the identity commitment on the provider chain, as that
		/// defines what information can be revealed as part of the commitment
		/// proof. Additional info to satisfy requirements according to the
		/// `LocalIdentityInfo` (e.g., a signature) must also be provided in the
		/// proof.
		type ProofVerifier: IdentityProofVerifier<Self>;
		/// The aggregated `Call` type.
		type RuntimeCall: Parameter
			+ Dispatchable<PostInfo = PostDispatchInfo, RuntimeOrigin = <Self as Config>::RuntimeOrigin>
			+ GetDispatchInfo;
		/// The aggregated `Origin` type, which must include the origin exposed
		/// by this pallet.
		type RuntimeOrigin: From<Origin<Self>> + From<<Self as frame_system::Config>::RuntimeOrigin>;
		type WeightInfo: WeightInfo;
	}

	/// The pallet contains a single storage element, the `IdentityEntries` map.
	/// It maps from a subject `Identifier` to an instance of
	/// `LocalIdentityInfo`.
	#[pallet::storage]
	#[pallet::getter(fn identity_proofs)]
	pub(crate) type IdentityEntries<T> =
		StorageMap<_, Twox64Concat, <T as Config>::Identifier, <T as Config>::LocalIdentityInfo>;

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

	/// The origin is created after the identity proof has been successfully
	/// verified by the proof verifier, and it includes the identifier of the
	/// subject, the address of the tx submitter, and the result returned by the
	/// proof verifier upon successful verification.
	#[pallet::origin]
	pub type Origin<T> =
		DipOrigin<<T as Config>::Identifier, <T as frame_system::Config>::AccountId, VerificationResultOf<T>>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Try to dispatch a new local call only if it passes all the DIP
		/// requirements. Specifically, the call will be dispatched if it passes
		/// the preliminary `DipCallOriginFilter` and if the proof verifier
		/// returns a `Ok(verification_result)` value. The value is then added
		/// to the `DipOrigin` and passed down as the origin for the specified
		/// `Call`. If the whole execution terminates successfully, any changes
		/// applied to the `LocalIdentityInfo` by the proof verifier are
		/// persisted to the pallet storage.
		#[pallet::call_index(0)]
		#[pallet::weight({
			let extrinsic_weight = <T as Config>::WeightInfo::dispatch_as();
			let call_weight = call.get_dispatch_info().weight;
			extrinsic_weight.saturating_add(call_weight)
		})]
		pub fn dispatch_as(
			origin: OriginFor<T>,
			identifier: T::Identifier,
			proof: IdentityProofOf<T>,
			call: Box<RuntimeCallOf<T>>,
		) -> DispatchResultWithPostInfo {
			let submitter = T::DispatchOriginCheck::ensure_origin(origin, &identifier)?;
			ensure!(T::DipCallOriginFilter::contains(&*call), Error::<T>::Filtered);
			let proof_verification_result = IdentityEntries::<T>::try_mutate(&identifier, |identity_entry| {
				T::ProofVerifier::verify_proof_for_call_against_details(
					&*call,
					&identifier,
					&submitter,
					identity_entry,
					proof,
				)
				.map_err(|e| Error::<T>::InvalidProof(e.into()))
			})?;
			let did_origin: DipOrigin<
				T::Identifier,
				T::AccountId,
				<T::ProofVerifier as IdentityProofVerifier<T>>::VerificationResult,
			> = DipOrigin {
				identifier,
				account_address: submitter,
				details: proof_verification_result,
			};

			// TODO: Maybe find a nicer way to exclude the call dispatched from the
			// benchmarks while making sure the call is actually dispatched and passes any
			// filters the consumer proof verifier has set.
			cfg_if::cfg_if! {
				if #[cfg(not(feature = "runtime-benchmark"))] {
					call.dispatch(did_origin.into())
				} else {
					().into()
				}
			}
		}
	}
}
