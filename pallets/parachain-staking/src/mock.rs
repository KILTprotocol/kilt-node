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
use super::*;
use crate::{self as stake};
use frame_support::{
	assert_noop, assert_ok, construct_runtime, parameter_types,
	traits::{FindAuthor, GenesisBuild, OnFinalize, OnInitialize},
	weights::Weight,
};
use kilt_primitives::constants::YEARS;
use pallet_authorship::EventHandler;
use sp_core::H256;
use sp_io;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup, Zero},
	Perbill, Perquintill,
};

pub type AccountId = u64;
pub type Balance = u128;
pub type BlockNumber = u64;
pub const BLOCKS_PER_ROUND: u32 = 5;
pub const DECIMALS: Balance = 10u128.pow(15);
pub const COLLATOR: AccountId = 1337;

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
		Stake: stake::{Pallet, Call, Storage, Config<T>, Event<T>},
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
	pub const ExistentialDeposit: u128 = 1;
}
impl pallet_balances::Config for Test {
	type MaxLocks = ();
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
	pub const MinBlocksPerRound: u32 = 3; // 20
	pub const BondDuration: u32 = 2;
	pub const DefaultBlocksPerRound: u32 = BLOCKS_PER_ROUND;
	pub const MinSelectedCandidates: u32 = 5;
	pub const MaxDelegatorsPerCollator: u32 = 4;
	pub const MaxCollatorsPerDelegator: u32 = 4;
	pub const DefaultCollatorCommission: Perbill = Perbill::from_percent(20);
	pub const MinCollatorStk: u128 = 10;
	pub const MaxCollatorCandidateStk: u128 = 160_000_000 * DECIMALS;
	pub const MinDelegatorStk: u128 = 5;
	pub const MinDelegation: u128 = 3;
}
impl Config for Test {
	type Event = Event;
	type Currency = Balances;
	type CurrencyBalance = <Self as pallet_balances::Config>::Balance;
	type MinBlocksPerRound = MinBlocksPerRound;
	type DefaultBlocksPerRound = DefaultBlocksPerRound;
	type BondDuration = BondDuration;
	type MinSelectedCandidates = MinSelectedCandidates;
	type MaxDelegatorsPerCollator = MaxDelegatorsPerCollator;
	type MaxCollatorsPerDelegator = MaxCollatorsPerDelegator;
	type MinCollatorStk = MinCollatorStk;
	type MinCollatorCandidateStk = MinCollatorStk;
	type MaxCollatorCandidateStk = MaxCollatorCandidateStk;
	type MinDelegatorStk = MinDelegatorStk;
	type MinDelegation = MinDelegation;
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
	blocks_per_round: u32,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: vec![],
			delegators: vec![],
			collators: vec![],
			blocks_per_round: BLOCKS_PER_ROUND,
			inflation_config: InflationInfo {
				collator: StakingInfo {
					max_rate: Perbill::from_percent(10),
					reward_rate: RewardRate {
						annual: Perbill::from_percent(15),
						round: Perbill::from_parts(Perbill::from_percent(15).deconstruct() / 8640),
					},
				},
				delegator: StakingInfo {
					max_rate: Perbill::from_percent(40),
					reward_rate: RewardRate {
						annual: Perbill::from_percent(10),
						round: Perbill::from_parts(Perbill::from_percent(10).deconstruct() / 8640),
					},
				},
			},
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
		col_max: u32,
		col_rewards: u32,
		d_max: u32,
		d_rewards: u32,
		blocks_per_round: u32,
	) -> Self {
		let blocks_per_year = (YEARS as u32) / blocks_per_round;

		self.inflation_config = InflationInfo {
			collator: StakingInfo {
				max_rate: Perbill::from_percent(col_max),
				reward_rate: RewardRate {
					annual: Perbill::from_percent(col_rewards),
					round: Perbill::from_parts(Perbill::from_percent(col_rewards).deconstruct() / blocks_per_year),
				},
			},
			delegator: StakingInfo {
				max_rate: Perbill::from_percent(d_max),
				reward_rate: RewardRate {
					annual: Perbill::from_percent(d_rewards),
					round: Perbill::from_parts(Perbill::from_percent(d_rewards).deconstruct() / blocks_per_year),
				},
			},
		};
		self.blocks_per_round = blocks_per_round;

		self
	}

	pub(crate) fn set_blocks_per_round(mut self, blocks_per_round: u32) -> Self {
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
				Stake::set_blocks_per_round(Origin::root(), self.blocks_per_round)
					.expect("Ran into issues when setting blocks_per_round");
			});
		}

		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

