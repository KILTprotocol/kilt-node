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

//! The KILT runtime. This can be compiled with `#[no_std]`, ready for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
// The `from_over_into` warning originates from `construct_runtime` macro.
#![allow(clippy::from_over_into)]
// Triggered by `impl_runtime_apis` macro
#![allow(clippy::empty_structs_with_brackets)]

// Make the WASM binary available
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use frame_support::{
	construct_runtime,
	genesis_builder_helper::{build_state, get_preset},
	parameter_types,
	traits::{Everything, InstanceFilter},
	weights::{constants::RocksDbWeight, ConstantMultiplier, IdentityFee, Weight},
};
pub use frame_system::Call as SystemCall;
use frame_system::EnsureRoot;
use pallet_web3_names::Web3NameOf;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};

#[cfg(feature = "try-runtime")]
use frame_try_runtime::UpgradeCheckSelect;

use pallet_grandpa::{fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use pallet_transaction_payment::{FeeDetails, FungibleAdapter};
use scale_info::TypeInfo;
use sp_api::impl_runtime_apis;
use sp_consensus_aura::{ed25519::AuthorityId as AuraId, SlotDuration};
use sp_core::{ConstBool, ConstU32, ConstU64, OpaqueMetadata};
use sp_genesis_builder::PresetId;
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, NumberFor, OpaqueKeys},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, RuntimeDebug,
};
use sp_std::prelude::*;
use sp_version::RuntimeVersion;

use delegation::DelegationAc;
use kilt_support::traits::ItemFilter;
use pallet_did_lookup::linkable_account::LinkableAccountId;
use runtime_common::{
	assets::{AssetDid, PublicCredentialsFilter},
	authorization::{AuthorizationId, PalletAuthorize},
	constants::{self, EXISTENTIAL_DEPOSIT, KILT},
	errors::PublicCredentialsApiError,
	AccountId, Balance, BlockNumber, DidIdentifier, Hash, Nonce, Signature, SlowAdjustingFeeUpdate, Web3Name,
};

pub use pallet_timestamp::Call as TimestampCall;
pub use sp_runtime::{Perbill, Permill};

pub use attestation;
pub use ctype;
pub use delegation;
pub use did;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_web3_names;
pub use public_credentials;

#[cfg(feature = "runtime-benchmarks")]
use frame_system::EnsureSigned;
#[cfg(feature = "std")]
use sp_version::NativeVersion;

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

/// Digest item type.
pub type DigestItem = generic::DigestItem;

/// Opaque types. These are used by the CLI to instantiate machinery that don't
/// need to know the specifics of the runtime. They can then be made to be
/// agnostic over specific formats of data like extrinsics, allowing for them to
/// continue syncing the network through upgrades to even the core data
/// structures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
		}
	}
}

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("kilt-kestrel"),
	impl_name: create_runtime_str!("kilt-kestrel"),
	authoring_version: 4,
	spec_version: 11600,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 6,
	state_version: 0,
};

/// The version information used to identify this runtime when compiled
/// natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const BlockHashCount: BlockNumber = 2400;
	pub const SS58Prefix: u8 = 38;
}

// Configure FRAME pallets to include in runtime.
impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = Everything;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = runtime_common::BlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = runtime_common::BlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	/// The lookup mechanism to get account ID from whatever is passed in
	/// dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The block type as expected in this runtime
	type Block = Block;
	/// The nonce type for storing how many extrinsics an account has signed.
	type Nonce = Nonce;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeTask = RuntimeTask;
	/// Maximum number of block number to block hash mappings to keep (oldest
	/// pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// Converts a Pallet to the index of the Pallet in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// This is used as an identifier of the chain. 42 is the generic substrate
	/// prefix.
	type SS58Prefix = SS58Prefix;
	/// The set code logic, just the default since we're not a parachain.
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
	type MultiBlockMigrator = ();
	type PostInherents = ();
	type PostTransactions = ();
	type PreInherents = ();
	type SingleBlockMigrations = ();
}

/// Maximum number of nominators per validator.
const MAX_NOMINATORS: u32 = 8;

