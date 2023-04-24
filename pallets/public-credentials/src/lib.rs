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

//! # Public credentials Pallet
//!
//! Provides means of issuing public KILT credentials to subjects.
//!
//! ### Terminology
//!
//! - **Attester:**: An entity which issues a set of claims (i.e., a credential)
//!   to a subject. This could be an NFT Marketplace which issues authenticity
//!   certificates to NFT collections.
//!
//! - **Subject:**: The subject of a credential, i.e., the entity which the
//!   claims in the credential refer to.
#![cfg_attr(not(feature = "std"), no_std)]

mod access_control;
pub mod credentials;
pub mod default_weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod mock;
#[cfg(test)]
mod tests;

pub use crate::{
	access_control::AccessControl as PublicCredentialsAccessControl, credentials::*, default_weights::WeightInfo,
	pallet::*,
};

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
	use sp_std::{boxed::Box, vec::Vec};

	pub use ctype::CtypeHashOf;
	use kilt_support::{
		deposit::Deposit,
		traits::{CallSources, StorageDepositCollector},
	};

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
	pub type CredentialEntryOf<T> = CredentialEntry<
		CtypeHashOf<T>,
		AttesterOf<T>,
		BlockNumberOf<T>,
		AccountIdOf<T>,
		BalanceOf<T>,
		AuthorizationIdOf<T>,
	>;
	/// Type of an attester identifier.
	pub type AttesterOf<T> = <T as Config>::AttesterId;
	/// The type of account's balances.
	pub type BalanceOf<T> = <CurrencyOf<T> as Currency<AccountIdOf<T>>>::Balance;
	pub(crate) type AuthorizationIdOf<T> = <T as Config>::AuthorizationId;
	pub type CredentialIdOf<T> = <<T as Config>::CredentialHash as sp_runtime::traits::Hash>::Output;

	/// The type of a public credential as the pallet expects it.
	pub type InputCredentialOf<T> =
		Credential<CtypeHashOf<T>, InputSubjectIdOf<T>, InputClaimsContentOf<T>, <T as Config>::AccessControl>;

	#[pallet::config]
	pub trait Config: frame_system::Config + ctype::Config {
		/// The access control logic.
		type AccessControl: Parameter
			+ PublicCredentialsAccessControl<
				Self::AttesterId,
				Self::AuthorizationId,
				CtypeHashOf<Self>,
				CredentialIdOf<Self>,
			>;
		/// The identifier of the credential attester.
		type AttesterId: Parameter + MaxEncodedLen;
		/// The identifier of the authorization info to perform access control
		/// for the different operations.
		type AuthorizationId: Parameter + MaxEncodedLen;
		/// The origin allowed to issue/revoke/remove public credentials.
		type EnsureOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = <Self as Config>::OriginSuccess>;
		/// The ubiquitous event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The hashing algorithm to derive a credential identifier from the
		/// credential content.
		type CredentialHash: Hash<Output = Self::CredentialId>;
		/// The type of a credential identifier.
		type CredentialId: Parameter + MaxEncodedLen;
		/// The currency that is used to reserve funds for each credential.
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

	/// A reverse index mapping from credential ID to the subject the credential
	/// was issued to.
	///
	/// It it used to perform efficient lookup of credentials given their ID.
	#[pallet::storage]
	#[pallet::getter(fn get_credential_subject)]
	pub type CredentialSubjects<T> = StorageMap<_, Blake2_128Concat, CredentialIdOf<T>, <T as Config>::SubjectId>;

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
		/// A public credential has been revoked.
		CredentialRevoked {
			/// The id of the revoked credential.
			credential_id: CredentialIdOf<T>,
		},
		/// A public credential has been unrevoked.
		CredentialUnrevoked {
			/// The id of the unrevoked credential.
			credential_id: CredentialIdOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// A credential with the same root hash has already issued to the
		/// specified subject.
		AlreadyAttested,
		/// No credential with the specified root hash has been issued to the
		/// specified subject.
		NotFound,
		/// Not enough tokens to pay for the fees or the deposit.
		UnableToPayFees,
		/// The credential input is invalid.
		InvalidInput,
		/// The caller is not authorized to performed the operation.
		NotAuthorized,
		/// Catch-all for any other errors that should not happen, yet it
		/// happened.
		Internal,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a new public credential on chain.
		///
		/// This function fails if a credential with the same identifier already
		/// exists for the specified subject.
		///
		/// Emits `CredentialStored`.
		#[allow(clippy::boxed_local)]
		#[pallet::call_index(0)]
		#[pallet::weight({
			let xt_weight = <T as Config>::WeightInfo::add(credential.claims.len().saturated_into::<u32>());
			let ac_weight = credential.authorization.as_ref().map(|ac| ac.can_issue_weight()).unwrap_or(Weight::zero());
			xt_weight.saturating_add(ac_weight)
		})]
		pub fn add(origin: OriginFor<T>, credential: Box<InputCredentialOf<T>>) -> DispatchResultWithPostInfo {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;

			let attester = source.subject();
			let payer = source.sender();

			let deposit_amount = <T as Config>::Deposit::get();

			let Credential {
				ctype_hash,
				subject,
				claims: _,
				authorization,
			} = *credential.clone();

			ensure!(
				ctype::Ctypes::<T>::contains_key(ctype_hash),
				ctype::Error::<T>::NotFound
			);

			// Credential ID = H(<scale_encoded_credential_input> ||
			// <scale_encoded_attester_identifier>)
			let credential_id =
				T::CredentialHash::hash(&[&credential.encode()[..], &attester.encode()[..]].concat()[..]);

			// Check for validity of the authorization info if specified.
			let ac_weight = authorization
				.as_ref()
				.map(|ac| ac.can_issue(&attester, &ctype_hash, &credential_id))
				.transpose()
				.map_err(|_| Error::<T>::NotAuthorized)?;
			let authorization_id = authorization.as_ref().map(|ac| ac.authorization_id());

			// Try to decode subject ID to something structured
			let subject = T::SubjectId::try_from(subject.into_inner()).map_err(|_| Error::<T>::InvalidInput)?;

			ensure!(
				!Credentials::<T>::contains_key(&subject, &credential_id),
				Error::<T>::AlreadyAttested
			);

			let deposit = kilt_support::reserve_deposit::<T::AccountId, CurrencyOf<T>>(payer, deposit_amount)
				.map_err(|_| Error::<T>::UnableToPayFees)?;

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
					authorization_id,
				},
			);
			CredentialSubjects::<T>::insert(&credential_id, subject.clone());

			Self::deposit_event(Event::CredentialStored {
				subject_id: subject,
				credential_id,
			});

			Ok(Some(
				<T as Config>::WeightInfo::add(credential.claims.len().saturated_into::<u32>())
					.saturating_add(ac_weight.unwrap_or(Weight::zero())),
			)
			.into())
		}

		/// Revokes a public credential.
		///
		/// If a credential was already revoked, this function does not fail but
		/// simply results in a noop.
		///
		/// The dispatch origin must be authorized to revoke the credential.
		///
		/// Emits `CredentialRevoked`.
		#[pallet::call_index(1)]
		#[pallet::weight({
			let xt_weight = <T as Config>::WeightInfo::revoke();
			let ac_weight = authorization.as_ref().map(|ac| ac.can_revoke_weight()).unwrap_or(Weight::zero());
			xt_weight.saturating_add(ac_weight)
		})]
		pub fn revoke(
			origin: OriginFor<T>,
			credential_id: CredentialIdOf<T>,
			authorization: Option<T::AccessControl>,
		) -> DispatchResultWithPostInfo {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;
			let caller = source.subject();

			let credential_subject = CredentialSubjects::<T>::get(&credential_id).ok_or(Error::<T>::NotFound)?;

			let ac_weight_used = Self::set_credential_revocation_status(
				&caller,
				&credential_subject,
				&credential_id,
				authorization,
				true,
			)?;

			Self::deposit_event(Event::CredentialRevoked { credential_id });

			Ok(Some(<T as Config>::WeightInfo::revoke().saturating_add(ac_weight_used)).into())
		}

		/// Unrevokes a public credential.
		///
		/// If a credential was not revoked, this function does not fail but
		/// simply results in a noop.
		///
		/// The dispatch origin must be authorized to unrevoke the
		/// credential.
		///
		/// Emits `CredentialUnrevoked`.
		#[pallet::call_index(2)]
		#[pallet::weight({
			let xt_weight = <T as Config>::WeightInfo::unrevoke();
			let ac_weight = authorization.as_ref().map(|ac| ac.can_unrevoke_weight()).unwrap_or(Weight::zero());
			xt_weight.saturating_add(ac_weight)
		})]
		pub fn unrevoke(
			origin: OriginFor<T>,
			credential_id: CredentialIdOf<T>,
			authorization: Option<T::AccessControl>,
		) -> DispatchResultWithPostInfo {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;
			let caller = source.subject();

			let credential_subject = CredentialSubjects::<T>::get(&credential_id).ok_or(Error::<T>::NotFound)?;

			let ac_weight_used = Self::set_credential_revocation_status(
				&caller,
				&credential_subject,
				&credential_id,
				authorization,
				false,
			)?;

			Self::deposit_event(Event::CredentialUnrevoked { credential_id });

			Ok(Some(<T as Config>::WeightInfo::unrevoke().saturating_add(ac_weight_used)).into())
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
		/// The dispatch origin must be authorized to remove the credential.
		///
		/// Emits `CredentialRemoved`.
		#[pallet::call_index(3)]
		#[pallet::weight({
			let xt_weight = <T as Config>::WeightInfo::remove();
			let ac_weight = authorization.as_ref().map(|ac| ac.can_remove_weight()).unwrap_or(Weight::zero());
			xt_weight.saturating_add(ac_weight)
		})]
		pub fn remove(
			origin: OriginFor<T>,
			credential_id: CredentialIdOf<T>,
			authorization: Option<T::AccessControl>,
		) -> DispatchResultWithPostInfo {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;
			let caller = source.subject();

			let (credential_subject, credential_entry) = Self::retrieve_credential_entry(&credential_id)?;

			let ac_weight_used = if credential_entry.attester == caller {
				Weight::zero()
			} else {
				let credential_auth_id = credential_entry
					.authorization_id
					.as_ref()
					.ok_or(Error::<T>::NotAuthorized)?;
				authorization
					.ok_or(Error::<T>::NotAuthorized)?
					.can_remove(
						&caller,
						&credential_entry.ctype_hash,
						&credential_id,
						credential_auth_id,
					)
					.map_err(|_| Error::<T>::NotAuthorized)?
			};

			// Removes the credential from storage and generates a `CredentialRemoved`
			// event.
			Self::remove_credential_entry(credential_subject, credential_id, credential_entry);

			Ok(Some(<T as Config>::WeightInfo::remove().saturating_add(ac_weight_used)).into())
		}

		/// Removes the information pertaining a public credential from the
		/// chain and returns the deposit to its payer.
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
		/// The dispatch origin must be the owner of the deposit, hence not the
		/// credential's attester.
		///
		/// Emits `CredentialRemoved`.
		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::reclaim_deposit())]
		pub fn reclaim_deposit(origin: OriginFor<T>, credential_id: CredentialIdOf<T>) -> DispatchResult {
			let submitter = ensure_signed(origin)?;

			let (credential_subject, credential_entry) = Self::retrieve_credential_entry(&credential_id)?;

			ensure!(submitter == credential_entry.deposit.owner, Error::<T>::NotAuthorized);

			// Removes the credential from storage and generates a `CredentialRemoved`
			// event.
			Self::remove_credential_entry(credential_subject, credential_id, credential_entry);

			Ok(())
		}

		/// Changes the deposit owner.
		///
		/// The balance that is reserved by the current deposit owner will be
		/// freed and balance of the new deposit owner will get reserved.
		///
		/// The subject of the call must be the owner of the credential.
		/// The sender of the call will be the new deposit owner.
		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::change_deposit_owner())]
		pub fn change_deposit_owner(origin: OriginFor<T>, credential_id: CredentialIdOf<T>) -> DispatchResult {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;
			let subject = source.subject();

			let (_, credential_entry) = Self::retrieve_credential_entry(&credential_id)?;

			ensure!(subject == credential_entry.attester, Error::<T>::NotAuthorized);

			PublicCredentialDepositCollector::<T>::change_deposit_owner(&credential_id, source.sender())?;

			Ok(())
		}

		/// Updates the deposit amount to the current deposit rate.
		///
		/// The sender must be the deposit owner.
		#[pallet::call_index(6)]
		#[pallet::weight(<T as Config>::WeightInfo::update_deposit())]
		pub fn update_deposit(origin: OriginFor<T>, credential_id: CredentialIdOf<T>) -> DispatchResult {
			let source = ensure_signed(origin)?;
			let (_, credential_entry) = Self::retrieve_credential_entry(&credential_id)?;

			ensure!(source == credential_entry.deposit.owner, Error::<T>::NotAuthorized);

			PublicCredentialDepositCollector::<T>::update_deposit(&credential_id)?;

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// Simple wrapper to remove entries from both storages when deleting a
		// credential and generate a `CredentialRemoved` event.
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

		fn retrieve_credential_entry(
			credential_id: &CredentialIdOf<T>,
		) -> Result<(T::SubjectId, CredentialEntryOf<T>), Error<T>> {
			// Verify that the credential exists
			let credential_subject = CredentialSubjects::<T>::get(credential_id).ok_or(Error::<T>::NotFound)?;

			// Should never happen if the line above succeeds
			Credentials::<T>::get(&credential_subject, credential_id)
				.map(|entry| (credential_subject, entry))
				.ok_or(Error::<T>::Internal)
		}

		fn set_credential_revocation_status(
			caller: &AttesterOf<T>,
			credential_subject: &T::SubjectId,
			credential_id: &CredentialIdOf<T>,
			authorization: Option<T::AccessControl>,
			revocation: bool,
		) -> Result<Weight, Error<T>> {
			// Fails if the credential does not exist OR the caller is different than the
			// original attester. If successful, saves the additional weight used for access
			// control and returns it at the end of the function.
			Credentials::<T>::try_mutate(credential_subject, credential_id, |credential_entry| {
				if let Some(credential) = credential_entry {
					// Additional weight is 0 if the caller is the attester, otherwise it's the
					// value returned by the access control check, if it does not fail.
					let additional_weight = if *caller == credential.attester {
						Weight::zero()
					} else {
						let credential_auth_id =
							credential.authorization_id.as_ref().ok_or(Error::<T>::NotAuthorized)?;
						authorization
							.ok_or(Error::<T>::NotAuthorized)?
							.can_revoke(caller, &credential.ctype_hash, credential_id, credential_auth_id)
							.map_err(|_| Error::<T>::NotAuthorized)?
					};
					// If authorization checks are ok, update the revocation status.
					credential.revoked = revocation;
					Ok(additional_weight)
				} else {
					// No weight is computed as the error is an early return.
					Err(Error::<T>::NotFound)
				}
			})
		}
	}

	struct PublicCredentialDepositCollector<T: Config>(PhantomData<T>);
	impl<T: Config> StorageDepositCollector<AccountIdOf<T>, CredentialIdOf<T>> for PublicCredentialDepositCollector<T> {
		type Currency = <T as Config>::Currency;

		fn deposit(
			credential_id: &CredentialIdOf<T>,
		) -> Result<Deposit<AccountIdOf<T>, <Self::Currency as Currency<AccountIdOf<T>>>::Balance>, DispatchError> {
			let (_, credential_entry) = Pallet::<T>::retrieve_credential_entry(credential_id)?;
			Ok(credential_entry.deposit)
		}

		fn deposit_amount(_credential_id: &CredentialIdOf<T>) -> <Self::Currency as Currency<AccountIdOf<T>>>::Balance {
			T::Deposit::get()
		}

		fn store_deposit(
			credential_id: &CredentialIdOf<T>,
			deposit: Deposit<AccountIdOf<T>, <Self::Currency as Currency<AccountIdOf<T>>>::Balance>,
		) -> Result<(), DispatchError> {
			let credential_subject = CredentialSubjects::<T>::get(credential_id).ok_or(Error::<T>::NotFound)?;
			Credentials::<T>::try_mutate(&credential_subject, credential_id, |credential_entry| {
				if let Some(credential) = credential_entry {
					credential.deposit = deposit;
					Ok(())
				} else {
					Err(Error::<T>::NotFound.into())
				}
			})
		}
	}
}