/// Simulate a longer cycle of the chain to check collator and delegator
/// rewards.
///
/// * base_balance: The balance to mint for each user (should be 160 Mio. in
///   sum)
/// * collator_stake: The balance each collator stakes
/// * delegator_stake: The balance each delegator stakes (1 collator per
///   delegator)
/// * max_collator_rate: The percentage of the maximum collator staking rate for
///   the `InflationInfo`
/// * collator_reward_rate: The percentage of the annual collator reward rate
///   for the `InflationInfo`
/// * max_delegator_rate: The percentage of the maximum collator staking rate
///   for the `InflationInfo`
/// * delegator_reward_rate: The percentage of the annual delegator reward rate
///   for the `InflationInfo`
/// * blocks_per_round: The number of blocks for each round. 7200 corresponds to
///   rounds of length 12.
/// * num_of_years: The number of years we want to simulate. Ideally, this
///   should be between zero and one.
pub(crate) fn check_yearly_inflation(
	base_balance: Balance,
	collator_stake: Balance,
	delegator_stake: Balance,
	max_collator_rate: u32,
	collator_reward_rate: u32,
	max_delegator_rate: u32,
	delegator_reward_rate: u32,
	blocks_per_round: u32,
	num_of_years: Perbill,
) {
	let expected_issuance = 160_000_000 * DECIMALS;
	let num_of_users = (expected_issuance / base_balance) as u64;
	let num_of_collators = (Perbill::from_percent(max_collator_rate) * expected_issuance / collator_stake) as u64;
	let num_of_delegators = (Perbill::from_percent(max_delegator_rate) * expected_issuance / delegator_stake) as u64;
	assert!(num_of_users >= num_of_collators + num_of_delegators);
	let end_block = num_of_years * YEARS as u64;

	// mint 160 Mio total issuance
	let balances: Vec<(<Test as frame_system::Config>::AccountId, BalanceOf<Test>)> =
		(1u64..=num_of_users).map(|i| (i, base_balance)).collect();
	assert!(!balances.is_empty());

	// goal: max_collator_rate in % (default: 10%) staked by collators
	let collator_ids: Vec<AccountId> = (1u64..=num_of_collators).collect();
	let collators: Vec<(<Test as frame_system::Config>::AccountId, BalanceOf<Test>)> =
		collator_ids.clone().into_iter().map(|i| (i, collator_stake)).collect();

	// goal: max_delegator_rate in % (default: 40%) of the network staked by
	// delegators
	let delegators: Vec<(AccountId, <Test as frame_system::Config>::AccountId, BalanceOf<Test>)> =
		((num_of_collators + 1)..=(num_of_collators + num_of_delegators))
			.map(|i| (i, (i - 1) % num_of_collators + 1, delegator_stake))
			.collect();

	// generate round robin author list
	let authors: Vec<Option<AccountId>> = (1u64..=end_block)
		.map(|i| Some((i - 1) % num_of_collators + 1))
		.collect();

	ExtBuilder::default()
		.with_balances(balances)
		.with_collators(collators.clone())
		.with_delegators(delegators.clone())
		.with_inflation(
			max_collator_rate,
			collator_reward_rate,
			max_delegator_rate,
			delegator_reward_rate,
			blocks_per_round,
		)
		.build()
		.execute_with(|| {
			let total_issuance = <Test as Config>::Currency::total_issuance();
			assert_eq!(total_issuance, expected_issuance);
			let (total_collator_stake, total_delegator_stake) = Stake::total();
			assert_eq!(total_collator_stake, num_of_collators as u128 * collator_stake);
			assert_eq!(total_delegator_stake, num_of_delegators as u128 * delegator_stake);
			assert_eq!(Stake::round().length, blocks_per_round);

			// for each round, give each collator the same amount of points
			for collator in collator_ids.clone() {
				let collator_state = Stake::collator_state(collator).expect("Collator should have state");
				assert_eq!(collator_state.id, collator);
				assert_eq!(collator_state.bond, collator_stake);
				assert!(collator_state.total >= collator_state.bond);
				assert_eq!(collator_state.state, CollatorStatus::Active);
			}

			// increase number of selected candidates
			if num_of_collators as usize > Stake::selected_candidates().len() {
				assert_noop!(
					Stake::set_total_selected(Origin::root(), <Test as Config>::MinSelectedCandidates::get() - 1),
					Error::<Test>::CannotSetBelowMin
				);
				assert_ok!(Stake::set_total_selected(Origin::root(), num_of_collators as u32));

				// roll to round 2 to check for update of TotalSelected
				roll_to_new((2 * blocks_per_round + 2).into(), authors.clone());
				assert_eq!(Stake::selected_candidates(), collator_ids);
			}

			// get inflation and expected rewards
			let inflation = Stake::inflation_config();
			let collator_rewards: BalanceOf<Test> = inflation
				.collator
				.compute_block_rewards::<Test>(total_collator_stake, total_issuance)
				* (end_block as u128);
			let delegator_rewards: BalanceOf<Test> = inflation
				.delegator
				.compute_block_rewards::<Test>(total_delegator_stake, total_issuance)
				* (end_block as u128);

			// fast-forward to num_of_years * blocks
			roll_to_new(end_block, authors);

			// check collator rewards
			let mut single_collator_reward: Balance = Balance::zero();
			for (collator_acc, _) in collators {
				single_collator_reward =
					Balances::free_balance(collator_acc).saturating_sub(base_balance - collator_stake);

				// collator should have received about the annual reward rate of their initial
				// stake
				assert!(almost_equal(
					num_of_years * inflation.collator.reward_rate.annual * Balances::reserved_balance(collator_acc),
					single_collator_reward,
					Perbill::from_perthousand(15)
				));
				assert!(almost_equal(
					num_of_years * Perbill::from_percent(collator_reward_rate) * collator_stake,
					single_collator_reward,
					Perbill::from_perthousand(15)
				));
				// expected collator rewards should match what was minted
				assert!(almost_equal(
					collator_rewards,
					num_of_collators as u128 * single_collator_reward,
					Perbill::from_perthousand(15)
				));
			}
			assert!(!single_collator_reward.is_zero());

			// check delegator rewards
			let mut single_delegator_reward: Balance = Balance::zero();
			for (delegator_acc, _, _) in delegators {
				single_delegator_reward =
					Balances::free_balance(delegator_acc).saturating_sub(base_balance - delegator_stake);

				// delegator should have received about the annual reward rate of their initial
				// stake
				assert!(almost_equal(
					num_of_years * inflation.delegator.reward_rate.annual * Balances::reserved_balance(delegator_acc),
					single_delegator_reward,
					Perbill::from_percent(2),
				));
				assert!(almost_equal(
					num_of_years * Perbill::from_percent(delegator_reward_rate) * delegator_stake,
					single_delegator_reward,
					Perbill::from_percent(2),
				));
				// expected delegator rewards should match what was minted
				assert!(almost_equal(
					delegator_rewards,
					num_of_delegators as u128 * single_delegator_reward,
					Perbill::from_percent(2),
				));
			}
			assert!(!single_delegator_reward.is_zero());

			// collators should have received better reward rate than delegators
			assert!(
				Perbill::from_rational(single_collator_reward, collator_stake)
					> Perbill::from_rational(single_delegator_reward, delegator_stake)
			);
		});
}

