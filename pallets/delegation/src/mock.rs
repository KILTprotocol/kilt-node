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

use frame_support::{parameter_types, weights::constants::RocksDbWeight};
use frame_system::EnsureSigned;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
};

#[cfg(test)]
use codec::Encode;

use crate as delegation;
use crate::*;
use ctype::mock as ctype_mock;

pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub type Block = frame_system::mocking::MockBlock<Test>;

pub type TestDidIdentifier = kilt_primitives::AccountId;
pub type TestCtypeOwner = TestDidIdentifier;
pub type TestCtypeHash = kilt_primitives::Hash;
pub type TestDelegationNodeId = kilt_primitives::Hash;
pub type TestDelegatorId = TestDidIdentifier;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Ctype: ctype::{Pallet, Call, Storage, Event<T>},
		Delegation: delegation::{Pallet, Call, Storage, Event<T>},
		Did: did::{Pallet, Call, Storage, Event<T>, Origin<T>},
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
	type Hash = kilt_primitives::Hash;
	type Hashing = BlakeTwo256;
	type AccountId = <<kilt_primitives::Signature as Verify>::Signer as IdentifyAccount>::AccountId;
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

parameter_types! {
	pub const MaxNewKeyAgreementKeys: u32 = 10u32;
	pub const MaxVerificationKeysToRevoke: u32 = 10u32;
	pub const MaxUrlLength: u32 = 200u32;
}

impl did::Config for Test {
	type DidIdentifier = TestDidIdentifier;
	type Origin = Origin;
	type Call = Call;
	type Event = ();
	type MaxNewKeyAgreementKeys = MaxNewKeyAgreementKeys;
	type MaxUrlLength = MaxUrlLength;
	type MaxVerificationKeysToRevoke = MaxVerificationKeysToRevoke;
	type WeightInfo = ();
}

impl ctype::Config for Test {
	type CtypeCreatorId = TestCtypeOwner;
	type EnsureOrigin = EnsureSigned<TestCtypeOwner>;
	type Event = ();
}

impl Config for Test {
	type DelegationNodeId = TestDelegationNodeId;
	type EnsureOrigin = EnsureSigned<TestDelegatorId>;
	type Event = ();
}

impl did::DeriveDidCallAuthorizationVerificationKeyRelationship for Call {
	fn derive_verification_key_relationship(&self) -> Option<did::DidVerificationKeyRelationship> {
		// Not used in this pallet
		None
	}

	// Always return a System::remark() extrinsic call
	#[cfg(feature = "runtime-benchmarks")]
	fn get_call_for_did_call_benchmark() -> Self {
		Call::System(frame_system::Call::remark(vec![]))
	}
}

#[cfg(test)]
pub(crate) const ALICE: TestDelegatorId = TestDelegatorId::new([0u8; 32]);
#[cfg(test)]
pub(crate) const BOB: TestDelegatorId = TestDelegatorId::new([1u8; 32]);
#[cfg(test)]
pub(crate) const CHARLIE: TestDelegatorId = TestDelegatorId::new([2u8; 32]);

const DEFAULT_ROOT_ID_SEED: u64 = 1u64;
const ALTERNATIVE_ROOT_ID_SEED: u64 = 2u64;
const DEFAULT_DELEGATION_ID_SEED: u64 = 3u64;
const ALTERNATIVE_DELEGATION_ID_SEED: u64 = 4u64;
const DEFAULT_DELEGATION_ID_2_SEED: u64 = 3u64;
const ALTERNATIVE_DELEGATION_ID_2_SEED: u64 = 4u64;

pub fn get_origin(account: TestDelegatorId) -> Origin {
	Origin::signed(account)
}

pub fn get_delegation_root_id(default: bool) -> TestDelegationNodeId {
	if default {
		TestCtypeHash::from_low_u64_be(DEFAULT_ROOT_ID_SEED)
	} else {
		TestCtypeHash::from_low_u64_be(ALTERNATIVE_ROOT_ID_SEED)
	}
}

pub fn get_delegation_id(default: bool) -> TestDelegationNodeId {
	if default {
		TestCtypeHash::from_low_u64_be(DEFAULT_DELEGATION_ID_SEED)
	} else {
		TestCtypeHash::from_low_u64_be(ALTERNATIVE_DELEGATION_ID_SEED)
	}
}

pub fn get_delegation_id_2(default: bool) -> TestDelegationNodeId {
	if default {
		TestCtypeHash::from_low_u64_be(DEFAULT_DELEGATION_ID_2_SEED)
	} else {
		TestCtypeHash::from_low_u64_be(ALTERNATIVE_DELEGATION_ID_2_SEED)
	}
}

#[cfg(test)]
pub(crate) fn hash_to_u8<T: Encode>(hash: T) -> Vec<u8> {
	hash.encode()
}

pub struct DelegationRootCreationDetails {
	pub root_id: TestDelegationNodeId,
	pub ctype_hash: TestCtypeHash,
}

pub fn generate_base_delegation_root_creation_details(
	root_id: TestDelegationNodeId,
	root_node: DelegationRoot<Test>,
) -> DelegationRootCreationDetails {
	DelegationRootCreationDetails {
		ctype_hash: root_node.ctype_hash,
		root_id,
	}
}

pub struct DelegationCreationDetails {
	pub delegation_id: TestDelegationNodeId,
	pub root_id: TestDelegationNodeId,
	pub parent_id: Option<TestDelegationNodeId>,
	pub delegate: TestDelegatorId,
	pub permissions: Permissions,
	pub delegate_signature: did::DidSignature,
}

pub fn generate_base_delegation_creation_details(
	delegation_id: TestDelegationNodeId,
	delegate_signature: did::DidSignature,
	delegation_node: DelegationNode<Test>,
) -> DelegationCreationDetails {
	DelegationCreationDetails {
		delegation_id,
		parent_id: delegation_node.parent,
		root_id: delegation_node.root_id,
		delegate: delegation_node.owner,
		delegate_signature,
		permissions: delegation_node.permissions,
	}
}

pub struct DelegationRootRevocationDetails {
	pub root_id: TestDelegationNodeId,
	pub max_children: u32,
}

pub fn generate_base_delegation_root_revocation_details(
	root_id: TestDelegationNodeId,
) -> DelegationRootRevocationDetails {
	DelegationRootRevocationDetails {
		root_id,
		max_children: 0u32,
	}
}

pub struct DelegationRevocationDetails {
	pub delegation_id: TestDelegationNodeId,
	pub max_parent_checks: u32,
	pub max_revocations: u32,
}

pub fn generate_base_delegation_revocation_details(delegation_id: TestDelegationNodeId) -> DelegationRevocationDetails {
	DelegationRevocationDetails {
		delegation_id,
		max_parent_checks: 0u32,
		max_revocations: 0u32,
	}
}

pub fn generate_base_delegation_root(owner: TestDelegatorId) -> DelegationRoot<Test> {
	DelegationRoot {
		owner,
		ctype_hash: ctype_mock::get_ctype_hash(true),
		revoked: false,
	}
}

pub fn generate_base_delegation_node(root_id: TestDelegationNodeId, owner: TestDelegatorId) -> DelegationNode<Test> {
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

	pub fn build(self, ext: Option<sp_io::TestExternalities>) -> sp_io::TestExternalities {
		let mut ext = if let Some(ext) = ext {
			ext
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