parameter_types! {
	pub const MaxAuthorities: u32 = constants::staking::MAX_CANDIDATES;
	pub const MaxNominators: u32 = MAX_NOMINATORS;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = MaxAuthorities;
	type AllowMultipleBlocksPerSlot = ConstBool<false>;
	type SlotDuration = ConstU64<6000>;
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type KeyOwnerProof = sp_core::Void;
	type MaxNominators = MaxNominators;
	type WeightInfo = ();
	type MaxAuthorities = MaxAuthorities;
	// This is a purely random value
	type MaxSetIdSessionEntries = ConstU64<100>;
	type EquivocationReportSystem = ();
}

parameter_types! {
	// Minimum period 500ms -> block time 1s
	pub const MinimumPeriod: u64 = 500;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = Nonce;
	type Currency = Balances;
	type Deposit = constants::IndicesDeposit;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = EXISTENTIAL_DEPOSIT;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
	pub const MaxFreezes: u32 = 50;
	pub const MaxHolds: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type FreezeIdentifier = RuntimeFreezeReason;
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type MaxFreezes = MaxFreezes;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const MaxClaims: u32 = 50;
	pub const AutoUnlockBound: u32 = 100;
	pub const UsableBalance: Balance = KILT;
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = FungibleAdapter<Balances, runtime_common::fees::ToAuthorCredit<Runtime>>;
	type OperationalFeeMultiplier = constants::fee::OperationalFeeMultiplier;
	type WeightToFee = IdentityFee<Balance>;
	type LengthToFee = ConstantMultiplier<Balance, constants::fee::TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
}

impl pallet_sudo::Config for Runtime {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
}

parameter_types! {
	pub const MaxDelegatedAttestations: u32 = 1000;
	pub const AttestationDeposit: Balance = constants::attestation::ATTESTATION_DEPOSIT;
}

impl attestation::Config for Runtime {
	type RuntimeHoldReason = RuntimeHoldReason;
	type EnsureOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
	type OriginSuccess = did::DidRawOrigin<DidIdentifier, AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Currency = Balances;
	type Deposit = AttestationDeposit;
	type MaxDelegatedAttestations = MaxDelegatedAttestations;
	type AttesterId = DidIdentifier;
	type AuthorizationId = AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>;
	type AccessControl = PalletAuthorize<DelegationAc<Runtime>>;
	type BalanceMigrationManager = ();
}

parameter_types! {
	pub const MaxSignatureByteLength: u16 = constants::delegation::MAX_SIGNATURE_BYTE_LENGTH;
	pub const MaxParentChecks: u32 = constants::delegation::MAX_PARENT_CHECKS;
	pub const MaxRevocations: u32 = constants::delegation::MAX_REVOCATIONS;
	pub const MaxRemovals: u32 = constants::delegation::MAX_REMOVALS;
	#[derive(Clone, TypeInfo)]
	pub const MaxChildren: u32 = constants::delegation::MAX_CHILDREN;
	pub const DelegationDeposit: Balance = constants::delegation::DELEGATION_DEPOSIT;
}

impl delegation::Config for Runtime {
	type RuntimeHoldReason = RuntimeHoldReason;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type Signature = did::DidSignature;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type DelegationSignatureVerification = did::DidSignatureVerify<Self>;

	#[cfg(feature = "runtime-benchmarks")]
	type Signature = runtime_common::benchmarks::DummySignature;
	#[cfg(feature = "runtime-benchmarks")]
	type DelegationSignatureVerification = kilt_support::signature::AlwaysVerify<AccountId, Vec<u8>, Self::Signature>;

	type DelegationEntityId = DidIdentifier;
	type DelegationNodeId = Hash;
	type EnsureOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
	type OriginSuccess = did::DidRawOrigin<AccountId, DidIdentifier>;
	type RuntimeEvent = RuntimeEvent;
	type MaxSignatureByteLength = MaxSignatureByteLength;
	type MaxParentChecks = MaxParentChecks;
	type MaxRevocations = MaxRevocations;
	type MaxRemovals = MaxRemovals;
	type MaxChildren = MaxChildren;
	type WeightInfo = ();
	type Currency = Balances;
	type Deposit = DelegationDeposit;
	type BalanceMigrationManager = ();
}

parameter_types! {
	pub const Fee: Balance = 500;
}

impl ctype::Config for Runtime {
	type Currency = Balances;
	type Fee = Fee;
	type FeeCollector = runtime_common::fees::ToAuthorCredit<Runtime>;

