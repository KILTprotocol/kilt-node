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

//! # Parachain Staking
//! Minimal staking pallet that implements collator selection by total backed
//! stake. The main difference between this pallet and `frame/pallet-staking` is
//! that this pallet uses direct delegation. Delegators choose exactly who they
//! delegate and with what stake. This is different from `frame/pallet-staking`
//! where you approval vote and then run Phragmen.
//!
//! ### Rules
//! There is a new round every `BlocksPerRound` blocks.
//!
//! At the start of every round,
//! * issuance is distributed to collators for `BondDuration` rounds ago
//! in proportion to the points they received in that round (for authoring
//! blocks)
//! * queued collator exits are executed
//! * a new set of collators is chosen from the candidates
//!
//! To join the set of candidates, an account must call `join_candidates` with
//! stake >= `MinCollatorCandidateStk` and fee <= `MaxFee`. The fee is taken off
//! the top of any rewards for the collator before the remaining rewards are
//! distributed in proportion to stake to all delegators (including the
//! collator, who always self-delegates).
//!
//! To leave the set of candidates, the collator calls `leave_candidates`. If
//! the call succeeds, the collator is removed from the pool of candidates so
//! they cannot be selected for future collator sets, but they are not unstaked
//! until `BondDuration` rounds later. The exit request is stored in the
//! `ExitQueue` and processed `BondDuration` rounds later to unstake the
//! collator and all of its delegators.
//!
//! To join the set of delegators, an account must call `join_delegators` with
//! stake >= `MinDelegatorStk`. There are also runtime methods for delegating
//! additional collators and revoking delegations.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod inflation;
#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
pub(crate) mod tests;
use frame_support::pallet;
pub use inflation::{InflationInfo, RewardRate, StakingInfo};

pub use pallet::*;

#[pallet]
pub mod pallet {

