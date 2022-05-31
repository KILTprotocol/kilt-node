// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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

//! The KILT runtime. This can be compiled with `#[no_std]`, ready for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
// The `from_over_into` warning originates from `construct_runtime` macro.
#![allow(clippy::from_over_into)]

// Make the WASM binary available
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use codec::{Decode, Encode, MaxEncodedLen};
use delegation::DelegationAc;
pub use frame_support::{
	construct_runtime, parameter_types,
	traits::{Currency, FindAuthor, Imbalance, KeyOwnerProofSystem, OnUnbalanced, Randomness},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
		IdentityFee, Weight,
	},
	ConsensusEngineId, StorageValue,
};
use frame_support::{traits::InstanceFilter, weights::ConstantMultiplier};
use frame_system::EnsureRoot;
use pallet_grandpa::{fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use pallet_transaction_payment::{CurrencyAdapter, FeeDetails};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::{ed25519::AuthorityId as AuraId, SlotDuration};
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto, NumberFor, OpaqueKeys, Verify},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, RuntimeDebug,
};
pub use sp_runtime::{Perbill, Permill};
use sp_std::prelude::*;
use sp_version::RuntimeVersion;

pub use pallet_timestamp::Call as TimestampCall;

pub use attestation;
pub use ctype;
pub use delegation;
pub use did;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_web3_names;
use runtime_common::{
	authorization::{AuthorizationId, PalletAuthorize},
	constants::{self, KILT, MILLI_KILT},
	fees::ToAuthor,
	AccountId, Balance, BlockNumber, DidIdentifier, Hash, Index, Signature, SlowAdjustingFeeUpdate,
};

#[cfg(feature = "std")]
use sp_version::NativeVersion;

#[cfg(feature = "runtime-benchmarks")]
use frame_system::EnsureSigned;

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

/// Digest item type.
pub type DigestItem = generic::DigestItem;

pub type NegativeImbalance<T> =
	<pallet_balances::Pallet<T> as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

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
	spec_name: create_runtime_str!("mashnet-node"),
	impl_name: create_runtime_str!("mashnet-node"),
	authoring_version: 4,
	spec_version: 10620,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 3,
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

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const BlockHashCount: BlockNumber = 2400;
	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
		::with_sensible_defaults(2 * WEIGHT_PER_SECOND, NORMAL_DISPATCH_RATIO);
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 38;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = frame_support::traits::Everything;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = BlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = BlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in
	/// dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
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
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const MaxAuthorities: u32 = constants::staking::MAX_CANDIDATES;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = MaxAuthorities;
}

impl pallet_grandpa::Config for Runtime {
	type Event = Event;
	type Call = Call;

	type KeyOwnerProofSystem = ();

	type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

	type KeyOwnerIdentification =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::IdentificationTuple;

	type HandleEquivocation = ();

	type WeightInfo = ();
	type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
	pub const MinimumPeriod: u64 = constants::SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = Index;
	type Currency = Balances;
	type Deposit = constants::IndicesDeposit;
	type Event = Event;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: Balance = 10 * MILLI_KILT;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
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
	type OnChargeTransaction = CurrencyAdapter<Balances, runtime_common::fees::ToAuthor<Runtime>>;
	type OperationalFeeMultiplier = constants::fee::OperationalFeeMultiplier;
	type WeightToFee = IdentityFee<Balance>;
	type LengthToFee = ConstantMultiplier<Balance, constants::fee::TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
}

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

parameter_types! {
	pub const MaxDelegatedAttestations: u32 = 1000;
	pub const AttestationDeposit: Balance = constants::attestation::ATTESTATION_DEPOSIT;
}

impl attestation::Config for Runtime {
	type EnsureOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
	type OriginSuccess = did::DidRawOrigin<DidIdentifier, AccountId>;
	type Event = Event;
	type WeightInfo = ();
	type Currency = Balances;
	type Deposit = AttestationDeposit;
	type MaxDelegatedAttestations = MaxDelegatedAttestations;
	type AttesterId = DidIdentifier;
	type AuthorizationId = AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>;
	type AccessControl = PalletAuthorize<DelegationAc<Runtime>>;
}

