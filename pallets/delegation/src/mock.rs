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

use crate as delegation;
use crate::*;
use ctype::mock as ctype_mock;

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
	type DelegationNodeId = H256;
}

impl ctype::Config for Test {
	type Event = ();
	type WeightInfo = ();
}

impl did::Config for Test {
	type Event = ();
	type WeightInfo = ();
	type DidIdentifier = AccountId;
}

pub type TestDelegationNodeId = <Test as Config>::DelegationNodeId;
pub type TestDidIdentifier = <Test as did::Config>::DidIdentifier;

#[cfg(test)]
pub(crate) const DEFAULT_ACCOUNT: AccountId = AccountId::new([0u8; 32]);

const DEFAULT_ROOT_ID_SEED: u64 = 1u64;
const ALTERNATIVE_ROOT_ID_SEED: u64 = 2u64;
const DEFAULT_DELEGATION_ID_SEED: u64 = 3u64;
const ALTERNATIVE_DELEGATION_ID_SEED: u64 = 4u64;
const DEFAULT_DELEGATION_ID_2_SEED: u64 = 3u64;
const ALTERNATIVE_DELEGATION_ID_2_SEED: u64 = 4u64;

pub fn get_delegation_root_id(default: bool) -> H256 {
	if default {
		H256::from_low_u64_be(DEFAULT_ROOT_ID_SEED)
	} else {
		H256::from_low_u64_be(ALTERNATIVE_ROOT_ID_SEED)
	}
}

pub fn get_delegation_id(default: bool) -> H256 {
	if default {
		H256::from_low_u64_be(DEFAULT_DELEGATION_ID_SEED)
	} else {
		H256::from_low_u64_be(ALTERNATIVE_DELEGATION_ID_SEED)
	}
}

pub fn get_delegation_id_2(default: bool) -> H256 {
	if default {
		H256::from_low_u64_be(DEFAULT_DELEGATION_ID_2_SEED)
	} else {
		H256::from_low_u64_be(ALTERNATIVE_DELEGATION_ID_2_SEED)
	}
}

#[cfg(test)]
pub(crate) fn hash_to_u8<T: Encode>(hash: T) -> Vec<u8> {
	hash.encode()
}

pub fn generate_base_delegation_root_creation_operation(
	root_id: TestDelegationNodeId,
	root_node: DelegationRoot<Test>,
) -> DelegationRootCreationOperation<Test> {
	DelegationRootCreationOperation {
		caller_did: root_node.owner,
		ctype_hash: root_node.ctype_hash,
		root_id,
		tx_counter: 1u64,
	}
}

pub fn generate_base_delegation_creation_operation(
	delegator_did: TestDidIdentifier,
	delegation_id: TestDelegationNodeId,
	delegate_signature: did::DidSignature,
	delegation_node: DelegationNode<Test>,
) -> DelegationCreationOperation<Test> {
	DelegationCreationOperation {
		caller_did: delegator_did,
		delegate_did: delegation_node.owner,
		delegate_signature,
		delegation_id,
		parent_id: delegation_node.parent,
		root_id: delegation_node.root_id,
		permissions: delegation_node.permissions,
		tx_counter: 1u64,
	}
}

pub fn generate_base_delegation_root_revocation_operation(
	root_id: TestDelegationNodeId,
	root_node: DelegationRoot<Test>,
) -> DelegationRootRevocationOperation<Test> {
	DelegationRootRevocationOperation {
		caller_did: root_node.owner,
		root_id,
		max_children: 1u32,
		tx_counter: 1u64,
	}
}

pub fn generate_base_delegation_revocation_operation(
	delegation_id: TestDelegationNodeId,
	delegation_node: DelegationNode<Test>,
) -> DelegationRevocationOperation<Test> {
	DelegationRevocationOperation {
		caller_did: delegation_node.owner,
		delegation_id,
		max_parent_checks: 1u32,
		max_revocations: 1u32,
		tx_counter: 1u64,
	}
}

pub fn generate_base_delegation_root(owner: TestDidIdentifier) -> DelegationRoot<Test> {
	DelegationRoot {
		owner,
		ctype_hash: ctype_mock::get_ctype_hash(true),
		revoked: false,
	}
}

pub fn generate_base_delegation_node(root_id: TestDelegationNodeId, owner: TestDidIdentifier) -> DelegationNode<Test> {
	DelegationNode {
		owner,
		parent: None,
		root_id,
		permissions: Permissions::DELEGATE,
		revoked: false,
	}
}

#[derive(Clone)]
pub struct ExtBuilder {
	ctype_builder: Option<ctype_mock::ExtBuilder>,
	root_delegations_stored: Vec<(TestDelegationNodeId, DelegationRoot<Test>)>,
	delegations_stored: Vec<(TestDelegationNodeId, DelegationNode<Test>)>,
	children_stored: Vec<(TestDelegationNodeId, Vec<TestDelegationNodeId>)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			ctype_builder: None,
			root_delegations_stored: vec![],
			delegations_stored: vec![],
			children_stored: vec![],
		}
	}
}

impl From<ctype_mock::ExtBuilder> for ExtBuilder {
	fn from(ctype_builder: ctype_mock::ExtBuilder) -> Self {
		Self {
			ctype_builder: Some(ctype_builder),
			..Default::default()
		}
	}
}

impl ExtBuilder {
	pub fn with_root_delegations(
		mut self,
		root_delegations: Vec<(TestDelegationNodeId, DelegationRoot<Test>)>,
	) -> Self {
		self.root_delegations_stored = root_delegations;
		self
	}

	pub fn with_delegations(mut self, delegations: Vec<(TestDelegationNodeId, DelegationNode<Test>)>) -> Self {
		self.delegations_stored = delegations;
		self
	}

	pub fn with_children(mut self, children: Vec<(TestDelegationNodeId, Vec<TestDelegationNodeId>)>) -> Self {
		self.children_stored = children;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut ext = if let Some(ctype_builder) = self.ctype_builder.clone() {
			ctype_builder.build()
		} else {
			let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
			sp_io::TestExternalities::new(storage)
		};

		if !self.root_delegations_stored.is_empty() {
			ext.execute_with(|| {
				self.root_delegations_stored.iter().for_each(|root_delegation| {
					delegation::Roots::<Test>::insert(root_delegation.0, root_delegation.1.clone());
				})
			});
		}

		if !self.delegations_stored.is_empty() {
			ext.execute_with(|| {
				self.delegations_stored.iter().for_each(|del| {
					delegation::Delegations::<Test>::insert(del.0, del.1.clone());
				})
			});
		}

		if !self.children_stored.is_empty() {
			ext.execute_with(|| {
				self.children_stored.iter().for_each(|child| {
					delegation::Children::<Test>::insert(child.0, child.1.clone());
				})
			});
		}

		ext
	}
}
