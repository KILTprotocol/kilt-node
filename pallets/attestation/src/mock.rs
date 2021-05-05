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

use crate as attestation;
use crate::{Attestation as AttestationStruct, *};
use ctype::mock as ctype_mock;
use delegation::mock as delegation_mock;

use frame_support::{parameter_types, weights::constants::RocksDbWeight};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
};

pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub type Block = frame_system::mocking::MockBlock<Test>;

pub type TestDidIdentifier = kilt_primitives::AccountId;
pub type TestCtypeOwner = TestDidIdentifier;
pub type TestCtypeHash = kilt_primitives::Hash;
pub type TestDelegationNodeId = kilt_primitives::Hash;
pub type TestDelegatorId = TestDidIdentifier;
pub type TestClaimHash = kilt_primitives::Hash;
pub type TestAttester = TestDidIdentifier;

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
	type Hash = H256;
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

impl did::Config for Test {
	type DidIdentifier = TestDidIdentifier;
	type Origin = Origin;
	type Call = Call;
	type Event = ();
	type WeightInfo = ();
}

impl ctype::Config for Test {
	type EnsureOrigin = did::EnsureDidOrigin<TestCtypeOwner>;
	type Event = ();
	type WeightInfo = ();
}

impl delegation::Config for Test {
	type DelegationNodeId = TestDelegationNodeId;
	type EnsureOrigin = did::EnsureDidOrigin<TestDelegatorId>;
	type Event = ();
	type WeightInfo = ();
}

impl Config for Test {
	type EnsureOrigin = did::EnsureDidOrigin<TestAttester>;
	type Event = ();
	type WeightInfo = ();
}

impl did::DeriveDidCallAuthorizationVerificationKeyRelationship for Call {
	// Only interested in attestation operations
	fn derive_verification_key_relationship(&self) -> Option<did::DidVerificationKeyRelationship> {
		// Not used in this pallet
		None
	}
}

#[cfg(test)]
pub(crate) const ALICE: TestAttester = TestAttester::new([0u8; 32]);
#[cfg(test)]
pub(crate) const BOB: TestAttester = TestAttester::new([1u8; 32]);

const DEFAULT_CLAIM_HASH_SEED: u64 = 1u64;
const ALTERNATIVE_CLAIM_HASH_SEED: u64 = 2u64;

pub fn get_origin(account: TestAttester) -> Origin {
	Origin::from(did::DidRawOrigin { id: account })
}

pub fn get_claim_hash(default: bool) -> TestClaimHash {
	if default {
		TestClaimHash::from_low_u64_be(DEFAULT_CLAIM_HASH_SEED)
	} else {
		TestClaimHash::from_low_u64_be(ALTERNATIVE_CLAIM_HASH_SEED)
	}
}

pub struct AttestationCreationDetails {
	pub claim_hash: TestClaimHash,
	pub ctype_hash: TestCtypeHash,
	pub delegation_id: Option<TestDelegationNodeId>,
}

pub fn generate_base_attestation_creation_details(
	claim_hash: TestClaimHash,
	attestation: AttestationStruct<Test>,
) -> AttestationCreationDetails {
	AttestationCreationDetails {
		claim_hash,
		ctype_hash: attestation.ctype_hash,
		delegation_id: attestation.delegation_id,
	}
}

pub struct AttestationRevocationDetails {
	pub claim_hash: TestClaimHash,
	pub max_parent_checks: u32,
}

pub fn generate_base_attestation_revocation_details(claim_hash: TestClaimHash) -> AttestationRevocationDetails {
	AttestationRevocationDetails {
		claim_hash,
		max_parent_checks: 0u32,
	}
}

pub fn generate_base_attestation(attester: TestAttester) -> AttestationStruct<Test> {
	AttestationStruct {
		attester,
		delegation_id: None,
		ctype_hash: ctype_mock::get_ctype_hash(true),
		revoked: false,
	}
}

#[derive(Clone)]
pub struct ExtBuilder {
	delegation_builder: Option<delegation_mock::ExtBuilder>,
	attestations_stored: Vec<(TestClaimHash, AttestationStruct<Test>)>,
	delegated_attestations_stored: Vec<(TestDelegationNodeId, Vec<TestClaimHash>)>,
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
		Self {
			delegation_builder: Some(delegation_builder),
			..Default::default()
		}
	}
}

impl ExtBuilder {
	pub fn with_attestations(mut self, attestations: Vec<(TestClaimHash, AttestationStruct<Test>)>) -> Self {
		self.attestations_stored = attestations;
		self
	}

	pub fn with_delegated_attestations(
		mut self,
		delegated_attestations: Vec<(TestDelegationNodeId, Vec<TestClaimHash>)>,
	) -> Self {
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
				self.delegated_attestations_stored
					.iter()
					.for_each(|delegated_attestation| {
						attestation::DelegatedAttestations::<Test>::insert(
							delegated_attestation.0,
							delegated_attestation.1.clone(),
						);
					})
			});
		}

		ext
	}
}
