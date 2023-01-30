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

use frame_support::traits::{GetStorageVersion, OnRuntimeUpgrade};
use sp_runtime::traits::{Get, Zero};
use sp_std::marker::PhantomData;

use ctype::{CtypeCreatorOf, CtypeEntryOf};

#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

pub struct AddCTypeBlockNumber<R>(PhantomData<R>);

impl<T: ctype::Config> OnRuntimeUpgrade for AddCTypeBlockNumber<T> {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		// Missed the migration when v1 was introduced, so now Spiritnet and Peregrine
		// are on v0 although they should be on v1.
		assert!(ctype::Pallet::<T>::on_chain_storage_version() <= 1,);

		// Use iter_keys() on new storage so it won't try to decode values.
		let ctypes_to_migrate = ctype::Ctypes::<T>::iter_keys().count() as u64;

		log::info!("🪪  CType pallet pre check: {:?} CTypes to migrate", ctypes_to_migrate);
		Ok(ctypes_to_migrate.to_be_bytes().into())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let current = ctype::Pallet::<T>::current_storage_version();
		let onchain = ctype::Pallet::<T>::on_chain_storage_version();

		log::info!(
			"💰 Running CType migration with current storage version {:?} / onchain {:?}",
			current,
			onchain
		);

		let mut num_translations = 0u64;
		let default_block_number = <T as frame_system::Config>::BlockNumber::zero();

		ctype::Ctypes::<T>::translate_values(|old: CtypeCreatorOf<T>| {
			num_translations = num_translations.saturating_add(1);
			Some(CtypeEntryOf::<T> {
				creator: old,
				created_at: default_block_number,
			})
		});
		current.put::<ctype::Pallet<T>>();

		// Num translations + old version read and new version write
		T::DbWeight::get().reads_writes(num_translations.saturating_add(1), num_translations.saturating_add(1))
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		assert_eq!(ctype::Pallet::<T>::on_chain_storage_version(), 2);

		let initial_ctype_count = u64::from_be_bytes(state.try_into().expect("input state should be 8 bytes"));
		assert_eq!(initial_ctype_count, ctype::Ctypes::<T>::iter().count() as u64);
		// Verify all migrated ctypes can be decoded under the new type.
		ctype::Ctypes::<T>::iter_values().for_each(|v| assert!(v.created_at.is_zero()));

		log::info!(
			"🪪  CType pallet post checks ok, all {:} CTypes have been migrated ✅",
			initial_ctype_count
		);
		Ok(())
	}
}

pub struct MigrateToNewStorageVersion<R>(PhantomData<R>);

impl<R> MigrateToNewStorageVersion<R>
where
	R: attestation::Config + pallet_web3_names::Config + public_credentials::Config,
{
	fn migrate() -> frame_support::weights::Weight {
		type AttestationPallet<R> = attestation::Pallet<R>;
		type Web3NamesPallet<R> = pallet_web3_names::Pallet<R>;
		type PublicCredentialsPallet<R> = public_credentials::Pallet<R>;

		AttestationPallet::<R>::current_storage_version().put::<AttestationPallet<R>>();
		// Not an issue with Peregrine, but it is with Spiritnet.
		Web3NamesPallet::<R>::current_storage_version().put::<Web3NamesPallet<R>>();
		PublicCredentialsPallet::<R>::current_storage_version().put::<PublicCredentialsPallet<R>>();

		<R as frame_system::Config>::DbWeight::get().writes(3)
	}
}

