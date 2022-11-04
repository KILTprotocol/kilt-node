// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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

use frame_support::traits::{Currency, Get};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, CheckedSub, Saturating, Zero},
	RuntimeDebug,
};
use sp_staking::SessionIndex;
use sp_std::{
	cmp::Ordering,
	fmt::Debug,
	ops::{Add, Sub},
};

use crate::{set::OrderedSet, Config};

/// A struct represented an amount of staked funds.
///
/// The stake has a destination account (to which the stake is directed) and an
/// amount of funds staked.
#[derive(Default, Clone, Encode, Decode, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[codec(mel_bound(AccountId: MaxEncodedLen, Balance: MaxEncodedLen))]
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

// We order by stake and only return an equal order, if both account ids match.
// This prevents the same account ids to be in the same OrderedSet. Otherwise,
// it is ordered from greatest to lowest stake (primary) and from first joined
// to last joined (primary).
impl<AccountId: Ord, Balance: PartialEq + Ord> Ord for Stake<AccountId, Balance> {
	fn cmp(&self, other: &Self) -> Ordering {
		match (self.owner.cmp(&other.owner), self.amount.cmp(&other.amount)) {
			// enforce unique account ids
			(Ordering::Equal, _) => Ordering::Equal,
			// prioritize existing members if stakes match
			(_, Ordering::Equal) => Ordering::Greater,
			// order by stake
			(_, ord) => ord,
		}
	}
}

/// The activity status of the collator.
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum CandidateStatus {
	/// Committed to be online and producing valid blocks (not equivocating)
	Active,
	/// Staked until the inner round
	Leaving(SessionIndex),
}

impl Default for CandidateStatus {
	fn default() -> CandidateStatus {
		CandidateStatus::Active
	}
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxDelegatorsPerCandidate))]
#[codec(mel_bound(AccountId: MaxEncodedLen, Balance: MaxEncodedLen))]
/// Global collator state with commission fee, staked funds, and delegations
pub struct Candidate<AccountId, Balance, MaxDelegatorsPerCandidate>
where
	AccountId: Eq + Ord + Debug,
	Balance: Eq + Ord + Debug,
	MaxDelegatorsPerCandidate: Get<u32> + Debug + PartialEq,
{
	/// Account id of the candidate.
	pub id: AccountId,

	/// The stake that the candidate put down.
	pub stake: Balance,

	/// The delegators that back the candidate.
	pub delegators: OrderedSet<Stake<AccountId, Balance>, MaxDelegatorsPerCandidate>,

	/// The total backing a collator has.
	///
	/// Should equal the sum of all delegators stake adding collators stake
	pub total: Balance,

	/// The current status of the candidate. Indicates whether a candidate is
	/// active or leaving the candidate pool
	pub status: CandidateStatus,
}

impl<A, B, S> Candidate<A, B, S>
where
	A: Ord + Clone + Debug,
	B: AtLeast32BitUnsigned + Ord + Copy + Saturating + Debug + Zero,
	S: Get<u32> + Debug + PartialEq,
{
	pub fn new(id: A, stake: B) -> Self {
		let total = stake;
		Candidate {
			id,
			stake,
			delegators: OrderedSet::new(),
			total,
			status: CandidateStatus::default(), // default active
		}
	}

	pub fn is_active(&self) -> bool {
		self.status == CandidateStatus::Active
	}

	pub fn is_leaving(&self) -> bool {
		matches!(self.status, CandidateStatus::Leaving(_))
	}

	pub fn can_exit(&self, when: u32) -> bool {
		matches!(self.status, CandidateStatus::Leaving(at) if at <= when )
	}

	pub fn revert_leaving(&mut self) {
		self.status = CandidateStatus::Active;
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

	pub fn inc_delegator(&mut self, delegator: A, more: B) {
		if let Ok(i) = self.delegators.linear_search(&Stake::<A, B> {
			owner: delegator,
			amount: B::zero(),
		}) {
			self.delegators
				.mutate(|vec| vec[i].amount = vec[i].amount.saturating_add(more));
			self.total = self.total.saturating_add(more);
			self.delegators.sort_greatest_to_lowest()
		}
	}

	pub fn dec_delegator(&mut self, delegator: A, less: B) {
		if let Ok(i) = self.delegators.linear_search(&Stake::<A, B> {
			owner: delegator,
			amount: B::zero(),
		}) {
			self.delegators
				.mutate(|vec| vec[i].amount = vec[i].amount.saturating_sub(less));
			self.total = self.total.saturating_sub(less);
			self.delegators.sort_greatest_to_lowest()
		}
	}

	pub fn leave_candidates(&mut self, round: SessionIndex) {
		self.status = CandidateStatus::Leaving(round);
	}
}

pub type Delegator<AccountId, Balance> = Stake<AccountId, Balance>;
impl<AccountId, Balance> Delegator<AccountId, Balance>
where
	AccountId: Eq + Ord + Clone + Debug,
	Balance: Copy + Add<Output = Balance> + Saturating + PartialOrd + Eq + Ord + Debug + Zero + Default + CheckedSub,
{
	/// Returns Ok if the delegation for the
	/// collator exists and `Err` otherwise.
	pub fn try_clear(&mut self, collator: AccountId) -> Result<(), ()> {
		if self.owner == collator {
			self.amount = Balance::zero();
			Ok(())
		} else {
			Err(())
		}
	}

	/// Returns Ok(delegated_amount) if successful, `Err` if delegation was
	/// not found.
	pub fn try_increment(&mut self, collator: AccountId, more: Balance) -> Result<Balance, ()> {
		if self.owner == collator {
			self.amount = self.amount.saturating_add(more);
			Ok(self.amount)
		} else {
			Err(())
		}
	}

	/// Returns Ok(Some(delegated_amount)) if successful, `Err` if delegation
	/// was not found and Ok(None) if delegated stake would underflow.
	pub fn try_decrement(&mut self, collator: AccountId, less: Balance) -> Result<Option<Balance>, ()> {
		if self.owner == collator {
			Ok(self.amount.checked_sub(&less).map(|new| {
				self.amount = new;
				self.amount
			}))
		} else {
			Err(())
		}
	}
}

/// The current round index and transition information.
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
#[derive(Default, Clone, Encode, Decode, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct TotalStake<Balance: Default> {
	pub collators: Balance,
	pub delegators: Balance,
}

/// The number of delegations a delegator has done within the last session in
/// which they delegated.
#[derive(Default, Clone, Encode, Decode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct DelegationCounter {
	/// The index of the last delegation.
	pub round: SessionIndex,
	/// The number of delegations made within round.
	pub counter: u32,
}

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
pub type CandidateOf<T, S> = Candidate<AccountIdOf<T>, BalanceOf<T>, S>;
pub type StakeOf<T> = Stake<AccountIdOf<T>, BalanceOf<T>>;
pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::NegativeImbalance;
