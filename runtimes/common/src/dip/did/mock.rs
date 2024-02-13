// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

use did::{
	did_details::{DidDetails, DidVerificationKey},
	mock_utils::generate_base_did_details,
};
use frame_support::traits::Currency;
use pallet_did_lookup::{account::AccountId20, linkable_account::LinkableAccountId};
use pallet_web3_names::Web3NameOf;
use sp_runtime::AccountId32;

use crate::{
	constants::KILT,
	dip::mock::{Balances, Did, DidLookup, TestRuntime, Web3Names},
	AccountId, DidIdentifier,
};

pub(crate) const ACCOUNT: AccountId = AccountId::new([100u8; 32]);
pub(crate) const DID_IDENTIFIER: DidIdentifier = DidIdentifier::new([150u8; 32]);
pub(crate) const SUBMITTER: AccountId = AccountId::new([150u8; 32]);

pub(crate) fn create_linked_info(
	auth_key: DidVerificationKey<AccountId>,
	include_web3_name: bool,
	linked_accounts: u32,
) -> (
	DidDetails<TestRuntime>,
	Option<Web3NameOf<TestRuntime>>,
	Vec<LinkableAccountId>,
) {
	let did_details: DidDetails<TestRuntime> = generate_base_did_details(auth_key, Some(SUBMITTER));
	let web3_name: Option<Web3NameOf<TestRuntime>> = if include_web3_name {
		Some(b"ntn_x2".to_vec().try_into().unwrap())
	} else {
		None
	};
	let linked_accounts = (0..linked_accounts)
		.map(|i| {
			let bytes = i.to_be_bytes();
			if i % 2 == 0 {
				let mut buffer = <[u8; 20]>::default();
				buffer[..4].copy_from_slice(&bytes);
				LinkableAccountId::AccountId20(AccountId20(buffer))
			} else {
				let mut buffer = <[u8; 32]>::default();
				buffer[..4].copy_from_slice(&bytes);
				LinkableAccountId::AccountId32(AccountId32::new(buffer))
			}
		})
		.collect::<Vec<_>>();
	(did_details, web3_name, linked_accounts)
}

#[derive(Default)]
pub(crate) struct ExtBuilder(
	#[allow(clippy::type_complexity)]
	Vec<(
		DidIdentifier,
		DidDetails<TestRuntime>,
		Option<Web3NameOf<TestRuntime>>,
		Vec<LinkableAccountId>,
		AccountId,
	)>,
	Vec<DidIdentifier>,
);

impl ExtBuilder {
	#[allow(clippy::type_complexity)]
	pub(crate) fn with_dids(
		mut self,
		dids: Vec<(
			DidIdentifier,
			DidDetails<TestRuntime>,
			Option<Web3NameOf<TestRuntime>>,
			Vec<LinkableAccountId>,
			AccountId,
		)>,
	) -> Self {
		self.0 = dids;
		self
	}

	pub(crate) fn with_deleted_dids(mut self, dids: Vec<DidIdentifier>) -> Self {
		self.1 = dids;
		self
	}
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut ext = sp_io::TestExternalities::default();

		ext.execute_with(|| {
			for (did_identifier, did_details, web3_name, linked_accounts, submitter) in self.0 {
				Balances::make_free_balance_be(&submitter, 100_000 * KILT);
				Did::try_insert_did(did_identifier.clone(), did_details, submitter.clone())
					.unwrap_or_else(|_| panic!("Failed to insert DID {:#?}.", did_identifier));
				if let Some(name) = web3_name {
					Web3Names::register_name(name.clone(), did_identifier.clone(), submitter.clone())
						.unwrap_or_else(|_| panic!("Failed to insert web3name{:#?}.", name));
				}
				for linked_account in linked_accounts {
					DidLookup::add_association(submitter.clone(), did_identifier.clone(), linked_account.clone())
						.unwrap_or_else(|_| panic!("Failed to insert linked account{:#?}.", linked_account));
				}
			}

			for did_identifier in self.1 {
				Balances::make_free_balance_be(&SUBMITTER, 100_000 * KILT);
				// Ignore error if the DID already exists
				let _ = Did::try_insert_did(
					did_identifier.clone(),
					did::mock_utils::generate_base_did_details(DidVerificationKey::Account(ACCOUNT), Some(SUBMITTER)),
					SUBMITTER,
				);
				did::Pallet::<TestRuntime>::delete_did(did_identifier, 0)
					.expect("Should not fail to mark DID as deleted.");
			}
		});

		ext
	}
}