	type CtypeCreatorId = DidIdentifier;
	type EnsureOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
	type OriginSuccess = did::DidRawOrigin<AccountId, DidIdentifier>;
	type OverarchingOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

parameter_types! {
	#[derive(Debug, Clone, Eq, PartialEq, TypeInfo, Decode, Encode)]
	pub const MaxNewKeyAgreementKeys: u32 = constants::did::MAX_KEY_AGREEMENT_KEYS;
	#[derive(Clone)]
	pub const MaxPublicKeysPerDid: u32 = constants::did::MAX_PUBLIC_KEYS_PER_DID;
	#[derive(Debug, Clone, Eq, PartialEq)]
	pub const MaxTotalKeyAgreementKeys: u32 = constants::did::MAX_TOTAL_KEY_AGREEMENT_KEYS;
	// Standalone block time is half the duration of a parachain block.
	pub const MaxBlocksTxValidity: BlockNumber = constants::did::MAX_BLOCKS_TX_VALIDITY * 2;
	pub const DidFee: Balance = constants::did::DID_FEE;
	pub const MaxNumberOfServicesPerDid: u32 = constants::did::MAX_NUMBER_OF_SERVICES_PER_DID;
	pub const MaxServiceIdLength: u32 = constants::did::MAX_SERVICE_ID_LENGTH;
	pub const MaxServiceTypeLength: u32 = constants::did::MAX_SERVICE_TYPE_LENGTH;
	pub const MaxServiceUrlLength: u32 = constants::did::MAX_SERVICE_URL_LENGTH;
	pub const MaxNumberOfTypesPerService: u32 = constants::did::MAX_NUMBER_OF_TYPES_PER_SERVICE;
	pub const MaxNumberOfUrlsPerService: u32 = constants::did::MAX_NUMBER_OF_URLS_PER_SERVICE;
}

impl did::Config for Runtime {
	type RuntimeHoldReason = RuntimeHoldReason;
	type DidIdentifier = DidIdentifier;
	type KeyDeposit = constants::did::KeyDeposit;
	type ServiceEndpointDeposit = constants::did::ServiceEndpointDeposit;
	type BaseDeposit = constants::did::DidBaseDeposit;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type RuntimeOrigin = RuntimeOrigin;
	type Currency = Balances;
	type Fee = DidFee;
	type FeeCollector = runtime_common::fees::ToAuthorCredit<Runtime>;

	#[cfg(not(feature = "runtime-benchmarks"))]
	type EnsureOrigin = did::EnsureDidOrigin<Self::DidIdentifier, AccountId>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type OriginSuccess = did::DidRawOrigin<AccountId, Self::DidIdentifier>;

	#[cfg(feature = "runtime-benchmarks")]
	type EnsureOrigin = EnsureSigned<Self::DidIdentifier>;
	#[cfg(feature = "runtime-benchmarks")]
	type OriginSuccess = Self::DidIdentifier;

	type MaxNewKeyAgreementKeys = MaxNewKeyAgreementKeys;
	type MaxTotalKeyAgreementKeys = MaxTotalKeyAgreementKeys;
	type MaxPublicKeysPerDid = MaxPublicKeysPerDid;
	type MaxBlocksTxValidity = MaxBlocksTxValidity;
	type MaxNumberOfServicesPerDid = MaxNumberOfServicesPerDid;
	type MaxServiceIdLength = MaxServiceIdLength;
	type MaxServiceTypeLength = MaxServiceTypeLength;
	type MaxServiceUrlLength = MaxServiceUrlLength;
	type MaxNumberOfTypesPerService = MaxNumberOfTypesPerService;
	type MaxNumberOfUrlsPerService = MaxNumberOfUrlsPerService;
	type WeightInfo = ();
	type BalanceMigrationManager = ();
	// This differs from the implementation of the other runtimes.
	type DidLifecycleHooks = ();
}

impl pallet_did_lookup::Config for Runtime {
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeEvent = RuntimeEvent;

	type DidIdentifier = DidIdentifier;

	type Currency = Balances;
	type Deposit = constants::did_lookup::DidLookupDeposit;

