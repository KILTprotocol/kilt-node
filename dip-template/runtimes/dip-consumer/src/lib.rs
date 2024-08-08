// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

//! Runtime template of a Decentralized Identity Provider (DIP) consumer, which
//! does not itself include any identity-related pallets, but only the
//! [`pallet_dip_consumer::Pallet`] pallet (configured to work with the
//! [`dip_provider_runtime_template::Runtime`] template runtime), the
//! [`pallet_relay_store::Pallet`] pallet to keep track of finalized relaychain
//! state roots, and the example [`pallet_postit::Pallet`], which allows any
//! entity that can be identified with a username (e.g., a web3name carried over
//! from the provider chain) to post a message on chain, reply to another
//! on-chain message (including another reply), or like a message and/or any of
//! its replies.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use dip_provider_runtime_template::Web3Name;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
pub use sp_runtime::{MultiAddress, Perbill, Permill};

use cumulus_pallet_parachain_system::{ParachainSetCode, RelayNumberMonotonicallyIncreases};
use cumulus_primitives_core::{AggregateMessageOrigin, CollationInfo};
use frame_support::{
	instances::{Instance1, Instance2},
	construct_runtime,
	dispatch::DispatchClass,
	parameter_types,
	traits::{ConstU32, ConstU64, ConstU8, EnqueueWithOrigin, Everything, EitherOfDiverse,
			AsEnsureOriginWithArg},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
		IdentityFee, Weight,
	},
	PalletId, BoundedVec,
};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	ChainContext, EnsureRoot, EnsureSigned,
};
use pallet_balances::AccountData;
use pallet_collator_selection::IdentityCollator;
use pallet_session::{FindAccountFromAuthorIndex, PeriodicSessions};
use pallet_transaction_payment::{FeeDetails, FungibleAdapter, RuntimeDispatchInfo};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::SlotDuration;
use sp_core::{crypto::KeyTypeId, ConstBool, ConstU128, ConstU16, OpaqueMetadata};
use sp_inherents::{CheckInherentsResult, InherentData};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, OpaqueKeys, Verify},
	transaction_validity::{TransactionSource, TransactionValidity},
	AccountId32, ApplyExtrinsicResult, MultiSignature, OpaqueExtrinsic,
};
use sp_std::prelude::*;
use sp_version::RuntimeVersion;
use pallet_nfts::PalletFeatures;

mod dip;
mod origin_adapter;
mod weights;
pub use crate::{dip::*, origin_adapter::*};
pub mod constants;
use constants::{currency::*, time::*};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

#[cfg(feature = "std")]
use sp_version::NativeVersion;

pub type AccountId = AccountId32;
pub type Address = MultiAddress<AccountId, ()>;
pub type Balance = u128;
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
pub type BlockNumber = u64;
pub type DidIdentifier = AccountId;
pub type Hasher = BlakeTwo256;
pub type Hash = sp_core::H256;
pub type Header = generic::Header<BlockNumber, Hasher>;
pub type Nonce = u64;
pub type Signature = MultiSignature;

pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;
pub type Executive = frame_executive::Executive<Runtime, Block, ChainContext<Runtime>, Runtime, AllPalletsWithSystem>;
pub type NodeBlock = generic::Block<Header, OpaqueExtrinsic>;
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

pub const MILLISECS_PER_BLOCK: u64 = 12000;
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;

pub const UNIT: Balance = 1_000_000_000_000;
pub const MILLIUNIT: Balance = UNIT / 1_000;

construct_runtime!(
	pub enum Runtime
	{
		// System
		System: frame_system = 0,
		ParachainSystem: cumulus_pallet_parachain_system = 1,
		Timestamp: pallet_timestamp = 2,
		ParachainInfo: parachain_info = 3,
		Sudo: pallet_sudo = 4,
		Utility: pallet_utility = 5,

		// Money
		Balances: pallet_balances = 10,
		TransactionPayment: pallet_transaction_payment = 11,
		Nfts: pallet_nfts = 12,
		Assets: pallet_assets::<Instance1> = 13,
		NftFractionalization: pallet_nft_fractionalization = 14,

		// Collators
		Authorship: pallet_authorship = 20,
		CollatorSelection: pallet_collator_selection = 21,
		Session: pallet_session = 22,
		Aura: pallet_aura = 23,
		AuraExt: cumulus_pallet_aura_ext = 24,
		Council: pallet_collective::<Instance1> = 25,
		TechnicalCommittee: pallet_collective::<Instance2> = 26,
		AllianceMotion: pallet_collective::<Instance3> = 27,

		// Custom
		PostIt: pallet_postit = 30,
		XcavateWhitelist: pallet_xcavate_whitelist = 31,
		NftMarketplace: pallet_nft_marketplace = 32,
		PropertyManagement: pallet_property_management = 33,
		PropertyGovernance: pallet_property_governance = 34,

		// DIP
		DipConsumer: pallet_dip_consumer = 40,
		RelayStore: pallet_relay_store = 41,
	}
);

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("dip-consumer-runtime-template"),
	impl_name: create_runtime_str!("dip-consumer-runtime-template"),
	authoring_version: 1,
	spec_version: 11400,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}

