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
#![allow(dead_code)]

use frame_support::{parameter_types, weights::constants::RocksDbWeight};
use frame_system::EnsureSigned;
use kilt_primitives::AccountId;
use sp_core::{ecdsa, ed25519, sr25519, Pair};
use sp_keystore::{testing::KeyStore, KeystoreExt};
use sp_runtime::{
	testing::{Header, H256},
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSigner,
};
use sp_std::sync::Arc;

use crate as did;
use crate::*;

pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub type Block = frame_system::mocking::MockBlock<Test>;

pub type TestDidIdentifier = kilt_primitives::AccountId;
pub type TestKeyId = did::KeyIdOf<Test>;
pub type TestBlockNumber = kilt_primitives::BlockNumber;
pub type TestCtypeOwner = TestDidIdentifier;
pub type TestCtypeHash = kilt_primitives::Hash;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		Did: did::{Pallet, Call, Storage, Event<T>, Origin<T>},
		Ctype: ctype::{Pallet, Call, Storage, Event<T>},
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
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
	type BaseCallFilter = frame_support::traits::Everything;
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const MaxNewKeyAgreementKeys: u32 = 10u32;
	#[derive(Debug, Clone, PartialEq)]
	pub const MaxUrlLength: u32 = 200u32;
	#[derive(Debug, Clone, PartialEq)]
	pub const MaxTotalKeyAgreementKeys: u32 = 10u32;
	// IMPORTANT: Needs to be at least MaxTotalKeyAgreementKeys + 3 (auth, delegation, attestation keys) for benchmarks!
	#[derive(Debug, Clone)]
	pub const MaxPublicKeysPerDid: u32 = 13u32;
	pub const MaxBlocksTxValidity: u64 = 300u64;
}

impl Config for Test {
	type DidIdentifier = TestDidIdentifier;
	type Origin = Origin;
	type Call = Call;
	type EnsureOrigin = EnsureSigned<TestDidIdentifier>;
	type OriginSuccess = AccountId;
	type Event = ();
	type MaxNewKeyAgreementKeys = MaxNewKeyAgreementKeys;
	type MaxTotalKeyAgreementKeys = MaxTotalKeyAgreementKeys;
	type MaxPublicKeysPerDid = MaxPublicKeysPerDid;
	type MaxBlocksTxValidity = MaxBlocksTxValidity;
	type WeightInfo = ();
}

impl ctype::Config for Test {
	type CtypeCreatorId = TestCtypeOwner;
	type Event = ();
	type WeightInfo = ();

	#[cfg(feature = "runtime-benchmarks")]
	type EnsureOrigin = EnsureSigned<TestDidIdentifier>;
	#[cfg(feature = "runtime-benchmarks")]
	type OriginSuccess = kilt_primitives::AccountId;

	#[cfg(not(feature = "runtime-benchmarks"))]
	type EnsureOrigin = did::EnsureDidOrigin<TestCtypeOwner, kilt_primitives::AccountId>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type OriginSuccess = did::DidRawOrigin<kilt_primitives::AccountId, TestCtypeOwner>;
}

#[cfg(test)]
pub(crate) const ACCOUNT_00: kilt_primitives::AccountId = kilt_primitives::AccountId::new([0u8; 32]);
#[cfg(test)]
pub(crate) const ACCOUNT_01: kilt_primitives::AccountId = kilt_primitives::AccountId::new([1u8; 32]);

const DEFAULT_AUTH_SEED: [u8; 32] = [4u8; 32];
const ALTERNATIVE_AUTH_SEED: [u8; 32] = [40u8; 32];
const DEFAULT_ENC_SEED: [u8; 32] = [254u8; 32];
const ALTERNATIVE_ENC_SEED: [u8; 32] = [255u8; 32];
const DEFAULT_ATT_SEED: [u8; 32] = [6u8; 32];
const ALTERNATIVE_ATT_SEED: [u8; 32] = [60u8; 32];
const DEFAULT_DEL_SEED: [u8; 32] = [7u8; 32];
const ALTERNATIVE_DEL_SEED: [u8; 32] = [70u8; 32];

/// Solely used to fill public keys in unit tests to check for correct error
/// throws. Thus, it does not matter whether the correct key types get added
/// such that we can use the ed25519 for all key types per default.
pub(crate) fn fill_public_keys(mut did_details: DidDetails<Test>) -> DidDetails<Test> {
	while (did_details.public_keys.len() as u32) < <Test as Config>::MaxPublicKeysPerDid::get() {
		did_details
			.public_keys
			.try_insert(
				H256::random(),
				did::DidPublicKeyDetails {
					key: did::DidPublicKey::from(did::DidVerificationKey::from(ed25519::Public::from_h256(
						H256::random(),
					))),
					block_number: 0u64,
				},
			)
			.expect("Should not exceed BoundedBTreeMap size due to prior check");
	}
	did_details
}

