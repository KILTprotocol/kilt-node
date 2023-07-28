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

pub use crate::default_weights::WeightInfo;
pub use pallet::*;
pub mod default_weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(any(test, feature = "runtime-benchmarks"))]
mod mock;
#[cfg(test)]
mod test;

#[frame_support::pallet]
pub mod pallet {
	use attestation::ClaimHashOf;
	use delegation::DelegationNodeIdOf;
	use did::DidIdentifierOf;
	use frame_support::{
		pallet_prelude::{DispatchResult, *},
		traits::{LockableCurrency, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use pallet_did_lookup::linkable_account::LinkableAccountId;
	use pallet_web3_names::Web3NameOf;
	use public_credentials::{CredentialIdOf, SubjectIdOf};
	use sp_runtime::traits::Hash;
	use sp_std::vec::Vec;

	use crate::default_weights::WeightInfo;

	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	type HashOf<T> = <T as frame_system::Config>::Hash;

	#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq)]
	pub struct EntriesToMigrate<T>
	where
		T: did::Config,
		T: delegation::Config,
		T: frame_system::Config,
		T: pallet_web3_names::Config,
		T: public_credentials::Config,
		T: Config,
	{
		pub attestation: BoundedVec<ClaimHashOf<T>, <T as Config>::MaxMigrations>,
		pub delegation: BoundedVec<DelegationNodeIdOf<T>, <T as Config>::MaxMigrations>,
		pub did: BoundedVec<DidIdentifierOf<T>, <T as Config>::MaxMigrations>,
		pub lookup: BoundedVec<LinkableAccountId, <T as Config>::MaxMigrations>,
		pub w3n: BoundedVec<Web3NameOf<T>, <T as Config>::MaxMigrations>,
		pub staking: BoundedVec<AccountIdOf<T>, <T as Config>::MaxMigrations>,
		pub public_credentials: BoundedVec<(SubjectIdOf<T>, CredentialIdOf<T>), <T as Config>::MaxMigrations>,
	}

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ attestation::Config
		+ delegation::Config
		+ did::Config
		+ pallet_did_lookup::Config
		+ pallet_web3_names::Config
		+ parachain_staking::Config
		+ public_credentials::Config
		+ TypeInfo
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The max amount on migrations for each pallet
		#[pallet::constant]
		type MaxMigrations: Get<u32>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn connected_dids)]
	pub type MigratedKeys<T> = StorageMap<_, Blake2_128Concat, HashOf<T>, ()>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		EntriesUpdated(EntriesToMigrate<T>),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Some constants to distinguish between the different pallets
	pub(crate) const ATTESTATION_PALLET: &[u8] = b"attestation";
	pub(crate) const DELEGATION_PALLET: &[u8] = b"delegation";
	pub(crate) const DID_PALLET: &[u8] = b"did";
	pub(crate) const DID_LOOKUP_PALLET: &[u8] = b"pallet-did-lookup";
	pub(crate) const W3N_PALLET: &[u8] = b"pallet-web3-names";
	pub(crate) const BALANCES_PALLET: &[u8] = b"pallet-balances";
	pub(crate) const PUBLIC_CREDENTIALS_PALLET: &[u8] = b"public-credentials";