	type AssociateOrigin = Self::EnsureOrigin;
	type EnsureOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
	type OriginSuccess = did::DidRawOrigin<AccountId, DidIdentifier>;
	type BalanceMigrationManager = ();
	type WeightInfo = ();
	// Do not change the below flag to `true` without also deploying a runtime
	// migration which removes any links that point to the same DID!
	type UniqueLinkingEnabled = ConstBool<false>;
}

impl pallet_web3_names::Config for Runtime {
	type RuntimeHoldReason = RuntimeHoldReason;
	type BanOrigin = EnsureRoot<AccountId>;
	type ClaimOrigin = Self::OwnerOrigin;
	type OwnerOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
	type OriginSuccess = did::DidRawOrigin<AccountId, DidIdentifier>;
	type Currency = Balances;
	type Deposit = constants::web3_names::Web3NameDeposit;
	type RuntimeEvent = RuntimeEvent;
	type MaxNameLength = constants::web3_names::MaxNameLength;
	type MinNameLength = constants::web3_names::MinNameLength;
	type Web3Name = Web3Name<{ Self::MinNameLength::get() }, { Self::MaxNameLength::get() }>;
	type Web3NameOwner = DidIdentifier;
	type WeightInfo = ();
	type BalanceMigrationManager = ();

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

parameter_types! {
	pub const Period: u64 = 0xFFFF_FFFF_FFFF_FFFF;
	pub const Offset: u64 = 0xFFFF_FFFF_FFFF_FFFF;
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = AccountId;
	type ValidatorIdOf = ();
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = ();
	type SessionManager = ();
	type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = opaque::SessionKeys;
	type WeightInfo = ();
}

parameter_types! {
	pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type EventHandler = ();
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}

impl public_credentials::Config for Runtime {
	type RuntimeHoldReason = RuntimeHoldReason;
	type AccessControl = PalletAuthorize<DelegationAc<Runtime>>;
	type AttesterId = DidIdentifier;
	type AuthorizationId = AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>;
	type CredentialId = Hash;
	type CredentialHash = BlakeTwo256;
	type Currency = Balances;
	type Deposit = runtime_common::constants::public_credentials::Deposit;
	type EnsureOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
	type MaxEncodedClaimsLength = runtime_common::constants::public_credentials::MaxEncodedClaimsLength;
	type MaxSubjectIdLength = runtime_common::constants::public_credentials::MaxSubjectIdLength;
	type OriginSuccess = did::DidRawOrigin<AccountId, DidIdentifier>;
	type RuntimeEvent = RuntimeEvent;
	type SubjectId = runtime_common::assets::AssetDid;
	type WeightInfo = ();
	type BalanceMigrationManager = ();
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug, MaxEncodedLen, scale_info::TypeInfo,
)]
pub enum ProxyType {
	/// Allow for any call.
	Any,
	/// Allow for calls that do not move tokens out of the caller's account.
	NonTransfer,
	/// Allow for staking-related calls.
	CancelProxy,
	/// Allow for calls that do not result in a deposit being claimed (e.g., for
	/// attestations, delegations, or DIDs).
	NonDepositClaiming,
}

impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}

impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => matches!(
				c,
				RuntimeCall::Attestation(..)
					// Excludes `Balances`
					| RuntimeCall::Ctype(..)
					| RuntimeCall::Delegation(..)
					| RuntimeCall::Did(..)
					| RuntimeCall::DidLookup(..)
					| RuntimeCall::Indices(
						// Excludes `force_transfer`, and `transfer`
						pallet_indices::Call::claim { .. }
							| pallet_indices::Call::free { .. }
							| pallet_indices::Call::freeze { .. }
					)
					| RuntimeCall::Proxy(..)
					| RuntimeCall::PublicCredentials(..)
					| RuntimeCall::Session(..)
					// Excludes `Sudo`
					| RuntimeCall::System(..)
					| RuntimeCall::Timestamp(..)
					| RuntimeCall::Utility(..)
					| RuntimeCall::Web3Names(..),
			),
			ProxyType::NonDepositClaiming => matches!(
				c,
				RuntimeCall::Attestation(
						// Excludes `reclaim_deposit`
						attestation::Call::add { .. }
							| attestation::Call::remove { .. }
							| attestation::Call::revoke { .. }
							| attestation::Call::change_deposit_owner { .. }
							| attestation::Call::update_deposit { .. }
					)
					// Excludes `Balances`
					| RuntimeCall::Ctype(..)
					| RuntimeCall::Delegation(
						// Excludes `reclaim_deposit`
						delegation::Call::add_delegation { .. }
							| delegation::Call::create_hierarchy { .. }
							| delegation::Call::remove_delegation { .. }
							| delegation::Call::revoke_delegation { .. }
							| delegation::Call::update_deposit { .. }
							| delegation::Call::change_deposit_owner { .. }
					)
					| RuntimeCall::Did(
						// Excludes `reclaim_deposit`
						did::Call::add_key_agreement_key { .. }
							| did::Call::add_service_endpoint { .. }
							| did::Call::create { .. }
							| did::Call::delete { .. }
							| did::Call::remove_attestation_key { .. }
							| did::Call::remove_delegation_key { .. }
							| did::Call::remove_key_agreement_key { .. }
							| did::Call::remove_service_endpoint { .. }
							| did::Call::set_attestation_key { .. }
							| did::Call::set_authentication_key { .. }
							| did::Call::set_delegation_key { .. }
							| did::Call::submit_did_call { .. }
							| did::Call::update_deposit { .. }
							| did::Call::change_deposit_owner { .. }
					)
					| RuntimeCall::DidLookup(
						// Excludes `reclaim_deposit`
						pallet_did_lookup::Call::associate_account { .. }
							| pallet_did_lookup::Call::associate_sender { .. }
							| pallet_did_lookup::Call::remove_account_association { .. }
							| pallet_did_lookup::Call::remove_sender_association { .. }
							| pallet_did_lookup::Call::update_deposit { .. }
							| pallet_did_lookup::Call::change_deposit_owner { .. }
					)
					| RuntimeCall::Indices(..)
					| RuntimeCall::Proxy(..)
					| RuntimeCall::PublicCredentials(
						// Excludes `reclaim_deposit`
						public_credentials::Call::add { .. }
						| public_credentials::Call::revoke { .. }
						| public_credentials::Call::unrevoke { .. }
						| public_credentials::Call::remove { .. }
						| public_credentials::Call::update_deposit { .. }
						| public_credentials::Call::change_deposit_owner { .. }
					)
					| RuntimeCall::Session(..)
					// Excludes `Sudo`
					| RuntimeCall::System(..)
					| RuntimeCall::Timestamp(..)
					| RuntimeCall::Utility(..)
					| RuntimeCall::Web3Names(
						// Excludes `ban`, and `reclaim_deposit`
						pallet_web3_names::Call::claim { .. }
							| pallet_web3_names::Call::release_by_owner { .. }
							| pallet_web3_names::Call::unban { .. }
							| pallet_web3_names::Call::update_deposit { .. }
							| pallet_web3_names::Call::change_deposit_owner { .. }
					),
			),
			ProxyType::CancelProxy => matches!(c, RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. })),
		}
	}

	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			// "anything" always contains any subset
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			// reclaiming deposits is part of NonTransfer but not in NonDepositClaiming
			(ProxyType::NonDepositClaiming, ProxyType::NonTransfer) => false,
			// everything except NonTransfer and Any is part of NonDepositClaiming
			(ProxyType::NonDepositClaiming, _) => true,
			// Transfers are part of NonDepositClaiming but not in NonTransfer
			(ProxyType::NonTransfer, ProxyType::NonDepositClaiming) => false,
			// everything except NonDepositClaiming and Any is part of NonTransfer
			(ProxyType::NonTransfer, _) => true,
			_ => false,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = constants::proxy::ProxyDepositBase;
	type ProxyDepositFactor = constants::proxy::ProxyDepositFactor;
	type MaxProxies = constants::proxy::MaxProxies;
	type MaxPending = constants::proxy::MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = constants::proxy::AnnouncementDepositBase;
	type AnnouncementDepositFactor = constants::proxy::AnnouncementDepositFactor;
	type WeightInfo = ();
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = constants::multisig::DepositBase;
	type DepositFactor = constants::multisig::DepositFactor;
	type MaxSignatories = constants::multisig::MaxSignitors;
	type WeightInfo = ();
}

