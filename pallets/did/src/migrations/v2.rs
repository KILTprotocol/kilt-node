// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

use crate::*;

use sp_runtime::traits::Zero;

#[cfg(feature = "try-runtime")]
pub(crate) fn pre_migrate<T: Config>() -> Result<(), &'static str> {
	ensure!(
		StorageVersion::<T>::get() == DidStorageVersion::V2,
		"Current deployed version is not v2."
	);

	log::info!("Version storage migrating from v2 to v3");
	Ok(())
}

pub(crate) fn migrate<T: Config>() -> Weight {
	log::info!("v2 -> v3 DID storage migrator started!");
	let mut total_weight = Weight::zero();

	Did::<T>::translate_values(|old_did_details: deprecated::v2::DidDetails<T>| {
		// Add a read from the old storage and a write for the new storage
		total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
		Some(old_to_new_did_details(old_did_details))
	});

	StorageVersion::<T>::set(DidStorageVersion::V3);
	// Adds a write from StorageVersion::set() weight.
	total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
	log::debug!("Total weight consumed: {}", total_weight);
	log::info!("v1 -> v2 DID storage migrator finished!");

	total_weight
}

fn old_to_new_did_details<T: Config>(old: deprecated::v2::DidDetails<T>) -> DidDetails<T> {
	DidDetails {
		authentication_key: old.authentication_key,
		attestation_key: old.attestation_key,
		delegation_key: old.delegation_key,
		key_agreement_keys: old.key_agreement_keys,
		last_tx_counter: old.last_tx_counter,
		public_keys: old.public_keys,
		service_endpoints: old.service_endpoints,
		creation_block_number: <frame_system::Pallet<T>>::block_number(),
	}
}

#[cfg(feature = "try-runtime")]
pub(crate) fn post_migrate<T: Config>() -> Result<(), &'static str> {
	ensure!(
		StorageVersion::<T>::get() == DidStorageVersion::V3,
		"The version after deployment is not 3 as expected."
	);
	// Make sure all DIDs have the same block number set.
	let block_number_set = Did::<T>::iter_values().next().map_or(BlockNumberOf::<T>::default(), |details| details.creation_block_number);
	ensure!(
		!Did::<T>::iter_values().any(|did_details| { did_details.creation_block_number != block_number_set}),
		"Some DIDs have a different block number than the expected one"
	);
	log::info!("Version storage migrated from v1 to v2");
	Ok(())
}

// Tests for the v1 storage migrator.
#[cfg(test)]
mod tests {
	use frame_support::StorageMap;
	use sp_core::Pair;

	use super::*;
	use crate::mock::{Test as TestRuntime, System};
	use mock::{get_did_identifier_from_ed25519_key, get_ed25519_authentication_key, ExtBuilder};

	#[test]
	fn ok_no_dids() {
		let mut ext = ExtBuilder::default()
			.with_storage_version(DidStorageVersion::V2)
			.build(None);
		ext.execute_with(|| {
			#[cfg(feature = "try-runtime")]
			assert!(
				pre_migrate::<TestRuntime>().is_ok(),
				"Pre-migration for v2 should not fail."
			);

			migrate::<TestRuntime>();

			#[cfg(feature = "try-runtime")]
			assert!(
				post_migrate::<TestRuntime>().is_ok(),
				"Post-migration for v2 should not fail."
			);
		});
	}

	#[test]
	fn ok_with_some_dids() {
		let mut ext = ExtBuilder::default()
			.with_storage_version(DidStorageVersion::V2)
			.build(None);
		ext.execute_with(|| {
			let auth_key = get_ed25519_authentication_key(true);
			let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
			let alice_did_details =
				deprecated::v2::DidDetails::<TestRuntime>::new(DidVerificationKey::from(auth_key.public()), 0u64).expect("Alice DID creation should not fail.");

			deprecated::v2::storage::Did::insert(&alice_did, alice_did_details);

			#[cfg(feature = "try-runtime")]
			assert!(
				pre_migrate::<TestRuntime>().is_ok(),
				"Pre-migration for v2 should not fail."
			);

			let migration_block_number: BlockNumberOf<TestRuntime> = 10;

			System::set_block_number(migration_block_number);

			migrate::<TestRuntime>();

			#[cfg(feature = "try-runtime")]
			assert!(
				post_migrate::<TestRuntime>().is_ok(),
				"Post-migration for v2 should not fail."
			);

			let new_stored_details =
				Did::<TestRuntime>::get(&alice_did).expect("New DID details should exist in the storage.");
			assert_eq!(new_stored_details.creation_block_number, migration_block_number);
		});
	}
}