	// Some constants to distinguish between the different storage maps.
	pub(crate) const ATTESTATION_STORAGE_NAME: &[u8] = b"Attestations";
	pub(crate) const DELEGATION_STORAGE_NAME: &[u8] = b"DelegationNodes";
	pub(crate) const DID_STORAGE_NAME: &[u8] = b"Did";
	pub(crate) const DID_LOOKUP_STORAGE_NAME: &[u8] = b"ConnectedDids";
	pub(crate) const W3N_STORAGE_NAME: &[u8] = b"Owner";
	pub(crate) const BALANCES_STORAGE_NAME: &[u8] = b"Reserves";
	pub(crate) const PUBLIC_CREDENTIALS_STORAGE_NAME: &[u8] = b"Credentials";

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<T as attestation::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as delegation::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as did::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as pallet_did_lookup::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as pallet_web3_names::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as parachain_staking::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as parachain_staking::Config>::Currency: LockableCurrency<AccountIdOf<T>>,
		<T as public_credentials::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as frame_system::Config>::AccountId: Into<[u8; 32]>,
	{
		#[pallet::call_index(0)]
		#[pallet::weight({
			//TODO: Placeholder.
			Weight::from_parts(1_000_000, 1_000_000)
		})]
		pub fn update_balance(origin: OriginFor<T>, requested_migrations: EntriesToMigrate<T>) -> DispatchResult {
			ensure_signed(origin)?;

			requested_migrations
				.attestation
				.iter()
				.filter(|key| Self::is_key_already_migrated(key.as_ref(), ATTESTATION_PALLET, ATTESTATION_STORAGE_NAME))
				.try_for_each(|attestation_hash| {
					attestation::migrations::update_balance_for_entry::<T>(attestation_hash)
				})?;

			requested_migrations
				.delegation
				.iter()
				.filter(|key| Self::is_key_already_migrated(key.as_ref(), DELEGATION_PALLET, DELEGATION_STORAGE_NAME))
				.try_for_each(|delegation_hash| {
					delegation::migrations::update_balance_for_entry::<T>(delegation_hash)
				})?;

			requested_migrations
				.did
				.iter()
				.filter(|key| Self::is_key_already_migrated(key.as_ref(), DID_PALLET, DID_STORAGE_NAME))
				.try_for_each(|did_hash| did::migrations::update_balance_for_entry::<T>(did_hash))?;

			requested_migrations
				.lookup
				.iter()
				.filter(|key| Self::is_key_already_migrated(key.as_ref(), DID_LOOKUP_PALLET, DID_LOOKUP_STORAGE_NAME))
				.try_for_each(|did_lookup_hash| {
					pallet_did_lookup::migrations::update_balance_for_entry::<T>(did_lookup_hash)
				})?;

			requested_migrations
				.w3n
				.iter()
				.filter(|key| Self::is_key_already_migrated(key.as_ref(), W3N_PALLET, W3N_STORAGE_NAME))
				.try_for_each(|w3n| pallet_web3_names::migrations::update_balance_for_entry::<T>(w3n))?;

			requested_migrations
				.staking
				.iter()
				.filter(|&key| {
					let account_bytes: [u8; 32] = key.clone().into();
					Self::is_key_already_migrated(&account_bytes, BALANCES_PALLET, BALANCES_STORAGE_NAME)
				})
				.try_for_each(|account| parachain_staking::migrations::update_or_create_freeze::<T>(account))?;

			requested_migrations
				.public_credentials
				.iter()
				.filter(|(subject_id, credential_id)| {
					Self::filter_disclosure_public_credentials(subject_id, credential_id)
				})
				.try_for_each(|(subject_id, credential_id)| {
					public_credentials::migrations::update_balance_for_entry::<T>(subject_id, credential_id)
				})?;

			Self::deposit_event(Event::EntriesUpdated(requested_migrations));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn calculate_full_key(storage_key: &[u8], pallet_name: &[u8], storage_name: &[u8]) -> HashOf<T> {
			let vec_capacity = storage_key.len() + pallet_name.len() + storage_name.len();
			let mut full_key: Vec<u8> = Vec::with_capacity(vec_capacity);
			full_key.extend_from_slice(storage_key);
			full_key.extend_from_slice(pallet_name);
			full_key.extend_from_slice(storage_name);
			<T as frame_system::Config>::Hashing::hash_of(&full_key)
		}

		fn is_key_already_migrated(key: &[u8], pallet_name: &[u8], storage_name: &[u8]) -> bool {
			let full_key = Self::calculate_full_key(key, pallet_name, storage_name);
			if MigratedKeys::<T>::contains_key(full_key) {
				return false;
			}
			MigratedKeys::<T>::insert(full_key, ());
			true
		}

		pub fn calculate_public_credentials_key(
			subject_id: &<T as public_credentials::Config>::SubjectId,
			credential_id: &<T as public_credentials::Config>::CredentialId,
		) -> Vec<u8> {
			let subject_id_encoded = subject_id.encode();
			let subject_id_ref: &[u8] = subject_id_encoded.as_ref();
			let credential_id_ref = credential_id.as_ref();
			let vec_capacity = subject_id_ref.len() + credential_id_ref.len();
			let mut key: Vec<u8> = Vec::with_capacity(vec_capacity);
			key.extend_from_slice(subject_id_ref);
			key.extend_from_slice(credential_id_ref);
			key
		}

		fn filter_disclosure_public_credentials(
			subject_id: &<T as public_credentials::Config>::SubjectId,
			credential_id: &<T as public_credentials::Config>::CredentialId,
		) -> bool {
			let key = Self::calculate_public_credentials_key(subject_id, credential_id);
			Self::is_key_already_migrated(key.as_ref(), PUBLIC_CREDENTIALS_PALLET, PUBLIC_CREDENTIALS_STORAGE_NAME)
		}
	}
}