construct_runtime!(
	pub enum Runtime
	{
		System: frame_system = 0,
		// DELETED: RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip = 1,

		Timestamp: pallet_timestamp = 2,
		Aura: pallet_aura = 3,
		Grandpa: pallet_grandpa = 4,
		Indices: pallet_indices = 5,
		Balances: pallet_balances = 6,
		TransactionPayment: pallet_transaction_payment = 7,
		Sudo: pallet_sudo = 8,

		Ctype: ctype = 9,
		Attestation: attestation = 10,
		Delegation: delegation = 11,
		Did: did = 12,
		DidLookup: pallet_did_lookup = 13,

		Session: pallet_session = 15,
		Authorship: pallet_authorship = 16,
		// Democracy: pallet_democracy = 25,
		// Council: pallet_collective = 26,
		// TechnicalCommittee: pallet_collective = 27,
		// ElectionsPhragmen: pallet_elections_phragmen = 28,
		// TechnicalMembership: pallet_membership = 29,
		// Treasury: pallet_treasury = 30,

		// Scheduler: pallet_scheduler = 32,

		// DELETED: Vesting: pallet_vesting = 33,
		// DELETED: KiltLaunch: kilt_launch = 34,
		Utility: pallet_utility = 35,
		// DELETED CrowdloanContributors: 36,

		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 37,

		Web3Names: pallet_web3_names = 38,
		PublicCredentials: public_credentials = 39,

		Multisig: pallet_multisig = 47,
	}
);

impl did::DeriveDidCallAuthorizationVerificationKeyRelationship for RuntimeCall {
	fn derive_verification_key_relationship(&self) -> did::DeriveDidCallKeyRelationshipResult {
		fn single_key_relationship(calls: &[RuntimeCall]) -> did::DeriveDidCallKeyRelationshipResult {
			let init = calls
				.first()
				.ok_or(did::RelationshipDeriveError::InvalidCallParameter)?
				.derive_verification_key_relationship()?;
			calls
				.iter()
				.skip(1)
				.map(RuntimeCall::derive_verification_key_relationship)
				.try_fold(init, |acc, next| {
					if Ok(acc) == next {
						Ok(acc)
					} else {
						Err(did::RelationshipDeriveError::InvalidCallParameter)
					}
				})
		}
		match self {
			RuntimeCall::Attestation { .. } => Ok(did::DidVerificationKeyRelationship::AssertionMethod),
			RuntimeCall::Ctype { .. } => Ok(did::DidVerificationKeyRelationship::AssertionMethod),
			RuntimeCall::Delegation { .. } => Ok(did::DidVerificationKeyRelationship::CapabilityDelegation),
			// DID creation is not allowed through the DID proxy.
			RuntimeCall::Did(did::Call::create { .. }) => Err(did::RelationshipDeriveError::NotCallableByDid),
			RuntimeCall::Did { .. } => Ok(did::DidVerificationKeyRelationship::Authentication),
			RuntimeCall::Web3Names { .. } => Ok(did::DidVerificationKeyRelationship::Authentication),
			RuntimeCall::DidLookup { .. } => Ok(did::DidVerificationKeyRelationship::Authentication),
			RuntimeCall::PublicCredentials { .. } => Ok(did::DidVerificationKeyRelationship::AssertionMethod),
			RuntimeCall::Utility(pallet_utility::Call::batch { calls }) => single_key_relationship(&calls[..]),
			RuntimeCall::Utility(pallet_utility::Call::batch_all { calls }) => single_key_relationship(&calls[..]),
			RuntimeCall::Utility(pallet_utility::Call::force_batch { calls }) => single_key_relationship(&calls[..]),
			#[cfg(not(feature = "runtime-benchmarks"))]
			_ => Err(did::RelationshipDeriveError::NotCallableByDid),
			// By default, returns the authentication key
			#[cfg(feature = "runtime-benchmarks")]
			_ => Ok(did::DidVerificationKeyRelationship::Authentication),
		}
	}

	// Always return a System::remark() extrinsic call
	#[cfg(feature = "runtime-benchmarks")]
	fn get_call_for_did_call_benchmark() -> Self {
		RuntimeCall::System(frame_system::Call::remark { remark: vec![] })
	}
}

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
	frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;
