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

//! The KILT runtime. This can be compiled with `#[no_std]`, ready for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
// The `from_over_into` warning originates from `construct_runtime` macro.
#![allow(clippy::from_over_into)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use ctype::PayFee;
use did::DidSignature;
use frame_support::{
	traits::{Currency, ExistenceRequirement, LockIdentifier},
	PalletId,
};
#[cfg(feature = "runtime-benchmarks")]
use frame_system::EnsureSigned;
use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureOneOf, EnsureRoot,
};
use kilt_primitives::{
	constants::{
		did::{
			MAX_BLOCKS_TX_VALIDITY, MAX_ENDPOINT_URLS_COUNT, MAX_KEY_AGREEMENT_KEYS, MAX_PUBLIC_KEYS_PER_DID,
			MAX_TOTAL_KEY_AGREEMENT_KEYS, MAX_URL_LENGTH,
		},
		governance::{
			COOLOFF_PERIOD, COUNCIL_MOTION_DURATION, ENACTMENT_PERIOD, FAST_TRACK_VOTING_PERIOD, LAUNCH_PERIOD,
			SPEND_PERIOD, TECHNICAL_MOTION_DURATION, VOTING_PERIOD,
		},
		staking::{DEFAULT_BLOCKS_PER_ROUND, MAX_CANDIDATES, MIN_BLOCKS_PER_ROUND, MIN_COLLATORS, STAKE_DURATION},
		AVERAGE_ON_INITIALIZE_RATIO, KILT, MAXIMUM_BLOCK_WEIGHT, MICRO_KILT, MILLI_KILT, MIN_VESTED_TRANSFER_AMOUNT,
		NORMAL_DISPATCH_RATIO, SLOT_DURATION,
	},
	AccountId, AuthorityId, Balance, BlockNumber, DidIdentifier, Hash, Header, Index, Signature,
};
use pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};
use sp_api::impl_runtime_apis;
use sp_core::{
	u32_trait::{_1, _2, _3, _5},
	OpaqueMetadata,
};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto, OpaqueKeys},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, FixedPointNumber, Perquintill, SaturatedConversion,
};
use sp_std::prelude::*;
use sp_version::RuntimeVersion;

mod fee;
#[cfg(test)]
mod tests;
mod weights;

#[cfg(feature = "std")]
use sp_version::NativeVersion;

// A few exports that help ease life for downstream crates.
pub use frame_support::{
	construct_runtime, parameter_types,
	traits::{Get, Randomness},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
		DispatchClass, IdentityFee, Weight,
	},
	StorageValue,
};
pub use parachain_staking::{InflationInfo, RewardRate, StakingInfo};

pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};

pub use attestation;
pub use ctype;
pub use delegation;
pub use did;

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("mashnet-node"),
	impl_name: create_runtime_str!("mashnet-node"),
	authoring_version: 4,
	spec_version: 26,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 2,
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

// Pallet accounts of runtime
parameter_types! {
	pub const TreasuryPalletId: PalletId = PalletId(*b"kilt/tsy");
	pub const ElectionsPalletId: LockIdentifier = *b"kilt/elc";
}

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
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
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u8 = 38;
}

impl frame_system::Config for Runtime {
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
	type Header = Header;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest
	/// pruned first).
	type BlockHashCount = BlockHashCount;
	/// Runtime version.
	type Version = Version;
	/// Converts a module to an index of this module in the runtime.
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = RocksDbWeight;
	type BaseCallFilter = frame_support::traits::Everything;
	type SystemWeightInfo = weights::frame_system::WeightInfo<Runtime>;
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	type SS58Prefix = SS58Prefix;
	/// The set code logic, just the default since we're not a parachain.
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Runtime>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = weights::pallet_timestamp::WeightInfo<Runtime>;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 10 * MILLI_KILT;
	pub const TransactionByteFee: u128 = MICRO_KILT;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = Index;
	type Currency = pallet_balances::Pallet<Runtime>;
	type Deposit = ExistentialDeposit;
	type Event = Event;
	type WeightInfo = weights::pallet_indices::WeightInfo<Runtime>;
}

