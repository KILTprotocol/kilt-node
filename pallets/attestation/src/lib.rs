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

//! # Attestation Pallet
//!
//! Provides means of adding KILT attestations on chain and revoking them.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ### Terminology
//!
//! - **Claimer:**: A user which claims properties about themselves in the
//!   format of a CType. This could be a person which claims to have a valid
//!   driver's license.
//!
//! - **Attester:**: An entity which checks a user's claim and approves its
//!   validity. This could be a Citizens Registration Office which issues
//!   drivers licenses.
//!
//! - **Verifier:**: An entity which wants to check a user's claim by checking
//!   the provided attestation.
//!
//! - **CType:**: CTypes are claim types. In everyday language, they are
//!   standardised structures for credentials. For example, a company may need a
//!   standard identification credential to identify workers that includes their
//!   full name, date of birth, access level and id number. Each of these are
//!   referred to as an attribute of a credential.
//!
//! - **Attestation:**: An approved or revoked user's claim in the format of a
//!   CType.
//!
//! - **Delegation:**: An attestation which is not issued by the attester
//!   directly but via a (chain of) delegations which entitle the delegated
//!   attester. This could be an employe of a company which is authorized to
//!   sign documents for their superiors.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//! - `add` - Create a new attestation for a given claim which is based on a
//!   CType. The attester can optionally provide a reference to an existing
//!   delegation that will be saved along with the attestation itself in the
//!   form of an attested delegation.
//! - `revoke` - Revoke an existing attestation for a given claim. The revoker
//!   must be either the creator of the attestation being revoked or an entity
//!   that in the delegation tree is an ancestor of the attester, i.e., it was
//!   either the delegator of the attester or an ancestor thereof.
//!
//! ## Assumptions
//!
//! - The claim which shall be attested is based on a CType and signed by the
//!   claimer.
//! - The Verifier trusts the Attester. Otherwise, the attestation is worthless
//!   for the Verifier

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

pub mod attestations;
pub mod default_weights;

#[cfg(any(feature = "mock", test))]
pub mod mock;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

mod access_control;
#[cfg(test)]
mod tests;