const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
	WEIGHT_REF_TIME_PER_SECOND.saturating_div(2),
	cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64,
);

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub RuntimeBlockLength: BlockLength =
	BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
	.base_block(BlockExecutionWeight::get())
	.for_class(DispatchClass::all(), |weights| {
		weights.base_extrinsic = ExtrinsicBaseWeight::get();
	})
	.for_class(DispatchClass::Normal, |weights| {
		weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
	})
	.for_class(DispatchClass::Operational, |weights| {
		weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
		weights.reserved = Some(
			MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
		);
	})
	.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
	.build_or_panic();
}

pub const SS58_PREFIX: u16 = 101;

impl frame_system::Config for Runtime {
	type AccountData = AccountData<Balance>;
	type AccountId = AccountId;
	type BaseCallFilter = Everything;
	type BlockHashCount = ConstU64<256>;
	type BlockLength = RuntimeBlockLength;
	type Block = Block;
	type BlockWeights = RuntimeBlockWeights;
	type DbWeight = RocksDbWeight;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type Lookup = AccountIdLookup<AccountId, ()>;
	type MaxConsumers = ConstU32<16>;
	type Nonce = u64;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ParachainSetCode<Self>;
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeTask = RuntimeTask;
	type SS58Prefix = ConstU16<SS58_PREFIX>;
	type SystemWeightInfo = weights::frame_system::WeightInfo<Runtime>;
	type Version = Version;
}

/// Maximum number of blocks simultaneously accepted by the Runtime, not yet included into the
/// relay chain.
const UNINCLUDED_SEGMENT_CAPACITY: u32 = 1;
/// How many parachain blocks are processed by the relay chain per parent. Limits the number of
/// blocks authored per slot.
const BLOCK_PROCESSING_VELOCITY: u32 = 1;
/// Relay chain slot duration, in milliseconds.
const RELAY_CHAIN_SLOT_DURATION_MILLIS: u32 = 6000;

/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

type ConsensusHook = cumulus_pallet_aura_ext::FixedVelocityConsensusHook<
	Runtime,
	RELAY_CHAIN_SLOT_DURATION_MILLIS,
	BLOCK_PROCESSING_VELOCITY,
	UNINCLUDED_SEGMENT_CAPACITY,
>;

parameter_types! {
	pub const RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type CheckAssociatedRelayNumber = RelayNumberMonotonicallyIncreases;
	type OnSystemEvent = ();
	type OutboundXcmpMessageSource = ();
	type ReservedDmpWeight = ();
	type ReservedXcmpWeight = ();
	type RuntimeEvent = RuntimeEvent;
	type SelfParaId = ParachainInfo;
	type XcmpMessageHandler = ();
	type ConsensusHook = ConsensusHook;
	type WeightInfo = ();
	type DmpQueue = EnqueueWithOrigin<(), RelayOrigin>;
}

impl pallet_timestamp::Config for Runtime {
	type MinimumPeriod = ConstU64<{ MILLISECS_PER_BLOCK / 2 }>;
	type Moment = u64;
	type OnTimestampSet = Aura;
	type WeightInfo = ();
}

impl parachain_info::Config for Runtime {}

impl pallet_sudo::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

impl pallet_utility::Config for Runtime {
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

pub const EXISTENTIAL_DEPOSIT: Balance = MILLIUNIT;

impl pallet_balances::Config for Runtime {
	type AccountStore = System;
	type Balance = Balance;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxFreezes = ConstU32<50>;
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type WeightInfo = ();
	type RuntimeFreezeReason = RuntimeFreezeReason;
}

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = FungibleAdapter<Balances, ()>;
	type FeeMultiplierUpdate = ();
	type LengthToFee = IdentityFee<Balance>;
	type OperationalFeeMultiplier = ConstU8<1>;
	type RuntimeEvent = RuntimeEvent;
	type WeightToFee = IdentityFee<Balance>;
}

