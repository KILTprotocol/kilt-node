// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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

//! # Public credentials Pallet
//!
//! Provides means of issuing public KILT credentials to subjects, optionally on
//! request of a third-party claimer.
//!
//! ### Terminology
//!
//! - **Attester:**: An entity which issues a set of claims (i.e., a credential)
//!   to a subject. This could be an NFT Marketplace which issues authenticity
//!   certificates to NFT collections.
//!
//! - **Claimer:**: A user that requests an attester to issue a credential to a
//!   subject, i.e., an NFT collection. The request contains the signature of
//!   the claimer, proving their involvement in the issuance process.
//!
//! - **Subject:**: The subject of a credential, i.e., the entity which the
//!   claims in the credential refer to.
//!
//! ## Assumptions
//!
//! - This pallet is heavily relying on the functionalities of the `attestation`
//!   pallet for all of its functions.
//! - This pallet does not expose a `revoke` extrinsic, as revocation
//!   functionalities are entirely delegated to the `attestation` pallet.
#![cfg_attr(not(feature = "std"), no_std)]

pub mod credentials;
pub mod default_weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod mock;
#[cfg(test)]
mod tests;

pub use crate::{credentials::*, default_weights::WeightInfo, pallet::*};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::{
		pallet_prelude::*,
		sp_runtime::traits::Saturating,
		traits::{Currency, IsType, ReservableCurrency, StorageVersion},
		Parameter,
	};
	use frame_system::pallet_prelude::*;
	use sp_core::H256;
	use sp_runtime::SaturatedConversion;
	use sp_std::{boxed::Box, vec::Vec};

	use attestation::{AttestationAccessControl, AttesterOf, ClaimHashOf};
	use ctype::CtypeHashOf;
	use kilt_support::{
		signature::{SignatureVerificationError, VerifySignature},
		traits::CallSources,
	};

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	// No easy way to check whether the two currencies are the same and check for
	// `can_withdraw` conditions. Maybe with #[transactional] we could stop caring
	// and simply rollback if the two are the same and there is not enough
	// for both operations.
	/// The type of currency to use to reserve the required deposits.
	/// Must match the definition of [<T as attestation::Config>::Currency].
	pub(crate) type CurrencyOf<T> = <T as attestation::Config>::Currency;
	/// The type of account's balances.
	pub(crate) type BalanceOf<T> = <CurrencyOf<T> as Currency<attestation::AccountIdOf<T>>>::Balance;

	/// The type of the credential subject input. It is bound in max length.
	/// It is transformed inside the `add` operation into a [<T as
	/// Config>::SubjectId].
	pub type InputSubjectIdOf<T> = BoundedVec<u8, <T as Config>::MaxSubjectIdLength>;
	/// The type of the credential subject input. It is bound in max length.
	pub type InputClaimsContentOf<T> = BoundedVec<u8, <T as Config>::MaxEncodedClaimsLength>;
	pub type ClaimerSignatureOf<T> =
		ClaimerSignatureInfo<<T as Config>::ClaimerIdentifier, <T as Config>::ClaimerSignature>;

	/// The type of a public credential as the pallet expects it.
	pub type InputCredentialOf<T> = Credential<
		CtypeHashOf<T>,
		InputSubjectIdOf<T>,
		InputClaimsContentOf<T>,
		ClaimHashOf<T>,
		H256,
		ClaimerSignatureOf<T>,
		<T as attestation::Config>::AccessControl,
	>;

	#[pallet::config]
	pub trait Config: frame_system::Config + ctype::Config + attestation::Config {
		/// The identifier of the credential claimer.
		type ClaimerIdentifier: Parameter;
		/// The signature of the credential claimer.
		type ClaimerSignature: Parameter;
		/// The claimer's signature verification logic.
		type ClaimerSignatureVerification: VerifySignature<
			SignerId = Self::ClaimerIdentifier,
			Payload = Vec<u8>,
			Signature = Self::ClaimerSignature,
		>;
		/// The origin allowed to issue/revoke/remove public credentials.
		type EnsureOrigin: EnsureOrigin<Success = <Self as Config>::OriginSuccess, Self::Origin>;
		/// The ubiquitous event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The error to return when the provided credential input is not valid.
		type InputError: Into<DispatchError>;
		/// The type of the origin when successfully converted from the outer
		/// origin.
		type OriginSuccess: CallSources<Self::AccountId, AttesterOf<Self>>;
		/// The type of the credential subject ID after being parsed from the
		/// raw attester-provided input.
		type SubjectId: Parameter + MaxEncodedLen + TryFrom<InputSubjectIdOf<Self>, Error = Self::InputError>;
		/// The weight info.
		type WeightInfo: WeightInfo;

		/// The amount of tokens to reserve when attesting a public credential.
		#[pallet::constant]
		type Deposit: Get<BalanceOf<Self>>;
		/// The maximum length in bytes of the encoded claims of a credential.
		#[pallet::constant]
		type MaxEncodedClaimsLength: Get<u32>;
		/// The maximum length in bytes of the raw credential subject
		/// identifier.
		#[pallet::constant]
		type MaxSubjectIdLength: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	/// The map of public credentials already attested.
	/// It maps from a (subject id + credential root hash) -> the creation
	/// details of the credential.
	#[pallet::storage]
	#[pallet::getter(fn get_credential_info)]
	pub type Credentials<T> = StorageDoubleMap<
		_,
		Twox64Concat,
		<T as Config>::SubjectId,
		Blake2_128Concat,
		ClaimHashOf<T>,
		CredentialEntryOf<T>,
	>;

	// Reverse map to make sure that the same claim hash cannot be issued to two
	// different subjects by issuing it to subject #1, then removing it only from
	// the attestation pallet and then issuing it to subject #2.
	// This map ensures that at any time a claim hash is only linked (i.e., issued)
	// to a single subject.
	// Not exposed to the outside world.
	#[pallet::storage]
	#[pallet::getter(fn attested_claim_hashes)]
	pub(crate) type CredentialsUnicityIndex<T> =
		StorageMap<_, Blake2_128Concat, ClaimHashOf<T>, <T as Config>::SubjectId>;

	/// The events generated by this pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new public credential has been issued.
		CredentialStored {
			/// The subject of the new credential.
			subject_id: T::SubjectId,
			/// The root hash of the new credential.
			claim_hash: ClaimHashOf<T>,
		},
		/// A public credentials has been removed.
		CredentialRemoved {
			/// The subject of the removed credential.
			subject_id: T::SubjectId,
			/// The root hash of the removed credential.
			claim_hash: ClaimHashOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// A credential with the same root hash has already issued to the
		/// specified subject.
		CredentialAlreadyIssued,
		/// No credential with the specified root hash has been issued to the
		/// specified subject.
		CredentialNotFound,
		/// Not enough tokens to pay for the fees or the deposit.
		UnableToPayFees,
		/// The credential claimer's information cannot be found, hence the
		/// signature cannot be verified.
		ClaimerInfoNotFound,
		/// The credential claimer's signature is invalid.
		InvalidClaimerSignature,
		/// The credential input is invalid.
		InvalidInput,
		/// The credential exceeds the maximum configured length in bytes.
		CredentialTooLong,
		/// The subject exceeds the maximum configured length in bytes.
		SubjectTooLong,
		/// Catch-all for any other errors that should not happen, yet it
		/// happened.
		InternalError,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a new public credential on chain.
		///
		/// The preconditions of the `attestation::add()` function must all be
		/// fulfilled, as this function calls `attestation:add()` internally.
		///
		/// Furthermore, the input must meet the requirements as part of the
		/// pallet's configuration, and the attester must be able to pay for the
		/// deposit of both the underlying attestation and the public credential
		/// info.
		///
		/// This function fails if a credential already exists for the specified
		/// subject, regardless of the identity of the attester.
		///
		/// Emits `CredentialStored`.
		#[allow(clippy::boxed_local)]
		#[pallet::weight({
			let signature_weight = credential.claimer_signature.as_ref().map(|_| <T as Config>::ClaimerSignatureVerification::weight(credential.claim_hash.encoded_size())).unwrap_or(0);
			let ac_weight = credential.authorization_info.as_ref().map(|ac| ac.can_attest_weight()).unwrap_or(0);
			<T as Config>::WeightInfo::add(credential.claim.contents.len().saturated_into::<u32>()).saturating_add(signature_weight).saturating_add(ac_weight)
		})]
		pub fn add(origin: OriginFor<T>, credential: Box<InputCredentialOf<T>>) -> DispatchResult {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;

			let attester = source.subject();
			let payer = source.sender();

			let deposit_amount = <T as Config>::Deposit::get();
			let attestation_deposit_amount = <T as attestation::Config>::Deposit::get();

			let Credential {
				claim: Claim {
					ctype_hash,
					subject,
					contents: _,
				},
				claim_hash,
				nonce: _,
				claimer_signature,
				authorization_info,
			} = *credential;

			// Try to decode subject ID to something structured
			let subject = T::SubjectId::try_from(subject).map_err(|e| e.into())?;

			// Check that the same attestation has not already been issued previously
			// (potentially to a different subject)
			ensure!(
				!CredentialsUnicityIndex::<T>::contains_key(&claim_hash),
				Error::<T>::CredentialAlreadyIssued
			);

			// Check that enough funds can be reserved to pay for both attestation and
			// public info deposits.
			// It is harder to use two potentially different currencies while making sure
			// that, if the same, the sum can be reserved, but if they are not, then each
			// deposit could be reserved separately. We could switch to using
			// `NamedReservableCurrency` to do that and check whether the name of the
			// attestation currency matches the name of this pallet currency.
			ensure!(
				<CurrencyOf<T> as ReservableCurrency<T::AccountId>>::can_reserve(
					&payer,
					deposit_amount.saturating_add(attestation_deposit_amount)
				),
				Error::<T>::UnableToPayFees
			);

			// Check the validity of the claimer's signature, if present, over the
			// credential root hash.
			if let Some(ClaimerSignatureInfo {
				claimer_id,
				signature_payload,
			}) = claimer_signature
			{
				T::ClaimerSignatureVerification::verify(&claimer_id, &claim_hash.encode(), &signature_payload)
					.map_err(|err| match err {
						SignatureVerificationError::SignerInformationNotPresent => Error::<T>::ClaimerInfoNotFound,
						SignatureVerificationError::SignatureInvalid => Error::<T>::InvalidClaimerSignature,
					})?;
			}

			// Delegate to the attestation pallet writing the attestation information and
			// reserve its part of the deposit
			attestation::Pallet::<T>::write_attestation(
				ctype_hash,
				claim_hash,
				attester,
				payer.clone(),
				authorization_info,
			)?;

			// *** No Fail beyond this point ***

			// Take the rest of the deposit. Should never fail since we made sure above that
			// enough funds can be reserved.
			let deposit = kilt_support::reserve_deposit::<T::AccountId, CurrencyOf<T>>(payer, deposit_amount)
				.map_err(|_| Error::<T>::InternalError)?;

			let block_number = frame_system::Pallet::<T>::block_number();

			Credentials::<T>::insert(&subject, &claim_hash, CredentialEntryOf { deposit, block_number });
			CredentialsUnicityIndex::<T>::insert(&claim_hash, subject.clone());

			Self::deposit_event(Event::CredentialStored {
				subject_id: subject,
				claim_hash,
			});

			Ok(())
		}

		/// Removes the information pertaining a public credential from the
		/// chain.
		///
		/// The preconditions of the `attestation::remove()` function must all
		/// be fulfilled, as this function calls `attestation::remove()`
		/// internally. Nevertheless, the opposite is not true.
		/// Removing an attestation from the attestation pallet still requires
		/// the attester to also remove any traces of the corresponding public
		/// credential from this pallet.
		///
		/// The removal of the credential does not delete it entirely from the
		/// blockchain history, but only its link *from* the blockchain state
		/// *to* the blockchain history is removed.
		///
		/// Clients parsing public credentials should interpret
		/// the lack of such a link as the fact that the credential has been
		/// removed by its attester some time in the past.
		///
		/// This function fails if a credential already exists for the specified
		/// subject, regardless of the identity of the attester.
		///
		/// Emits `CredentialRemoved`.
		#[pallet::weight({
			let ac_weight = authorization.as_ref().map(|ac| ac.can_attest_weight()).unwrap_or(0);
			<T as pallet::Config>::WeightInfo::remove().saturating_add(ac_weight)
		})]
		pub fn remove(
			origin: OriginFor<T>,
			claim_hash: ClaimHashOf<T>,
			authorization: Option<T::AccessControl>,
		) -> DispatchResultWithPostInfo {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;
			let attester = source.subject();

			// Verify that the credential exists
			let credential_subject =
				CredentialsUnicityIndex::<T>::get(&claim_hash).ok_or(Error::<T>::CredentialNotFound)?;
			// Should never happen if the line above succeeds
			let credential_entry =
				Credentials::<T>::get(&credential_subject, &claim_hash).ok_or(Error::<T>::InternalError)?;

			// Delegate to the attestation pallet the removal logic
			// This guarantees that the authorized attester is calling this function
			let result = attestation::Pallet::<T>::remove_attestation(attester, claim_hash, authorization)?;

			Self::remove_credential_entry(credential_subject, claim_hash, credential_entry);

			Ok(result)
		}

		/// Performs the same function as the `remove` extrinsic, with the
		/// difference that the caller of this function must be the original
		/// payer of the deposit, and not the original attester of the
		/// credential.
		///
		/// It calls `attestation::reclaim_deposit()` internally, nevertheless
		/// the opposite is not true. Removing an attestation from the
		/// attestation pallet still requires the deposit payer to also remove
		/// any traces of the corresponding public credential from this pallet.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::reclaim_deposit())]
		pub fn reclaim_deposit(origin: OriginFor<T>, claim_hash: ClaimHashOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Verify that the credential exists
			let credential_subject =
				CredentialsUnicityIndex::<T>::get(&claim_hash).ok_or(Error::<T>::CredentialNotFound)?;
			// Should never happen if the line above succeeds
			let credential_entry =
				Credentials::<T>::get(&credential_subject, &claim_hash).ok_or(Error::<T>::InternalError)?;

			// Delegate to the attestation pallet the removal logic.
			// This guarantees that the authorized payer is calling this function.
			attestation::Pallet::<T>::unlock_deposit(who, claim_hash)?;

			Self::remove_credential_entry(credential_subject, claim_hash, credential_entry);

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// Simple wrapper to remove entries from both storages when deleting a
		// credential.
		fn remove_credential_entry(
			credential_subject: T::SubjectId,
			claim_hash: ClaimHashOf<T>,
			credential: CredentialEntryOf<T>,
		) {
			kilt_support::free_deposit::<T::AccountId, CurrencyOf<T>>(&credential.deposit);
			Credentials::<T>::remove(&credential_subject, &claim_hash);
			CredentialsUnicityIndex::<T>::remove(&claim_hash);

			Self::deposit_event(Event::CredentialRemoved {
				subject_id: credential_subject,
				claim_hash,
			});
		}
	}
}