pub use crate::{
	access_control::AttestationAccessControl, attestations::AttestationDetails, default_weights::WeightInfo, pallet::*,
};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::{DispatchResult, DispatchResultWithPostInfo},
		pallet_prelude::*,
		traits::{Currency, Get, ReservableCurrency, StorageVersion},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::DispatchError;

	use ctype::CtypeHashOf;
	use kilt_support::{deposit::Deposit, traits::CallSources};

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	/// Type of a claim hash.
	pub type ClaimHashOf<T> = <T as frame_system::Config>::Hash;

	/// Type of an attester identifier.
	pub(crate) type AttesterOf<T> = <T as Config>::AttesterId;

	/// Authorization id type
	pub(crate) type AuthorizationIdOf<T> = <T as Config>::AuthorizationId;

	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

	pub(crate) type CurrencyOf<T> = <T as Config>::Currency;

	#[pallet::config]
	pub trait Config: frame_system::Config + ctype::Config {
		type EnsureOrigin: EnsureOrigin<
			Success = <Self as Config>::OriginSuccess,
			<Self as frame_system::Config>::Origin,
		>;
		type OriginSuccess: CallSources<AccountIdOf<Self>, AttesterOf<Self>>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;

		/// The currency that is used to reserve funds for each attestation.
		type Currency: ReservableCurrency<AccountIdOf<Self>>;

		/// The deposit that is required for storing an attestation.
		#[pallet::constant]
		type Deposit: Get<BalanceOf<Self>>;

		/// The maximum number of delegated attestations which can be made by
		/// the same delegation.
		#[pallet::constant]
		type MaxDelegatedAttestations: Get<u32>;

		type AttesterId: Parameter + MaxEncodedLen;

		type AuthorizationId: Parameter + MaxEncodedLen;

		type AccessControl: Parameter
			+ AttestationAccessControl<Self::AttesterId, Self::AuthorizationId, CtypeHashOf<Self>, ClaimHashOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// Attestations stored on chain.
	///
	/// It maps from a claim hash to the full attestation.
	#[pallet::storage]
	#[pallet::getter(fn attestations)]
	pub type Attestations<T> = StorageMap<_, Blake2_128Concat, ClaimHashOf<T>, AttestationDetails<T>>;

	/// Delegated attestations stored on chain.
	///
	/// It maps from a delegation ID to a vector of claim hashes.
	#[pallet::storage]
	#[pallet::getter(fn external_attestations)]
	pub type ExternalAttestations<T> =
		StorageDoubleMap<_, Twox64Concat, AuthorizationIdOf<T>, Blake2_128Concat, ClaimHashOf<T>, bool, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new attestation has been created.
		/// \[attester ID, claim hash, CType hash, (optional) delegation ID\]
		AttestationCreated(
			AttesterOf<T>,
			ClaimHashOf<T>,
			CtypeHashOf<T>,
			Option<AuthorizationIdOf<T>>,
		),
		/// An attestation has been revoked.
		/// \[account id, claim hash\]
		AttestationRevoked(AttesterOf<T>, ClaimHashOf<T>),
		/// An attestation has been removed.
		/// \[account id, claim hash\]
		AttestationRemoved(AttesterOf<T>, ClaimHashOf<T>),
		/// The deposit owner reclaimed a deposit by removing an attestation.
		/// \[account id, claim hash\]
		DepositReclaimed(AccountIdOf<T>, ClaimHashOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// There is already an attestation with the same claim hash stored on
		/// chain.
		AlreadyAttested,
		/// The attestation has already been revoked.
		AlreadyRevoked,
		/// No attestation on chain matching the claim hash.
		AttestationNotFound,
		/// The attestation CType does not match the CType specified in the
		/// delegation hierarchy root.
		CTypeMismatch,
		/// The call origin is not authorized to change the attestation.
		Unauthorized,
		/// The maximum number of delegated attestations has already been
		/// reached for the corresponding delegation id such that another one
		/// cannot be added.
		MaxDelegatedAttestationsExceeded,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new attestation.
		///
		/// The attester can optionally provide a reference to an existing
		/// delegation that will be saved along with the attestation itself in
		/// the form of an attested delegation.
		///
		/// The referenced CType hash must already be present on chain.
		///
		/// If an optional delegation id is provided, the dispatch origin must
		/// be the owner of the delegation. Otherwise, it could be any
		/// `DelegationEntityId`.
		///
		/// Emits `AttestationCreated`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: [Origin Account], Ctype, Attestations
		/// - Reads if delegation id is provided: Delegations, Roots,
		///   DelegatedAttestations
		/// - Writes: Attestations, (DelegatedAttestations)
		/// # </weight>
		#[pallet::weight(
			<T as pallet::Config>::WeightInfo::add()
			.saturating_add(authorization.as_ref().map(|ac| ac.can_attest_weight()).unwrap_or(0))
		)]
		pub fn add(
			origin: OriginFor<T>,
			claim_hash: ClaimHashOf<T>,
			ctype_hash: CtypeHashOf<T>,
			authorization: Option<T::AccessControl>,
		) -> DispatchResult {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;
			let payer = source.sender();
			let who = source.subject();
			let deposit_amount = <T as Config>::Deposit::get();

			ensure!(
				<ctype::Ctypes<T>>::contains_key(&ctype_hash),
				ctype::Error::<T>::CTypeNotFound
			);
			ensure!(
				!<Attestations<T>>::contains_key(&claim_hash),
				Error::<T>::AlreadyAttested
			);

			// Check for validity of the delegation node if specified.
			authorization
				.as_ref()
				.map(|ac| ac.can_attest(&who, &ctype_hash, &claim_hash))
				.transpose()?;
			let authorization_id = authorization.as_ref().map(|ac| ac.authorization_id());

			let deposit = Pallet::<T>::reserve_deposit(payer, deposit_amount)?;

			// *** No Fail beyond this point ***

			log::debug!("insert Attestation");

			Attestations::<T>::insert(
				&claim_hash,
				AttestationDetails {
					ctype_hash,
					attester: who.clone(),
					authorization_id: authorization_id.clone(),
					revoked: false,
					deposit,
				},
			);
			if let Some(authorization_id) = &authorization_id {
				ExternalAttestations::<T>::insert(authorization_id, claim_hash, true);
			}

			Self::deposit_event(Event::AttestationCreated(who, claim_hash, ctype_hash, authorization_id));

			Ok(())
		}

		/// Revoke an existing attestation.
		///
		/// The revoker must be either the creator of the attestation being
		/// revoked or an entity that in the delegation tree is an ancestor of
		/// the attester, i.e., it was either the delegator of the attester or
		/// an ancestor thereof.
		///
		/// Emits `AttestationRevoked`.
		///
		/// # <weight>
		/// Weight: O(P) where P is the number of steps required to verify that
		/// the dispatch Origin controls the delegation entitled to revoke the
		/// attestation. It is bounded by `max_parent_checks`.
		/// - Reads: [Origin Account], Attestations, delegation::Roots
		/// - Reads per delegation step P: delegation::Delegations
		/// - Writes: Attestations, DelegatedAttestations
		/// # </weight>
		#[pallet::weight(
			<T as pallet::Config>::WeightInfo::revoke()
			.saturating_add(authorization.as_ref().map(|ac| ac.can_revoke_weight()).unwrap_or(0))
		)]
		pub fn revoke(
			origin: OriginFor<T>,
			claim_hash: ClaimHashOf<T>,
			authorization: Option<T::AccessControl>,
		) -> DispatchResultWithPostInfo {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;
			let who = source.subject();

			let attestation = <Attestations<T>>::get(&claim_hash).ok_or(Error::<T>::AttestationNotFound)?;

			ensure!(!attestation.revoked, Error::<T>::AlreadyRevoked);

			if attestation.attester != who {
				let attestation_auth_id = attestation.authorization_id.as_ref().ok_or(Error::<T>::Unauthorized)?;
				authorization.ok_or(Error::<T>::Unauthorized)?.can_revoke(
					&who,
					&attestation.ctype_hash,
					&claim_hash,
					attestation_auth_id,
				)?;
			}

			// *** No Fail beyond this point ***

			log::debug!("revoking Attestation");
			Attestations::<T>::insert(
				&claim_hash,
				AttestationDetails {
					revoked: true,
					..attestation
				},
			);

			Self::deposit_event(Event::AttestationRevoked(who, claim_hash));

			Ok(Some(<T as pallet::Config>::WeightInfo::revoke()).into())
		}

		/// Remove an attestation.
		///
		/// The origin must be either the creator of the attestation or an
		/// entity which is an ancestor of the attester in the delegation tree,
		/// i.e., it was either the delegator of the attester or an ancestor
		/// thereof.
		///
		/// Emits `AttestationRemoved`.
		///
		/// # <weight>
		/// Weight: O(P) where P is the number of steps required to verify that
		/// the dispatch Origin controls the delegation entitled to revoke the
		/// attestation. It is bounded by `max_parent_checks`.
		/// - Reads: [Origin Account], Attestations, delegation::Roots
		/// - Reads per delegation step P: delegation::Delegations
		/// - Writes: Attestations, DelegatedAttestations
		/// # </weight>
		#[pallet::weight(
			<T as pallet::Config>::WeightInfo::remove()
			.saturating_add(authorization.as_ref().map(|ac| ac.can_remove_weight()).unwrap_or(0))
		)]
		pub fn remove(
			origin: OriginFor<T>,
			claim_hash: ClaimHashOf<T>,
			authorization: Option<T::AccessControl>,
		) -> DispatchResultWithPostInfo {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;
			let who = source.subject();

			let attestation = Attestations::<T>::get(&claim_hash).ok_or(Error::<T>::AttestationNotFound)?;

			if attestation.attester != who {
				let attestation_auth_id = attestation.authorization_id.as_ref().ok_or(Error::<T>::Unauthorized)?;
				authorization.ok_or(Error::<T>::Unauthorized)?.can_remove(
					&who,
					&attestation.ctype_hash,
					&claim_hash,
					attestation_auth_id,
				)?;
			}

			// *** No Fail beyond this point ***

			log::debug!("removing Attestation");

			Self::remove_attestation(attestation, claim_hash);
			Self::deposit_event(Event::AttestationRemoved(who, claim_hash));

			Ok(Some(<T as pallet::Config>::WeightInfo::remove()).into())
		}

		/// Reclaim a storage deposit by removing an attestation
		///
		/// Emits `DepositReclaimed`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: [Origin Account], Attestations, DelegatedAttestations
		/// - Writes: Attestations, DelegatedAttestations
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::reclaim_deposit())]
		pub fn reclaim_deposit(origin: OriginFor<T>, claim_hash: ClaimHashOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let attestation = Attestations::<T>::get(&claim_hash).ok_or(Error::<T>::AttestationNotFound)?;

			ensure!(attestation.deposit.owner == who, Error::<T>::Unauthorized);

			// *** No Fail beyond this point ***

			log::debug!("removing Attestation");

			Self::remove_attestation(attestation, claim_hash);
			Self::deposit_event(Event::DepositReclaimed(who, claim_hash));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Reserve the deposit and record the deposit on chain.
		///
		/// Fails if the `payer` has a balance less than deposit.
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

		fn remove_attestation(attestation: AttestationDetails<T>, claim_hash: ClaimHashOf<T>) {
			kilt_support::free_deposit::<AccountIdOf<T>, CurrencyOf<T>>(&attestation.deposit);
			Attestations::<T>::remove(&claim_hash);
			if let Some(authorization_id) = &attestation.authorization_id {
				ExternalAttestations::<T>::remove(authorization_id, claim_hash);
			}
		}
	}
}