impl pallet_balances::Config for Runtime {
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = weights::pallet_balances::WeightInfo<Runtime>;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	/// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
	/// that combined with `AdjustmentVariable`, we can recover from the minimum.
	/// See `multiplier_can_grow_from_zero`.
	pub Minimum: Multiplier = Multiplier::saturating_from_rational(1, 1);
	/// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
	/// change the fees more rapidly.
	pub Variability: Multiplier = Multiplier::saturating_from_rational(3, 100_000);
}

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = fee::WeightToFee;
	type FeeMultiplierUpdate = TargetedFeeAdjustment<Runtime, TargetBlockFullness, Variability, Minimum>;
}

impl pallet_sudo::Config for Runtime {
	type Call = Call;
	type Event = Event;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type OnValidationData = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type OutboundXcmpMessageSource = ();
	type DmpMessageHandler = ();
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = ();
	type ReservedXcmpWeight = ReservedXcmpWeight;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuthorityId;
	//TODO: handle disabled validators
	type DisabledValidators = ();
}

parameter_types! {
	pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = ParachainStaking;
}

parameter_types! {
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

impl pallet_session::Config for Runtime {
	type Event = Event;
	type ValidatorId = AccountId;
	type ValidatorIdOf = ConvertInto;
	type ShouldEndSession = ParachainStaking;
	type NextSessionRotation = ParachainStaking;
	type SessionManager = ParachainStaking;
	type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
	type WeightInfo = weights::pallet_session::WeightInfo<Runtime>;
}

parameter_types! {
	pub const MinVestedTransfer: Balance = MIN_VESTED_TRANSFER_AMOUNT;
}

impl pallet_vesting::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	// disable vested transfers by setting min amount to max balance
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = weights::pallet_vesting::WeightInfo<Runtime>;
}

parameter_types! {
	pub const MaxClaims: u32 = 50;
	pub const UsableBalance: Balance = KILT;
}

impl kilt_launch::Config for Runtime {
	type Event = Event;
	type MaxClaims = MaxClaims;
	type UsableBalance = UsableBalance;
	type WeightInfo = weights::kilt_launch::WeightInfo<Runtime>;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * RuntimeBlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = weights::pallet_scheduler::WeightInfo<Runtime>;
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = LAUNCH_PERIOD;
	pub const VotingPeriod: BlockNumber = VOTING_PERIOD;
	pub const FastTrackVotingPeriod: BlockNumber = FAST_TRACK_VOTING_PERIOD;
	pub const MinimumDeposit: Balance = KILT;
	pub const EnactmentPeriod: BlockNumber = ENACTMENT_PERIOD;
	pub const CooloffPeriod: BlockNumber = COOLOFF_PERIOD;
	// One cent: $10,000 / MB
	pub const PreimageByteDeposit: Balance = 10 * MILLI_KILT;
	pub const InstantAllowed: bool = true;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 100;
}

impl pallet_democracy::Config for Runtime {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type MinimumDeposit = MinimumDeposit;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin = pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>;
	/// A majority can have the next scheduled referendum be a straight
	/// majority-carries vote.
	type ExternalMajorityOrigin = pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>;
	/// A unanimous council can have the next scheduled referendum be a straight
	/// default-carries (NTB) vote.
	type ExternalDefaultOrigin = pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>;
	/// Two thirds of the technical committee can have an
	/// ExternalMajority/ExternalDefault vote be tabled immediately and with a
	/// shorter voting/enactment period.
	type FastTrackOrigin = pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, TechnicalCollective>;
	type InstantOrigin = pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to
	// it.
	type CancellationOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollective>,
	>;
	// To cancel a proposal before it has been passed, the technical committee must
	// be unanimous or Root must agree.
	type CancelProposalOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>,
	>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// Any single technical committee member may veto a coming council proposal,
	// however they can only do it once and it lasts only for the cooloff period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
	type CooloffPeriod = CooloffPeriod;
	type PreimageByteDeposit = PreimageByteDeposit;
	type Slash = Treasury;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = MaxVotes;
	type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
	type MaxProposals = MaxProposals;

	type WeightInfo = weights::pallet_democracy::WeightInfo<Runtime>;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 20 * KILT; // TODO: how much?
	pub const SpendPeriod: BlockNumber = SPEND_PERIOD;
	pub const Burn: Permill = Permill::zero();
	pub const MaxApprovals: u32 = 100;
}

