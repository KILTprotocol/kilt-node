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
use crate::*;
use ctype::mock as ctype_mock;

use frame_support::{parameter_types, weights::constants::RocksDbWeight};
use frame_system::EnsureSigned;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
};

pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub type Block = frame_system::mocking::MockBlock<Test>;

pub type TestDidIdentifier = kilt_primitives::AccountId;
pub type TestCtypeOwner = kilt_primitives::AccountId;
pub type TestCtypeHash = kilt_primitives::Hash;
pub type TestDelegationNodeId = kilt_primitives::Hash;
pub type TestDelegatorId = kilt_primitives::AccountId;
pub type TestClaimHash = kilt_primitives::Hash;
pub type TestAttester = TestDelegatorId;

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

impl delegation::Config for Test {
	type DelegationNodeId = TestDelegationNodeId;
	type EnsureOrigin = EnsureSigned<TestDelegatorId>;
	type Event = ();
}

impl Config for Test {
	type EnsureOrigin = EnsureSigned<TestAttester>;
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
pub(crate) const ALICE: TestAttester = TestAttester::new([0u8; 32]);
#[cfg(test)]
pub(crate) const BOB: TestAttester = TestAttester::new([1u8; 32]);

const DEFAULT_CLAIM_HASH_SEED: u64 = 1u64;
const ALTERNATIVE_CLAIM_HASH_SEED: u64 = 2u64;

pub fn get_origin(account: TestAttester) -> Origin {
	Origin::signed(account)
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
	attestation: AttestationDetails<Test>,
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

pub fn generate_base_attestation(attester: TestAttester) -> AttestationDetails<Test> {
	AttestationDetails {
		attester,
		delegation_id: None,
		ctype_hash: ctype_mock::get_ctype_hash(true),
		revoked: false,
	}
}

#[derive(Clone)]
pub struct ExtBuilder {
	attestations_stored: Vec<(TestClaimHash, AttestationDetails<Test>)>,
	delegated_attestations_stored: Vec<(TestDelegationNodeId, Vec<TestClaimHash>)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			attestations_stored: vec![],
			delegated_attestations_stored: vec![],
		}
	}
}

impl ExtBuilder {
	pub fn with_attestations(mut self, attestations: Vec<(TestClaimHash, AttestationDetails<Test>)>) -> Self {
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

	pub fn build(self, ext: Option<sp_io::TestExternalities>) -> sp_io::TestExternalities {
		let mut ext = if let Some(ext) = ext {
			ext
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
