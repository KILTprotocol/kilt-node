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
use crate::{self as stake, migrations::StakingStorageVersion};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{GenesisBuild, OnFinalize, OnInitialize},
	weights::Weight,
};
use kilt_primitives::constants::KILT;
use pallet_authorship::EventHandler;
use sp_consensus_aura::sr25519::AuthorityId;
use sp_core::H256;
use sp_runtime::{
	impl_opaque_keys,
	testing::{Header, UintAuthorityId},
	traits::{BlakeTwo256, ConvertInto, IdentityLookup, OpaqueKeys},
	Perbill, Perquintill,
};
use sp_std::fmt::Debug;

pub use kilt_primitives::BlockNumber;

pub type AccountId = u64;
pub type Balance = u128;
pub const BLOCKS_PER_ROUND: BlockNumber = 5;
pub const DECIMALS: Balance = KILT;

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
		Authorship: pallet_authorship::{Pallet, Call, Storage, Inherent},
		StakePallet: stake::{Pallet, Call, Storage, Config<T>, Event<T>},
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},
		Aura: pallet_aura::{Pallet, Storage},
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

impl pallet_aura::Config for Test {
	type AuthorityId = AuthorityId;
	type DisabledValidators = ();
}

impl pallet_authorship::Config for Test {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type UncleGenerations = ();
	type FilterUncle = ();
	type EventHandler = Pallet<Test>;
}

parameter_types! {
	pub const MinBlocksPerRound: BlockNumber = 3;
	pub const StakeDuration: u32 = 2;
	pub const ExitQueueDelay: u32 = 2;
	pub const DefaultBlocksPerRound: BlockNumber = BLOCKS_PER_ROUND;
	pub const MinSelectedCandidates: u32 = 2;
	#[derive(Debug, PartialEq)]
	pub const MaxDelegatorsPerCollator: u32 = 4;
	#[derive(Debug, PartialEq)]
	pub const MaxCollatorsPerDelegator: u32 = 4;
	pub const DefaultCollatorCommission: Perbill = Perbill::from_percent(20);
	pub const MinCollatorStake: Balance = 10;
	#[derive(Debug, PartialEq)]
	pub const MaxCollatorCandidates: u32 = 10;
	pub const MinDelegatorStake: Balance = 5;
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
	type MinRequiredCollators = MinSelectedCandidates;
	type MaxDelegationsPerRound = MaxDelegatorsPerCollator;
	type MaxDelegatorsPerCollator = MaxDelegatorsPerCollator;
	type MaxCollatorsPerDelegator = MaxCollatorsPerDelegator;
	type MinCollatorStake = MinCollatorStake;
	type MinCollatorCandidateStake = MinCollatorStake;
	type MaxCollatorCandidates = MaxCollatorCandidates;
	type MinDelegatorStake = MinDelegatorStake;
	type MinDelegation = MinDelegation;
	type MaxUnstakeRequests = MaxUnstakeRequests;
	type WeightInfo = ();
}

impl_opaque_keys! {
	pub struct MockSessionKeys {
		pub aura: Aura,
	}
}

parameter_types! {
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

impl pallet_session::Config for Test {
	type Event = Event;
	type ValidatorId = AccountId;
	type ValidatorIdOf = ConvertInto;
	type ShouldEndSession = StakePallet;
	type NextSessionRotation = StakePallet;
	type SessionManager = StakePallet;
	type SessionHandler = <MockSessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = MockSessionKeys;
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
	type WeightInfo = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = 1;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
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
	// version of storage
	storage_version: StakingStorageVersion,
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
			storage_version: StakingStorageVersion::default(),
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

	pub(crate) fn with_storage_version(mut self, storage_version: StakingStorageVersion) -> Self {
		self.storage_version = storage_version;
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
			max_candidate_stake: 160_000_000 * DECIMALS,
		}
		.assimilate_storage(&mut t)
		.expect("Parachain Staking's storage can be assimilated");

		// stashes are the AccountId
		let session_keys: Vec<_> = self
			.collators
			.iter()
			.map(|(k, _)| {
				(
					*k,
					*k,
					MockSessionKeys {
						aura: UintAuthorityId(*k).to_public_key(),
					},
				)
			})
			.collect();

		// NOTE: this will initialize the aura authorities
		// through OneSessionHandler::on_genesis_session
		pallet_session::GenesisConfig::<Test> { keys: session_keys }
			.assimilate_storage(&mut t)
			.expect("Session Pallet's storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);

		if self.blocks_per_round != BLOCKS_PER_ROUND {
			ext.execute_with(|| {
				StakePallet::set_blocks_per_round(Origin::root(), self.blocks_per_round)
					.expect("Ran into issues when setting blocks_per_round");
			});
		}

		ext.execute_with(|| {
			crate::StorageVersion::<Test>::set(self.storage_version);
		});

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
		<AllPallets as OnFinalize<u64>>::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		<AllPallets as OnInitialize<u64>>::on_initialize(System::block_number());
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
