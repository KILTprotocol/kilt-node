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

pub use pallet::*;

// #[cfg(feature = "runtime-benchmarks")] TODO
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::{DispatchResult, *},
		traits::{LockableCurrency, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;

	use attestation::ClaimHashOf;
	use delegation::DelegationNodeIdOf;
	use did::DidIdentifierOf;
	use pallet_did_lookup::linkable_account::LinkableAccountId;
	use pallet_web3_names::Web3NameOf;
	use public_credentials::{CredentialIdOf, SubjectIdOf};

	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq)]
	pub struct PalletToMigrate<T>
	where
		T: did::Config,
		T: delegation::Config,
		T: frame_system::Config,
		T: pallet_web3_names::Config,
		T: public_credentials::Config,
		T: Config,
	{
		attestation: BoundedVec<ClaimHashOf<T>, <T as Config>::MaxMigrations>,
		delegation: BoundedVec<DelegationNodeIdOf<T>, <T as Config>::MaxMigrations>,
		did: BoundedVec<DidIdentifierOf<T>, <T as Config>::MaxMigrations>,
		lookup: BoundedVec<LinkableAccountId, <T as Config>::MaxMigrations>,
		w3n: BoundedVec<Web3NameOf<T>, <T as Config>::MaxMigrations>,
		staking: BoundedVec<AccountIdOf<T>, <T as Config>::MaxMigrations>,
		public_credentials: BoundedVec<(SubjectIdOf<T>, CredentialIdOf<T>), <T as Config>::MaxMigrations>,
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
		type MaxMigrations: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::event]
	pub enum Event<T: Config> {
		UserUpdated(<T as frame_system::Config>::AccountId),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

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
	{
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0))]
		pub fn update_single_entry(origin: OriginFor<T>, requested_migrations: PalletToMigrate<T>) -> DispatchResult {
			ensure_signed(origin)?;

			requested_migrations
				.attestation
				.iter()
				.try_for_each(|attestation_hash| {
					attestation::migrations::update_balance_for_entry::<T>(attestation_hash)
				})?;

			requested_migrations.delegation.iter().try_for_each(|delegation_hash| {
				delegation::migrations::update_balance_for_entry::<T>(delegation_hash)
			})?;

			requested_migrations
				.did
				.iter()
				.try_for_each(|did_hash| did::migrations::update_balance_for_entry::<T>(did_hash))?;

			requested_migrations.lookup.iter().try_for_each(|did_lookup_hash| {
				pallet_did_lookup::migrations::update_balance_for_entry::<T>(did_lookup_hash)
			})?;

			requested_migrations
				.w3n
				.iter()
				.try_for_each(|w3n| pallet_web3_names::migrations::update_balance_for_entry::<T>(w3n))?;

			requested_migrations
				.staking
				.iter()
				.try_for_each(|account| parachain_staking::migrations::update_or_create_freeze::<T>(account))?;

			requested_migrations
				.public_credentials
				.iter()
				.try_for_each(|(subject_id, credential_id)| {
					public_credentials::migrations::update_balance_for_entry::<T>(subject_id, credential_id)
				})
		}
	}
}
