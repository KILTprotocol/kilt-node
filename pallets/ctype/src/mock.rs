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

#![allow(clippy::from_over_into)]

use crate as ctype;
use did::mock as did_mock;

use frame_support::{parameter_types, weights::constants::RocksDbWeight};
use kilt_primitives::{AccountId, Signature};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
};

pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Ctype: ctype::{Pallet, Call, Storage, Event<T>},
		Did: did::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const SS58Prefix: u8 = 38;
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type DbWeight = RocksDbWeight;
	type Version = ();

	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

impl ctype::Config for Test {
	type Event = ();
	type WeightInfo = ();
}

impl did::Config for Test {
	type Event = ();
	type Call = Call;
	type WeightInfo = ();
	type DidIdentifier = AccountId;
}

pub type TestCtypeHash = <Test as frame_system::Config>::Hash;
pub type TestDidIdentifier = <Test as did::Config>::DidIdentifier;

#[cfg(test)]
pub(crate) const DEFAULT_ACCOUNT: AccountId = AccountId::new([0u8; 32]);

const DEFAULT_CTYPE_HASH_SEED: u64 = 1u64;
const ALTERNATIVE_CTYPE_HASH_SEED: u64 = 2u64;

pub fn get_ctype_hash(default: bool) -> H256 {
	if default {
		H256::from_low_u64_be(DEFAULT_CTYPE_HASH_SEED)
	} else {
		H256::from_low_u64_be(ALTERNATIVE_CTYPE_HASH_SEED)
	}
}

// Given a creator DID, it returns a CtypeCreationOperation
// that would result in a CTYPE with a default hash being written on chain.
pub fn generate_base_ctype_creation_operation(creator: TestDidIdentifier) -> ctype::CtypeCreationOperation<Test> {
	ctype::CtypeCreationOperation {
		creator_did: creator,
		hash: get_ctype_hash(true),
		tx_counter: 1u64,
	}
}

#[derive(Clone)]
pub struct ExtBuilder {
	did_builder: Option<did_mock::ExtBuilder>,
	ctypes_stored: Vec<(TestCtypeHash, TestDidIdentifier)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			ctypes_stored: vec![],
			did_builder: None,
		}
	}
}

impl From<did_mock::ExtBuilder> for ExtBuilder {
	fn from(did_builder: did_mock::ExtBuilder) -> Self {
		ExtBuilder {
			did_builder: Some(did_builder),
			..Default::default()
		}
	}
}

impl ExtBuilder {
	pub fn with_ctypes(mut self, ctypes: Vec<(TestCtypeHash, TestDidIdentifier)>) -> Self {
		self.ctypes_stored = ctypes;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut ext = if let Some(did_builder) = self.did_builder.clone() {
			did_builder.build()
		} else {
			let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
			sp_io::TestExternalities::new(storage)
		};

		if !self.ctypes_stored.is_empty() {
			ext.execute_with(|| {
				self.ctypes_stored.iter().for_each(|ctype| {
					ctype::Ctypes::<Test>::insert(ctype.0, ctype.1.clone());
				})
			});
		}

		ext
	}
}
