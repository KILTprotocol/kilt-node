use frame_support::traits::Currency;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Saturating},
	RuntimeDebug,
};
use sp_std::{cmp::Ordering, vec, vec::Vec};

use crate::{set::OrderedSet, Config};

#[derive(Default, Clone, Encode, Decode, RuntimeDebug)]
pub struct Bond<AccountId, Balance> {
	pub owner: AccountId,
	pub amount: Balance,
}

impl<A, B: Default> From<A> for Bond<A, B> {
	fn from(owner: A) -> Self {
		Bond {
			owner,
			amount: B::default(),
		}
	}
}

impl<A, B: Default> Bond<A, B> {
	pub fn from_owner(owner: A) -> Self {
		Bond {
			owner,
			amount: B::default(),
		}
	}
}

impl<AccountId: Ord, Balance: PartialEq> Eq for Bond<AccountId, Balance> {}

impl<AccountId: Ord, Balance: PartialEq> Ord for Bond<AccountId, Balance> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.owner.cmp(&other.owner)
	}
}

impl<AccountId: Ord, Balance: PartialEq> PartialOrd for Bond<AccountId, Balance> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<AccountId: Ord, Balance: PartialEq> PartialEq for Bond<AccountId, Balance> {
	fn eq(&self, other: &Self) -> bool {
		self.owner == other.owner && self.amount == other.amount
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
/// The activity status of the collator
pub enum CollatorStatus {
	/// Committed to be online and producing valid blocks (not equivocating)
	Active,
	/// Temporarily inactive and excused for inactivity
	Idle,
	/// Bonded until the inner round
	Leaving(RoundIndex),
}

impl Default for CollatorStatus {
	fn default() -> CollatorStatus {
		CollatorStatus::Active
	}
}

#[derive(Default, Encode, Decode, RuntimeDebug)]
/// Snapshot of collator state at the start of the round for which they are
/// selected
pub struct CollatorSnapshot<AccountId, Balance> {
	pub bond: Balance,
	pub delegators: Vec<Bond<AccountId, Balance>>,
	pub total: Balance,
}

impl<AccountId: Ord, Balance: PartialEq> PartialEq for CollatorSnapshot<AccountId, Balance> {
	fn eq(&self, other: &Self) -> bool {
		self.bond == other.bond && self.total == other.total && self.delegators.eq(&other.delegators)
	}
}

impl<AccountId: Ord, Balance: PartialEq> Eq for CollatorSnapshot<AccountId, Balance> {}

#[derive(Encode, Decode, RuntimeDebug)]
/// Global collator state with commission fee, bonded stake, and delegations
pub struct Collator<AccountId, Balance> {
	pub id: AccountId,
	pub bond: Balance,
	pub delegators: OrderedSet<Bond<AccountId, Balance>>,
	pub total: Balance,
	pub state: CollatorStatus,
}

impl<A: Ord + Clone, B: AtLeast32BitUnsigned + Ord + Copy + sp_std::ops::AddAssign + sp_std::ops::SubAssign>
	Collator<A, B>
{
	pub fn new(id: A, bond: B) -> Self {
		let total = bond;
		Collator {
			id,
			bond,
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

	pub fn bond_more(&mut self, more: B) {
		// TODO: use saturaring_add instead?
		self.bond += more;
		self.total += more;
	}

	// Returns None if underflow or less == self.bond (in which case collator should
	// leave)
	pub fn bond_less(&mut self, less: B) -> Option<B> {
		if self.bond > less {
			self.bond -= less;
			self.total -= less;
			Some(self.bond)
		} else {
			None
		}
	}

	pub fn inc_delegator(&mut self, delegator: A, more: B) {
		for x in &mut self.delegators.0 {
			if x.owner == delegator {
				x.amount += more;
				self.total += more;
				return;
			}
		}
	}

	pub fn dec_delegator(&mut self, delegator: A, less: B) {
		for x in &mut self.delegators.0 {
			if x.owner == delegator {
				x.amount -= less;
				self.total -= less;
				return;
			}
		}
	}

	pub fn go_offline(&mut self) {
		self.state = CollatorStatus::Idle;
	}

	pub fn go_online(&mut self) {
		self.state = CollatorStatus::Active;
	}

	pub fn leave_candidates(&mut self, round: RoundIndex) {
		self.state = CollatorStatus::Leaving(round);
	}
}

impl<A: Clone, B: Copy> From<Collator<A, B>> for CollatorSnapshot<A, B> {
	fn from(other: Collator<A, B>) -> CollatorSnapshot<A, B> {
		CollatorSnapshot {
			bond: other.bond,
			delegators: other.delegators.0,
			total: other.total,
		}
	}
}

#[derive(Encode, Decode, RuntimeDebug)]
pub struct Delegator<AccountId, Balance> {
	pub delegations: OrderedSet<Bond<AccountId, Balance>>,
	pub total: Balance,
}

impl<AccountId, Balance> Delegator<AccountId, Balance>
where
	AccountId: Ord + Clone,
	Balance: Copy + sp_std::ops::AddAssign + sp_std::ops::Add<Output = Balance> + sp_std::ops::SubAssign + PartialOrd,
{
	pub fn new(collator: AccountId, amount: Balance) -> Self {
		Delegator {
			delegations: OrderedSet::from(vec![Bond {
				owner: collator,
				amount,
			}]),
			total: amount,
		}
	}

	pub fn add_delegation(&mut self, bond: Bond<AccountId, Balance>) -> bool {
		let amt = bond.amount;
		if self.delegations.insert(bond) {
			self.total += amt;
			true
		} else {
			false
		}
	}

	// Returns Some(remaining balance), must be more than MinDelegatorStk
	// Returns None if delegation not found
	pub fn rm_delegation(&mut self, collator: AccountId) -> Option<Balance> {
		let mut amt: Option<Balance> = None;
		let delegations = self
			.delegations
			.0
			.iter()
			.filter_map(|x| {
				if x.owner == collator {
					amt = Some(x.amount);
					None
				} else {
					Some(x.clone())
				}
			})
			.collect();
		if let Some(balance) = amt {
			self.delegations = OrderedSet::from(delegations);
			self.total -= balance;
			Some(self.total)
		} else {
			None
		}
	}

	// Returns None if delegation not found
	pub fn inc_delegation(&mut self, collator: AccountId, more: Balance) -> Option<Balance> {
		for x in &mut self.delegations.0 {
			if x.owner == collator {
				x.amount += more;
				self.total += more;
				return Some(x.amount);
			}
		}
		None
	}

	// Returns Some(Some(balance)) if successful
	// None if delegation not found
	// Some(None) if underflow
	pub fn dec_delegation(&mut self, collator: AccountId, less: Balance) -> Option<Option<Balance>> {
		for x in &mut self.delegations.0 {
			if x.owner == collator {
				if x.amount > less {
					x.amount -= less;
					self.total -= less;
					return Some(Some(x.amount));
				} else {
					// underflow error; should rm entire delegation if x.amount == collator
					return Some(None);
				}
			}
		}
		None
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
/// The current round index and transition information
pub struct RoundInfo<BlockNumber> {
	/// Current round index
	pub current: RoundIndex,
	/// The first block of the current round
	pub first: BlockNumber,
	/// The length of the current round in number of blocks
	pub length: u32,
}

impl<B: Copy + Saturating + sp_std::ops::Add<Output = B> + sp_std::ops::Sub<Output = B> + From<u32> + PartialOrd>
	RoundInfo<B>
{
	pub fn new(current: RoundIndex, first: B, length: u32) -> RoundInfo<B> {
		RoundInfo { current, first, length }
	}
	/// Check if the round should be updated
	pub fn should_update(&self, now: B) -> bool {
		let l = now.saturating_sub(self.first);
		l >= self.length.into()
	}
	/// New round
	pub fn update(&mut self, now: B) {
		self.current = self.current.saturating_add(1u32);
		self.first = now;
	}
}

impl<B: Copy + Saturating + sp_std::ops::Add<Output = B> + sp_std::ops::Sub<Output = B> + From<u32> + PartialOrd>
	Default for RoundInfo<B>
{
	fn default() -> RoundInfo<B> {
		RoundInfo::new(0u32, 0u32.into(), 20u32.into())
	}
}

pub type RoundIndex = u32;
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