type ApproveOrigin = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_3, _5, AccountId, CouncilCollective>,
>;

type MoreThanHalfCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
>;

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = ApproveOrigin;
	type RejectOrigin = MoreThanHalfCouncil;
	type Event = Event;
	type OnSlash = Treasury;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendFunds = ();
	type WeightInfo = weights::pallet_treasury::WeightInfo<Runtime>;
	type MaxApprovals = MaxApprovals;
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = COUNCIL_MOTION_DURATION;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
}

parameter_types! {
	pub const TechnicalMotionDuration: BlockNumber = TECHNICAL_MOTION_DURATION;
	pub const TechnicalMaxProposals: u32 = 100;
	pub const TechnicalMaxMembers: u32 = 100;
}

type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = TechnicalMaxProposals;
	type MaxMembers = TechnicalMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
}

impl pallet_membership::Config for Runtime {
	type Event = Event;
	type AddOrigin = MoreThanHalfCouncil;
	type RemoveOrigin = MoreThanHalfCouncil;
	type SwapOrigin = MoreThanHalfCouncil;
	type ResetOrigin = MoreThanHalfCouncil;
	type PrimeOrigin = MoreThanHalfCouncil;
	type MembershipInitialized = TechnicalCommittee;
	type MembershipChanged = TechnicalCommittee;
	type MaxMembers = TechnicalMaxMembers;
	type WeightInfo = weights::pallet_membership::WeightInfo<Runtime>;
}

pub struct DelegationSignatureVerifier<R>(sp_std::marker::PhantomData<R>);
impl<R: did::Config> delegation::VerifyDelegateSignature for DelegationSignatureVerifier<R> {
	type DelegateId = <R as did::Config>::DidIdentifier;
	type Payload = Vec<u8>;
	type Signature = DidSignature;

	fn verify(
		delegate: &Self::DelegateId,
		payload: &Self::Payload,
		signature: &Self::Signature,
	) -> delegation::SignatureVerificationResult {
		let delegate_details =
			did::Did::<R>::get(delegate).ok_or(delegation::SignatureVerificationError::SignerInformationNotPresent)?;

		did::Pallet::verify_payload_signature_with_did_key_type(
			payload,
			signature,
			&delegate_details,
			did::DidVerificationKeyRelationship::Authentication,
		)
		.map_err(|err| match err {
			// Should never happen as a DID has always a valid authentication key and UrlErrors are never thrown here.
			did::DidError::SignatureError(_) => delegation::SignatureVerificationError::SignatureInvalid,
			_ => delegation::SignatureVerificationError::SignerInformationNotPresent,
		})
	}
}

parameter_types! {
	pub const MaxDelegatedAttestations: u32 = 1000;
	pub const AttestationDeposit: Balance = 100 * MILLI_KILT;
}

impl attestation::Config for Runtime {
	#[cfg(not(feature = "runtime-benchmarks"))]
	type EnsureOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type OriginSuccess = did::DidRawOrigin<AccountId, DidIdentifier>;

	#[cfg(feature = "runtime-benchmarks")]
	type EnsureOrigin = EnsureSigned<DidIdentifier>;
	#[cfg(feature = "runtime-benchmarks")]
	type OriginSuccess = DidIdentifier;

	type Event = Event;
	type WeightInfo = weights::attestation::WeightInfo<Runtime>;

	type Currency = Balances;
	type Deposit = AttestationDeposit;
	type MaxDelegatedAttestations = MaxDelegatedAttestations;
}

parameter_types! {
	pub const MaxSignatureByteLength: u16 = 64;
	pub const MaxParentChecks: u32 = 5;
	pub const MaxRevocations: u32 = 5;
	// TODO: Find reasonable number
	#[derive(Clone)]
	pub const MaxChildren: u32 = 1000;
}

impl delegation::Config for Runtime {
	type Signature = DidSignature;
	type DelegationSignatureVerification = DelegationSignatureVerifier<Runtime>;
	type DelegationEntityId = AccountId;
	type DelegationNodeId = Hash;