pub fn get_did_identifier_from_ed25519_key(public_key: ed25519::Public) -> TestDidIdentifier {
	MultiSigner::from(public_key).into_account()
}

pub fn get_did_identifier_from_sr25519_key(public_key: sr25519::Public) -> TestDidIdentifier {
	MultiSigner::from(public_key).into_account()
}

pub fn get_did_identifier_from_ecdsa_key(public_key: ecdsa::Public) -> TestDidIdentifier {
	MultiSigner::from(public_key).into_account()
}

pub fn get_ed25519_authentication_key(default: bool) -> ed25519::Pair {
	if default {
		ed25519::Pair::from_seed(&DEFAULT_AUTH_SEED)
	} else {
		ed25519::Pair::from_seed(&ALTERNATIVE_AUTH_SEED)
	}
}

pub fn get_sr25519_authentication_key(default: bool) -> sr25519::Pair {
	if default {
		sr25519::Pair::from_seed(&DEFAULT_AUTH_SEED)
	} else {
		sr25519::Pair::from_seed(&ALTERNATIVE_AUTH_SEED)
	}
}

pub fn get_ecdsa_authentication_key(default: bool) -> ecdsa::Pair {
	if default {
		ecdsa::Pair::from_seed(&DEFAULT_AUTH_SEED)
	} else {
		ecdsa::Pair::from_seed(&ALTERNATIVE_AUTH_SEED)
	}
}

pub fn get_x25519_encryption_key(default: bool) -> DidEncryptionKey {
	if default {
		DidEncryptionKey::X25519(DEFAULT_ENC_SEED)
	} else {
		DidEncryptionKey::X25519(ALTERNATIVE_ENC_SEED)
	}
}

pub fn get_ed25519_attestation_key(default: bool) -> ed25519::Pair {
	if default {
		ed25519::Pair::from_seed(&DEFAULT_ATT_SEED)
	} else {
		ed25519::Pair::from_seed(&ALTERNATIVE_ATT_SEED)
	}
}

pub fn get_sr25519_attestation_key(default: bool) -> sr25519::Pair {
	if default {
		sr25519::Pair::from_seed(&DEFAULT_ATT_SEED)
	} else {
		sr25519::Pair::from_seed(&ALTERNATIVE_ATT_SEED)
	}
}

pub fn get_ecdsa_attestation_key(default: bool) -> ecdsa::Pair {
	if default {
		ecdsa::Pair::from_seed(&DEFAULT_ATT_SEED)
	} else {
		ecdsa::Pair::from_seed(&ALTERNATIVE_ATT_SEED)
	}
}

pub fn get_ed25519_delegation_key(default: bool) -> ed25519::Pair {
	if default {
		ed25519::Pair::from_seed(&DEFAULT_DEL_SEED)
	} else {
		ed25519::Pair::from_seed(&ALTERNATIVE_DEL_SEED)
	}
}

pub fn get_sr25519_delegation_key(default: bool) -> sr25519::Pair {
	if default {
		sr25519::Pair::from_seed(&DEFAULT_DEL_SEED)
	} else {
		sr25519::Pair::from_seed(&ALTERNATIVE_DEL_SEED)
	}
}

pub fn get_ecdsa_delegation_key(default: bool) -> ecdsa::Pair {
	if default {
		ecdsa::Pair::from_seed(&DEFAULT_DEL_SEED)
	} else {
		ecdsa::Pair::from_seed(&ALTERNATIVE_DEL_SEED)
	}
}

pub fn generate_key_id(key: &did::DidPublicKey) -> TestKeyId {
	utils::calculate_key_id::<Test>(key)
}