/// Compare whether the difference of both sides is at most `precision * left`.
pub(crate) fn almost_equal(left: Balance, right: Balance, precision: Perbill) -> bool {
	let err = precision * left;
	left.max(right) - left.min(right) <= err
}

pub(crate) fn check_inflation_update(less_rounds: InflationInfo, more_rounds: InflationInfo) -> bool {
	less_rounds.collator.max_rate == more_rounds.collator.max_rate
		&& less_rounds.collator.reward_rate.annual == more_rounds.collator.reward_rate.annual
		&& less_rounds.collator.reward_rate.round > more_rounds.collator.reward_rate.round
		&& less_rounds.delegator.max_rate == more_rounds.delegator.max_rate
		&& less_rounds.delegator.reward_rate.annual == more_rounds.delegator.reward_rate.annual
		&& less_rounds.delegator.reward_rate.round > more_rounds.delegator.reward_rate.round
}

pub(crate) fn roll_to(n: u64) {
	while System::block_number() < n {
		Stake::on_finalize(System::block_number());
		Balances::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Balances::on_initialize(System::block_number());
		Stake::on_initialize(System::block_number());
	}
}

pub(crate) fn roll_to_new(n: u64, authors: Vec<Option<AccountId>>) {
	while System::block_number() < n {
		if let Some(Some(author)) = authors.get((System::block_number()) as usize) {
			Stake::note_author(*author);
		}
		Stake::on_finalize(System::block_number());
		Balances::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Balances::on_initialize(System::block_number());
		Stake::on_initialize(System::block_number());
	}
}

pub(crate) fn last_event() -> Event {
	System::events().pop().expect("Event expected").event
}

pub(crate) fn events() -> Vec<pallet::Event<Test>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| if let Event::stake(inner) = e { Some(inner) } else { None })
		.collect::<Vec<_>>()
}
