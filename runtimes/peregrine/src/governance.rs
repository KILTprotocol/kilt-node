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

use cfg_if::cfg_if;
use frame_support::{
	parameter_types,
	traits::{
		fungible::HoldConsideration,
		tokens::{PayFromAccount, UnityAssetBalanceConversion},
		ChangeMembers, EitherOfDiverse, LinearStoragePrice, PrivilegeCmp,
	},
	weights::Weight,
};
use frame_system::{EnsureRoot, EnsureSigned};
use runtime_common::{
	constants::{self, KILT},
	pallet_id, AccountId, Balance, BlockWeights, Tippers,
};
use sp_core::{ConstBool, ConstU128, ConstU32, ConstU64};
use sp_runtime::{traits::AccountIdLookup, Perbill, Permill};
use sp_std::cmp::Ordering;

use crate::{
	benchmarks, weights, Balances, OriginCaller, Preimage, Runtime, RuntimeCall, RuntimeEvent, RuntimeHoldReason,
	RuntimeOrigin, Scheduler, TechnicalCommittee, Treasury,
};

pub type RootOrCollectiveProportion<Collective, const NUM: u32, const DEN: u32> =
	EitherOfDiverse<EnsureRoot<AccountId>, pallet_collective::EnsureProportionAtLeast<AccountId, Collective, 2, 3>>;

impl pallet_democracy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type EnactmentPeriod = constants::governance::EnactmentPeriod;
	type VoteLockingPeriod = constants::governance::VotingPeriod;
	type LaunchPeriod = constants::governance::LaunchPeriod;
	type VotingPeriod = constants::governance::VotingPeriod;
	type MinimumDeposit = constants::governance::MinimumDeposit;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>;
	/// A majority can have the next scheduled referendum be a straight
	/// majority-carries vote.
	type ExternalMajorityOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>;
	/// A unanimous council can have the next scheduled referendum be a straight
	/// default-carries (NTB) vote.
	type ExternalDefaultOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>;
	/// Two thirds of the technical committee can have an
	/// ExternalMajority/ExternalDefault vote be tabled immediately and with a
	/// shorter voting/enactment period.
	type FastTrackOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 2, 3>;
	type InstantOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>;
	type InstantAllowed = ConstBool<true>;
	type FastTrackVotingPeriod = constants::governance::FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to
	// it.
	type CancellationOrigin = RootOrCollectiveProportion<CouncilCollective, 2, 3>;
	// To cancel a proposal before it has been passed, the technical committee must
	// be unanimous or Root must agree.
	type CancelProposalOrigin = RootOrCollectiveProportion<TechnicalCollective, 1, 1>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// Any single technical committee member may veto a coming council proposal,
	// however they can only do it once and it lasts only for the cooloff period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
	type CooloffPeriod = constants::governance::CooloffPeriod;
	type Slash = Treasury;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = ConstU32<100>;
	type WeightInfo = weights::pallet_democracy::WeightInfo<Runtime>;
	type MaxProposals = ConstU32<100>;
	type Preimages = Preimage;
	type MaxDeposits = ConstU32<100>;
	type MaxBlacklisted = ConstU32<100>;
	type SubmitOrigin = EnsureSigned<AccountId>;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const Burn: Permill = Permill::zero();
	pub const TreasuryAccount: AccountId = Treasury::account_id();
}

impl pallet_treasury::Config for Runtime {
	type PalletId = pallet_id::Treasury;
	type Currency = Balances;
	type ApproveOrigin = RootOrCollectiveProportion<CouncilCollective, 3, 5>;
	type RejectOrigin = RootOrCollectiveProportion<CouncilCollective, 1, 2>;
	type RuntimeEvent = RuntimeEvent;
	type OnSlash = Treasury;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ConstU128<{ 20 * KILT }>;
	type ProposalBondMaximum = ();
	type SpendPeriod = ConstU64<{ constants::governance::SPEND_PERIOD }>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<Balance>;
	#[cfg(feature = "runtime-benchmarks")]
	type SpendOrigin = frame_system::EnsureWithSuccess<EnsureRoot<AccountId>, AccountId, benchmarks::MaxBalance>;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendFunds = ();
	type WeightInfo = weights::pallet_treasury::WeightInfo<Runtime>;
	type MaxApprovals = ConstU32<100>;
	type AssetKind = ();
	type BalanceConverter = UnityAssetBalanceConversion;
	type Beneficiary = AccountId;
	type BeneficiaryLookup = AccountIdLookup<Self::Beneficiary, ()>;
	type Paymaster = PayFromAccount<Balances, TreasuryAccount>;
	type PayoutPeriod = runtime_common::constants::treasury::PayoutPeriod;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = runtime_common::benchmarks::treasury::BenchmarkHelper<Runtime>;
}

impl pallet_tips::Config for Runtime {
	type MaximumReasonLength = constants::tips::MaximumReasonLength;
	type DataDepositPerByte = constants::ByteDeposit;
	type Tippers = Tippers<Runtime, TipsMembershipProvider>;
	type TipCountdown = constants::tips::TipCountdown;
	type TipFindersFee = constants::tips::TipFindersFee;
	type TipReportDepositBase = constants::tips::TipReportDepositBase;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_tips::WeightInfo<Runtime>;
	type MaxTipAmount = constants::tips::MaxTipAmount;
}

