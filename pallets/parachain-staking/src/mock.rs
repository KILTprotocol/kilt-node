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
//! Test utilities

#![allow(clippy::from_over_into)]

use super::*;
use crate::{self as stake, types::CreditOf};
use frame_support::{
	assert_ok, construct_runtime, parameter_types,
	traits::{fungible::Balanced, OnFinalize, OnInitialize, OnUnbalanced},
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_authorship::EventHandler;
use sp_consensus_aura::sr25519::AuthorityId;
use sp_core::{ConstBool, ConstU64, H256};
use sp_runtime::{
	impl_opaque_keys,
	testing::UintAuthorityId,
	traits::{BlakeTwo256, ConvertInto, IdentityLookup, OpaqueKeys},
	BuildStorage, Perbill, Perquintill,
};
use sp_std::fmt::Debug;

pub(crate) type Block = frame_system::mocking::MockBlock<Test>;
pub(crate) type Balance = u128;
pub(crate) type AccountId = u64;
pub(crate) type BlockNumber = u64;

pub(crate) const MILLI_KILT: Balance = 10u128.pow(12);
pub(crate) const MAX_COLLATOR_STAKE: Balance = 200_000 * 1000 * MILLI_KILT;
pub(crate) const BLOCKS_PER_ROUND: BlockNumber = 5;
pub(crate) const DECIMALS: Balance = 1000 * MILLI_KILT;
pub(crate) const TREASURY_ACC: AccountId = u64::MAX;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		Aura: pallet_aura,
		Session: pallet_session,
		StakePallet: stake,
		Authorship: pallet_authorship,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
	type RuntimeTask = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Block = Block;
	type Nonce = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
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
	type BlockWeights = ();
	type BlockLength = ();
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
	pub const ExistentialDeposit: Balance = 1;
	pub const MaxFreezes : u32 = 50;
}

impl pallet_balances::Config for Test {
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = RuntimeFreezeReason;
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxFreezes = MaxFreezes;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

impl pallet_aura::Config for Test {
	type AuthorityId = AuthorityId;
	type DisabledValidators = ();
	type MaxAuthorities = MaxCollatorCandidates;
	type AllowMultipleBlocksPerSlot = ConstBool<false>;
	type SlotDuration = ConstU64<500>;
}

impl pallet_authorship::Config for Test {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type EventHandler = Pallet<Test>;
}

parameter_types! {
	pub const MinBlocksPerRound: BlockNumber = 3;
	pub const StakeDuration: u32 = 2;
	pub const ExitQueueDelay: u32 = 2;
	pub const DefaultBlocksPerRound: BlockNumber = BLOCKS_PER_ROUND;
	pub const MinCollators: u32 = 2;
	pub const MaxDelegationsPerRound: u32 = 2;
	#[derive(Debug, Eq, PartialEq)]
	pub const MaxDelegatorsPerCollator: u32 = 4;
	pub const MinCollatorStake: Balance = 10;
	#[derive(Debug, Eq, PartialEq)]
	pub const MaxCollatorCandidates: u32 = 10;
	pub const MinDelegatorStake: Balance = 5;
	pub const MaxUnstakeRequests: u32 = 6;
	pub const NetworkRewardRate: Perquintill = Perquintill::from_percent(10);
	pub const NetworkRewardStart: BlockNumber = 5 * 5 * 60 * 24 * 36525 / 100;
}

pub struct ToBeneficiary;
impl OnUnbalanced<CreditOf<Test>> for ToBeneficiary {
	fn on_nonzero_unbalanced(amount: CreditOf<Test>) {
		// Must resolve into existing but better to be safe.
		let _ = <Test as Config>::Currency::resolve(&TREASURY_ACC, amount);
	}
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type CurrencyBalance = <Self as pallet_balances::Config>::Balance;
	type MinBlocksPerRound = MinBlocksPerRound;
	type DefaultBlocksPerRound = DefaultBlocksPerRound;
	type StakeDuration = StakeDuration;
	type ExitQueueDelay = ExitQueueDelay;
	type MinCollators = MinCollators;
	type MinRequiredCollators = MinCollators;
	type MaxDelegationsPerRound = MaxDelegationsPerRound;
	type MaxDelegatorsPerCollator = MaxDelegatorsPerCollator;
	type MinCollatorStake = MinCollatorStake;
	type MinCollatorCandidateStake = MinCollatorStake;
	type MaxTopCandidates = MaxCollatorCandidates;
	type MinDelegatorStake = MinDelegatorStake;
	type MaxUnstakeRequests = MaxUnstakeRequests;
	type NetworkRewardRate = NetworkRewardRate;
	type NetworkRewardStart = NetworkRewardStart;
	type NetworkRewardBeneficiary = ToBeneficiary;
	type WeightInfo = ();
	type FreezeIdentifier = RuntimeFreezeReason;
	const BLOCKS_PER_YEAR: BlockNumberFor<Test> = 5 * 60 * 24 * 36525 / 100;
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
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = AccountId;
	type ValidatorIdOf = ConvertInto;
	type ShouldEndSession = StakePallet;
	type NextSessionRotation = StakePallet;
	type SessionManager = StakePallet;
	type SessionHandler = <MockSessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = MockSessionKeys;
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
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: vec![],
			delegators: vec![],
			collators: vec![],
			blocks_per_round: BLOCKS_PER_ROUND,
			inflation_config: InflationInfo::new(
				<Test as Config>::BLOCKS_PER_YEAR,
				Perquintill::from_percent(10),
				Perquintill::from_percent(15),
				Perquintill::from_percent(40),
				Perquintill::from_percent(10),
			),
		}
	}
}

