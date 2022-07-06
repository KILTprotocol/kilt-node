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
//! Provides means of issuing public KILT credentials on chain and revoking
//! them.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod credentials;
pub mod default_weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use crate::{credentials::*, default_weights::WeightInfo, pallet::*};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::{
		pallet_prelude::*,
		sp_runtime::traits::{CheckedConversion, Saturating},
		traits::{Currency, IsType, ReservableCurrency, StorageVersion},
		Parameter,
	};
	use frame_system::pallet_prelude::*;
	use sp_core::H256;
	use sp_std::{boxed::Box, vec::Vec};

	use attestation::{AttesterOf, ClaimHashOf};
	use ctype::CtypeHashOf;
	use kilt_support::{
		signature::{SignatureVerificationError, VerifySignature},
		traits::CallSources,
	};

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	pub(crate) type AccountIdOf<T> = attestation::AccountIdOf<T>;
	pub(crate) type BalanceOf<T> = <<T as attestation::Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	pub(crate) type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
	// No easy way to check whether the two currencies are the same and check for
	// `can_withdraw` conditions. Maybe with #[transactional] we could stop caring
	// and simply rollback if the two are the same and there is not enough
	// for both operations.
	pub(crate) type CurrencyOf<T> = <T as attestation::Config>::Currency;
	pub(crate) type SubjectIdOf<T> = <T as Config>::SubjectId;

	pub type CredentialOf<T> = Credential<
		CtypeHashOf<T>,
		BoundedVec<u8, <T as Config>::MaxSubjectIdLength>,
		Vec<u8>,
		ClaimHashOf<T>,
		H256,
		ClaimerSignatureInfo<<T as Config>::ClaimerIdentifier, <T as Config>::ClaimerSignature>,
		<T as attestation::Config>::AccessControl,
	>;

	#[pallet::config]
	pub trait Config: frame_system::Config + ctype::Config + attestation::Config {
		type ClaimerIdentifier: Parameter;
		type ClaimerSignature: Parameter;
		type ClaimerSignatureVerification: VerifySignature<
			SignerId = Self::ClaimerIdentifier,
			Payload = Vec<u8>,
			Signature = Self::ClaimerSignature,
		>;
		#[pallet::constant]
		type Deposit: Get<BalanceOf<Self>>;
		type EnsureOrigin: EnsureOrigin<
			Success = <Self as Config>::OriginSuccess,
			<Self as frame_system::Config>::Origin,
		>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type InputError: Into<DispatchError>;
		#[pallet::constant]
		type MaxEncodedCredentialLength: Get<u32>;
		#[pallet::constant]
		type MaxSubjectIdLength: Get<u32>;
		type OriginSuccess: CallSources<AccountIdOf<Self>, AttesterOf<Self>>;
		type SubjectId: Parameter
			+ MaxEncodedLen
			+ TryFrom<BoundedVec<u8, Self::MaxSubjectIdLength>, Error = Self::InputError>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::storage]
	#[pallet::getter(fn get_credential_info)]
	pub type Credentials<T> =
		StorageDoubleMap<_, Twox64Concat, SubjectIdOf<T>, Blake2_128Concat, ClaimHashOf<T>, CredentialEntryOf<T>>;

	// Reverse map to make sure that the same claim hash cannot be issued to two
	// different subjects by issuing it to subject #1, then removing it only from
	// the attestation pallet and then issuing it to subject #2.
	// This map ensures that at any time a claim hash is only linked (i.e., issued)
	// to a single subject.
	// Not exposed to the outside world.
	#[pallet::storage]
	#[pallet::getter(fn attested_claim_hashes)]
	pub(crate) type CredentialsUnicityIndex<T> = StorageMap<_, Blake2_128Concat, ClaimHashOf<T>, SubjectIdOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CredentialStored {
			subject_id: SubjectIdOf<T>,
			claim_hash: ClaimHashOf<T>,
			block_number: BlockNumberFor<T>,
		},
		CredentialRemoved {
			subject_id: SubjectIdOf<T>,
			claim_hash: ClaimHashOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		CredentialAlreadyIssued,
		CredentialNotFound,
		UnableToPayFees,
		ClaimerInfoNotFound,
		InvalidClaimerSignature,
		InvalidInput,
		CredentialTooLong,
		InternalError,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[allow(clippy::boxed_local)]
		#[pallet::weight(0)]
		pub fn add(origin: OriginFor<T>, credential: Box<CredentialOf<T>>) -> DispatchResult {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;

			let encoded_credential_len: u32 = credential
				.encode()
				.len()
				.checked_into()
				.ok_or(Error::<T>::CredentialTooLong)?;
			ensure!(
				encoded_credential_len <= T::MaxEncodedCredentialLength::get(),
				Error::<T>::CredentialTooLong
			);

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

			// Try to decode subject ID to something usable
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
			// `NamedReservableCurrency` to do that.
			ensure!(
				<CurrencyOf<T> as ReservableCurrency<AccountIdOf<T>>>::can_reserve(
					&payer,
					deposit_amount.saturating_add(attestation_deposit_amount)
				),
				Error::<T>::UnableToPayFees
			);

			// Check the validity of the claimer's signature, if present.
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
			let deposit = kilt_support::reserve_deposit::<AccountIdOf<T>, CurrencyOf<T>>(payer, deposit_amount)
				.map_err(|_| Error::<T>::InternalError)?;

			let block_number = frame_system::Pallet::<T>::block_number();

			Credentials::<T>::insert(&subject, &claim_hash, CredentialEntryOf { deposit, block_number });
			CredentialsUnicityIndex::<T>::insert(&claim_hash, subject.clone());

			Self::deposit_event(Event::CredentialStored {
				subject_id: subject,
				claim_hash,
				block_number,
			});

			Ok(())
		}

		#[pallet::weight(0)]
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
			// This guarantees that the authorized owner is calling this function
			let result = attestation::Pallet::<T>::remove_attestation(attester, claim_hash, authorization)?;

			Self::remove_credential_entry(credential_subject, claim_hash, credential_entry);

			// TODO: return the actual fee used.
			Ok(result)
		}

		#[pallet::weight(0)]
		pub fn reclaim_deposit(origin: OriginFor<T>, claim_hash: ClaimHashOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Verify that the credential exists
			let credential_subject =
				CredentialsUnicityIndex::<T>::get(&claim_hash).ok_or(Error::<T>::CredentialNotFound)?;
			// Should never happen if the line above succeeds
			let credential_entry =
				Credentials::<T>::get(&credential_subject, &claim_hash).ok_or(Error::<T>::InternalError)?;

			// Delegate to the attestation pallet the removal logic.
			// This guarantees that the authorized owner is calling this function.
			attestation::Pallet::<T>::unlock_deposit(who, claim_hash)?;

			Self::remove_credential_entry(credential_subject, claim_hash, credential_entry);

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn remove_credential_entry(
			credential_subject: SubjectIdOf<T>,
			claim_hash: ClaimHashOf<T>,
			credential: CredentialEntryOf<T>,
		) {
			kilt_support::free_deposit::<AccountIdOf<T>, CurrencyOf<T>>(&credential.deposit);
			Credentials::<T>::remove(&credential_subject, &claim_hash);
			CredentialsUnicityIndex::<T>::remove(&claim_hash);

			Self::deposit_event(Event::CredentialRemoved {
				subject_id: credential_subject,
				claim_hash,
			});
		}
	}
}