parameter_types! {
	pub const MaxSignatureByteLength: u16 = constants::delegation::MAX_SIGNATURE_BYTE_LENGTH;
	pub const MaxParentChecks: u32 = constants::delegation::MAX_PARENT_CHECKS;
	pub const MaxRevocations: u32 = constants::delegation::MAX_REVOCATIONS;
	pub const MaxRemovals: u32 = constants::delegation::MAX_REMOVALS;
	#[derive(Clone)]
	pub const MaxChildren: u32 = constants::delegation::MAX_CHILDREN;
	pub const DelegationDeposit: Balance = constants::delegation::DELEGATION_DEPOSIT;
}

impl delegation::Config for Runtime {
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
	type Event = Event;
	type MaxSignatureByteLength = MaxSignatureByteLength;
	type MaxParentChecks = MaxParentChecks;
	type MaxRevocations = MaxRevocations;
	type MaxRemovals = MaxRemovals;
	type MaxChildren = MaxChildren;
	type WeightInfo = ();
	type Currency = Balances;
	type Deposit = DelegationDeposit;
}

parameter_types! {
	pub const Fee: Balance = 500;
}

impl ctype::Config for Runtime {
	type Currency = Balances;
	type Fee = Fee;
	type FeeCollector = runtime_common::fees::ToAuthor<Runtime>;

	type CtypeCreatorId = DidIdentifier;
	type EnsureOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
	type OriginSuccess = did::DidRawOrigin<AccountId, DidIdentifier>;
	type Event = Event;
	type WeightInfo = ();
}

parameter_types! {
	pub const MaxNewKeyAgreementKeys: u32 = constants::did::MAX_KEY_AGREEMENT_KEYS;
	#[derive(Debug, Clone, PartialEq)]
	pub const MaxUrlLength: u32 = constants::did::MAX_URL_LENGTH;
	pub const MaxPublicKeysPerDid: u32 = constants::did::MAX_PUBLIC_KEYS_PER_DID;
	#[derive(Debug, Clone, PartialEq)]
	pub const MaxTotalKeyAgreementKeys: u32 = constants::did::MAX_TOTAL_KEY_AGREEMENT_KEYS;
	#[derive(Debug, Clone, PartialEq)]
	pub const MaxEndpointUrlsCount: u32 = constants::did::MAX_ENDPOINT_URLS_COUNT;
	// Standalone block time is half the duration of a parachain block.
	pub const MaxBlocksTxValidity: BlockNumber = constants::did::MAX_BLOCKS_TX_VALIDITY * 2;
	pub const DidDeposit: Balance = constants::did::DID_DEPOSIT;
	pub const DidFee: Balance = constants::did::DID_FEE;
	pub const MaxNumberOfServicesPerDid: u32 = constants::did::MAX_NUMBER_OF_SERVICES_PER_DID;
	pub const MaxServiceIdLength: u32 = constants::did::MAX_SERVICE_ID_LENGTH;
	pub const MaxServiceTypeLength: u32 = constants::did::MAX_SERVICE_TYPE_LENGTH;
	pub const MaxServiceUrlLength: u32 = constants::did::MAX_SERVICE_URL_LENGTH;
	pub const MaxNumberOfTypesPerService: u32 = constants::did::MAX_NUMBER_OF_TYPES_PER_SERVICE;
	pub const MaxNumberOfUrlsPerService: u32 = constants::did::MAX_NUMBER_OF_URLS_PER_SERVICE;
}

impl did::Config for Runtime {
	type DidIdentifier = DidIdentifier;
	type Event = Event;
	type Call = Call;
	type Origin = Origin;
	type Currency = Balances;
	type Deposit = DidDeposit;
	type Fee = DidFee;
	type FeeCollector = ToAuthor<Runtime>;

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
}

parameter_types! {
	pub const DidLookupDeposit: Balance = constants::did_lookup::DID_CONNECTION_DEPOSIT;
}

impl pallet_did_lookup::Config for Runtime {
	type Event = Event;
	type Signature = Signature;
	type Signer = <Signature as Verify>::Signer;
	type DidIdentifier = DidIdentifier;

	type Currency = Balances;
	type Deposit = DidLookupDeposit;

	type EnsureOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
	type OriginSuccess = did::DidRawOrigin<AccountId, DidIdentifier>;

	type WeightInfo = ();
}

parameter_types! {
	pub const Web3NameDeposit: Balance = constants::web3_names::DEPOSIT;
	pub const MinNameLength: u32 = constants::web3_names::MIN_LENGTH;
	pub const MaxNameLength: u32 = constants::web3_names::MAX_LENGTH;
}