impl pallet_authorship::Config for Runtime {
	type EventHandler = (CollatorSelection,);
	type FindAuthor = FindAccountFromAuthorIndex<Self, Aura>;
}

parameter_types! {
	pub const PotId: PalletId = PalletId(*b"PotStake");
}

impl pallet_collator_selection::Config for Runtime {
	type Currency = Balances;
	type PotId = PotId;
	type KickThreshold = ConstU64<{ 6 * HOURS }>;
	type MaxCandidates = ConstU32<1_000>;
	type MaxInvulnerables = ConstU32<100>;
	type MinEligibleCollators = ConstU32<5>;
	type RuntimeEvent = RuntimeEvent;
	type UpdateOrigin = EnsureRoot<AccountId>;
	type ValidatorId = AccountId;
	type ValidatorIdOf = IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = ();
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

impl pallet_session::Config for Runtime {
	type Keys = SessionKeys;
	type NextSessionRotation = PeriodicSessions<ConstU64<HOURS>, ConstU64<0>>;
	type RuntimeEvent = RuntimeEvent;
	type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type SessionManager = CollatorSelection;
	type ShouldEndSession = PeriodicSessions<ConstU64<HOURS>, ConstU64<0>>;
	type ValidatorId = AccountId;
	type ValidatorIdOf = IdentityCollator;
	type WeightInfo = ();
}

impl pallet_aura::Config for Runtime {
	type AllowMultipleBlocksPerSlot = ConstBool<false>;
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<100_000>;
}

impl cumulus_pallet_aura_ext::Config for Runtime {}

impl pallet_postit::Config for Runtime {
	type MaxTextLength = ConstU32<160>;
	type OriginCheck = EnsureDipOriginAdapter;
	type OriginSuccess = DipOriginAdapter;
	type RuntimeEvent = RuntimeEvent;
	type Username = Web3Name;
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
	pub MaxCollectivesProposalWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type SetMembersOrigin = EnsureRoot<Self::AccountId>;
	type MaxProposalWeight = MaxCollectivesProposalWeight;
}

parameter_types! {
	pub const TechnicalMotionDuration: BlockNumber = 5 * DAYS;
	pub const TechnicalMaxProposals: u32 = 100;
	pub const TechnicalMaxMembers: u32 = 100;
}

type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = TechnicalMaxProposals;
	type MaxMembers = TechnicalMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type SetMembersOrigin = EnsureRoot<Self::AccountId>;
	type MaxProposalWeight = MaxCollectivesProposalWeight;
}

const ALLIANCE_MOTION_DURATION_IN_BLOCKS: BlockNumber = 5 * DAYS;

parameter_types! {
	pub const AllianceMotionDuration: BlockNumber = ALLIANCE_MOTION_DURATION_IN_BLOCKS;
	pub const AllianceMaxProposals: u32 = 100;
	pub const AllianceMaxMembers: u32 = 100;
}

type AllianceCollective = pallet_collective::Instance3;
impl pallet_collective::Config<AllianceCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = AllianceMotionDuration;
	type MaxProposals = AllianceMaxProposals;
	type MaxMembers = AllianceMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type SetMembersOrigin = EnsureRoot<Self::AccountId>;
	type MaxProposalWeight = MaxCollectivesProposalWeight;
}

parameter_types! {
	pub Features: PalletFeatures = PalletFeatures::all_enabled();
	pub const MaxAttributesPerCall: u32 = 10;
	pub const CollectionDeposit: Balance = DOLLARS;
	pub const ItemDeposit: Balance = DOLLARS;
	pub const MetadataDepositBase: Balance = DOLLARS;
	pub const MetadataDepositPerByte: Balance = DOLLARS / 100;
	pub const StringLimit: u32 = 5000;
	pub const KeyLimit: u32 = 32;
	pub const ValueLimit: u32 = 256;
	pub const ApprovalsLimit: u32 = 20;
	pub const ItemAttributesApprovalsLimit: u32 = 20;
	pub const MaxTips: u32 = 10;
	pub const MaxDeadlineDuration: BlockNumber = 12 * 30 * DAYS;

	pub const UserStringLimit: u32 = 5;

}

