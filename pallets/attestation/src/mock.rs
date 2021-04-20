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

use crate as attestation;
use crate::*;
use delegation::mock as delegation_mock;

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
		Attestation: attestation::{Pallet, Call, Storage, Event<T>},
		Ctype: ctype::{Pallet, Call, Storage, Event<T>},
		Delegation: delegation::{Pallet, Call, Storage, Event<T>},
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

impl Config for Test {
	type Event = ();
	type WeightInfo = ();
}

impl ctype::Config for Test {
	type Event = ();
	type WeightInfo = ();
}

impl delegation::Config for Test {
	type Event = ();
	type WeightInfo = ();
	type DelegationNodeId = H256;
}

impl did::Config for Test {
	type Event = ();
	type WeightInfo = ();
	type DidIdentifier = AccountId;
}

pub type TestHash = <Test as frame_system::Config>::Hash;
pub type TestDelegationNodeId = <Test as delegation::Config>::DelegationNodeId;

pub(crate) const DEFAULT_ACCOUNT: AccountId = AccountId::new([0u8; 32]);

#[derive(Clone)]
pub struct ExtBuilder {
	delegation_builder: Option<delegation_mock::ExtBuilder>,
	attestations_stored: Vec<(TestHash, attestation::Attestation<Test>)>,
	delegated_attestations_stored: Vec<(TestDelegationNodeId, Vec<TestHash>)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			delegation_builder: None,
			attestations_stored: vec![],
			delegated_attestations_stored: vec![],
		}
	}
}

impl From<delegation_mock::ExtBuilder> for ExtBuilder {
	fn from(delegation_builder: delegation_mock::ExtBuilder) -> Self {
		let mut instance = Self::default();
		instance.delegation_builder = Some(delegation_builder);
		instance
	}
}

impl ExtBuilder {
	pub fn with_attestations(
		mut self,
		attestations: Vec<(TestHash, attestation::Attestation<Test>)>,
	) -> Self {
		self.attestations_stored = attestations;
		self
	}

	pub fn with_delegated_attestations(mut self, delegated_attestations: Vec<(TestDelegationNodeId, Vec<TestHash>)>) -> Self {
		self.delegated_attestations_stored = delegated_attestations;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut ext = if let Some(delegation_builder) = self.delegation_builder.clone() {
			delegation_builder.build()
		} else {
			let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
			sp_io::TestExternalities::new(storage)
		};

		if !self.attestations_stored.is_empty() {
			ext.execute_with(|| {
				self.attestations_stored.iter().for_each(|attestation| {
					attestation::Attestations::<Test>::insert(attestation.0, attestation.1.clone());
				})
			});
		}

		if !self.delegated_attestations_stored.is_empty() {
			ext.execute_with(|| {
				self.delegated_attestations_stored.iter().for_each(|delegated_attestation| {
					attestation::DelegatedAttestations::<Test>::insert(delegated_attestation.0, delegated_attestation.1.clone());
				})
			});
		}

		ext
	}
}
