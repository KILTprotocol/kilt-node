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

use crate::{DidLookup, Runtime, Weight};
use frame_support::traits::{GetStorageVersion, OnRuntimeUpgrade};
use sp_std::marker::PhantomData;

pub struct LookupReverseIndexMigration<T: pallet_did_lookup::Config>(PhantomData<T>);

impl OnRuntimeUpgrade for LookupReverseIndexMigration<Runtime> {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		assert!(DidLookup::on_chain_storage_version() < DidLookup::current_storage_version());
		assert_eq!(pallet_did_lookup::ConnectedAccounts::<Runtime>::iter().count(), 0);

		Ok(())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		// Account for the new storage version written below.
		let initial_weight = <Runtime as frame_system::Config>::DbWeight::get().writes(1);

		let total_weight: Weight = pallet_did_lookup::ConnectedDids::<Runtime>::iter().fold(
			initial_weight,
			|total_weight, (account, record)| {
				pallet_did_lookup::ConnectedAccounts::<Runtime>::insert(record.did, account, ());
				// One read for the `ConnectedDids` entry, one write for the new
				// `ConnectedAccounts` entry.
				total_weight.saturating_add(<Runtime as frame_system::Config>::DbWeight::get().reads_writes(1, 1))
			},
		);

		DidLookup::current_storage_version().put::<DidLookup>();

		total_weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		assert_eq!(
			DidLookup::on_chain_storage_version(),
			DidLookup::current_storage_version()
		);

		// Verify DID -> Account integrity.
		pallet_did_lookup::ConnectedDids::<Runtime>::iter().for_each(|(account, record)| {
			assert!(pallet_did_lookup::ConnectedAccounts::<Runtime>::contains_key(
				record.did, account
			));
		});
		// Verify Account -> DID integrity.
		pallet_did_lookup::ConnectedAccounts::<Runtime>::iter().for_each(|(did, account, _)| {
			let entry = pallet_did_lookup::ConnectedDids::<Runtime>::get(account)
				.expect("Should find a record for the given account.");
			assert_eq!(entry.did, did);
		});

		Ok(())
	}
}