impl pallet_nfts::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = u32;
	type ItemId = u32;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type CollectionDeposit = CollectionDeposit;
	type ItemDeposit = ItemDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type AttributeDepositBase = MetadataDepositBase;
	type DepositPerByte = MetadataDepositPerByte;
	type StringLimit = StringLimit;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type ApprovalsLimit = ApprovalsLimit;
	type ItemAttributesApprovalsLimit = ItemAttributesApprovalsLimit;
	type MaxTips = MaxTips;
	type MaxDeadlineDuration = MaxDeadlineDuration;
	type MaxAttributesPerCall = MaxAttributesPerCall;
	type Features = Features;
	type OffchainSignature = Signature;
	type OffchainPublic = <Signature as Verify>::Signer;
	type WeightInfo = pallet_nfts::weights::SubstrateWeight<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
	//type CreateOrigin = AsEnsureOriginWithArg<EnsureSignedBy<CollectionCreationOrigin, AccountId>>;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type Locker = ();
}

parameter_types! {
	pub const AssetConversionPalletId: PalletId = PalletId(*b"py/ascon");
	pub const AssetDeposit: Balance = 100 * DOLLARS;
	pub const ApprovalDeposit: Balance = DOLLARS;

}

impl pallet_assets::Config<Instance1> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u128;
	type AssetId = u32;
	type AssetIdParameter = parity_scale_codec::Compact<u32>;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = ConstU128<DOLLARS>;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type Extra = ();
	type CallbackHandle = ();
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
	type RemoveItemsLimit = ConstU32<1000>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

parameter_types! {
	pub const NftFractionalizationPalletId: PalletId = PalletId(*b"fraction");
	pub NewAssetSymbol: BoundedVec<u8, StringLimit> = (*b"BRIX").to_vec().try_into().unwrap();
	pub NewAssetName: BoundedVec<u8, StringLimit> = (*b"Brix").to_vec().try_into().unwrap();
	pub const Deposit: Balance = DOLLARS;
}

impl pallet_nft_fractionalization::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Deposit = Deposit;
	type Currency = Balances;
	type NewAssetSymbol = NewAssetSymbol;
	type NewAssetName = NewAssetName;
	type NftCollectionId = <Self as pallet_nfts::Config>::CollectionId;
	type NftId = <Self as pallet_nfts::Config>::ItemId;
	type AssetBalance = <Self as pallet_balances::Config>::Balance;
	type AssetId = <Self as pallet_assets::Config<Instance1>>::AssetId;
	type Assets = Assets;
	type Nfts = Nfts;
	type PalletId = NftFractionalizationPalletId;
	type WeightInfo = ();
	type StringLimit = StringLimit;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
	type RuntimeHoldReason = RuntimeHoldReason;
}

parameter_types! {
	pub const MaxWhitelistUsers: u32 = 1000;
}

/// Configure the pallet-xcavate-whitelist in pallets/xcavate-whitelist.
impl pallet_xcavate_whitelist::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_xcavate_whitelist::weights::SubstrateWeight<Runtime>;
	type WhitelistOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
	>;
	type MaxUsersInWhitelist = MaxWhitelistUsers;
}

parameter_types! {
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const CommunityProjectPalletId: PalletId = PalletId(*b"py/cmprj");
	pub const NftMarketplacePalletId: PalletId = PalletId(*b"py/nftxc");
	pub const MaxNftTokens: u32 = 250;
	pub const Postcode: u32 = 10;
}

/// Configure the pallet-nft-marketplace in pallets/nft-marketplace.
impl pallet_nft_marketplace::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_nft_marketplace::weights::SubstrateWeight<Runtime>;
	type Currency = Balances;
	type PalletId = NftMarketplacePalletId;
	type MaxNftToken = MaxNftTokens;
	type LocationOrigin = EnsureRoot<Self::AccountId>;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = pallet_nft_marketplace::NftHelper;
	type CollectionId = u32;
	type ItemId = u32;
	type TreasuryId = TreasuryPalletId;
	type CommunityProjectsId = CommunityProjectPalletId;
	type FractionalizeCollectionId = <Self as pallet_nfts::Config>::CollectionId;
	type FractionalizeItemId = <Self as pallet_nfts::Config>::ItemId;
	type AssetId = <Self as pallet_assets::Config<Instance1>>::AssetId;
	type AssetId2 = u32;
	type PostcodeLimit = Postcode;
	type OriginCheck = EnsureDipOriginAdapter;
	type OriginSuccess = DipOriginAdapter;
	type Username = Web3Name;
}

