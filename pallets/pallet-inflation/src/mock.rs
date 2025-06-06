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

use crate as pallet_inflation;
use crate::CreditOf;
use frame_support::{
	parameter_types,
	traits::{fungible::Balanced, OnFinalize, OnInitialize, OnUnbalanced},
};

use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::{
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage, MultiSignature,
};

type Block = frame_system::mocking::MockBlock<Test>;
type Hash = sp_core::H256;
type Balance = u128;
type Signature = MultiSignature;
type AccountPublic = <Signature as Verify>::Signer;
type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

pub(crate) const TREASURY_ACC: AccountId = AccountId::new([1u8; 32]);

pub const BLOCKS_PER_YEAR: BlockNumberFor<Test> = 60_000 / 12_000 * 60 * 24 * 36525 / 100;
pub const KILT: Balance = 10u128.pow(15);
pub const INITIAL_PERIOD_LENGTH: BlockNumberFor<Test> = BLOCKS_PER_YEAR.saturating_mul(5);
const YEARLY_REWARD: Balance = 2_000_000u128 * KILT;
pub const INITIAL_PERIOD_REWARD_PER_BLOCK: Balance = YEARLY_REWARD / (BLOCKS_PER_YEAR as Balance);

frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		Inflation: pallet_inflation,
	}
);

parameter_types! {
	pub const SS58Prefix: u8 = 38;
	pub const BlockHashCount: BlockNumberFor<Test> = 2400;
}

impl frame_system::Config for Test {
	type RuntimeTask = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Block = Block;
	type Nonce = u64;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
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
	pub const ExistentialDeposit: Balance = 500;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type RuntimeFreezeReason = ();
	type FreezeIdentifier = ();
	type RuntimeHoldReason = ();
	type MaxFreezes = ();
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

pub struct ToBeneficiary;
impl OnUnbalanced<CreditOf<Test>> for ToBeneficiary {
	fn on_nonzero_unbalanced(amount: CreditOf<Test>) {
		// Must resolve into existing but better to be safe.
		let _ = <Test as pallet_inflation::Config>::Currency::resolve(&TREASURY_ACC, amount);
	}
}

parameter_types! {
	pub const InitialPeriodLength: BlockNumberFor<Test> = INITIAL_PERIOD_LENGTH;
	pub const InitialPeriodReward: Balance = INITIAL_PERIOD_REWARD_PER_BLOCK;
}

impl pallet_inflation::Config for Test {
	type Currency = Balances;
	type InitialPeriodLength = InitialPeriodLength;
	type InitialPeriodReward = InitialPeriodReward;
	type Beneficiary = ToBeneficiary;
	type WeightInfo = ();
}

pub(crate) fn roll_to(n: BlockNumberFor<Test>) {
	while System::block_number() < n {
		<AllPalletsWithSystem as OnFinalize<u64>>::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		<AllPalletsWithSystem as OnInitialize<u64>>::on_initialize(System::block_number());
	}
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap()
		.into()
}