#[cfg(feature = "try-runtime")]
impl<R> OnRuntimeUpgrade for MigrateToNewStorageVersion<R>
where
	R: attestation::Config
		+ ctype::Config
		+ delegation::Config
		+ did::Config
		+ pallet_did_lookup::Config
		+ pallet_inflation::Config
		+ pallet_web3_names::Config
		+ parachain_staking::Config
		+ public_credentials::Config,
{
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		type AttestationPallet<R> = attestation::Pallet<R>;
		type DelegationPallet<R> = delegation::Pallet<R>;
		type DidPallet<R> = did::Pallet<R>;
		type LookupPallet<R> = pallet_did_lookup::Pallet<R>;
		type InflationPallet<R> = pallet_inflation::Pallet<R>;
		type Web3NamesPallet<R> = pallet_web3_names::Pallet<R>;
		type ParachainStakingPallet<R> = parachain_staking::Pallet<R>;
		type PublicCredentialsPallet<R> = public_credentials::Pallet<R>;

		log::info!("💿  Storage version pre checks");

		if AttestationPallet::<R>::on_chain_storage_version() != AttestationPallet::<R>::current_storage_version() {
			log::warn!(
				"🚨 Attestation pallet on chain version {:?} != declared storage version {:?}.",
				AttestationPallet::<R>::on_chain_storage_version(),
				AttestationPallet::<R>::current_storage_version()
			)
		}
		if DelegationPallet::<R>::on_chain_storage_version() != DelegationPallet::<R>::current_storage_version() {
			log::warn!(
				"🚨 Delegation pallet on chain version {:?} != declared storage version {:?}.",
				DelegationPallet::<R>::on_chain_storage_version(),
				DelegationPallet::<R>::current_storage_version()
			)
		}
		if DidPallet::<R>::on_chain_storage_version() != DidPallet::<R>::current_storage_version() {
			log::warn!(
				"🚨 Did pallet on chain version {:?} != declared storage version {:?}.",
				DidPallet::<R>::on_chain_storage_version(),
				DidPallet::<R>::current_storage_version()
			)
		}
		if LookupPallet::<R>::on_chain_storage_version() != LookupPallet::<R>::current_storage_version() {
			log::warn!(
				"🚨 Lookup pallet on chain version {:?} != declared storage version {:?}.",
				LookupPallet::<R>::on_chain_storage_version(),
				LookupPallet::<R>::current_storage_version()
			)
		}
		if InflationPallet::<R>::on_chain_storage_version() != InflationPallet::<R>::current_storage_version() {
			log::warn!(
				"🚨 Inflation pallet on chain version {:?} != declared storage version {:?}.",
				InflationPallet::<R>::on_chain_storage_version(),
				InflationPallet::<R>::current_storage_version()
			)
		}
		if Web3NamesPallet::<R>::on_chain_storage_version() != Web3NamesPallet::<R>::current_storage_version() {
			log::warn!(
				"🚨 Web3names pallet on chain version {:?} != declared storage version {:?}.",
				Web3NamesPallet::<R>::on_chain_storage_version(),
				Web3NamesPallet::<R>::current_storage_version()
			)
		}
		if ParachainStakingPallet::<R>::on_chain_storage_version()
			!= ParachainStakingPallet::<R>::current_storage_version()
		{
			log::warn!(
				"🚨 Parachain staking pallet on chain version {:?} != declared storage version {:?}.",
				ParachainStakingPallet::<R>::on_chain_storage_version(),
				ParachainStakingPallet::<R>::current_storage_version()
			)
		}
		if PublicCredentialsPallet::<R>::on_chain_storage_version()
			!= PublicCredentialsPallet::<R>::current_storage_version()
		{
			log::warn!(
				"🚨 Public credentials pallet on chain version {:?} != declared storage version {:?}.",
				PublicCredentialsPallet::<R>::on_chain_storage_version(),
				PublicCredentialsPallet::<R>::current_storage_version()
			)
		}

		Ok(Vec::default())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		Self::migrate()
	}

	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		type AttestationPallet<R> = attestation::Pallet<R>;
		type CTypePallet<R> = ctype::Pallet<R>;
		type DelegationPallet<R> = delegation::Pallet<R>;
		type DidPallet<R> = did::Pallet<R>;
		type LookupPallet<R> = pallet_did_lookup::Pallet<R>;
		type InflationPallet<R> = pallet_inflation::Pallet<R>;
		type Web3NamesPallet<R> = pallet_web3_names::Pallet<R>;
		type ParachainStakingPallet<R> = parachain_staking::Pallet<R>;
		type PublicCredentialsPallet<R> = public_credentials::Pallet<R>;

		assert_eq!(
			AttestationPallet::<R>::on_chain_storage_version(),
			AttestationPallet::<R>::current_storage_version(),
			"Attestation pallet on chain version {:?} != declared storage version {:?}.",
			AttestationPallet::<R>::on_chain_storage_version(),
			AttestationPallet::<R>::current_storage_version()
		);
		// Although it's part of a different migration, we check that the CType pallet
		// storage version is also consistent.
		assert_eq!(
			CTypePallet::<R>::on_chain_storage_version(),
			CTypePallet::<R>::current_storage_version(),
			"CType pallet on chain version {:?} != declared storage version {:?}.",
			CTypePallet::<R>::on_chain_storage_version(),
			CTypePallet::<R>::current_storage_version()
		);
		assert_eq!(
			DelegationPallet::<R>::on_chain_storage_version(),
			DelegationPallet::<R>::current_storage_version(),
			"Delegation pallet on chain version {:?} != declared storage version {:?}.",
			DelegationPallet::<R>::on_chain_storage_version(),
			DelegationPallet::<R>::current_storage_version()
		);
		assert_eq!(
			DidPallet::<R>::on_chain_storage_version(),
			DidPallet::<R>::current_storage_version(),
			"Did pallet on chain version {:?} != declared storage version {:?}.",
			DidPallet::<R>::on_chain_storage_version(),
			DidPallet::<R>::current_storage_version()
		);
		assert_eq!(
			LookupPallet::<R>::on_chain_storage_version(),
			LookupPallet::<R>::current_storage_version(),
			"Lookup pallet on chain version {:?} != declared storage version {:?}.",
			LookupPallet::<R>::on_chain_storage_version(),
			LookupPallet::<R>::current_storage_version()
		);
		assert_eq!(
			InflationPallet::<R>::on_chain_storage_version(),
			InflationPallet::<R>::current_storage_version(),
			"Inflation pallet on chain version {:?} != declared storage version {:?}.",
			InflationPallet::<R>::on_chain_storage_version(),
			InflationPallet::<R>::current_storage_version()
		);
		assert_eq!(
			Web3NamesPallet::<R>::on_chain_storage_version(),
			Web3NamesPallet::<R>::current_storage_version(),
			"Web3names pallet on chain version {:?} != declared storage version {:?}.",
			Web3NamesPallet::<R>::on_chain_storage_version(),
			Web3NamesPallet::<R>::current_storage_version()
		);
		assert_eq!(
			ParachainStakingPallet::<R>::on_chain_storage_version(),
			ParachainStakingPallet::<R>::current_storage_version(),
			"Parachain staking pallet on chain version {:?} != declared storage version {:?}.",
			ParachainStakingPallet::<R>::on_chain_storage_version(),
			ParachainStakingPallet::<R>::current_storage_version()
		);
		assert_eq!(
			PublicCredentialsPallet::<R>::on_chain_storage_version(),
			PublicCredentialsPallet::<R>::current_storage_version(),
			"Public credentials pallet on chain version {:?} != declared storage version {:?}.",
			PublicCredentialsPallet::<R>::on_chain_storage_version(),
			PublicCredentialsPallet::<R>::current_storage_version()
		);

		log::info!("💿  Storage version post checks ok ✅");

		Ok(())
	}
}

#[cfg(not(feature = "try-runtime"))]
impl<R> OnRuntimeUpgrade for MigrateToNewStorageVersion<R>
where
	R: attestation::Config + pallet_web3_names::Config + public_credentials::Config,
{
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		Self::migrate()
	}
}
