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
//! Test utilities

#![allow(clippy::from_over_into)]

use super::*;
use crate::{self as stake};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{FindAuthor, GenesisBuild, OnFinalize, OnInitialize},
	weights::Weight,
};
use pallet_authorship::EventHandler;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill, Perquintill,
};

pub use kilt_primitives::BlockNumber;

pub type AccountId = u64;
pub type Balance = u128;
pub const BLOCKS_PER_ROUND: BlockNumber = 5;
pub const DECIMALS: Balance = 10u128.pow(15);

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		StakePallet: stake::{Pallet, Call, Storage, Config<T>, Event<T>},
		Authorship: pallet_authorship::{Pallet, Call, Storage, Inherent},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}
parameter_types! {
	pub const ExistentialDeposit: Balance = 1;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

/// Author of block is always 1337
pub struct Author1337;
impl FindAuthor<AccountId> for Author1337 {
	fn find_author<'a, I>(_digests: I) -> Option<AccountId>
	where
		I: 'a + IntoIterator<Item = (frame_support::ConsensusEngineId, &'a [u8])>,
	{
		Some(1337)
	}
}
impl pallet_authorship::Config for Test {
	type FindAuthor = Author1337;
	type UncleGenerations = ();
	type FilterUncle = ();
	type EventHandler = Pallet<Test>;
}

parameter_types! {
	pub const MinBlocksPerRound: BlockNumber = 3; // 20
	pub const StakeDuration: u32 = 2;
	pub const ExitQueueDelay: u32 = 2;
	pub const DefaultBlocksPerRound: BlockNumber = BLOCKS_PER_ROUND;
	pub const MinSelectedCandidates: u32 = 2;
	pub const MaxDelegatorsPerCollator: u32 = 4;
	pub const MaxCollatorsPerDelegator: u32 = 4;
	pub const DefaultCollatorCommission: Perbill = Perbill::from_percent(20);
	pub const MinCollatorStk: Balance = 10;
	pub const MaxCollatorCandidateStk: Balance = 160_000_000 * DECIMALS;
	pub const MaxCollatorCandidates: u32 = 10;
	pub const MinDelegatorStk: Balance = 5;
	pub const MinDelegation: Balance = 3;
	pub const MaxUnstakeRequests: u32 = 5;
}

impl Config for Test {
	type Event = Event;
	type Currency = Balances;
	type CurrencyBalance = <Self as pallet_balances::Config>::Balance;
	type MinBlocksPerRound = MinBlocksPerRound;
	type DefaultBlocksPerRound = DefaultBlocksPerRound;
	type StakeDuration = StakeDuration;
	type ExitQueueDelay = ExitQueueDelay;
	type MinSelectedCandidates = MinSelectedCandidates;
	type MaxDelegatorsPerCollator = MaxDelegatorsPerCollator;
	type MaxCollatorsPerDelegator = MaxCollatorsPerDelegator;
	type MinCollatorStk = MinCollatorStk;
	type MinCollatorCandidateStk = MinCollatorStk;
	type MaxCollatorCandidateStk = MaxCollatorCandidateStk;
	type MaxCollatorCandidates = MaxCollatorCandidates;
	type MinDelegatorStk = MinDelegatorStk;
	type MinDelegation = MinDelegation;
	type MaxUnstakeRequests = MaxUnstakeRequests;
	type WeightInfo = ();
}

pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
	// [collator, amount]
	collators: Vec<(AccountId, Balance)>,
	// [delegator, collator, delegation_amount]
	delegators: Vec<(AccountId, AccountId, Balance)>,
	// inflation config
	inflation_config: InflationInfo,
	// blocks per round
	blocks_per_round: BlockNumber,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: vec![],
			delegators: vec![],
			collators: vec![],
			blocks_per_round: BLOCKS_PER_ROUND,
			inflation_config: InflationInfo::new(
				Perquintill::from_percent(10),
				Perquintill::from_percent(15),
				Perquintill::from_percent(40),
				Perquintill::from_percent(10),
			),
		}
	}
}

impl ExtBuilder {
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub(crate) fn with_collators(mut self, collators: Vec<(AccountId, Balance)>) -> Self {
		self.collators = collators;
		self
	}

	pub(crate) fn with_delegators(mut self, delegators: Vec<(AccountId, AccountId, Balance)>) -> Self {
		self.delegators = delegators;
		self
	}

	pub(crate) fn with_inflation(
		mut self,
		col_max: u64,
		col_rewards: u64,
		d_max: u64,
		d_rewards: u64,
		blocks_per_round: BlockNumber,
	) -> Self {
		self.inflation_config = InflationInfo::new(
			Perquintill::from_percent(col_max),
			Perquintill::from_percent(col_rewards),
			Perquintill::from_percent(d_max),
			Perquintill::from_percent(d_rewards),
		);
		self.blocks_per_round = blocks_per_round;

		self
	}

	pub(crate) fn set_blocks_per_round(mut self, blocks_per_round: BlockNumber) -> Self {
		self.blocks_per_round = blocks_per_round;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Test> {
			balances: self.balances.clone(),
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		let mut stakers: Vec<(AccountId, Option<AccountId>, Balance)> = Vec::new();
		for collator in self.collators.clone() {
			stakers.push((collator.0, None, collator.1));
		}
		for delegator in self.delegators.clone() {
			stakers.push((delegator.0, Some(delegator.1), delegator.2));
		}
		stake::GenesisConfig::<Test> {
			stakers,
			inflation_config: self.inflation_config.clone(),
		}
		.assimilate_storage(&mut t)
		.expect("Parachain Staking's storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);

		if self.blocks_per_round != BLOCKS_PER_ROUND {
			ext.execute_with(|| {
				StakePallet::set_blocks_per_round(Origin::root(), self.blocks_per_round)
					.expect("Ran into issues when setting blocks_per_round");
			});
		}

		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

/// Compare whether the difference of both sides is at most `precision * left`.
pub(crate) fn almost_equal(left: Balance, right: Balance, precision: Perbill) -> bool {
	let err = precision * left;
	left.max(right) - left.min(right) <= err
}

pub(crate) fn roll_to(n: BlockNumber, authors: Vec<Option<AccountId>>) {
	while System::block_number() < n {
		if let Some(Some(author)) = authors.get((System::block_number()) as usize) {
			StakePallet::note_author(*author);
		}
		StakePallet::on_finalize(System::block_number());
		Balances::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Balances::on_initialize(System::block_number());
		StakePallet::on_initialize(System::block_number());
	}
}

pub(crate) fn last_event() -> Event {
	System::events().pop().expect("Event expected").event
}

pub(crate) fn events() -> Vec<pallet::Event<Test>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let Event::StakePallet(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()
}