pub(crate) fn get_attestation_key_test_input() -> TestCtypeHash {
	TestCtypeHash::from_slice(&[0u8; 32])
}
pub(crate) fn get_attestation_key_call() -> Call {
	Call::Ctype(ctype::Call::add(get_attestation_key_test_input()))
}
pub(crate) fn get_authentication_key_test_input() -> TestCtypeHash {
	TestCtypeHash::from_slice(&[1u8; 32])
}
pub(crate) fn get_authentication_key_call() -> Call {
	Call::Ctype(ctype::Call::add(get_authentication_key_test_input()))
}
pub(crate) fn get_delegation_key_test_input() -> TestCtypeHash {
	TestCtypeHash::from_slice(&[2u8; 32])
}
pub(crate) fn get_delegation_key_call() -> Call {
	Call::Ctype(ctype::Call::add(get_delegation_key_test_input()))
}
pub(crate) fn get_none_key_test_input() -> TestCtypeHash {
	TestCtypeHash::from_slice(&[3u8; 32])
}
pub(crate) fn get_none_key_call() -> Call {
	Call::Ctype(ctype::Call::add(get_none_key_test_input()))
}

impl did::DeriveDidCallAuthorizationVerificationKeyRelationship for Call {
	fn derive_verification_key_relationship(&self) -> Option<did::DidVerificationKeyRelationship> {
		if *self == get_attestation_key_call() {
			Some(did::DidVerificationKeyRelationship::AssertionMethod)
		} else if *self == get_authentication_key_call() {
			Some(did::DidVerificationKeyRelationship::Authentication)
		} else if *self == get_delegation_key_call() {
			Some(did::DidVerificationKeyRelationship::CapabilityDelegation)
		} else {
			#[cfg(feature = "runtime-benchmarks")]
			if *self == Self::get_call_for_did_call_benchmark() {
				// Always require an authentication key to dispatch calls during benchmarking
				return Some(did::DidVerificationKeyRelationship::Authentication);
			}
			None
		}
	}

	// Always return a System::remark() extrinsic call
	#[cfg(feature = "runtime-benchmarks")]
	fn get_call_for_did_call_benchmark() -> Self {
		Call::System(frame_system::Call::remark(vec![]))
	}
}

pub fn generate_test_did_call(
	verification_key_required: did::DidVerificationKeyRelationship,
	caller: TestDidIdentifier,
	submitter: kilt_primitives::AccountId,
) -> did::DidAuthorizedCallOperationWithVerificationRelationship<Test> {
	let call = match verification_key_required {
		DidVerificationKeyRelationship::AssertionMethod => get_attestation_key_call(),
		DidVerificationKeyRelationship::Authentication => get_authentication_key_call(),
		DidVerificationKeyRelationship::CapabilityDelegation => get_delegation_key_call(),
		_ => get_none_key_call(),
	};
	did::DidAuthorizedCallOperationWithVerificationRelationship {
		operation: did::DidAuthorizedCallOperation {
			did: caller,
			call,
			tx_counter: 1u64,
			block_number: 0u64,
			submitter,
		},
		verification_key_relationship: verification_key_required,
	}
}

#[allow(unused_must_use)]
pub fn initialize_logger() {
	env_logger::builder().is_test(true).try_init();
}

#[derive(Clone, Default)]
pub struct ExtBuilder {
	dids_stored: Vec<(TestDidIdentifier, did::DidDetails<Test>)>,
	deleted_dids: Vec<TestDidIdentifier>,
	storage_version: DidStorageVersion,
}

impl ExtBuilder {
	pub fn with_dids(mut self, dids: Vec<(TestDidIdentifier, did::DidDetails<Test>)>) -> Self {
		self.dids_stored = dids;
		self
	}

	pub fn with_deleted_dids(mut self, dids: Vec<TestDidIdentifier>) -> Self {
		self.deleted_dids = dids;
		self
	}

	pub fn with_storage_version(mut self, storage_version: DidStorageVersion) -> Self {
		self.storage_version = storage_version;
		self
	}

	pub fn build(self, ext: Option<sp_io::TestExternalities>) -> sp_io::TestExternalities {
		let mut ext = if let Some(ext) = ext {
			ext
		} else {
			let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
			sp_io::TestExternalities::new(storage)
		};

		ext.execute_with(|| {
			for did in self.dids_stored.iter() {
				did::Did::<Test>::insert(did.0.clone(), did.1.clone());
			}
			for did in self.deleted_dids.iter() {
				did::DidBlacklist::<Test>::insert(did.clone(), ());
			}
			did::StorageVersion::<Test>::set(self.storage_version);
		});

		ext
	}

	// allowance only required for clippy, this function is actually used
	#[allow(dead_code)]
	pub fn build_with_keystore(self, ext: Option<sp_io::TestExternalities>) -> sp_io::TestExternalities {
		let mut ext = self.build(ext);

		let keystore = KeyStore::new();
		ext.register_extension(KeystoreExt(Arc::new(keystore)));

		ext
	}
}