	#[cfg(not(feature = "runtime-benchmarks"))]
	type EnsureOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type OriginSuccess = did::DidRawOrigin<AccountId, DidIdentifier>;

	#[cfg(feature = "runtime-benchmarks")]
	type EnsureOrigin = EnsureSigned<DidIdentifier>;
	#[cfg(feature = "runtime-benchmarks")]
	type OriginSuccess = DidIdentifier;

	type Event = Event;
	type MaxSignatureByteLength = MaxSignatureByteLength;
	type MaxParentChecks = MaxParentChecks;
	type MaxRevocations = MaxRevocations;
	type MaxChildren = MaxChildren;
	type WeightInfo = weights::delegation::WeightInfo<Runtime>;
}

pub struct CtypeFeeHandler<R>(sp_std::marker::PhantomData<R>);
impl<R> PayFee<<R as frame_system::Config>::AccountId> for CtypeFeeHandler<R>
where
	R: pallet_balances::Config + frame_system::Config + pallet_treasury::Config,
	<R as pallet_balances::Config>::Balance: From<Balance>,
{
	fn secure_fee(payer: <R as frame_system::Config>::AccountId, ctype_size: usize) -> Result<(), ()> {
		let fee = ctype_size.saturated_into::<Balance>() * MICRO_KILT;

		if pallet_balances::Pallet::<R>::free_balance(payer) > fee.into() {
			Ok(())
		} else {
			Err(())
		}
	}

	fn pay_fee(payer: <R as frame_system::Config>::AccountId, ctype_size: usize) {
		let treasury_account = pallet_treasury::Pallet::<R>::account_id();
		let fee = ctype_size.saturated_into::<Balance>() * MICRO_KILT;

		// this could fail if the balance was reduced in the meantime.
		// Since we know that this is not happening in the ctype pallet, this call should never fail.
		let result = <pallet_balances::Pallet<R> as Currency<_>>::transfer(
			&payer,
			&treasury_account,
			fee.into(),
			ExistenceRequirement::AllowDeath,
		);
		debug_assert!(result.is_ok());
	}
}

impl ctype::Config for Runtime {
	type CtypeCreatorId = AccountId;
	type FeeHandler = CtypeFeeHandler<Runtime>;

	#[cfg(not(feature = "runtime-benchmarks"))]
	type EnsureOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type OriginSuccess = did::DidRawOrigin<AccountId, DidIdentifier>;

	#[cfg(feature = "runtime-benchmarks")]
	type EnsureOrigin = EnsureSigned<DidIdentifier>;
	#[cfg(feature = "runtime-benchmarks")]
	type OriginSuccess = DidIdentifier;

	type Event = Event;
	type WeightInfo = weights::ctype::WeightInfo<Runtime>;
}

parameter_types! {
	pub const MaxNewKeyAgreementKeys: u32 = MAX_KEY_AGREEMENT_KEYS;
	#[derive(Debug, Clone, PartialEq)]
	pub const MaxUrlLength: u32 = MAX_URL_LENGTH;
	pub const MaxPublicKeysPerDid: u32 = MAX_PUBLIC_KEYS_PER_DID;
	#[derive(Debug, Clone, PartialEq)]
	pub const MaxTotalKeyAgreementKeys: u32 = MAX_TOTAL_KEY_AGREEMENT_KEYS;
	#[derive(Debug, Clone, PartialEq)]
	pub const MaxEndpointUrlsCount: u32 = MAX_ENDPOINT_URLS_COUNT;
	// Standalone block time is half the duration of a parachain block.
	pub const MaxBlocksTxValidity: BlockNumber = MAX_BLOCKS_TX_VALIDITY;
}

impl did::Config for Runtime {
	type DidIdentifier = DidIdentifier;
	type Event = Event;
	type Call = Call;
	type Origin = Origin;

	#[cfg(not(feature = "runtime-benchmarks"))]
	type EnsureOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type OriginSuccess = did::DidRawOrigin<AccountId, DidIdentifier>;

	#[cfg(feature = "runtime-benchmarks")]
	type EnsureOrigin = EnsureSigned<DidIdentifier>;
	#[cfg(feature = "runtime-benchmarks")]
	type OriginSuccess = DidIdentifier;

