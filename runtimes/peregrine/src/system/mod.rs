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

use frame_support::{
	parameter_types,
	traits::{AsEnsureOriginWithArg, Everything},
};
use frame_system::EnsureRoot;
use runtime_common::{
	asset_switch::EnsureRootAsTreasury,
	constants::{self, UnvestedFundsAllowedWithdrawReasons, EXISTENTIAL_DEPOSIT},
	fees::{ToAuthorCredit, WeightToFee},
	AccountId, AuthorityId, Balance, BlockHashCount, BlockLength, BlockWeights, FeeSplit, Hash, Nonce,
	SendDustAndFeesToTreasury, SlowAdjustingFeeUpdate,
};
use sp_core::{ConstBool, ConstU128, ConstU16, ConstU32, ConstU64};
use sp_runtime::{
	impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, ConvertInto, OpaqueKeys},
};
use sp_std::vec::Vec;
use sp_version::RuntimeVersion;
use sp_weights::ConstantMultiplier;
use xcm::v4::Location;

use crate::{
	weights, Aura, Balances, Block, OriginCaller, PalletInfo, ParachainStaking, Runtime, RuntimeCall, RuntimeEvent,
	RuntimeFreezeReason, RuntimeHoldReason, RuntimeOrigin, RuntimeTask, System, VERSION,
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
	type DustRemoval = runtime_common::SendDustAndFeesToTreasury<Runtime>;
	type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
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

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = AccountId;
	type ValidatorIdOf = ConvertInto;
	type ShouldEndSession = ParachainStaking;
	type NextSessionRotation = ParachainStaking;
	type SessionManager = ParachainStaking;
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
	type EventHandler = ParachainStaking;
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
	type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
	const MAX_VESTING_SCHEDULES: u32 = constants::MAX_VESTING_SCHEDULES;
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = ConstU128<{ constants::multisig::DEPOSIT_BASE }>;
	type DepositFactor = ConstU128<{ constants::multisig::DEPOSIT_FACTOR }>;
	type MaxSignatories = ConstU32<{ constants::multisig::MAX_SIGNITORS }>;
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