impl ExtBuilder {
	#[must_use]
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	#[must_use]
	pub(crate) fn with_collators(mut self, collators: Vec<(AccountId, Balance)>) -> Self {
		self.collators = collators;
		self
	}

	#[must_use]
	pub(crate) fn with_delegators(mut self, delegators: Vec<(AccountId, AccountId, Balance)>) -> Self {
		self.delegators = delegators;
		self
	}

	#[must_use]
	pub(crate) fn with_inflation(
		mut self,
		col_max: u64,
		col_rewards: u64,
		d_max: u64,
		d_rewards: u64,
		blocks_per_round: BlockNumber,
	) -> Self {
		self.inflation_config = InflationInfo::new(
			<Test as Config>::BLOCKS_PER_YEAR,
			Perquintill::from_percent(col_max),
			Perquintill::from_percent(col_rewards),
			Perquintill::from_percent(d_max),
			Perquintill::from_percent(d_rewards),
		);
		self.blocks_per_round = blocks_per_round;

		self
	}

	#[must_use]
	pub(crate) fn set_blocks_per_round(mut self, blocks_per_round: BlockNumber) -> Self {
		self.blocks_per_round = blocks_per_round;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Test>::default()
			.build_storage()
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
		pallet_session::GenesisConfig::<Test> {
			keys: session_keys,
			..Default::default()
		}
		.assimilate_storage(&mut t)
		.expect("Session Pallet's storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);

		if self.blocks_per_round != BLOCKS_PER_ROUND {
			ext.execute_with(|| {
				StakePallet::set_blocks_per_round(RuntimeOrigin::root(), self.blocks_per_round)
					.expect("Ran into issues when setting blocks_per_round");
			});
		}

		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	pub fn build_and_execute_with_sanity_tests(self, test: impl FnOnce()) {
		self.build().execute_with(|| {
			test();
			crate::try_state::do_try_state::<Test>().expect("Sanity test for parachain staking failed.");
		})
	}
}

/// Compare whether the difference of both sides is at most `precision * left`.
pub(crate) fn almost_equal(left: Balance, right: Balance, precision: Perbill) -> bool {
	let err = precision * left;
	left.max(right) - left.min(right) <= err
}

/// Incrementelly traverses from the current block to the provided one and
/// potentially sets block authors.
///
/// If for a block `i` the corresponding index of the authors input is set, this
/// account is regarded to be the block author and thus gets noted.
///
/// NOTE: At most, this updates the RewardCount of the block author but does not
/// increment rewards or claim them. Please use `roll_to_claim_rewards` in that
/// case.
pub(crate) fn roll_to(n: BlockNumber, authors: Vec<Option<AccountId>>) {
	while System::block_number() < n {
		if let Some(Some(author)) = authors.get((System::block_number()) as usize) {
			StakePallet::note_author(*author);
		}
		<AllPalletsWithSystem as OnFinalize<u64>>::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		<AllPalletsWithSystem as OnInitialize<u64>>::on_initialize(System::block_number());
	}
}

#[allow(unused_must_use)]
/// Incrementelly traverses from the current block to the provided one and
/// potentially sets block authors.
///
/// If existent, rewards of the block author and their delegators are
/// incremented and claimed.
///
/// If for a block `i` the corresponding index of the authors input is set, this
/// account is regarded to be the block author and thus gets noted.
pub(crate) fn roll_to_claim_rewards(n: BlockNumber, authors: Vec<Option<AccountId>>) {
	while System::block_number() < n {
		if let Some(Some(author)) = authors.get((System::block_number()) as usize) {
			StakePallet::note_author(*author);
			// author has to increment rewards before claiming
			assert_ok!(StakePallet::increment_collator_rewards(RuntimeOrigin::signed(*author)));
			// author claims rewards
			assert_ok!(StakePallet::claim_rewards(RuntimeOrigin::signed(*author)));

			// claim rewards for delegators
			let col_state = StakePallet::candidate_pool(author).expect("Block author must be candidate");
			for delegation in col_state.delegators {
				// delegator has to increment rewards before claiming
				StakePallet::increment_delegator_rewards(RuntimeOrigin::signed(delegation.owner));
				// NOTE: cannot use assert_ok! as we sometimes expect zero rewards for
				// delegators such that the claiming would throw
				StakePallet::claim_rewards(RuntimeOrigin::signed(delegation.owner));
			}
		}
		<AllPalletsWithSystem as OnFinalize<u64>>::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		<AllPalletsWithSystem as OnInitialize<u64>>::on_initialize(System::block_number());
	}
}

pub(crate) fn last_event() -> pallet::Event<Test> {
	events().pop().expect("Event expected")
}

pub(crate) fn events() -> Vec<pallet::Event<Test>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let RuntimeEvent::StakePallet(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()
}