#[allow(clippy::arithmetic_side_effects)]
#[inline]
fn maximum_proposal_weight() -> Weight {
	Perbill::from_percent(80) * BlockWeights::get().max_block
}

parameter_types! {
	pub MaxProposalWeight: Weight = maximum_proposal_weight();
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MaxProposalWeight = MaxProposalWeight;
	type MotionDuration = constants::governance::CouncilMotionDuration;
	type MaxProposals = constants::governance::CouncilMaxProposals;
	type MaxMembers = constants::governance::CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
	type SetMembersOrigin = EnsureRoot<AccountId>;
}

type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type MaxProposalWeight = MaxProposalWeight;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = constants::governance::TechnicalMotionDuration;
	type MaxProposals = constants::governance::TechnicalMaxProposals;
	type MaxMembers = constants::governance::TechnicalMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = weights::pallet_technical_committee_collective::WeightInfo<Runtime>;
	type SetMembersOrigin = EnsureRoot<AccountId>;
}

pub type RootOrMoreThanHalfCouncil = RootOrCollectiveProportion<CouncilCollective, 1, 2>;

type TechnicalMembershipProvider = pallet_membership::Instance1;
impl pallet_membership::Config<TechnicalMembershipProvider> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddOrigin = RootOrMoreThanHalfCouncil;
	type RemoveOrigin = RootOrMoreThanHalfCouncil;
	type SwapOrigin = RootOrMoreThanHalfCouncil;
	type ResetOrigin = RootOrMoreThanHalfCouncil;
	type PrimeOrigin = RootOrMoreThanHalfCouncil;
	type MembershipInitialized = TechnicalCommittee;
	type MembershipChanged = TechnicalCommittee;
	type MaxMembers = constants::governance::TechnicalMaxMembers;
	type WeightInfo = weights::pallet_technical_membership::WeightInfo<Runtime>;
}

// Implementation of `MembershipChanged` equivalent to using `()` but that
// returns `Some(AccountId::new([0; 32]))` in `get_prime()` only when
// benchmarking. TODO: Remove once we upgrade with a version containing the fix: https://github.com/paritytech/polkadot-sdk/pull/6439
pub struct MockMembershipChangedForBenchmarks;

impl ChangeMembers<AccountId> for MockMembershipChangedForBenchmarks {
	fn change_members_sorted(incoming: &[AccountId], outgoing: &[AccountId], sorted_new: &[AccountId]) {
		<()>::change_members_sorted(incoming, outgoing, sorted_new)
	}

	fn get_prime() -> Option<AccountId> {
		cfg_if! {
			if #[cfg(feature = "runtime-benchmark")] {
				Some(AccountId::new([0; 32]))
			} else {
				<()>::get_prime()
			}
		}
	}
}

type TipsMembershipProvider = pallet_membership::Instance2;
impl pallet_membership::Config<TipsMembershipProvider> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddOrigin = RootOrMoreThanHalfCouncil;
	type RemoveOrigin = RootOrMoreThanHalfCouncil;
	type SwapOrigin = RootOrMoreThanHalfCouncil;
	type ResetOrigin = RootOrMoreThanHalfCouncil;
	type PrimeOrigin = RootOrMoreThanHalfCouncil;
	type MembershipInitialized = ();
	type MembershipChanged = MockMembershipChangedForBenchmarks;
	type MaxMembers = constants::governance::TipperMaxMembers;
	type WeightInfo = weights::pallet_membership::WeightInfo<Runtime>;
}

parameter_types! {
	pub const PreImageHoldReason: RuntimeHoldReason = RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
}

impl pallet_preimage::Config for Runtime {
	type WeightInfo = weights::pallet_preimage::WeightInfo<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type Consideration = HoldConsideration<
		AccountId,
		Balances,
		PreImageHoldReason,
		LinearStoragePrice<constants::preimage::PreimageBaseDeposit, constants::ByteDeposit, Balance>,
	>;
}

#[allow(clippy::arithmetic_side_effects)]
#[inline]
fn maximum_scheduler_weight() -> Weight {
	Perbill::from_percent(80) * BlockWeights::get().max_block
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = maximum_scheduler_weight();
}

/// Used the compare the privilege of an origin inside the scheduler.
pub struct OriginPrivilegeCmp;

impl PrivilegeCmp<OriginCaller> for OriginPrivilegeCmp {
	fn cmp_privilege(left: &OriginCaller, right: &OriginCaller) -> Option<Ordering> {
		if left == right {
			return Some(Ordering::Equal);
		}

		match (left, right) {
			// Root is greater than anything.
			(OriginCaller::system(frame_system::RawOrigin::Root), _) => Some(Ordering::Greater),
			// Check which one has more yes votes.
			(
				OriginCaller::Council(pallet_collective::RawOrigin::Members(l_yes_votes, l_count)),
				OriginCaller::Council(pallet_collective::RawOrigin::Members(r_yes_votes, r_count)),
			) => Some((l_yes_votes.saturating_mul(*r_count)).cmp(&(r_yes_votes.saturating_mul(*l_count)))),
			// For every other origin we don't care, as they are not used for `ScheduleOrigin`.
			_ => None,
		}
	}
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = RootOrCollectiveProportion<CouncilCollective, 1, 2>;
	type MaxScheduledPerBlock = ConstU32<50>;
	type WeightInfo = weights::pallet_scheduler::WeightInfo<Runtime>;
	type OriginPrivilegeCmp = OriginPrivilegeCmp;
	type Preimages = Preimage;
}