impl pallet_web3_names::Config for Runtime {
	type BanOrigin = EnsureRoot<AccountId>;
	type OwnerOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
	type OriginSuccess = did::DidRawOrigin<AccountId, DidIdentifier>;
	type Currency = Balances;
	type Deposit = Web3NameDeposit;
	type Event = Event;
	type MaxNameLength = MaxNameLength;
	type MinNameLength = MinNameLength;
	type Web3Name = pallet_web3_names::web3_name::AsciiWeb3Name<Runtime>;
	type Web3NameOwner = DidIdentifier;
	type WeightInfo = ();
}

parameter_types! {
	pub const Period: u64 = 0xFFFF_FFFF_FFFF_FFFF;
	pub const Offset: u64 = 0xFFFF_FFFF_FFFF_FFFF;
}

impl pallet_session::Config for Runtime {
	type Event = Event;
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
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = ();
}

impl pallet_vesting::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	// disable vested transfers by setting min amount to max balance
	type MinVestedTransfer = constants::MinVestedTransfer;
	const MAX_VESTING_SCHEDULES: u32 = constants::MAX_VESTING_SCHEDULES;
	type WeightInfo = ();
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}

impl pallet_randomness_collective_flip::Config for Runtime {}

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

impl InstanceFilter<Call> for ProxyType {
	fn filter(&self, c: &Call) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => matches!(
				c,
				Call::Attestation(..)
					| Call::Authorship(..)
					// Excludes `Balances`
					| Call::Ctype(..)
					| Call::Delegation(..)
					| Call::Did(..)
					| Call::DidLookup(..)
					| Call::Indices(
						// Excludes `force_transfer`, and `transfer`
						pallet_indices::Call::claim { .. }
							| pallet_indices::Call::free { .. }
							| pallet_indices::Call::freeze { .. }
					)
					| Call::Proxy(..)
					| Call::Session(..)
					// Excludes `Sudo`
					| Call::System(..)
					| Call::Timestamp(..)
					| Call::Utility(..)
					| Call::Web3Names(..),
			),
			ProxyType::NonDepositClaiming => matches!(
				c,
				Call::Attestation(
						// Excludes `reclaim_deposit`
						attestation::Call::add { .. }
							| attestation::Call::remove { .. }
							| attestation::Call::revoke { .. }
					)
					| Call::Authorship(..)
					// Excludes `Balances`
					| Call::Ctype(..)
					| Call::Delegation(
						// Excludes `reclaim_deposit`
						delegation::Call::add_delegation { .. }
							| delegation::Call::create_hierarchy { .. }
							| delegation::Call::remove_delegation { .. }
							| delegation::Call::revoke_delegation { .. }
					)
					| Call::Did(
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
					)
					| Call::DidLookup(
						// Excludes `reclaim_deposit`
						pallet_did_lookup::Call::associate_account { .. }
							| pallet_did_lookup::Call::associate_sender { .. }
							| pallet_did_lookup::Call::remove_account_association { .. }
							| pallet_did_lookup::Call::remove_sender_association { .. }
					)
					| Call::Indices(..)
					| Call::Proxy(..)
					| Call::Session(..)
					// Excludes `Sudo`
					| Call::System(..)
					| Call::Timestamp(..)
					| Call::Utility(..)
					| Call::Web3Names(
						// Excludes `ban`, and `reclaim_deposit`
						pallet_web3_names::Call::claim { .. }
							| pallet_web3_names::Call::release_by_owner { .. }
							| pallet_web3_names::Call::unban { .. }
					),
			),
			ProxyType::CancelProxy => matches!(c, Call::Proxy(pallet_proxy::Call::reject_announcement { .. })),
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
	type Event = Event;
	type Call = Call;
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

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system = 0,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip = 1,

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

		// // Governance stuff; uncallable initially.
		// Democracy: pallet_democracy = 25,
		// Council: pallet_collective = 26,
		// TechnicalCommittee: pallet_collective = 27,
		// ElectionsPhragmen: pallet_elections_phragmen = 28,
		// TechnicalMembership: pallet_membership = 29,
		// Treasury: pallet_treasury = 30,

		// // System scheduler.
		// Scheduler: pallet_scheduler = 32,

