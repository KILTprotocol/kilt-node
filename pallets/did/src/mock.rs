// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

#[cfg(not(feature = "runtime-benchmarks"))]
use crate::{DidRawOrigin, EnsureDidOrigin};

use frame_support::{
	parameter_types,
	traits::{
		fungible::{Balanced, Credit, MutateHold},
		OnUnbalanced,
	},
	weights::constants::RocksDbWeight,
};
use frame_system::EnsureSigned;
use pallet_balances::Pallet as PalletBalance;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::{ecdsa, ed25519, sr25519, Pair};
use sp_runtime::{
	testing::H256,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage, MultiSignature, MultiSigner, SaturatedConversion,
};
use sp_std::vec::Vec;

use crate::{
	self as did,
	did_details::{
		DeriveDidCallAuthorizationVerificationKeyRelationship, DeriveDidCallKeyRelationshipResult,
		DidAuthorizedCallOperation, DidAuthorizedCallOperationWithVerificationRelationship, DidDetails,
		DidEncryptionKey, DidPublicKey, DidPublicKeyDetails, DidVerificationKey, DidVerificationKeyRelationship,
		RelationshipDeriveError,
	},
	service_endpoints::DidEndpoint,
	utils as crate_utils, AccountIdOf, Config, CurrencyOf, DidBlacklist, DidEndpointsCount, HoldReason, KeyIdOf,
	ServiceEndpoints,
};

pub(crate) type Block = frame_system::mocking::MockBlock<Test>;
pub(crate) type Hash = sp_core::H256;
pub(crate) type Balance = u128;
pub(crate) type Signature = MultiSignature;
pub(crate) type AccountPublic = <Signature as Verify>::Signer;
pub(crate) type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
type CreditOf<T> = Credit<<T as frame_system::Config>::AccountId, PalletBalance<T, ()>>;
pub(crate) type DidIdentifier = AccountId;
pub(crate) type CtypeHash = Hash;

const MICRO_KILT: Balance = 10u128.pow(9);
const MILLI_KILT: Balance = 10u128.pow(12);
const KILT: Balance = 10u128.pow(15);

frame_support::construct_runtime!(
	pub enum Test
	{
		Did: did,
		Ctype: ctype,
		Balances: pallet_balances,
		System: frame_system,
	}
);

parameter_types! {
	pub const SS58Prefix: u8 = 38;
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type RuntimeTask = ();
	type Block = Block;
	type Nonce = u64;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = ();
	type BlockHashCount = BlockHashCount;
	type DbWeight = RocksDbWeight;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type MultiBlockMigrator = ();
	type SingleBlockMigrations = ();
	type PostInherents = ();
	type PostTransactions = ();
	type PreInherents = ();
}

parameter_types! {
	#[derive(Clone, TypeInfo, Debug, PartialEq, Eq, Encode, Decode)]
	pub const MaxNewKeyAgreementKeys: u32 = 10u32;
	#[derive(Debug, Clone, Eq, PartialEq)]
	pub const MaxTotalKeyAgreementKeys: u32 = 10u32;
	// IMPORTANT: Needs to be at least MaxTotalKeyAgreementKeys + 3 (auth, delegation, attestation keys) for benchmarks!
	#[derive(Debug, Clone)]
	pub const MaxPublicKeysPerDid: u32 = 13u32;
	pub const MaxBlocksTxValidity: u64 = 300u64;
	pub const DidFee: Balance = MICRO_KILT;
	pub const MaxNumberOfServicesPerDid: u32 = 25u32;
	pub const MaxServiceIdLength: u32 = 50u32;
	pub const MaxServiceTypeLength: u32 = 50u32;
	pub const MaxServiceUrlLength: u32 = 100u32;
	pub const MaxNumberOfTypesPerService: u32 = 1u32;
	pub const MaxNumberOfUrlsPerService: u32 = 1u32;
	pub const KeyDeposit :Balance = 32 * MICRO_KILT;
	pub const ServiceEndpointDeposit :Balance = 50 * MICRO_KILT;
	pub const BaseDeposit: Balance = 100 * MILLI_KILT;
}

pub struct ToAccount<R>(sp_std::marker::PhantomData<R>);

impl<R> OnUnbalanced<CreditOf<R>> for ToAccount<R>
where
	R: pallet_balances::Config,
	<R as frame_system::Config>::AccountId: From<AccountId>,
{
	fn on_nonzero_unbalanced(amount: CreditOf<R>) {
		let _ = pallet_balances::Pallet::<R>::resolve(&ACCOUNT_FEE.into(), amount);
	}
}

impl Config for Test {
	#[cfg(feature = "runtime-benchmarks")]
	type EnsureOrigin = EnsureSigned<DidIdentifier>;
	#[cfg(feature = "runtime-benchmarks")]
	type OriginSuccess = AccountId;

