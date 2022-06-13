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

#[cfg(test)]
mod mock;

pub use crate::{credentials::*, default_weights::WeightInfo, pallet::*};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use codec::MaxEncodedLen;

	use frame_support::{
		pallet_prelude::*,
		sp_runtime::traits::Saturating,
		traits::{Currency, IsType, ReservableCurrency, StorageVersion},
		BoundedVec,
	};
	use frame_system::pallet_prelude::*;
	use sp_core::H256;

	use attestation::{AttesterOf, ClaimHashOf};
	use ctype::CtypeHashOf;
	use kilt_support::{deposit::Deposit, traits::CallSources};

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	// TODO: Replace with an enum that includes KILT DIDs and asset DIDs.
	pub(crate) type SubjectIdOf<T> = AccountIdOf<T>;
	pub(crate) type BalanceOf<T> = <<T as attestation::Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	pub(crate) type CurrencyOf<T> = <T as attestation::Config>::Currency;

	pub type CredentialOf<T> = Credential<
		CtypeHashOf<T>,
		SubjectIdOf<T>,
		BoundedVec<u8, <T as Config>::MaxEncodedClaimContentLength>,
		ClaimHashOf<T>,
		H256,
	>;

	#[pallet::config]
	pub trait Config: frame_system::Config + attestation::Config {
		type CredentialClaimerIdentifier: Parameter + MaxEncodedLen;
		#[pallet::constant]
		type Deposit: Get<BalanceOf<Self>>;
		type EnsureOrigin: EnsureOrigin<
			Success = <Self as Config>::OriginSuccess,
			<Self as frame_system::Config>::Origin,
		>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		#[pallet::constant]
		type MaxEncodedClaimContentLength: Get<u32>;
		type OriginSuccess: CallSources<AccountIdOf<Self>, AttesterOf<Self>>;
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
	pub type Credentials<T> = StorageDoubleMap<
		_,
		Twox64Concat,
		SubjectIdOf<T>,
		Blake2_128Concat,
		ClaimHashOf<T>,
		CredentialEntry<T>,
	>;

	// Reverse map to make sure that the same claim hash cannot be issued to two
	// different subjects by issuing it to subject #1, then removing it only from
	// the attestation pallet and then issuing it to subject #2.
	// This map ensures that at any time a claim hash is only issued to a single
	// subject.
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
		CredentialIssued,
		CredentialNotFound,
		UnableToPayFees,
		InternalError,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn add(origin: OriginFor<T>, credential: CredentialOf<T>) -> DispatchResult {
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
			} = credential;

			// Check that the same attestation has not already been issued previously
			// (potentially to a different subject)
			ensure!(
				!CredentialsUnicityIndex::<T>::contains_key(&claim_hash),
				Error::<T>::CredentialIssued
			);

			// Check that enough funds can be reserved to pay for both attestation and
			// public info deposits
			ensure!(
				<<T as attestation::Config>::Currency as ReservableCurrency<AccountIdOf<T>>>::can_reserve(
					&payer,
					deposit_amount.saturating_add(attestation_deposit_amount)
				),
				Error::<T>::UnableToPayFees
			);

			// Delegate to the attestation pallet writing the attestation information and
			// reserve its part of the deposit
			attestation::Pallet::<T>::write_attestation(ctype_hash, claim_hash, attester, payer.clone(), None)?;

			// *** No Fail beyond this point ***

			// Take the rest of the deposit. Should never fail since we made sure that enough funds can be reserved.
			let deposit = Self::reserve_deposit(payer, deposit_amount).map_err(|_| Error::<T>::InternalError)?;

			let block_number = frame_system::Pallet::<T>::block_number();

			Credentials::<T>::insert(&subject, &claim_hash, CredentialEntry { deposit, block_number });
			CredentialsUnicityIndex::<T>::insert(&claim_hash, subject.clone());

			Self::deposit_event(Event::CredentialStored {
				subject_id: subject,
				claim_hash,
				block_number,
			});

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn remove(origin: OriginFor<T>, claim_hash: ClaimHashOf<T>) -> DispatchResultWithPostInfo {
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
			let result = attestation::Pallet::<T>::remove_attestation(attester, claim_hash, None)?;

			Self::remove_credential_entry(credential_subject, claim_hash, credential_entry);

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
			attestation::Pallet::<T>::reclaim_dep(who, claim_hash)?;

			Self::remove_credential_entry(credential_subject, claim_hash, credential_entry);

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn reserve_deposit(
			payer: AccountIdOf<T>,
			deposit: BalanceOf<T>,
		) -> Result<Deposit<AccountIdOf<T>, BalanceOf<T>>, DispatchError> {
			CurrencyOf::<T>::reserve(&payer, deposit)?;

			Ok(Deposit::<AccountIdOf<T>, BalanceOf<T>> {
				owner: payer,
				amount: deposit,
			})
		}

		pub(crate) fn remove_credential_entry(
			credential_subject: SubjectIdOf<T>,
			claim_hash: ClaimHashOf<T>,
			credential: CredentialEntry<T>,
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