	type MaxNewKeyAgreementKeys = MaxNewKeyAgreementKeys;
	type MaxTotalKeyAgreementKeys = MaxTotalKeyAgreementKeys;
	type MaxPublicKeysPerDid = MaxPublicKeysPerDid;
	type MaxUrlLength = MaxUrlLength;
	type MaxEndpointUrlsCount = MaxEndpointUrlsCount;
	type MaxBlocksTxValidity = MaxBlocksTxValidity;
	type WeightInfo = weights::did::WeightInfo<Runtime>;
}

parameter_types! {
	/// Minimum round length is 1 hour
	pub const MinBlocksPerRound: BlockNumber = MIN_BLOCKS_PER_ROUND;
	/// Default BlocksPerRound is every 6 hours
	pub const DefaultBlocksPerRound: BlockNumber = DEFAULT_BLOCKS_PER_ROUND;
	/// Unstaked balance can be unlocked after 7 days
	pub const StakeDuration: BlockNumber = STAKE_DURATION;
	/// Collator exit requests are delayed by 4 (2 rounds/sessions)
	pub const ExitQueueDelay: u32 = 2;
	/// Minimum 16 collators selected per round, default at genesis and minimum forever after
	pub const MinCollators: u32 = MIN_COLLATORS;
	/// At least 4 candidates which cannot leave the network if there are no other candidates.
	pub const MinRequiredCollators: u32 = 4;
	/// We only allow one delegation per round.
	pub const MaxDelegationsPerRound: u32 = 1;
	/// Maximum 25 delegators per collator at launch, might be increased later
	#[derive(Debug, PartialEq)]
	pub const MaxDelegatorsPerCollator: u32 = 25;
	/// Maximum 1 collator per delegator at launch, will be increased later
	#[derive(Debug, PartialEq)]
	pub const MaxCollatorsPerDelegator: u32 = 1;
	/// Minimum stake required to be reserved to be a collator is 10_000
	pub const MinCollatorStake: Balance = 10_000 * KILT;
	/// Minimum stake required to be reserved to be a delegator is 1000
	pub const MinDelegatorStake: Balance = 1000 * KILT;
	/// Maximum number of collator candidates
	#[derive(Debug, PartialEq)]
	pub const MaxCollatorCandidates: u32 = MAX_CANDIDATES;
	/// Maximum number of concurrent requests to unlock unstaked balance
	pub const MaxUnstakeRequests: u32 = 10;
}

impl parachain_staking::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type CurrencyBalance = Balance;
	type MinBlocksPerRound = MinBlocksPerRound;
	type DefaultBlocksPerRound = DefaultBlocksPerRound;
	type StakeDuration = StakeDuration;
	type ExitQueueDelay = ExitQueueDelay;
	type MinCollators = MinCollators;
	type MinRequiredCollators = MinRequiredCollators;
	type MaxDelegationsPerRound = MaxDelegationsPerRound;
	type MaxDelegatorsPerCollator = MaxDelegatorsPerCollator;
	type MaxCollatorsPerDelegator = MaxCollatorsPerDelegator;
	type MinCollatorStake = MinCollatorStake;
	type MinCollatorCandidateStake = MinCollatorStake;
	type MaxTopCandidates = MaxCollatorCandidates;
	type MinDelegation = MinDelegatorStake;
	type MinDelegatorStake = MinDelegatorStake;
	type MaxUnstakeRequests = MaxUnstakeRequests;
	type WeightInfo = weights::parachain_staking::WeightInfo<Runtime>;
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type WeightInfo = weights::pallet_utility::WeightInfo<Runtime>;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