	use super::InflationInfo;
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, Get, Imbalance, LockIdentifier, LockableCurrency, ReservableCurrency, WithdrawReasons},
	};
	use frame_system::pallet_prelude::*;
	use orml_utilities::OrderedSet;
	use parity_scale_codec::{Decode, Encode};
	use sp_runtime::{
		traits::{AtLeast32BitUnsigned, Saturating, Zero},
		Perbill, RuntimeDebug,
	};
	use sp_std::{cmp::Ordering, collections::btree_map::BTreeMap, prelude::*};

	pub const REWARDS_ID: LockIdentifier = *b"kiltrwrd";

	/// Pallet for parachain staking
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[derive(Default, Clone, Encode, Decode, RuntimeDebug)]
	pub struct Bond<AccountId, Balance> {
		pub owner: AccountId,
		pub amount: Balance,
	}

	impl<A, B: Default> Bond<A, B> {
		fn from_owner(owner: A) -> Self {
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

	impl<
			AccountId: Ord + Clone,
			Balance: Copy + sp_std::ops::AddAssign + sp_std::ops::Add<Output = Balance> + sp_std::ops::SubAssign + PartialOrd,
		> Delegator<AccountId, Balance>
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

	impl<B: Copy + sp_std::ops::Add<Output = B> + sp_std::ops::Sub<Output = B> + From<u32> + PartialOrd> RoundInfo<B> {
		pub fn new(current: RoundIndex, first: B, length: u32) -> RoundInfo<B> {
			RoundInfo { current, first, length }
		}
		/// Check if the round should be updated
		pub fn should_update(&self, now: B) -> bool {
			now - self.first >= self.length.into()
		}
		/// New round
		pub fn update(&mut self, now: B) {
			self.current += 1u32;
			self.first = now;
		}
	}

	impl<B: Copy + sp_std::ops::Add<Output = B> + sp_std::ops::Sub<Output = B> + From<u32> + PartialOrd> Default
		for RoundInfo<B>
	{
		fn default() -> RoundInfo<B> {
			RoundInfo::new(1u32, 1u32.into(), 20u32.into())
		}
	}

	type RoundIndex = u32;
	type RewardPoint = u32;
	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency type
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId>
			+ Eq;
		// TODO: Add CurrencyBalance as in pallet_gilt to make use of `From<u64`;

		/// Minimum number of blocks per round
		type MinBlocksPerRound: Get<u32>;
		/// Default number of blocks per round at genesis
		type DefaultBlocksPerRound: Get<u32>;
		/// Number of rounds that collators remain bonded before exit request is
		/// executed
		type BondDuration: Get<RoundIndex>;
		/// Minimum number of selected candidates every round
		type MinSelectedCandidates: Get<u32>;
		/// Maximum delegators per collator
		type MaxDelegatorsPerCollator: Get<u32>;
		/// Maximum collators per delegator
		type MaxCollatorsPerDelegator: Get<u32>;
		/// Minimum stake required for any account to be in `SelectedCandidates`
		/// for the round
		type MinCollatorStk: Get<BalanceOf<Self>>;
		/// Minimum stake required for any account to be a collator candidate
		type MinCollatorCandidateStk: Get<BalanceOf<Self>>;
		// Maximum stake possible for any collator to be a collator candidate
		type MaxCollatorCandidateStk: Get<BalanceOf<Self>>;
		/// Minimum stake for any registered on-chain account to delegate
		type MinDelegation: Get<BalanceOf<Self>>;
		/// Minimum stake for any registered on-chain account to become a
		/// delegator
		type MinDelegatorStk: Get<BalanceOf<Self>>;
		// TODO: Add MaxNumOfCollators
	}

	#[pallet::error]
	pub enum Error<T> {
		// Delegator Does Not Exist
		DelegatorDNE,
		CandidateDNE,
		DelegatorExists,
		CandidateExists,
		ValBondBelowMin,
		ValBondAboveMax,
		NomBondBelowMin,
		DelegationBelowMin,
		AlreadyOffline,
		AlreadyActive,
		AlreadyLeaving,
		TooManyDelegators,
		CannotActivateIfLeaving,
		ExceedMaxCollatorsPerNom,
		AlreadyDelegatedCollator,
		DelegationDNE,
		Underflow,
		InvalidSchedule,
		CannotSetBelowMin,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Starting Block, Round, Number of Collators Selected, Active Collator
		/// Stake, Delegator Stake
		NewRound(T::BlockNumber, RoundIndex, u32, BalanceOf<T>, BalanceOf<T>),
		/// Account, Amount Locked by candidate, New Total Locked Collator
		/// Candidate Amount
		JoinedCollatorCandidates(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Round, Collator Account, Collator Self Bond, Delegator Stake (total)
		CollatorChosen(RoundIndex, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Collator Account, Old Bond, New Bond
		CollatorBondedMore(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Collator Account, Old Bond, New Bond
		CollatorBondedLess(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		CollatorWentOffline(RoundIndex, T::AccountId),
		CollatorBackOnline(RoundIndex, T::AccountId),
		/// Round, Collator Account, Scheduled Exit
		CollatorScheduledExit(RoundIndex, T::AccountId, RoundIndex),
		/// Account, Amount Unlocked, New Total Collator Amount Locked, New
		/// Total Delegator Amount Locked
		CollatorLeft(T::AccountId, BalanceOf<T>, BalanceOf<T>, BalanceOf<T>),
		// Delegator, Collator, Old Delegation, New Delegation
		DelegationIncreased(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		// Delegator, Collator, Old Delegation, New Delegation
		DelegationDecreased(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Delegator, Amount Unstaked
		DelegatorLeft(T::AccountId, BalanceOf<T>),
		/// Delegator, Amount Locked, Collator, New Total Amount backing
		/// Collator
		Delegation(T::AccountId, BalanceOf<T>, T::AccountId, BalanceOf<T>),
		/// Delegator, Collator, Amount Unstaked, New Total Amount Staked for
		/// Collator
		DelegatorLeftCollator(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Paid the account (delegator or collator) the balance as liquid
		/// rewards
		Rewarded(T::AccountId, BalanceOf<T>),
		/// Round inflation range set with the provided annual inflation range
		RoundInflationSet(Perbill, Perbill, Perbill, Perbill),
		/// Set total selected candidates to this value [old, new]
		TotalSelectedSet(u32, u32),
		/// Set blocks per round [current_round, first_block, old, new]
		BlocksPerRoundSet(RoundIndex, T::BlockNumber, u32, u32),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: T::BlockNumber) {
			let mut round = <Round<T>>::get();
			if round.should_update(n) {
				// mutate round
				round.update(n);
				// pay all stakers for T::BondDuration rounds ago
				Self::pay_stakers(round.current);
				// execute all delayed collator exits
				Self::execute_delayed_collator_exits(round.current);
				// select top collator candidates for next round
				let (collator_count, collator_staked, delegator_staked) = Self::select_top_candidates(round.current);
				// start next round
				<Round<T>>::put(round);

				Self::deposit_event(Event::NewRound(
					round.first,
					round.current,
					collator_count,
					collator_staked,
					delegator_staked,
				));
			}
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn total_selected)]
	/// The total candidates selected every round
	type TotalSelected<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn round)]
	/// Current round index and next round scheduled transition
	pub type Round<T: Config> = StorageValue<_, RoundInfo<T::BlockNumber>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn delegator_state)]
	/// Get delegator state associated with an account if account is delegating
	/// else None
	type DelegatorState<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Delegator<T::AccountId, BalanceOf<T>>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn collator_state)]
	/// Get collator state associated with an account if account is collating
	/// else None
	type CollatorState<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Collator<T::AccountId, BalanceOf<T>>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn selected_candidates)]
	/// The collator candidates selected for the current round
	type SelectedCandidates<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total)]
	/// Total capital locked by this staking pallet
	// TODO: Might want to use Struct instead of Tuple
	type Total<T: Config> = StorageValue<_, (BalanceOf<T>, BalanceOf<T>), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn candidate_pool)]
	/// The pool of collator candidates, each with their total backing stake
	type CandidatePool<T: Config> = StorageValue<_, OrderedSet<Bond<T::AccountId, BalanceOf<T>>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn exit_queue)]
	/// A queue of collators awaiting exit `BondDuration` delay after request
	type ExitQueue<T: Config> = StorageValue<_, OrderedSet<Bond<T::AccountId, RoundIndex>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn at_stake)]
	/// Snapshot of collator delegation stake at the start of the round
	// TODO: Convert to StorageMap because we already note during the round
	pub type AtStake<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		RoundIndex,
		Twox64Concat,
		T::AccountId,
		CollatorSnapshot<T::AccountId, BalanceOf<T>>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn staked_collator)]
	/// Total backing stake for all collators in the current round
	pub type StakedCollator<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn staked_delegator)]
	/// Total backing stake for all delegators of the collators in the current
	/// round
	pub type StakedDelegator<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn inflation_config)]
	/// Inflation configuration
	pub type InflationConfig<T: Config> = StorageValue<_, InflationInfo, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn points)]
	/// Total points awarded to collators for block production in the round
	pub type Points<T: Config> = StorageMap<_, Twox64Concat, RoundIndex, RewardPoint, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn awarded_pts)]
	/// Points for each collator per round
	pub type AwardedPts<T: Config> =
		StorageDoubleMap<_, Twox64Concat, RoundIndex, Twox64Concat, T::AccountId, RewardPoint, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn reward_locks)]
	/// Total points awarded to collators for block production in the round
	pub type RewardLocks<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BTreeMap<T::BlockNumber, BalanceOf<T>>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub stakers: Vec<(T::AccountId, Option<T::AccountId>, BalanceOf<T>)>,
		pub inflation_config: InflationInfo,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				stakers: vec![],
				..Default::default()
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			assert!(self.inflation_config.is_valid(), "Invalid inflation configuration");
			<InflationConfig<T>>::put(self.inflation_config.clone());

			for &(ref actor, ref opt_val, balance) in &self.stakers {
				assert!(
					T::Currency::free_balance(&actor) >= balance,
					"Account does not have enough balance to bond."
				);
				let _ = if let Some(delegated_val) = opt_val {
					<Pallet<T>>::delegate(
						T::Origin::from(Some(actor.clone()).into()),
						delegated_val.clone(),
						balance,
					)
				} else {
					<Pallet<T>>::join_candidates(T::Origin::from(Some(actor.clone()).into()), balance)
				};
			}
			// Set total selected candidates to minimum config
			<TotalSelected<T>>::put(T::MinSelectedCandidates::get());
			// Choose top TotalSelected collator candidates
			let (v_count, collator_staked, delegator_staked) = <Pallet<T>>::select_top_candidates(1u32);
			// Start Round 1 at Block 0
			let round: RoundInfo<T::BlockNumber> = RoundInfo::new(1u32, 0u32.into(), T::DefaultBlocksPerRound::get());
			<Round<T>>::put(round);
			// Snapshot total stake
			<StakedCollator<T>>::put(collator_staked);
			<StakedDelegator<T>>::put(delegator_staked);
			<Pallet<T>>::deposit_event(Event::NewRound(
				T::BlockNumber::zero(),
				1u32,
				v_count,
				collator_staked,
				delegator_staked,
			));
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set the annual inflation rate to derive per-round inflation
		#[pallet::weight(0)]
		pub fn set_inflation(origin: OriginFor<T>, inflation: InflationInfo) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;

			Self::update_inflation(inflation)?;
			Ok(().into())
		}

		#[pallet::weight(0)]
		/// Set the total number of collator candidates selected per round
		/// - changes are not applied until the start of the next round
		pub fn set_total_selected(origin: OriginFor<T>, new: u32) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			ensure!(new >= T::MinSelectedCandidates::get(), Error::<T>::CannotSetBelowMin);
			let old = <TotalSelected<T>>::get();
			<TotalSelected<T>>::put(new);
			Self::deposit_event(Event::TotalSelectedSet(old, new));
			Ok(().into())
		}

		#[pallet::weight(0)]
		/// Set blocks per round
		/// - if called with `new` less than length of current round, will
		///   transition immediately
		/// in the next block
		pub fn set_blocks_per_round(origin: OriginFor<T>, new: u32) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			ensure!(new >= T::MinBlocksPerRound::get(), Error::<T>::CannotSetBelowMin);

			// Update inflation config
			let mut inflation = <InflationConfig<T>>::get();
			inflation.update_blocks_per_round(new);
			Self::update_inflation(inflation.clone())?;

			let mut round = <Round<T>>::get();
			let (now, first, old) = (round.current, round.first, round.length);
			round.length = new;
			<Round<T>>::put(round);

			Self::deposit_event(Event::BlocksPerRoundSet(now, first, old, new));
			Ok(().into())
		}

		/// Join the set of collator candidates
		#[pallet::weight(0)]
		pub fn join_candidates(origin: OriginFor<T>, bond: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			ensure!(!Self::is_candidate(&acc), Error::<T>::CandidateExists);
			ensure!(!Self::is_delegator(&acc), Error::<T>::DelegatorExists);
			ensure!(bond >= T::MinCollatorCandidateStk::get(), Error::<T>::ValBondBelowMin);
			ensure!(bond <= T::MaxCollatorCandidateStk::get(), Error::<T>::ValBondAboveMax);

			let mut candidates = <CandidatePool<T>>::get();
			ensure!(
				candidates.insert(Bond {
					owner: acc.clone(),
					amount: bond
				}),
				Error::<T>::CandidateExists
			);
			T::Currency::reserve(&acc, bond)?;

			let candidate = Collator::new(acc.clone(), bond);
			let (total_collators, total_delegators) = <Total<T>>::get();
			let total_collators = total_collators.saturating_add(bond);
			<Total<T>>::put((total_collators, total_delegators));
			<CollatorState<T>>::insert(&acc, candidate);
			<CandidatePool<T>>::put(candidates);

			Self::deposit_event(Event::JoinedCollatorCandidates(acc, bond, total_collators));
			Ok(().into())
		}

		/// Request to leave the set of candidates. If successful, the account
		/// is immediately removed from the candidate pool to prevent selection
		/// as a collator, but unbonding is executed with a delay of
		/// `BondDuration` rounds.
		#[pallet::weight(0)]
		pub fn leave_candidates(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_leaving(), Error::<T>::AlreadyLeaving);
			let mut exits = <ExitQueue<T>>::get();
			let now = <Round<T>>::get().current;
			let when = now + T::BondDuration::get();
			ensure!(
				exits.insert(Bond {
					owner: collator.clone(),
					amount: when
				}),
				Error::<T>::AlreadyLeaving
			);
			state.leave_candidates(when);
			let mut candidates = <CandidatePool<T>>::get();
			if candidates.remove(&Bond::from_owner(collator.clone())) {
				<CandidatePool<T>>::put(candidates);
			}
			<ExitQueue<T>>::put(exits);
			<CollatorState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CollatorScheduledExit(now, collator, when));
			Ok(().into())
		}

		/// Temporarily leave the set of collator candidates without unbonding
		#[pallet::weight(0)]
		pub fn go_offline(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(state.is_active(), Error::<T>::AlreadyOffline);
			state.go_offline();
			let mut candidates = <CandidatePool<T>>::get();
			if candidates.remove(&Bond::from_owner(collator.clone())) {
				<CandidatePool<T>>::put(candidates);
			}
			<CollatorState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CollatorWentOffline(<Round<T>>::get().current, collator));
			Ok(().into())
		}

		/// Rejoin the set of collator candidates if previously had called
		/// `go_offline`
		#[pallet::weight(0)]
		pub fn go_online(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_active(), Error::<T>::AlreadyActive);
			ensure!(!state.is_leaving(), Error::<T>::CannotActivateIfLeaving);
			state.go_online();
			let mut candidates = <CandidatePool<T>>::get();
			ensure!(
				candidates.insert(Bond {
					owner: collator.clone(),
					amount: state.total
				}),
				Error::<T>::AlreadyActive
			);
			<CandidatePool<T>>::put(candidates);
			<CollatorState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CollatorBackOnline(<Round<T>>::get().current, collator));
			Ok(().into())
		}

		/// Bond more for collator candidates
		#[pallet::weight(0)]
		// TODO: Make transactional
		pub fn candidate_bond_more(origin: OriginFor<T>, more: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;

			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_leaving(), Error::<T>::CannotActivateIfLeaving);
			ensure!(
				state.bond.saturating_add(more) <= T::MaxCollatorCandidateStk::get(),
				Error::<T>::ValBondAboveMax
			);

			T::Currency::reserve(&collator, more)?;
			let before = state.bond;
			state.bond_more(more);
			let after = state.bond;
			if state.is_active() {
				Self::update_active(collator.clone(), state.total);
			}
			<CollatorState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CollatorBondedMore(collator, before, after));
			Ok(().into())
		}
		/// Bond less for collator candidates
		#[pallet::weight(0)]
		pub fn candidate_bond_less(origin: OriginFor<T>, less: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_leaving(), Error::<T>::CannotActivateIfLeaving);
			let before = state.bond;
			let after = state.bond_less(less).ok_or(Error::<T>::Underflow)?;
			ensure!(after >= T::MinCollatorCandidateStk::get(), Error::<T>::ValBondBelowMin);
			T::Currency::unreserve(&collator, less);
			if state.is_active() {
				Self::update_active(collator.clone(), state.total);
			}
			<CollatorState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CollatorBondedLess(collator, before, after));
			Ok(().into())
		}
		/// If caller is not a delegator, then join the set of delegators
		/// If caller is a delegator, then makes delegation to change their
		/// delegation state
		#[pallet::weight(0)]
		pub fn delegate(
			origin: OriginFor<T>,
			collator: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			if let Some(mut delegator) = <DelegatorState<T>>::get(&acc) {
				// delegation after first
				ensure!(amount >= T::MinDelegation::get(), Error::<T>::DelegationBelowMin);
				ensure!(
					(delegator.delegations.0.len() as u32) < T::MaxCollatorsPerDelegator::get(),
					Error::<T>::ExceedMaxCollatorsPerNom
				);
				let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
				ensure!(
					delegator.add_delegation(Bond {
						owner: collator.clone(),
						amount
					}),
					Error::<T>::AlreadyDelegatedCollator
				);
				let delegation = Bond {
					owner: acc.clone(),
					amount,
				};
				ensure!(
					(state.delegators.0.len() as u32) < T::MaxDelegatorsPerCollator::get(),
					Error::<T>::TooManyDelegators
				);
				ensure!(state.delegators.insert(delegation), Error::<T>::DelegatorExists);
				T::Currency::reserve(&acc, amount)?;
				let new_total = state.total + amount;
				if state.is_active() {
					Self::update_active(collator.clone(), new_total);
				}

				// Update states
				let (total_collators, total_delegators) = <Total<T>>::get();
				<Total<T>>::put((total_collators, total_delegators.saturating_add(amount)));
				state.total = new_total;
				<CollatorState<T>>::insert(&collator, state);
				<DelegatorState<T>>::insert(&acc, delegator);
				Self::deposit_event(Event::Delegation(acc, amount, collator, new_total));
			} else {
				// first delegation
				ensure!(amount >= T::MinDelegatorStk::get(), Error::<T>::NomBondBelowMin);
				// cannot be a collator candidate and delegator with same AccountId
				ensure!(!Self::is_candidate(&acc), Error::<T>::CandidateExists);
				let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
				let delegation = Bond {
					owner: acc.clone(),
					amount,
				};
				ensure!(state.delegators.insert(delegation), Error::<T>::DelegatorExists);
				ensure!(
					(state.delegators.0.len() as u32) <= T::MaxDelegatorsPerCollator::get(),
					Error::<T>::TooManyDelegators
				);
				T::Currency::reserve(&acc, amount)?;
				let new_total = state.total + amount;

				if state.is_active() {
					Self::update_active(collator.clone(), new_total);
				}

				let (total_collators, total_delegators) = <Total<T>>::get();
				<Total<T>>::put((total_collators, total_delegators.saturating_add(amount)));
				state.total = new_total;
				<CollatorState<T>>::insert(&collator, state);
				<DelegatorState<T>>::insert(&acc, Delegator::new(collator.clone(), amount));
				Self::deposit_event(Event::Delegation(acc, amount, collator, new_total));
			}
			Ok(().into())
		}
		/// Leave the set of delegators and, by implication, revoke all ongoing
		/// delegations
		#[pallet::weight(0)]
		pub fn leave_delegators(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			let delegator = <DelegatorState<T>>::get(&acc).ok_or(Error::<T>::DelegatorDNE)?;
			for bond in delegator.delegations.0 {
				Self::delegator_leaves_collator(acc.clone(), bond.owner.clone())?;
			}
			<DelegatorState<T>>::remove(&acc);
			Self::deposit_event(Event::DelegatorLeft(acc, delegator.total));
			Ok(().into())
		}
		/// Revoke an existing delegation
		#[pallet::weight(0)]
		pub fn revoke_delegation(origin: OriginFor<T>, collator: T::AccountId) -> DispatchResultWithPostInfo {
			Self::delegator_revokes_collator(ensure_signed(origin)?, collator)
		}
		/// Bond more for delegators with respect to a specific collator
		/// candidate
		#[pallet::weight(0)]
		pub fn delegator_bond_more(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			let mut delegations = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorDNE)?;
			let mut collator = <CollatorState<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
			let _ = delegations
				.inc_delegation(candidate.clone(), more)
				.ok_or(Error::<T>::DelegationDNE)?;
			T::Currency::reserve(&delegator, more)?;
			let before = collator.total;
			collator.inc_delegator(delegator.clone(), more);
			let after = collator.total;

			if collator.is_active() {
				Self::update_active(candidate.clone(), collator.total);
			}

			<CollatorState<T>>::insert(&candidate, collator);
			<DelegatorState<T>>::insert(&delegator, delegations);
			Self::deposit_event(Event::DelegationIncreased(delegator, candidate, before, after));
			Ok(().into())
		}
		/// Bond less for delegators with respect to a specific delegator
		/// candidate
		#[pallet::weight(0)]
		pub fn delegator_bond_less(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			less: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			let mut delegations = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorDNE)?;
			let mut collator = <CollatorState<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
			let remaining = delegations
				.dec_delegation(candidate.clone(), less)
				.ok_or(Error::<T>::DelegationDNE)?
				.ok_or(Error::<T>::Underflow)?;

			ensure!(remaining >= T::MinDelegation::get(), Error::<T>::DelegationBelowMin);
			ensure!(
				delegations.total >= T::MinDelegatorStk::get(),
				Error::<T>::NomBondBelowMin
			);

			T::Currency::unreserve(&delegator, less);
			let before = collator.total;
			collator.dec_delegator(delegator.clone(), less);
			let after = collator.total;
			if collator.is_active() {
				Self::update_active(candidate.clone(), collator.total);
			}
			<CollatorState<T>>::insert(&candidate, collator);
			<DelegatorState<T>>::insert(&delegator, delegations);
			Self::deposit_event(Event::DelegationDecreased(delegator, candidate, before, after));
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn is_delegator(acc: &T::AccountId) -> bool {
			<DelegatorState<T>>::get(acc).is_some()
		}

		pub fn is_candidate(acc: &T::AccountId) -> bool {
			<CollatorState<T>>::get(acc).is_some()
		}

		pub fn is_selected_candidate(acc: &T::AccountId) -> bool {
			<SelectedCandidates<T>>::get().binary_search(acc).is_ok()
		}
		// ensure candidate is active before calling
		fn update_active(candidate: T::AccountId, total: BalanceOf<T>) {
			let mut candidates = <CandidatePool<T>>::get();
			candidates.remove(&Bond::from_owner(candidate.clone()));
			candidates.insert(Bond {
				owner: candidate,
				amount: total,
			});
			<CandidatePool<T>>::put(candidates);
		}

		fn compute_issuance(
			collator_stake: BalanceOf<T>,
			delegator_stake: BalanceOf<T>,
		) -> (BalanceOf<T>, BalanceOf<T>) {
			let config = <InflationConfig<T>>::get();
			config.round_issuance::<T>(collator_stake, delegator_stake)
		}

		fn compute_block_issuance(
			collator_stake: BalanceOf<T>,
			delegator_stake: BalanceOf<T>,
		) -> (BalanceOf<T>, BalanceOf<T>) {
			let config = <InflationConfig<T>>::get();
			config.block_issuance::<T>(collator_stake, delegator_stake)
		}

		fn delegator_revokes_collator(acc: T::AccountId, collator: T::AccountId) -> DispatchResultWithPostInfo {
			let mut delegator = <DelegatorState<T>>::get(&acc).ok_or(Error::<T>::DelegatorDNE)?;
			let old_total = delegator.total;
			let remaining = delegator
				.rm_delegation(collator.clone())
				.ok_or(Error::<T>::DelegationDNE)?;
			// edge case; if no delegations remaining, leave set of delegators
			if delegator.delegations.0.len().is_zero() {
				// leave the set of delegators because no delegations left
				Self::delegator_leaves_collator(acc.clone(), collator)?;
				<DelegatorState<T>>::remove(&acc);
				Self::deposit_event(Event::DelegatorLeft(acc, old_total));
				return Ok(().into());
			}
			ensure!(remaining >= T::MinDelegatorStk::get(), Error::<T>::NomBondBelowMin);
			Self::delegator_leaves_collator(acc.clone(), collator)?;
			<DelegatorState<T>>::insert(&acc, delegator);
			Ok(().into())
		}
		fn delegator_leaves_collator(delegator: T::AccountId, collator: T::AccountId) -> DispatchResultWithPostInfo {
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let mut exists: Option<BalanceOf<T>> = None;
			let noms = state
				.delegators
				.0
				.into_iter()
				.filter_map(|nom| {
					if nom.owner != delegator {
						Some(nom)
					} else {
						exists = Some(nom.amount);
						None
					}
				})
				.collect();
			let delegator_stake = exists.ok_or(Error::<T>::DelegatorDNE)?;
			let delegators = OrderedSet::from(noms);
			T::Currency::unreserve(&delegator, delegator_stake);
			state.delegators = delegators;
			state.total -= delegator_stake;
			if state.is_active() {
				Self::update_active(collator.clone(), state.total);
			}
			let (total_collators, total_delegators) = <Total<T>>::get();
			<Total<T>>::put((total_collators, total_delegators.saturating_sub(delegator_stake)));
			let new_total = state.total;
			<CollatorState<T>>::insert(&collator, state);
			Self::deposit_event(Event::DelegatorLeftCollator(
				delegator,
				collator,
				delegator_stake,
				new_total,
			));
			Ok(().into())
		}
		fn pay_stakers(next: RoundIndex) {
			let mint = |amt: BalanceOf<T>, to: T::AccountId| {
				if amt > T::Currency::minimum_balance() {
					if let Ok(imb) = T::Currency::deposit_into_existing(&to, amt) {
						Self::deposit_event(Event::Rewarded(to.clone(), imb.peek()));
					}
				}
			};
			let duration = T::BondDuration::get();
			if next > duration {
				let round_to_payout = next - duration;
				let total = <Points<T>>::get(round_to_payout);
				// TODO: We might just want to use this rounds stake such that large stakes of
				// inactive collators do not affect the rewards
				// let total_collator_stake = <StakedCollator<T>>::get();
				// let total_delegator_stake = <StakedDelegator<T>>::get();
				let (total_collator_stake, total_delegator_stake) = <Total<T>>::get();

				let (c_rewards, d_rewards) = Self::compute_issuance(total_collator_stake, total_delegator_stake);
				for (val, pts) in <AwardedPts<T>>::drain_prefix(round_to_payout) {
					let pct_due = Perbill::from_rational(pts, total);
					let amt_due_collator = pct_due * c_rewards;
					let amt_due_delegators = pct_due * d_rewards;

					// Take the snapshot of block author and delegations
					let state = <AtStake<T>>::take(round_to_payout, &val);
					// TODO: This can never fail, do we still need to ensure with saturating?
					let delegator_stake = state.total.saturating_sub(state.bond);

					// Pay collator
					if amt_due_collator > T::Currency::minimum_balance() {
						// println!("collator {:?} receives {:?}", val.clone(), amt_due_collator);

						mint(amt_due_collator, val.clone());
					}

					// Pay delegators
					if amt_due_delegators > T::Currency::minimum_balance() {
						// Pay delegators due portion
						for Bond { owner, amount } in state.delegators {
							// Compare this delegator's stake with the total amount of delegated stake for
							// this collator
							let percent = Perbill::from_rational(amount, delegator_stake);
							let due = percent * amt_due_delegators;
							// println!("owner {:?} receives {:?}", owner, due);
							mint(due, owner);
						}
					}
				}
			}
		}

		fn execute_delayed_collator_exits(next: RoundIndex) {
			let remain_exits = <ExitQueue<T>>::get()
				.0
				.into_iter()
				.filter_map(|x| {
					if x.amount > next {
						Some(x)
					} else {
						if let Some(state) = <CollatorState<T>>::get(&x.owner) {
							for bond in state.delegators.0 {
								// return stake to delegator
								T::Currency::unreserve(&bond.owner, bond.amount);
								// remove delegation from delegator state
								if let Some(mut delegator) = <DelegatorState<T>>::get(&bond.owner) {
									if let Some(remaining) = delegator.rm_delegation(x.owner.clone()) {
										if remaining.is_zero() {
											<DelegatorState<T>>::remove(&bond.owner);
										} else {
											<DelegatorState<T>>::insert(&bond.owner, delegator);
										}
									}
								}
							}
							// return stake to collator
							T::Currency::unreserve(&state.id, state.bond);

							let (total_collators, total_delegators) = <Total<T>>::get();
							let total_collators = total_collators.saturating_sub(state.bond);
							// safe because bond <= total at all times
							let total_delegators = total_delegators.saturating_sub(state.total - state.bond);
							<Total<T>>::put((total_collators, total_delegators));

							<CollatorState<T>>::remove(&x.owner);
							Self::deposit_event(Event::CollatorLeft(
								x.owner,
								state.total,
								total_collators,
								total_delegators,
							));
						}
						None
					}
				})
				.collect::<Vec<Bond<T::AccountId, RoundIndex>>>();
			<ExitQueue<T>>::put(OrderedSet::from(remain_exits));
		}

		/// Select the top `n` collators in terms of cumulated stake (self +
		/// from delegators) from the CandidatePool to become block authors for
		/// the next round.
		fn select_top_candidates(next: RoundIndex) -> (u32, BalanceOf<T>, BalanceOf<T>) {
			let (mut all_collators, mut total_collators, mut total_delegators) =
				(0u32, BalanceOf::<T>::zero(), BalanceOf::<T>::zero());
			let mut candidates = <CandidatePool<T>>::get().0;

			// Order candidates by their total stake
			// TODO: Safe to unwrap?
			candidates.sort_unstable_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap());
			let top_n = <TotalSelected<T>>::get() as usize;

			// Choose the top TotalSelected qualified candidates, ordered by stake (least to
			// greatest, thus requires `rev()`)
			let mut collators = candidates
				.into_iter()
				.rev()
				.take(top_n)
				.filter(|x| x.amount >= T::MinCollatorStk::get())
				.map(|x| x.owner)
				.collect::<Vec<T::AccountId>>();

			// Snapshot exposure for round for weighting reward distribution
			for account in collators.iter() {
				let state = <CollatorState<T>>::get(&account).expect("all members of CandidateQ must be candidates");
				let amount_collator = state.bond;
				let amount_delegators = state.total.saturating_sub(amount_collator);
				let exposure: CollatorSnapshot<T::AccountId, BalanceOf<T>> = state.into();
				<AtStake<T>>::insert(next, account, exposure);
				all_collators += 1u32;
				total_collators = total_collators.saturating_add(amount_collator);
				total_delegators = total_delegators.saturating_add(amount_delegators);
				Self::deposit_event(Event::CollatorChosen(
					next,
					account.clone(),
					amount_collator,
					amount_delegators,
				));
			}
			collators.sort();

			// Insert canonical collator set
			<SelectedCandidates<T>>::put(collators);
			(all_collators, total_collators, total_delegators)
		}

		fn update_inflation(inflation: InflationInfo) -> Result<(), DispatchError> {
			ensure!(inflation.is_valid(), Error::<T>::InvalidSchedule);
			Self::deposit_event(Event::RoundInflationSet(
				inflation.collator.max_rate,
				inflation.collator.reward_rate.round,
				inflation.delegator.max_rate,
				inflation.delegator.reward_rate.round,
			));
			<InflationConfig<T>>::put(inflation);
			Ok(())
		}

		fn do_reward(who: &T::AccountId, reward: BalanceOf<T>, now: T::BlockNumber) {
			// mint
			if reward > T::Currency::minimum_balance() {
				if let Ok(imb) = T::Currency::deposit_into_existing(who, reward) {
					// TODO: Remove?
					Self::deposit_event(Event::Rewarded(who.clone(), imb.peek()));
				}
			}

			// set & update lock
			let mut locks = <RewardLocks<T>>::get(who);
			// TODO: Fix dummy value
			let unlock_block: T::BlockNumber = now.saturating_add(222u32.into());
			locks.insert(unlock_block, reward);
			Self::do_update_reward_locks(who, locks, now);
		}
		fn do_update_reward_locks(
			who: &T::AccountId,
			mut locks: BTreeMap<T::BlockNumber, BalanceOf<T>>,
			now: T::BlockNumber,
		) {
			let mut total_locked: BalanceOf<T> = Zero::zero();
			let mut expired = Vec::new();

			// check potential unlocks
			for (block_number, locked_balance) in &locks {
				if block_number <= &now {
					expired.push(*block_number);
				} else {
					total_locked = total_locked.saturating_add(*locked_balance);
				}
			}
			for block_number in expired {
				locks.remove(&block_number);
			}

			// TODO: Check whether remove_lock is necessary if total_locked == 0
			<T::Currency as LockableCurrency<T::AccountId>>::set_lock(
				REWARDS_ID,
				who,
				total_locked,
				WithdrawReasons::except(WithdrawReasons::TRANSACTION_PAYMENT),
			);

			<RewardLocks<T>>::insert(who, locks);
		}
	}

	/// Reward author and their delegators
	// TODO: Remove author_inherent and author_filter once Aura is working for
	// parachains + make compatible with Aura
	impl<T: Config> author_inherent::EventHandler<T::AccountId> for Pallet<T> {
		fn note_author(author: T::AccountId) {
			let now = <Round<T>>::get().current;

			let state = <AtStake<T>>::take(now, author.clone());
			let (total_collator_stake, total_delegator_stake) = <Total<T>>::get();
			let (c_rewards, d_rewards) = Self::compute_block_issuance(total_collator_stake, total_delegator_stake);

			let amt_due_collator = Perbill::from_rational(state.bond, total_collator_stake) * c_rewards;
			let delegator_stake = state.total.saturating_sub(state.bond);
			let amt_due_delegators = Perbill::from_rational(delegator_stake, total_delegator_stake) * d_rewards;

			// Reward collator
			if amt_due_collator > T::Currency::minimum_balance() {
				Self::do_reward(&author, amt_due_collator, now.into());
			}

			// Reward delegators
			if amt_due_delegators > T::Currency::minimum_balance() {
				// Reward delegators due portion
				for Bond { owner, amount } in state.delegators {
					// Compare this delegator's stake with the total amount of delegated stake for
					// this collator
					let percent = Perbill::from_rational(amount, delegator_stake);
					let due = percent * amt_due_delegators;
					Self::do_reward(&owner, due, now.into());
				}
			}
		}
	}

	impl<T: Config> author_inherent::CanAuthor<T::AccountId> for Pallet<T> {
		fn can_author(account: &T::AccountId) -> bool {
			Self::is_selected_candidate(account)
		}
	}
}
