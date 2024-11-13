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

use frame_support::{parameter_types, traits::Everything};
use runtime_common::{
	constants::{self, UnvestedFundsAllowedWithdrawReasons, EXISTENTIAL_DEPOSIT},
	fees::{ToAuthorCredit, WeightToFee},
	AccountId, AuthorityId, Balance, Block, BlockHashCount, BlockLength, BlockWeights, FeeSplit, Hash, Nonce,
	SendDustAndFeesToTreasury, SlowAdjustingFeeUpdate,
};
use sp_core::{ConstBool, ConstU128, ConstU16, ConstU32, ConstU64};
use sp_runtime::{
	impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, ConvertInto, OpaqueKeys},
};
use sp_version::RuntimeVersion;
use sp_weights::ConstantMultiplier;

use crate::{
	weights, Aura, Balances, OriginCaller, PalletInfo, ParachainStaking, Runtime, RuntimeCall, RuntimeEvent,
	RuntimeFreezeReason, RuntimeHoldReason, RuntimeOrigin, RuntimeTask, System, VERSION,
};

pub mod proxy;

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