	#[cfg(not(feature = "runtime-benchmarks"))]
	type EnsureOrigin = EnsureDidOrigin<DidIdentifier, AccountId>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type OriginSuccess = DidRawOrigin<AccountId, DidIdentifier>;

	type DidIdentifier = DidIdentifier;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type KeyDeposit = KeyDeposit;
	type RuntimeHoldReason = RuntimeHoldReason;
	type ServiceEndpointDeposit = KeyDeposit;
	type RuntimeEvent = ();
	type Currency = Balances;
	type BaseDeposit = BaseDeposit;
	type Fee = DidFee;
	type FeeCollector = ToAccount<Test>;
	type MaxNewKeyAgreementKeys = MaxNewKeyAgreementKeys;
	type MaxTotalKeyAgreementKeys = MaxTotalKeyAgreementKeys;
	type MaxPublicKeysPerDid = MaxPublicKeysPerDid;
	type MaxBlocksTxValidity = MaxBlocksTxValidity;
	type WeightInfo = ();
	type MaxNumberOfServicesPerDid = MaxNumberOfServicesPerDid;
	type MaxServiceIdLength = MaxServiceIdLength;
	type MaxServiceTypeLength = MaxServiceTypeLength;
	type MaxServiceUrlLength = MaxServiceUrlLength;
	type MaxNumberOfTypesPerService = MaxNumberOfTypesPerService;
	type MaxNumberOfUrlsPerService = MaxNumberOfUrlsPerService;
	type BalanceMigrationManager = ();
	type DidLifecycleHooks = ();
}

parameter_types! {
	pub const ExistentialDeposit: Balance = 500;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
	pub const MaxFreezes: u32 = 50;
}

impl pallet_balances::Config for Test {
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = RuntimeFreezeReason;
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxFreezes = MaxFreezes;
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub const Fee: Balance = 0;
}

impl ctype::Config for Test {
	#[cfg(feature = "runtime-benchmarks")]
	type EnsureOrigin = EnsureSigned<DidIdentifier>;
	#[cfg(feature = "runtime-benchmarks")]
	type OriginSuccess = AccountId;

	#[cfg(not(feature = "runtime-benchmarks"))]
	type EnsureOrigin = EnsureDidOrigin<DidIdentifier, AccountId>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type OriginSuccess = DidRawOrigin<AccountId, DidIdentifier>;

	type CtypeCreatorId = DidIdentifier;
	type OverarchingOrigin = EnsureSigned<AccountId>;
	type RuntimeEvent = ();
	type WeightInfo = ();
	type Currency = Balances;
	type Fee = Fee;
	type FeeCollector = ();
}

pub(crate) const DEFAULT_BALANCE: Balance = 10 * KILT;

pub(crate) const ACCOUNT_00: AccountId = AccountId::new([1u8; 32]);
pub(crate) const ACCOUNT_01: AccountId = AccountId::new([2u8; 32]);
pub(crate) const ACCOUNT_02: AccountId = AccountId::new([3u8; 32]);
pub(crate) const ACCOUNT_FEE: AccountId = AccountId::new([u8::MAX; 32]);

pub(crate) const AUTH_SEED_0: [u8; 32] = [4u8; 32];
pub(crate) const AUTH_SEED_1: [u8; 32] = [40u8; 32];
pub(crate) const ENC_SEED_0: [u8; 32] = [254u8; 32];
pub(crate) const ENC_SEED_1: [u8; 32] = [255u8; 32];
pub(crate) const ATT_SEED_0: [u8; 32] = [6u8; 32];
pub(crate) const ATT_SEED_1: [u8; 32] = [60u8; 32];
pub(crate) const DEL_SEED_0: [u8; 32] = [7u8; 32];
pub(crate) const DEL_SEED_1: [u8; 32] = [70u8; 32];

/// Solely used to fill public keys in unit tests to check for correct error
/// throws. Thus, it does not matter whether the correct key types get added
/// such that we can use the ed25519 for all key types per default.
pub(crate) fn fill_public_keys(mut did_details: DidDetails<Test>) -> DidDetails<Test> {
	while (did_details.public_keys.len() as u32) < <Test as Config>::MaxPublicKeysPerDid::get() {
		did_details
			.public_keys
			.try_insert(
				H256::random(),
				DidPublicKeyDetails {
					key: DidPublicKey::from(DidVerificationKey::from(ed25519::Public::from_h256(H256::random()))),
					block_number: 0u64,
				},
			)
			.expect("Should not exceed BoundedBTreeMap size due to prior check");
	}
	did_details
}

pub fn get_did_identifier_from_ed25519_key(public_key: ed25519::Public) -> DidIdentifier {
	MultiSigner::from(public_key).into_account()
}

