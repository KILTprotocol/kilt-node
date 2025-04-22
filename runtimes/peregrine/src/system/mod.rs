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

use frame_support::{
	parameter_types,
	traits::{AsEnsureOriginWithArg, Everything, PrivilegeCmp},
	weights::Weight,
};
use frame_system::EnsureRoot;
use runtime_common::{
	asset_switch::EnsureRootAsTreasury,
	constants,
	fees::{ToAuthorCredit, WeightToFee},
	AccountId, AuthorityId, Balance, BlockHashCount, BlockLength, BlockWeights, FeeSplit, Hash, Nonce,
	SendDustAndFeesToTreasury, SlowAdjustingFeeUpdate,
};
use sp_core::{ConstBool, ConstU128, ConstU16, ConstU32, ConstU64};
use sp_runtime::{
	impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, ConvertInto, OpaqueKeys},
	Perbill,
};
use sp_std::{cmp::Ordering, vec::Vec};
use sp_version::RuntimeVersion;
use sp_weights::ConstantMultiplier;
use xcm::v4::Location;

use crate::{
	governance::{CouncilCollective, RootOrCollectiveProportion, RootOrMoreThanHalfCouncil},
	weights, Aura, Balances, Block, OriginCaller, PalletInfo, ParachainStaking, PermissionedCollator, Preimage,
	Runtime, RuntimeCall, RuntimeEvent, RuntimeFreezeReason, RuntimeHoldReason, RuntimeOrigin, RuntimeTask, System,
	VERSION,
};

pub(crate) mod proxy;

pub const SS_58_PREFIX: u16 = 38;

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
}

impl frame_system::Config for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	/// The lookup mechanism to get account ID from whatever is passed in
	/// dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The nonce type for storing how many extrinsics an account has signed.
	type Nonce = Nonce;
	/// The block type as expected in this runtime
	type Block = Block;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeTask = RuntimeTask;
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
	type DbWeight = weights::rocksdb_weights::constants::RocksDbWeight;
	type BaseCallFilter = Everything;
	type SystemWeightInfo = crate::weights::frame_system::WeightInfo<Runtime>;
	type BlockWeights = BlockWeights;
	type BlockLength = BlockLength;
	type SS58Prefix = ConstU16<SS_58_PREFIX>;
	/// The set code logic
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Runtime>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<{ constants::timestamp::MINIMUM_PERIOD }>;
	type WeightInfo = weights::pallet_timestamp::WeightInfo<Runtime>;
}

impl pallet_balances::Config for Runtime {
	/// The type for recording an account's balance.
	type Balance = Balance;
	type FreezeIdentifier = RuntimeFreezeReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxFreezes = ConstU32<50>;

	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = SendDustAndFeesToTreasury<Runtime>;
	type ExistentialDeposit = ConstU128<{ constants::EXISTENTIAL_DEPOSIT }>;
	type AccountStore = System;
	type WeightInfo = weights::pallet_balances::WeightInfo<Runtime>;
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<
		Balances,
		FeeSplit<Runtime, SendDustAndFeesToTreasury<Runtime>, ToAuthorCredit<Runtime>>,
	>;
	type OperationalFeeMultiplier = constants::fee::OperationalFeeMultiplier;
	type WeightToFee = WeightToFee<Runtime>;
	type LengthToFee = ConstantMultiplier<Balance, constants::fee::TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

pub struct SessionManager;

impl pallet_session::SessionManager<AccountId> for SessionManager {
	fn new_session(new_index: sp_staking::SessionIndex) -> Option<Vec<AccountId>> {
		let collators = PermissionedCollator::members().to_vec();

		log::debug!(
			"assembling new collators for new session {} at #{:?} with {:?}",
			new_index,
			System::block_number(),
			collators
		);

		System::register_extra_weight_unchecked(
			<Runtime as frame_system::Config>::DbWeight::get().reads(2),
			frame_support::pallet_prelude::DispatchClass::Mandatory,
		);

		if collators.is_empty() {
			// we never want to pass an empty set of collators. This would brick the chain.
			log::error!("💥 keeping old session because of empty collator set!");
			return None;
		}

		Some(collators)
	}

	fn start_session(_start_index: sp_staking::SessionIndex) {
		// We don't care
	}

	fn end_session(_end_index: sp_staking::SessionIndex) {
		// We don't care
	}
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = AccountId;
	type ValidatorIdOf = ConvertInto;
	type ShouldEndSession = ParachainStaking;
	type NextSessionRotation = ParachainStaking;
	type SessionManager = SessionManager;
	type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = weights::pallet_session::WeightInfo<Runtime>;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuthorityId;
	//TODO: handle disabled validators
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<{ constants::staking::MAX_CANDIDATES }>;
	type AllowMultipleBlocksPerSlot = ConstBool<false>;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type EventHandler = ();
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = weights::pallet_utility::WeightInfo<Runtime>;
}

impl pallet_vesting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	type BlockNumberProvider = System;
	// disable vested transfers by setting min amount to max balance
	type MinVestedTransfer = constants::MinVestedTransfer;
	type WeightInfo = weights::pallet_vesting::WeightInfo<Runtime>;
	type UnvestedFundsAllowedWithdrawReasons = constants::UnvestedFundsAllowedWithdrawReasons;
	const MAX_VESTING_SCHEDULES: u32 = constants::MAX_VESTING_SCHEDULES;
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = constants::multisig::DepositBase;
	type DepositFactor = constants::multisig::DepositFactor;
	type MaxSignatories = constants::multisig::MaxSignitors;
	type WeightInfo = weights::pallet_multisig::WeightInfo<Runtime>;
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = Nonce;
	type Currency = Balances;
	type Deposit = constants::IndicesDeposit;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_indices::WeightInfo<Runtime>;
}

impl pallet_sudo::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_sudo::WeightInfo<Runtime>;
}

// No deposit is taken since creation is permissioned. Only the root origin can
// create new assets, and the owner will be the treasury account.
impl pallet_assets::Config for Runtime {
	type ApprovalDeposit = constants::assets::ApprovalDeposit;
	type AssetAccountDeposit = constants::assets::AssetAccountDeposit;
	type AssetDeposit = constants::assets::AssetDeposit;
	type AssetId = Location;
	type AssetIdParameter = Location;
	type Balance = Balance;
	type CallbackHandle = ();
	type CreateOrigin = AsEnsureOriginWithArg<EnsureRootAsTreasury<Runtime>>;
	type Currency = Balances;
	type Extra = ();
	type ForceOrigin = EnsureRoot<AccountId>;
	type Freezer = ();
	type MetadataDepositBase = constants::assets::MetaDepositBase;
	type MetadataDepositPerByte = constants::assets::MetaDepositPerByte;
	type RemoveItemsLimit = constants::assets::RemoveItemsLimit;
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = constants::assets::StringLimit;
	type WeightInfo = weights::pallet_assets::WeightInfo<Runtime>;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = runtime_common::asset_switch::NoopBenchmarkHelper;
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

type CollatorMembershipProvider = pallet_membership::Instance3;
impl pallet_membership::Config<CollatorMembershipProvider> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddOrigin = RootOrMoreThanHalfCouncil;
	type RemoveOrigin = RootOrMoreThanHalfCouncil;
	type SwapOrigin = EnsureRoot<AccountId>;
	type ResetOrigin = EnsureRoot<AccountId>;
	type PrimeOrigin = EnsureRoot<AccountId>;
	type MembershipInitialized = ();
	type MembershipChanged = ();
	type MaxMembers = constants::governance::TechnicalMaxMembers;
	type WeightInfo = weights::pallet_technical_membership::WeightInfo<Runtime>;
}
