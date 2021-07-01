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

use frame_support::traits::Currency;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Saturating},
	RuntimeDebug,
};
use sp_staking::SessionIndex;
use sp_std::{
	cmp::Ordering,
	ops::{Add, Sub},
	vec,
};

use crate::{set::OrderedSet, Config};

/// A struct represented an amount of staked funds.
///
/// The stake has a destination account (to which the stake is directed) and an
/// amount of funds staked.
#[derive(Default, Clone, Encode, Decode, RuntimeDebug, PartialEq, Eq)]
pub struct Stake<AccountId, Balance>
where
	AccountId: Eq + Ord,
	Balance: Eq + Ord,
{
	/// The account that is backed by the stake.
	pub owner: AccountId,

	/// The amount of backing the `owner` received.
	pub amount: Balance,
}

impl<A, B> From<A> for Stake<A, B>
where
	A: Eq + Ord,
	B: Default + Eq + Ord,
{
	fn from(owner: A) -> Self {
		Stake {
			owner,
			amount: B::default(),
		}
	}
}

impl<AccountId: Ord, Balance: PartialEq + Ord> PartialOrd for Stake<AccountId, Balance> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

// We only establish an order based on the owner
impl<AccountId: Ord, Balance: PartialEq + Ord> Ord for Stake<AccountId, Balance> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.owner.cmp(&other.owner)
	}
}

/// The activity status of the collator.
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub enum CollatorStatus {
	/// Committed to be online and producing valid blocks (not equivocating)
	Active,
	/// Staked until the inner round
	Leaving(SessionIndex),
}

impl Default for CollatorStatus {
	fn default() -> CollatorStatus {
		CollatorStatus::Active
	}
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq)]
/// Global collator state with commission fee, staked funds, and delegations
pub struct Collator<AccountId, Balance>
where
	AccountId: Eq + Ord,
	Balance: Eq + Ord,
{
	/// The collators account id.
	pub id: AccountId,

	/// The stake that the collator put down.
	pub stake: Balance,

	/// The delegators that back the collator.
	pub delegators: OrderedSet<Stake<AccountId, Balance>>,

	/// The total backing a collator has.
	///
	/// Should equal the sum of all delegators stake adding collators stake
	pub total: Balance,

	/// The current status of the collator. Indicates whether a collator is
	/// active or leaving the collator set
	pub state: CollatorStatus,
}

impl<A, B> Collator<A, B>
where
	A: Ord + Clone,
	B: AtLeast32BitUnsigned + Ord + Copy + Saturating,
{
	pub fn new(id: A, stake: B) -> Self {
		let total = stake;
		Collator {
			id,
			stake,
			delegators: OrderedSet::new(),
			total,
			state: CollatorStatus::default(), // default active
		}
	}

	pub fn is_active(&self) -> bool {
		self.state == CollatorStatus::Active
	}

	pub fn is_leaving(&self) -> bool {
		matches!(self.state, CollatorStatus::Leaving(_))
	}

	pub fn can_exit(&self, when: u32) -> bool {
		matches!(self.state, CollatorStatus::Leaving(at) if at <= when )
	}

	pub fn revert_leaving(&mut self) {
		self.state = CollatorStatus::Active;
	}

	pub fn stake_more(&mut self, more: B) {
		self.stake = self.stake.saturating_add(more);
		self.total = self.total.saturating_add(more);
	}

	// Returns None if underflow or less == self.stake (in which case collator
	// should leave).
	pub fn stake_less(&mut self, less: B) -> Option<B> {
		if self.stake > less {
			self.stake = self.stake.saturating_sub(less);
			self.total = self.total.saturating_sub(less);
			Some(self.stake)
		} else {
			None
		}
	}

	pub fn inc_delegator(&mut self, delegator: &A, more: B) {
		if let Ok(i) = self.delegators.binary_search_by(|x| x.owner.cmp(delegator)) {
			self.delegators[i].amount = self.delegators[i].amount.saturating_add(more);
			self.total = self.total.saturating_add(more);
		}
	}

	pub fn dec_delegator(&mut self, delegator: &A, less: B) {
		if let Ok(i) = self.delegators.binary_search_by(|x| x.owner.cmp(delegator)) {
			self.delegators[i].amount = self.delegators[i].amount.saturating_sub(less);
			self.total = self.total.saturating_sub(less);
		}
	}

	pub fn leave_candidates(&mut self, round: SessionIndex) {
		self.state = CollatorStatus::Leaving(round);
	}
}

#[derive(Encode, Decode, RuntimeDebug, PartialEq)]
pub struct Delegator<AccountId: Eq + Ord, Balance: Eq + Ord> {
	pub delegations: OrderedSet<Stake<AccountId, Balance>>,
	pub total: Balance,
}