		// Vesting. Usable initially, but removed once all vesting is finished.
		Vesting: pallet_vesting = 33,
		// DELETED: KiltLaunch: kilt_launch = 34,
		Utility: pallet_utility = 35,
		// DELETED CrowdloanContributors: 36,

		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 37,
		Web3Names: pallet_web3_names = 38,
	}
);

impl did::DeriveDidCallAuthorizationVerificationKeyRelationship for Call {
	fn derive_verification_key_relationship(&self) -> did::DeriveDidCallKeyRelationshipResult {
		fn single_key_relationship(calls: &[Call]) -> did::DeriveDidCallKeyRelationshipResult {
			let init = calls
				.get(0)
				.ok_or(did::RelationshipDeriveError::InvalidCallParameter)?
				.derive_verification_key_relationship()?;
			calls
				.iter()
				.skip(1)
				.map(Call::derive_verification_key_relationship)
				.try_fold(init, |acc, next| {
					if Ok(acc) == next {
						Ok(acc)
					} else {
						Err(did::RelationshipDeriveError::InvalidCallParameter)
					}
				})
		}
		match self {
			Call::Attestation { .. } => Ok(did::DidVerificationKeyRelationship::AssertionMethod),
			Call::Ctype { .. } => Ok(did::DidVerificationKeyRelationship::AssertionMethod),
			Call::Delegation { .. } => Ok(did::DidVerificationKeyRelationship::CapabilityDelegation),
			// DID creation is not allowed through the DID proxy.
			Call::Did(did::Call::create { .. }) => Err(did::RelationshipDeriveError::NotCallableByDid),
			Call::Did { .. } => Ok(did::DidVerificationKeyRelationship::Authentication),
			Call::Web3Names { .. } => Ok(did::DidVerificationKeyRelationship::Authentication),
			Call::DidLookup { .. } => Ok(did::DidVerificationKeyRelationship::Authentication),
			Call::Utility(pallet_utility::Call::batch { calls }) => single_key_relationship(&calls[..]),
			Call::Utility(pallet_utility::Call::batch_all { calls }) => single_key_relationship(&calls[..]),
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
		Call::System(frame_system::Call::remark { remark: vec![] })
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
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various Pallets.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	(
		delegation::migrations::v3::DelegationMigrationV3<Runtime>,
		did::migrations::v4::DidMigrationV4<Runtime>,
	),
>;

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			frame_system::Pallet::<Runtime>::account_nonce(&account)
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
			Aura::authorities().into_inner()
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

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{list_benchmark, baseline, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			let mut list = Vec::<BenchmarkList>::new();

			list_benchmark!(list, extra, frame_benchmarking, BaselineBench::<Runtime>);
			list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
			list_benchmark!(list, extra, pallet_balances, Balances);
			list_benchmark!(list, extra, pallet_timestamp, Timestamp);

			list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
			list_benchmark!(list, extra, pallet_balances, Balances);
			list_benchmark!(list, extra, pallet_timestamp, Timestamp);
			list_benchmark!(list, extra, pallet_vesting, Vesting);

			list_benchmark!(list, extra, did, Did);
			list_benchmark!(list, extra, ctype, Ctype);
			list_benchmark!(list, extra, delegation, Delegation);
			list_benchmark!(list, extra, attestation, Attestation);

			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use baseline::Pallet as BaselineBench;
			use frame_system_benchmarking::Pallet as SystemBench;

			impl frame_system_benchmarking::Config for Runtime {}
			impl baseline::Config for Runtime {}

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

			add_benchmark!(params, batches, frame_benchmarking, BaselineBench::<Runtime>);
			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_balances, Balances);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);
			add_benchmark!(params, batches, pallet_vesting, Vesting);

			add_benchmark!(params, batches, did, Did);
			add_benchmark!(params, batches, ctype, Ctype);
			add_benchmark!(params, batches, delegation, Delegation);
			add_benchmark!(params, batches, attestation, Attestation);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade() -> (Weight, Weight) {
			log::info!("try-runtime::on_runtime_upgrade standalone runtime.");
			let weight = Executive::try_runtime_upgrade().map_err(|err|{
				log::info!("try-runtime::on_runtime_upgrade failed with: {:?}", err);
				err
			}).unwrap();
			(weight, BlockWeights::get().max_block)
		}
		fn execute_block_no_check(block: Block) -> Weight {
			Executive::execute_block_no_check(block)
		}
	}
}
