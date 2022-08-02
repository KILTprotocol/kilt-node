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
		traits::{Currency, IsType, ReservableCurrency, StorageVersion},
		Parameter,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{Hash, SaturatedConversion};
	use sp_std::vec::Vec;

	pub use ctype::CtypeHashOf;
	use kilt_support::traits::CallSources;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	// No easy way to check whether the two currencies are the same and check for
	// `can_withdraw` conditions. Maybe with #[transactional] we could stop caring
	// and simply rollback if the two are the same and there is not enough
	// for both operations.
	/// The type of currency to use to reserve the required deposits.
	pub(crate) type CurrencyOf<T> = <T as Config>::Currency;

	/// The type of the credential subject input. It is bound in max length.
	/// It is transformed inside the `add` operation into a [<T as
	/// Config>::SubjectId].
	pub type InputSubjectIdOf<T> = BoundedVec<u8, <T as Config>::MaxSubjectIdLength>;
	/// The type of the credential subject input. It is bound in max length.
	pub type InputClaimsContentOf<T> = BoundedVec<u8, <T as Config>::MaxEncodedClaimsLength>;
	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
	pub type CredentialEntryOf<T> = CredentialEntry<CtypeHashOf<T>, AttesterOf<T>, BlockNumberOf<T>, AccountIdOf<T>, BalanceOf<T>>;
	/// Type of an attester identifier.
	pub type AttesterOf<T> = <T as Config>::AttesterId;
	/// The type of account's balances.
	pub type BalanceOf<T> = <CurrencyOf<T> as Currency<AccountIdOf<T>>>::Balance;
	pub(crate) type CredentialIdOf<T> = <<T as Config>::CredentialHash as sp_runtime::traits::Hash>::Output;

	/// The type of a public credential as the pallet expects it.
	pub type InputCredentialOf<T> = Credential<CtypeHashOf<T>, InputSubjectIdOf<T>, InputClaimsContentOf<T>>;

	#[pallet::config]
	pub trait Config: frame_system::Config + ctype::Config {
		/// The identifier of the credential attester.
		type AttesterId: Parameter + MaxEncodedLen;
		/// The origin allowed to issue/revoke/remove public credentials.
		type EnsureOrigin: EnsureOrigin<Success = <Self as Config>::OriginSuccess, Self::Origin>;
		/// The ubiquitous event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The hashing algorithm to derive a credential identifier from the credential content.
		type CredentialHash: Hash<Output = Self::CredentialId>;
		/// The type of a credential identifier.
		type CredentialId: Parameter + MaxEncodedLen;
		/// The currency that is used to reserve funds for each attestation.
		type Currency: ReservableCurrency<AccountIdOf<Self>>;
		/// The type of the origin when successfully converted from the outer
		/// origin.
		type OriginSuccess: CallSources<Self::AccountId, AttesterOf<Self>>;
		/// The type of the credential subject ID after being parsed from the
		/// raw attester-provided input.
		// Vec<u8> instead of BoundedVec<u8, ...> because otherwise it becomes
		// impossible for runtime-common to be independent of the `T: Config`
		// constraint.
		type SubjectId: Parameter + MaxEncodedLen + TryFrom<Vec<u8>>;
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
	/// It maps from a (subject id + credential id) -> the creation
	/// details of the credential.
	#[pallet::storage]
	#[pallet::getter(fn get_credential_info)]
	pub type Credentials<T> = StorageDoubleMap<
		_,
		Twox64Concat,
		<T as Config>::SubjectId,
		Blake2_128Concat,
		CredentialIdOf<T>,
		CredentialEntryOf<T>,
	>;

	// This map ensures that at any time a credential is only linked (i.e., issued)
	// to a single subject, as it maps from a credential ID to the subject it was issued to.
	// Not exposed to the outside world.
	#[pallet::storage]
	#[pallet::getter(fn get_credential_subject)]
	pub(crate) type CredentialSubjects<T> =
		StorageMap<_, Blake2_128Concat, CredentialIdOf<T>, <T as Config>::SubjectId>;

	/// The events generated by this pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new public credential has been issued.
		CredentialStored {
			/// The subject of the new credential.
			subject_id: T::SubjectId,
			/// The id of the new credential.
			credential_id: CredentialIdOf<T>,
		},
		/// A public credentials has been removed.
		CredentialRemoved {
			/// The subject of the removed credential.
			subject_id: T::SubjectId,
			/// The id of the removed credential.
			credential_id: CredentialIdOf<T>,
		},
		/// A public credentials has been revoked.
		CredentialRevoked {
			/// The id of the revoked credential.
			credential_id: CredentialIdOf<T>,
		},
		/// A public credentials has been unrevoked.
		CredentialUnrevoked {
			/// The id of the unrevoked credential.
			credential_id: CredentialIdOf<T>,
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
		/// The credential input is invalid.
		InvalidInput,
		/// The caller is not authorized to performed the operation.
		Unauthorized,
		/// Catch-all for any other errors that should not happen, yet it
		/// happened.
		InternalError,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a new public credential on chain.
		///
		/// This function fails if a credential with the same identifier already exists for the specified
		/// subject.
		///
		/// Emits `CredentialStored`.
		#[allow(clippy::boxed_local)]
		#[pallet::weight(<T as Config>::WeightInfo::add(credential.claims.len().saturated_into::<u32>()))]
		pub fn add(origin: OriginFor<T>, credential: InputCredentialOf<T>) -> DispatchResult {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;

			let attester = source.subject();
			let payer = source.sender();

			let deposit_amount = <T as Config>::Deposit::get();

			let Credential {
				ctype_hash,
				subject,
				claims: _,
			} = credential.clone();

			ensure!(
				ctype::Ctypes::<T>::contains_key(&ctype_hash),
				ctype::Error::<T>::CTypeNotFound
			);

			// Credential ID = H(<encoded_credential_input> || <attester_identifier>)
			let credential_id =
				T::CredentialHash::hash(&[&credential.encode()[..], &attester.encode()[..]].concat()[..]);

			// Try to decode subject ID to something structured
			let subject = T::SubjectId::try_from(subject.into_inner()).map_err(|_| Error::<T>::InvalidInput)?;

			ensure!(
				!Credentials::<T>::contains_key(&subject, &credential_id),
				Error::<T>::CredentialAlreadyIssued
			);

			// Take the rest of the deposit. Should never fail since we made sure above that
			// enough funds can be reserved.
			let deposit = kilt_support::reserve_deposit::<T::AccountId, CurrencyOf<T>>(payer, deposit_amount)
				.map_err(|_| Error::<T>::UnableToPayFees)?;

			// *** No Fail beyond this point ***

			let block_number = frame_system::Pallet::<T>::block_number();

			Credentials::<T>::insert(
				&subject,
				&credential_id,
				CredentialEntryOf::<T> {
					revoked: false,
					attester,
					deposit,
					block_number,
					ctype_hash,
				},
			);
			CredentialSubjects::<T>::insert(&credential_id, subject.clone());

			Self::deposit_event(Event::CredentialStored {
				subject_id: subject,
				credential_id,
			});

			Ok(())
		}

		/// Revokes a public credential.
		///
		/// If a credential was already revoked, this function does not fail but simply results
		/// in a noop.
		///
		/// Only the original attester can revoke the credential.
		///
		/// Emits `CredentialRevoked`.
		#[pallet::weight(0)]
		pub fn revoke(origin: OriginFor<T>, credential_id: CredentialIdOf<T>) -> DispatchResult {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;
			let caller = source.subject();

			let credential_subject =
				CredentialSubjects::<T>::get(&credential_id).ok_or(Error::<T>::CredentialNotFound)?;

			// Fails if the credential does not exist OR the caller is different than the original attester.
			Credentials::<T>::try_mutate(&credential_subject, &credential_id, |credential_entry| {
				if let Some(credential) = credential_entry {
					if caller != credential.attester {
						Err(DispatchError::from(Error::<T>::Unauthorized))
					} else {
						credential.revoked = true;
						Ok(())
					}
				} else {
					Err(DispatchError::from(Error::<T>::CredentialNotFound))
				}
			})?;

			Self::deposit_event(Event::CredentialRevoked { credential_id });

			Ok(())
		}

		/// Unrevokes a public credential.
		///
		/// If a credential was not revoked, this function does not fail but simply results
		/// in a noop.
		///
		/// Only the original attester can unrevoke the credential.
		///
		/// Emits `CredentialUnrevoked`.
		#[pallet::weight(0)]
		pub fn unrevoke(origin: OriginFor<T>, credential_id: CredentialIdOf<T>) -> DispatchResult {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;
			let caller = source.subject();

			let credential_subject =
				CredentialSubjects::<T>::get(&credential_id).ok_or(Error::<T>::CredentialNotFound)?;

			// Fails if the credential does not exist OR the caller is different than the original attester.
			Credentials::<T>::try_mutate(&credential_subject, &credential_id, |credential_entry| {
				if let Some(credential) = credential_entry {
					if caller != credential.attester {
						Err(DispatchError::from(Error::<T>::Unauthorized))
					} else {
						credential.revoked = false;
						Ok(())
					}
				} else {
					Err(DispatchError::from(Error::<T>::CredentialNotFound))
				}
			})?;

			Self::deposit_event(Event::CredentialUnrevoked { credential_id });

			Ok(())
		}

		/// Removes the information pertaining a public credential from the
		/// chain.
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
		/// subject.
		///
		/// Emits `CredentialRemoved`.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::remove())]
		pub fn remove(origin: OriginFor<T>, credential_id: CredentialIdOf<T>) -> DispatchResult {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;
			let caller = source.subject();

			let (credential_subject, credential_entry) = Self::retrieve_credential_entry(&credential_id)?;

			ensure!(
				caller == credential_entry.attester,
				Error::<T>::Unauthorized
			);

			Self::remove_credential_entry(credential_subject, credential_id, credential_entry);

			Ok(())
		}

		/// Performs the same function as the `remove` extrinsic, with the
		/// difference that the caller of this function must be the original
		/// payer of the deposit, and not the original attester of the
		/// credential.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::reclaim_deposit())]
		pub fn reclaim_deposit(origin: OriginFor<T>, credential_id: CredentialIdOf<T>) -> DispatchResult {
			let submitter = ensure_signed(origin)?;

			let (credential_subject, credential_entry) = Self::retrieve_credential_entry(&credential_id)?;

			ensure!(
				submitter == credential_entry.deposit.owner,
				Error::<T>::Unauthorized
			);

			Self::remove_credential_entry(credential_subject, credential_id, credential_entry);

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn retrieve_credential_entry(
			credential_id: &CredentialIdOf<T>,
		) -> Result<(T::SubjectId, CredentialEntryOf<T>), Error<T>> {
			// Verify that the credential exists
			let credential_subject =
				CredentialSubjects::<T>::get(&credential_id).ok_or(Error::<T>::CredentialNotFound)?;

			// Should never happen if the line above succeeds
			Credentials::<T>::get(&credential_subject, &credential_id)
				.map(|entry| (credential_subject, entry))
				.ok_or(Error::<T>::InternalError)
		}

		// Simple wrapper to remove entries from both storages when deleting a
		// credential.
		fn remove_credential_entry(
			credential_subject: T::SubjectId,
			credential_id: CredentialIdOf<T>,
			credential: CredentialEntryOf<T>,
		) {
			kilt_support::free_deposit::<T::AccountId, CurrencyOf<T>>(&credential.deposit);
			Credentials::<T>::remove(&credential_subject, &credential_id);
			CredentialSubjects::<T>::remove(&credential_id);

			Self::deposit_event(Event::CredentialRemoved {
				subject_id: credential_subject,
				credential_id,
			});
		}
	}
}