pub fn get_did_identifier_from_sr25519_key(public_key: sr25519::Public) -> DidIdentifier {
	MultiSigner::from(public_key).into_account()
}

pub fn get_did_identifier_from_ecdsa_key(public_key: ecdsa::Public) -> DidIdentifier {
	MultiSigner::from(public_key).into_account()
}

pub fn get_ed25519_authentication_key(seed: &[u8; 32]) -> ed25519::Pair {
	ed25519::Pair::from_seed(seed)
}

pub fn get_sr25519_authentication_key(seed: &[u8; 32]) -> sr25519::Pair {
	sr25519::Pair::from_seed(seed)
}

pub fn get_ecdsa_authentication_key(seed: &[u8; 32]) -> ecdsa::Pair {
	ecdsa::Pair::from_seed(seed)
}

pub fn get_x25519_encryption_key(seed: &[u8; 32]) -> DidEncryptionKey {
	DidEncryptionKey::X25519(*seed)
}

pub fn get_ed25519_attestation_key(seed: &[u8; 32]) -> ed25519::Pair {
	ed25519::Pair::from_seed(seed)
}

pub fn get_sr25519_attestation_key(seed: &[u8; 32]) -> sr25519::Pair {
	sr25519::Pair::from_seed(seed)
}

pub fn get_ecdsa_attestation_key(seed: &[u8; 32]) -> ecdsa::Pair {
	ecdsa::Pair::from_seed(seed)
}

pub fn get_ed25519_delegation_key(seed: &[u8; 32]) -> ed25519::Pair {
	ed25519::Pair::from_seed(seed)
}

pub fn get_sr25519_delegation_key(seed: &[u8; 32]) -> sr25519::Pair {
	sr25519::Pair::from_seed(seed)
}

pub fn get_ecdsa_delegation_key(seed: &[u8; 32]) -> ecdsa::Pair {
	ecdsa::Pair::from_seed(seed)
}

pub fn generate_key_id(key: &DidPublicKey<AccountId>) -> KeyIdOf<Test> {
	crate_utils::calculate_key_id::<Test>(key)
}

pub(crate) fn get_attestation_key_test_input() -> Vec<u8> {
	[0u8; 32].to_vec()
}
pub(crate) fn get_attestation_key_call() -> RuntimeCall {
	RuntimeCall::Ctype(ctype::Call::add {
		ctype: get_attestation_key_test_input(),
	})
}
pub(crate) fn get_authentication_key_test_input() -> Vec<u8> {
	[1u8; 32].to_vec()
}
pub(crate) fn get_authentication_key_call() -> RuntimeCall {
	RuntimeCall::Ctype(ctype::Call::add {
		ctype: get_authentication_key_test_input(),
	})
}
pub(crate) fn get_delegation_key_test_input() -> Vec<u8> {
	[2u8; 32].to_vec()
}
pub(crate) fn get_delegation_key_call() -> RuntimeCall {
	RuntimeCall::Ctype(ctype::Call::add {
		ctype: get_delegation_key_test_input(),
	})
}
pub(crate) fn get_none_key_test_input() -> Vec<u8> {
	[3u8; 32].to_vec()
}
pub(crate) fn get_none_key_call() -> RuntimeCall {
	RuntimeCall::Ctype(ctype::Call::add {
		ctype: get_none_key_test_input(),
	})
}

#[cfg(not(feature = "runtime-benchmarks"))]
pub(crate) fn build_test_origin(account: AccountId, did: DidIdentifier) -> RuntimeOrigin {
	crate::DidRawOrigin::new(account, did).into()
}

#[cfg(feature = "runtime-benchmarks")]
pub(crate) fn build_test_origin(account: AccountId, _did: DidIdentifier) -> RuntimeOrigin {
	RuntimeOrigin::signed(account)
}

impl DeriveDidCallAuthorizationVerificationKeyRelationship for RuntimeCall {
	fn derive_verification_key_relationship(&self) -> DeriveDidCallKeyRelationshipResult {
		if *self == get_attestation_key_call() {
			Ok(DidVerificationKeyRelationship::AssertionMethod)
		} else if *self == get_authentication_key_call() {
			Ok(DidVerificationKeyRelationship::Authentication)
		} else if *self == get_delegation_key_call() {
			Ok(DidVerificationKeyRelationship::CapabilityDelegation)
		} else {
			#[cfg(feature = "runtime-benchmarks")]
			if *self == Self::get_call_for_did_call_benchmark() {
				// Always require an authentication key to dispatch calls during benchmarking
				return Ok(DidVerificationKeyRelationship::Authentication);
			}
			Err(RelationshipDeriveError::NotCallableByDid)
		}
	}