/// Executive: handles dispatch to the various Pallets.
pub type Executive =
	frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPalletsWithSystem, ()>;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	frame_benchmarking::define_benchmarks!(
		[frame_system, SystemBench::<Runtime>]
		[pallet_timestamp, Timestamp]
		[pallet_indices, Indices]
		[pallet_balances, Balances]
		[ctype, Ctype]
		[attestation, Attestation]
		[delegation, Delegation]
		[did, Did]
		[pallet_did_lookup, DidLookup]
		[pallet_utility, Utility]
		[pallet_proxy, Proxy]
		[pallet_web3_names, Web3Names]
		[public_credentials, PublicCredentials]
		[frame_benchmarking::baseline, Baseline::<Runtime>]
		[frame_system, SystemBench::<Runtime>]
		[pallet_balances, Balances]
		[pallet_indices, Indices]
		[pallet_timestamp, Timestamp]
		[pallet_utility, Utility]
		[pallet_proxy, Proxy]
		[pallet_multisig, Multisig]
	);
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			frame_system::Pallet::<Runtime>::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}

		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}

		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
	for Runtime
	{
		fn query_call_info(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}
	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> SlotDuration {
			SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			pallet_aura::Authorities::<Runtime>::get().into_inner()
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn current_set_id() -> fg_primitives::SetId {
			Grandpa::current_set_id()
		}

		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			_authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
		}
	}

	impl kilt_runtime_api_did::Did<
		Block,
		DidIdentifier,
		AccountId,
		LinkableAccountId,
		Balance,
		Hash,
		BlockNumber,
		(),
		()
	> for Runtime {
		fn query_by_web3_name(name: Vec<u8>) -> Option<kilt_runtime_api_did::RawDidLinkedInfo<
				DidIdentifier,
				AccountId,
				LinkableAccountId,
				Balance,
				Hash,
				BlockNumber
			>
		> {
			let parsed_name: Web3NameOf<Runtime> = name.try_into().ok()?;
			pallet_web3_names::Owner::<Runtime>::get(&parsed_name)
				.and_then(|owner_info| {
					did::Did::<Runtime>::get(&owner_info.owner).map(|details| (owner_info, details))
				})
				.map(|(owner_info, details)| {
					let accounts = pallet_did_lookup::ConnectedAccounts::<Runtime>::iter_key_prefix(
						&owner_info.owner,
					).collect();
					let service_endpoints = did::ServiceEndpoints::<Runtime>::iter_prefix(&owner_info.owner).map(|e| From::from(e.1)).collect();

					kilt_runtime_api_did::RawDidLinkedInfo{
						identifier: owner_info.owner,
						w3n: Some(parsed_name.into()),
						accounts,
						service_endpoints,
						details: details.into(),
					}
			})
		}

		fn batch_query_by_web3_name(names: Vec<Vec<u8>>) -> Vec<Option<kilt_runtime_api_did::RawDidLinkedInfo<
				DidIdentifier,
				AccountId,
				LinkableAccountId,
				Balance,
				Hash,
				BlockNumber
			>
		>> {
			names.into_iter().map(Self::query_by_web3_name).collect()
		}

		fn query_by_account(account: LinkableAccountId) -> Option<
			kilt_runtime_api_did::RawDidLinkedInfo<
				DidIdentifier,
				AccountId,
				LinkableAccountId,
				Balance,
				Hash,
				BlockNumber
			>
		> {
			pallet_did_lookup::ConnectedDids::<Runtime>::get(account)
				.and_then(|owner_info| {
					did::Did::<Runtime>::get(&owner_info.did).map(|details| (owner_info, details))
				})
				.map(|(connection_record, details)| {
					let w3n = pallet_web3_names::Names::<Runtime>::get(&connection_record.did).map(Into::into);
					let accounts = pallet_did_lookup::ConnectedAccounts::<Runtime>::iter_key_prefix(&connection_record.did).collect();
					let service_endpoints = did::ServiceEndpoints::<Runtime>::iter_prefix(&connection_record.did).map(|e| From::from(e.1)).collect();

					kilt_runtime_api_did::RawDidLinkedInfo {
						identifier: connection_record.did,
						w3n,
						accounts,
						service_endpoints,
						details: details.into(),
					}
				})
		}

		fn batch_query_by_account(accounts: Vec<LinkableAccountId>) -> Vec<Option<
			kilt_runtime_api_did::RawDidLinkedInfo<
				DidIdentifier,
				AccountId,
				LinkableAccountId,
				Balance,
				Hash,
				BlockNumber
			>
		>> {
			accounts.into_iter().map(Self::query_by_account).collect()
		}

		fn query(did: DidIdentifier) -> Option<
			kilt_runtime_api_did::RawDidLinkedInfo<
				DidIdentifier,
				AccountId,
				LinkableAccountId,
				Balance,
				Hash,
				BlockNumber
			>
		> {
			let details = did::Did::<Runtime>::get(&did)?;
			let w3n = pallet_web3_names::Names::<Runtime>::get(&did).map(Into::into);
			let accounts = pallet_did_lookup::ConnectedAccounts::<Runtime>::iter_key_prefix(&did).collect();
			let service_endpoints = did::ServiceEndpoints::<Runtime>::iter_prefix(&did).map(|e| From::from(e.1)).collect();

			Some(kilt_runtime_api_did::RawDidLinkedInfo {
				identifier: did,
				w3n,
				accounts,
				service_endpoints,
				details: details.into(),
			})
		}

		fn batch_query(dids: Vec<DidIdentifier>) -> Vec<Option<
			kilt_runtime_api_did::RawDidLinkedInfo<
				DidIdentifier,
				AccountId,
				LinkableAccountId,
				Balance,
				Hash,
				BlockNumber
			>
		>> {
			dids.into_iter().map(Self::query).collect()
		}

		// We don't return anything here, since the runtime does not require the resources to be cleaned up.
		fn linked_resources(_did: DidIdentifier) -> Vec<()> {
			[].into()
		}

		// We don't return anything here, since the runtime does not require the resources to be cleaned up.
		fn linked_resources_deletion_calls(_did: DidIdentifier) -> Vec<()> {
			[].into()
		}
	}

	impl kilt_runtime_api_public_credentials::PublicCredentials<Block, Vec<u8>, Hash, public_credentials::CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>>, PublicCredentialsFilter<Hash, AccountId>, PublicCredentialsApiError> for Runtime {
		fn get_by_id(credential_id: Hash) -> Option<public_credentials::CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>>> {
			let subject = public_credentials::CredentialSubjects::<Runtime>::get(credential_id)?;
			public_credentials::Credentials::<Runtime>::get(subject, credential_id)
		}

		fn get_by_subject(subject: Vec<u8>, filter: Option<PublicCredentialsFilter<Hash, AccountId>>) -> Result<Vec<(Hash, public_credentials::CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>>)>, PublicCredentialsApiError> {
			let asset_did = AssetDid::try_from(subject).map_err(|_| PublicCredentialsApiError::InvalidSubjectId)?;
			let credentials_prefix = public_credentials::Credentials::<Runtime>::iter_prefix(asset_did);
			if let Some(credentials_filter) = filter {
				Ok(credentials_prefix.filter(|(_, entry)| credentials_filter.should_include(entry)).collect())
			} else {
				Ok(credentials_prefix.collect())
			}
		}
	}


	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;

			use frame_system_benchmarking::Pallet as SystemBench;
			use frame_benchmarking::baseline::Pallet as Baseline;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();
			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch};
			use frame_system_benchmarking::Pallet as SystemBench;
			use frame_benchmarking::baseline::Pallet as Baseline;
			use frame_support::traits::TrackedStorageKey;

			impl frame_system_benchmarking::Config for Runtime {}
			impl frame_benchmarking::baseline::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac")
					.to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80")
					.to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a")
					.to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850")
					.to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7")
					.to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmarks!(params, batches);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_state::<RuntimeGenesisConfig>(config)
		}

		fn get_preset(id: &Option<PresetId>) -> Option<Vec<u8>> {
			get_preset::<RuntimeGenesisConfig>(id, |_| None)
		}

		fn preset_names() -> Vec<PresetId> {
			Default::default()
		}
	}


	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: UpgradeCheckSelect) -> (Weight, Weight) {
			log::info!("try-runtime::on_runtime_upgrade kestrel runtime.");
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, runtime_common::BlockWeights::get().max_block)
		}

		fn execute_block(block: Block, state_root_check: bool, sig_check: bool, select: frame_try_runtime::TryStateSelect) -> Weight {
			log::info!(
				target: "runtime::standalone", "try-runtime: executing block #{} ({:?}) / root checks: {:?} / sig check: {:?} / sanity-checks: {:?}",
				block.header.number,
				block.header.hash(),
				state_root_check,
				sig_check,
				select,
			);
			Executive::try_execute_block(block, state_root_check, sig_check, select).expect("try_execute_block failed")
		}
	}
}