construct_runtime! {
	pub enum Runtime where
		Block = Block,
		NodeBlock = kilt_primitives::Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		// Basic stuff; balances is uncallable initially.
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage} = 1,

		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 2,
		Indices: pallet_indices::{Pallet, Call, Storage, Event<T>} = 5,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 6,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 7,
		Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 8,

		// Consensus support.
		// The following order MUST NOT be changed: Authorship -> Staking -> Session -> Aura -> AuraExt
		Authorship: pallet_authorship::{Pallet, Call, Storage} = 20,
		ParachainStaking: parachain_staking::{Pallet, Call, Storage, Event<T>, Config<T>} = 21,
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 22,
		Aura: pallet_aura::{Pallet, Config<T>} = 23,
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Config} = 24,

		// Governance stuff; uncallable initially.
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>} = 30,
		Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 31,
		TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 32,
		// placeholder: parachain council election = 33,
		TechnicalMembership: pallet_membership::{Pallet, Call, Storage, Event<T>, Config<T>} = 34,
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 35,

		// Utility module.
		Utility: pallet_utility::{Pallet, Call, Storage, Event} = 40,

		// Vesting. Usable initially, but removed once all vesting is finished.
		Vesting: pallet_vesting::{Pallet, Call, Storage, Event<T>, Config<T>} = 41,

		// System scheduler.
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 42,

		// KILT Pallets. Start indices 60 to leave room
		KiltLaunch: kilt_launch::{Pallet, Call, Storage, Event<T>, Config<T>} = 60,
		Ctype: ctype::{Pallet, Call, Storage, Event<T>} = 61,
		Attestation: attestation::{Pallet, Call, Storage, Event<T>} = 62,
		Delegation: delegation::{Pallet, Call, Storage, Event<T>} = 63,
		Did: did::{Pallet, Call, Storage, Event<T>, Origin<T>} = 64,

		// Parachains pallets. Start indices at 80 to leave room.
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>, Config} = 80,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 81,

		// Pallet for sending XCM.
		// XcmHandler: cumulus_pallet_xcmp_queue::{Pallet, Call, Event<T>, Origin} = 100,
		// Tokens: orml_tokens::{Pallet, Call, Storage, Event<T>} = 101,
		// Currencies: orml_currencies::{Pallet, Call, Storage, Event<T>} = 102,
		// XTokens: orml_xtokens::{Pallet, Call, Storage, Event<T>} = 103,
		// UnknownTokens: orml_unknown_tokens::{Pallet, Storage, Event} = 104,

	}
}

impl did::DeriveDidCallAuthorizationVerificationKeyRelationship for Call {
	fn derive_verification_key_relationship(&self) -> Option<did::DidVerificationKeyRelationship> {
		match self {
			Call::Attestation(_) => Some(did::DidVerificationKeyRelationship::AssertionMethod),
			Call::Ctype(_) => Some(did::DidVerificationKeyRelationship::AssertionMethod),
			Call::Delegation(_) => Some(did::DidVerificationKeyRelationship::CapabilityDelegation),
			// DID creation is not allowed through the DID proxy.
			Call::Did(did::Call::create(..)) => None,
			Call::Did(_) => Some(did::DidVerificationKeyRelationship::Authentication),
			//TODO: add a batch call case that returns the right key type if all calls in the batch require the same
			// key type as well, otherwise it returns None and fails.
			#[cfg(not(feature = "runtime-benchmarks"))]
			_ => None,
			// By default, returns the authentication key
			#[cfg(feature = "runtime-benchmarks")]
			_ => Some(did::DidVerificationKeyRelationship::Authentication),
		}
	}