	// Always return a System::remark() extrinsic call
	#[cfg(feature = "runtime-benchmarks")]
	fn get_call_for_did_call_benchmark() -> Self {
		RuntimeCall::System(frame_system::Call::remark { remark: sp_std::vec![] })
	}
}

pub fn generate_test_did_call(
	verification_key_required: DidVerificationKeyRelationship,
	caller: DidIdentifier,
	submitter: AccountId,
) -> DidAuthorizedCallOperationWithVerificationRelationship<Test> {
	let call = match verification_key_required {
		DidVerificationKeyRelationship::AssertionMethod => get_attestation_key_call(),
		DidVerificationKeyRelationship::Authentication => get_authentication_key_call(),
		DidVerificationKeyRelationship::CapabilityDelegation => get_delegation_key_call(),
		_ => get_none_key_call(),
	};
	DidAuthorizedCallOperationWithVerificationRelationship {
		operation: DidAuthorizedCallOperation {
			did: caller,
			call,
			tx_counter: 1u64,
			block_number: 0u64,
			submitter,
		},
		verification_key_relationship: verification_key_required,
	}
}
#[cfg(test)]
#[allow(unused_must_use)]
pub(crate) fn initialize_logger() {
	env_logger::builder().is_test(true).try_init();
}

#[derive(Clone, Default)]
pub(crate) struct ExtBuilder {
	dids_stored: Vec<(DidIdentifier, DidDetails<Test>)>,
	service_endpoints: Vec<(DidIdentifier, Vec<DidEndpoint<Test>>)>,
	deleted_dids: Vec<DidIdentifier>,
	ctypes_stored: Vec<(CtypeHash, DidIdentifier)>,
	balances: Vec<(AccountIdOf<Test>, Balance)>,
}

impl ExtBuilder {
	#[must_use]
	pub fn with_dids(mut self, dids: Vec<(DidIdentifier, DidDetails<Test>)>) -> Self {
		self.dids_stored = dids;
		self
	}

	#[must_use]
	pub fn with_endpoints(mut self, endpoints: Vec<(DidIdentifier, Vec<DidEndpoint<Test>>)>) -> Self {
		self.service_endpoints = endpoints;
		self
	}

	#[must_use]
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountIdOf<Test>, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	#[must_use]
	pub fn with_ctypes(mut self, ctypes: Vec<(CtypeHash, DidIdentifier)>) -> Self {
		self.ctypes_stored = ctypes;
		self
	}

	#[must_use]
	pub fn with_deleted_dids(mut self, dids: Vec<DidIdentifier>) -> Self {
		self.deleted_dids = dids;
		self
	}

	pub fn build(self, ext: Option<sp_io::TestExternalities>) -> sp_io::TestExternalities {
		let mut ext = if let Some(ext) = ext {
			ext
		} else {
			let mut storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
			pallet_balances::GenesisConfig::<Test> {
				balances: self.balances.clone(),
			}
			.assimilate_storage(&mut storage)
			.expect("assimilate should not fail");
			sp_io::TestExternalities::new(storage)
		};

		ext.execute_with(|| {
			for (ctype_hash, owner) in self.ctypes_stored.iter() {
				ctype::Ctypes::<Test>::insert(
					ctype_hash,
					ctype::CtypeEntryOf::<Test> {
						creator: owner.to_owned(),
						created_at: System::block_number(),
					},
				);
			}

			for did in self.dids_stored.iter() {
				did::Did::<Test>::insert(&did.0, did.1.clone());
				CurrencyOf::<Test>::hold(&HoldReason::Deposit.into(), &did.1.deposit.owner, did.1.deposit.amount)
					.expect("Deposit owner should have enough balance");
			}
			for did in self.deleted_dids.iter() {
				DidBlacklist::<Test>::insert(did, ());
			}
			for (did, endpoints) in self.service_endpoints.iter() {
				for endpoint in endpoints.iter() {
					ServiceEndpoints::<Test>::insert(did, &endpoint.id, endpoint)
				}
				DidEndpointsCount::<Test>::insert(did, endpoints.len().saturated_into::<u32>());
			}
		});

		ext
	}

	pub fn build_and_execute_with_sanity_tests(self, ext: Option<sp_io::TestExternalities>, test: impl FnOnce()) {
		self.build(ext).execute_with(|| {
			test();
			crate::try_state::do_try_state::<Test>().expect("Sanity test for did failed.");
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub fn build_with_keystore(self) -> sp_io::TestExternalities {
		let mut ext = self.build(None);

		let keystore = sp_keystore::testing::MemoryKeystore::new();
		ext.register_extension(sp_keystore::KeystoreExt(sp_std::sync::Arc::new(keystore)));

		ext
	}
}