impl<AccountId, Balance> Delegator<AccountId, Balance>
where
	AccountId: Eq + Ord + Clone,
	Balance: Copy + Add<Output = Balance> + Saturating + PartialOrd + Eq + Ord,
{
	pub fn new(collator: AccountId, amount: Balance) -> Self {
		Delegator {
			delegations: OrderedSet::from(vec![Stake {
				owner: collator,
				amount,
			}]),
			total: amount,
		}
	}

	/// Adds a new delegation.
	///
	/// If already delegating to the same account, this call returns false and
	/// doesn't insert the new delegation.
	pub fn add_delegation(&mut self, stake: Stake<AccountId, Balance>) -> bool {
		let amt = stake.amount;
		if self.delegations.insert(stake) {
			self.total = self.total.saturating_add(amt);
			true
		} else {
			false
		}
	}

	/// Returns Some(remaining stake for delegator) if the delegation for the
	/// collator exists. Returns `None` otherwise.
	pub fn rm_delegation(&mut self, collator: &AccountId) -> Option<Balance> {
		let amt = self.delegations.remove_by(|x| x.owner.cmp(collator)).map(|f| f.amount);

		if let Some(balance) = amt {
			self.total = self.total.saturating_sub(balance);
			Some(self.total)
		} else {
			None
		}
	}

	/// Returns None if delegation was not found.
	pub fn inc_delegation(&mut self, collator: &AccountId, more: Balance) -> Option<Balance> {
		match self.delegations.binary_search_by(|x| x.owner.cmp(collator)) {
			Ok(i) => {
				self.delegations[i].amount = self.delegations[i].amount.saturating_add(more);
				self.total = self.total.saturating_add(more);
				Some(self.delegations[i].amount)
			}
			Err(_) => None,
		}
	}

	/// Returns Some(Some(balance)) if successful, None if delegation was not
	/// found and Some(None) if delegated stake would underflow.
	pub fn dec_delegation(&mut self, collator: &AccountId, less: Balance) -> Option<Option<Balance>> {
		match self.delegations.binary_search_by(|x| x.owner.cmp(collator)) {
			Ok(i) => {
				let mut x = &mut self.delegations[i];
				if x.amount > less {
					x.amount = x.amount.saturating_sub(less);
					self.total = self.total.saturating_sub(less);
					Some(Some(x.amount))
				} else {
					// underflow error; should rm entire delegation if x.amount == collator
					Some(None)
				}
			}
			Err(_) => None,
		}
	}
}

/// The current round index and transition information.
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct RoundInfo<BlockNumber> {
	/// Current round index.
	pub current: SessionIndex,
	/// The first block of the current round.
	pub first: BlockNumber,
	/// The length of the current round in blocks.
	pub length: BlockNumber,
}

impl<B> RoundInfo<B>
where
	B: Copy + Saturating + From<u32> + PartialOrd,
{
	pub fn new(current: SessionIndex, first: B, length: B) -> RoundInfo<B> {
		RoundInfo { current, first, length }
	}

	/// Checks if the round should be updated.
	///
	/// The round should update if `self.length` or more blocks where produced
	/// after `self.first`.
	pub fn should_update(&self, now: B) -> bool {
		let l = now.saturating_sub(self.first);
		l >= self.length
	}

	/// Starts a new round.
	pub fn update(&mut self, now: B) {
		self.current = self.current.saturating_add(1u32);
		self.first = now;
	}
}

impl<B> Default for RoundInfo<B>
where
	B: Copy + Saturating + Add<Output = B> + Sub<Output = B> + From<u32> + PartialOrd,
{
	fn default() -> RoundInfo<B> {
		RoundInfo::new(0u32, 0u32.into(), 20.into())
	}
}

/// The total stake of the pallet.
///
/// The stake includes both collators' and delegators' staked funds.
#[derive(Default, Clone, Encode, Decode, RuntimeDebug, PartialEq, Eq)]
pub struct TotalStake<Balance: Default> {
	pub collators: Balance,
	pub delegators: Balance,
}

/// The number of delegations a delegator has done within the last session in
/// which they delegated.
#[derive(Default, Clone, Encode, Decode, RuntimeDebug, PartialEq)]
pub struct DelegationCounter {
	/// The index of the last delegation.
	pub round: SessionIndex,
	/// The number of delegations made within round.
	pub counter: u32,
}

// A value placed in storage that represents the current version of the Staking
// storage. This value is used by the `on_runtime_upgrade` logic to determine
// whether we run storage migration logic. This should match directly with the
// semantic versions of the Rust crate.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
pub enum Releases {
	V1_0_0,
	V2_0_0, // New Reward calculation, MaxCollatorCandidateStake
	V3_0_0, // Update InflationConfig
}

impl Default for Releases {
	fn default() -> Self {
		Releases::V3_0_0
	}
}

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
pub type CollatorOf<T> = Collator<AccountIdOf<T>, BalanceOf<T>>;
pub type StakeOf<T> = Stake<AccountIdOf<T>, BalanceOf<T>>;