	// Always return a System::remark() extrinsic call
	#[cfg(feature = "runtime-benchmarks")]
	fn get_call_for_did_call_benchmark() -> Self {
		Call::System(frame_system::Call::remark(vec![]))
	}
}

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
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
/// Executive: handles dispatch to the various modules.
pub type Executive =
	frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPallets>;

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
			Runtime::metadata().into()
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

		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(
			extrinsic: <Block as BlockT>::Extrinsic,
		) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(block: Block, data: sp_inherents::InherentData) -> sp_inherents::CheckInherentsResult {
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
		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}

		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuthorityId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuthorityId> {
			Aura::authorities()
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info() -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info()
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{list_benchmark, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			// use cumulus_pallet_session_benchmarking::Pallet as SessionBench;

			let mut list = Vec::<BenchmarkList>::new();

			// Substrate
			list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
			list_benchmark!(list, extra, pallet_balances, Balances);
			list_benchmark!(list, extra, pallet_collective, Council);
			list_benchmark!(list, extra, pallet_democracy, Democracy);
			list_benchmark!(list, extra, pallet_indices, Indices);
			list_benchmark!(list, extra, pallet_membership, TechnicalMembership);
			list_benchmark!(list, extra, parachain_staking, ParachainStaking);
			list_benchmark!(list, extra, pallet_scheduler, Scheduler);
			// list_benchmark!(list, extra, pallet_session, Session);
			list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
			list_benchmark!(list, extra, pallet_timestamp, Timestamp);
			list_benchmark!(list, extra, pallet_treasury, Treasury);
			list_benchmark!(list, extra, pallet_utility, Utility);

			list_benchmark!(list, extra, attestation, Attestation);
			list_benchmark!(list, extra, ctype, Ctype);
			list_benchmark!(list, extra, delegation, Delegation);
			list_benchmark!(list, extra, did, Did);
			list_benchmark!(list, extra, kilt_launch, KiltLaunch);
			list_benchmark!(list, extra, pallet_vesting, Vesting);

			// No benchmarks for these pallets
			// list_benchmark!(list, extra, cumulus_pallet_parachain_system, ParachainSystem);
			// list_benchmark!(list, extra, parachain_info, ParachainInfo);
			// list_benchmark!(list, extra, cumulus_pallet_xcmp_queue, XcmHandler);
			// list_benchmark!(list, extra, orml_tokens, Tokens);
			// list_benchmark!(list, extra, orml_currencies, Currencies);
			// list_benchmark!(list, extra, orml_xtokens, XTokens);
			// list_benchmark!(list, extra, orml_unknown_tokens, UnknownTokens);

			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			// use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			// impl cumulus_pallet_session_benchmarking::Config for Runtime {}

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
				// KiltLaunch transfer account
				hex_literal::hex!("6a3c793cec9dbe330b349dc4eea6801090f5e71f53b1b41ad11afb4a313a282c").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, pallet_balances, Balances);
			add_benchmark!(params, batches, pallet_collective, Council);
			add_benchmark!(params, batches, pallet_democracy, Democracy);
			add_benchmark!(params, batches, pallet_indices, Indices);
			add_benchmark!(params, batches, pallet_membership, TechnicalMembership);
			add_benchmark!(params, batches, parachain_staking, ParachainStaking);
			add_benchmark!(params, batches, pallet_scheduler, Scheduler);
			// add_benchmark!(params, batches, pallet_session, SessionBench::<Runtime>);
			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);
			add_benchmark!(params, batches, pallet_treasury, Treasury);
			add_benchmark!(params, batches, pallet_utility, Utility);

			add_benchmark!(params, batches, attestation, Attestation);
			add_benchmark!(params, batches, ctype, Ctype);
			add_benchmark!(params, batches, delegation, Delegation);
			add_benchmark!(params, batches, did, Did);
			add_benchmark!(params, batches, kilt_launch, KiltLaunch);
			add_benchmark!(params, batches, pallet_vesting, Vesting);

			// No benchmarks for these pallets
			// add_benchmark!(params, batches, cumulus_pallet_parachain_system, ParachainSystem);
			// add_benchmark!(params, batches, parachain_info, ParachainInfo);
			// add_benchmark!(params, batches, cumulus_pallet_xcmp_queue, XcmHandler);
			// add_benchmark!(params, batches, orml_tokens, Tokens);
			// add_benchmark!(params, batches, orml_currencies, Currencies);
			// add_benchmark!(params, batches, orml_xtokens, XTokens);
			// add_benchmark!(params, batches, orml_unknown_tokens, UnknownTokens);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}

	// From the Polkadot repo: https://github.com/paritytech/polkadot/blob/1876963f254f31f8cd2d7b8d5fb26cd38b7836ab/runtime/polkadot/src/lib.rs#L1413
	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade() -> Result<(Weight, Weight), sp_runtime::RuntimeString> {
			log::info!("try-runtime::on_runtime_upgrade for peregrine runtime.");
			let weight = Executive::try_runtime_upgrade().map_err(|err|{
				log::info!("try-runtime::on_runtime_upgrade failed with: {:?}", err);
				err
			})?;
			Ok((weight, RuntimeBlockWeights::get().max_block))
		}
	}
}

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data = cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
			relay_chain_slot,
			sp_std::time::Duration::from_secs(6),
		)
		.create_inherent_data()
		.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(block)
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
}