parameter_types! {
	pub const MinimumStakingAmount: Balance = 100 * DOLLARS;
	pub const PropertyManagementPalletId: PalletId = PalletId(*b"py/ppmmt");
	pub const MaxProperty: u32 = 1000;
	pub const MaxLettingAgent: u32 = 100;
	pub const MaxLocation: u32 = 100;
	pub const PropertyReserves: Balance = 1000 * DOLLARS;
	pub const PolkadotJsMultiply: Balance = 1/* CENTS */;
}

/// Configure the pallet-property-management in pallets/property-management.
impl pallet_property_management::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_property_management::weights::SubstrateWeight<Runtime>;
	type Currency = Balances;
	type PalletId = PropertyManagementPalletId;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = pallet_property_management::AssetHelper;
	type AgentOrigin = EnsureRoot<Self::AccountId>;
	type MinStakingAmount = MinimumStakingAmount;
	type MaxProperties = MaxProperty;
	type MaxLettingAgents = MaxLettingAgent;
	type MaxLocations = MaxLocation;
	type GovernanceId = PropertyGovernancePalletId;
	type PropertyReserve = PropertyReserves;
	type AssetId = <Self as pallet_assets::Config<Instance1>>::AssetId;
	type PolkadotJsMultiplier = PolkadotJsMultiply;
}

parameter_types! {
	pub const PropertyVotingTime: BlockNumber = 20;
	pub const MaxVoteForBlock: u32 = 100;
	pub const MinimumSlashingAmount: Balance = 10 * DOLLARS;
	pub const MaximumVoter: u32 = 100;
	pub const VotingThreshold: u8 = 51;
	pub const HighVotingThreshold: u8 = 67;
	pub const LowProposal: Balance = 500 * CENTS;
	pub const HighProposal: Balance = 10_000 * CENTS;
	pub const PropertyGovernancePalletId: PalletId = PalletId(*b"py/gvrnc");
}

/// Configure the pallet-property-governance in pallets/property-governance.
impl pallet_property_governance::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_property_governance::weights::SubstrateWeight<Runtime>;
	type Currency = Balances;
	type VotingTime = PropertyVotingTime;
	type MaxVotesForBlock = MaxVoteForBlock;
	type Slash = ();
	type MinSlashingAmount = MinimumSlashingAmount;
	type MaxVoter = MaximumVoter;
	type Threshold = VotingThreshold;
	type HighThreshold = HighVotingThreshold;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = pallet_property_governance::AssetHelper;
	type LowProposal = LowProposal;
	type HighProposal = HighProposal;
	type PalletId = PropertyGovernancePalletId;
	type AssetId = <Self as pallet_assets::Config<Instance1>>::AssetId;
	type PolkadotJsMultiplier = PolkadotJsMultiply;
}

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	frame_benchmarking::define_benchmarks!(
		[frame_system, SystemBench::<Runtime>]
		[pallet_dip_consumer, DipConsumer]
		[pallet_relay_store, RelayStore]
		[pallet_relay_store, RelayStore]
		[pallet_xcavate_whitelist, XcavateWhitelist]
		[pallet_nft_marketplace, NftMarketplace]
		[pallet_property_management, PropertyManagement]
		[pallet_property_governance, PropertyGovernance]
		[pallet_nfts, Nfts]
		[pallet_assets, Assets]
		[pallet_nft_fractionalization, NftFractionalization]
	);
}

impl_runtime_apis! {

	impl cumulus_primitives_aura::AuraUnincludedSegmentApi<Block> for Runtime {
		fn can_build_upon(
			included_hash: <Block as BlockT>::Hash,
			slot: cumulus_primitives_aura::Slot,
		) -> bool {
			ConsensusHook::can_build_upon(included_hash, slot)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> SlotDuration {
			SlotDuration::from_millis(SLOT_DURATION)
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
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

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: InherentData,
		) -> CheckInherentsResult {
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
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> FeeDetails<Balance> {
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
		) -> RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(
			call: RuntimeCall,
			len: u32,
		) -> FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> CollationInfo {
			ParachainSystem::collect_collation_info(header)
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

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();
			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{BenchmarkError, Benchmarking, BenchmarkBatch};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {
				fn setup_set_code_requirements(code: &sp_std::vec::Vec<u8>) -> Result<(), BenchmarkError> {
					ParachainSystem::initialize_for_set_code_benchmark(code.len() as u32);
					Ok(())
				}

				fn verify_set_code() {
					System::assert_last_event(cumulus_pallet_parachain_system::Event::<Runtime>::ValidationFunctionStored.into());
				}
			}

			use frame_support::traits::WhitelistedStorageKeys;
			let whitelist = AllPalletsWithSystem::whitelisted_storage_keys();

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}
