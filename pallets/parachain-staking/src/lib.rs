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
#![allow(clippy::unused_unit)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
pub(crate) mod tests;

// TODO: Replace set with OrderedSet
mod inflation;
mod set;
mod types;

use frame_support::pallet;

pub use crate::pallet::*;

#[pallet]
pub mod pallet {
	pub use crate::inflation::{InflationInfo, RewardRate, StakingInfo};

	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, Get, Imbalance, LockIdentifier, LockableCurrency, ReservableCurrency, WithdrawReasons},
		transactional,
	};
	use frame_system::pallet_prelude::*;
	// TODO: Use ORML one once they point to Substrate master
	// use orml_utilities::OrderedSet;
	use sp_runtime::{
		traits::{Saturating, Zero},
		Perbill,
	};
	use sp_std::{collections::btree_map::BTreeMap, prelude::*};

	use crate::{
		set::OrderedSet,
		types::{BalanceOf, Bond, Collator, CollatorSnapshot, Delegator, RoundIndex, RoundInfo},
	};

	pub const REWARDS_ID: LockIdentifier = *b"kiltrwrd";

	/// Pallet for parachain staking
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency type
		// Note: Declaration of Balance taken from pallet_gilt
		type Currency: Currency<Self::AccountId, Balance = Self::CurrencyBalance>
			+ ReservableCurrency<Self::AccountId, Balance = Self::CurrencyBalance>
			+ LockableCurrency<Self::AccountId, Balance = Self::CurrencyBalance>
			+ Eq;

		/// Just the `Currency::Balance` type; we have this item to allow us to
		/// constrain it to `From<u64>`.
		// Note: Definition taken from pallet_gilt
		type CurrencyBalance: sp_runtime::traits::AtLeast32BitUnsigned
			+ parity_scale_codec::FullCodec
			+ Copy
			+ MaybeSerializeDeserialize
			+ sp_std::fmt::Debug
			+ Default
			+ From<u64>;

		/// Minimum number of blocks per round
		type MinBlocksPerRound: Get<u32>;
		/// Default number of blocks per round at genesis
		type DefaultBlocksPerRound: Get<u32>;
		/// Number of rounds that collators remain bonded before exit request is
		/// executed
		// TODO: Split into `BondDuration` and `ExitQueueDelay`
		type BondDuration: Get<RoundIndex>;
		/// Minimum number of selected candidates every round
		type MinSelectedCandidates: Get<u32>;
		/// Maximum delegators per collator
		type MaxDelegatorsPerCollator: Get<u32>;
		/// Maximum collators per delegator
		type MaxCollatorsPerDelegator: Get<u32>;
		/// Maximum number of collator candidates
		type MaxCollatorCandidates: Get<u32>;
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
	}

	#[pallet::error]
	// TODO: Add documentation
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
		AlreadyDelegating,
		NotYetDelegating,
		TooManyDelegators,
		TooManyCollatorCandidates,
		CannotActivateIfLeaving,
		ExceedMaxCollatorsPerDelegator,
		AlreadyDelegatedCollator,
		DelegationDNE,
		Underflow,
		InvalidSchedule,
		CannotSetBelowMin,
		RewardLockDNE,
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
		/// New Delegator, New Amount Bonded, Old Delegator, Old Amount Bonded,
		/// Collator, New Total Amount backing Collator
		DelegationReplaced(
			T::AccountId,
			BalanceOf<T>,
			T::AccountId,
			BalanceOf<T>,
			T::AccountId,
			BalanceOf<T>,
		),
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
		fn on_initialize(n: T::BlockNumber) -> frame_support::weights::Weight {
			let mut round = <Round<T>>::get();
			if round.should_update(n) {
				// kill snapshot of current round
				<AtStake<T>>::remove_prefix(round.current);
				// mutate round
				round.update(n);
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
			// TODO: Add post weight
			0
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
	pub(crate) type CollatorState<T: Config> =
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
	// TODO: Try to reduce storage footprint
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
	#[pallet::getter(fn inflation_config)]
	/// Inflation configuration
	pub type InflationConfig<T: Config> = StorageValue<_, InflationInfo, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn reward_locks)]
	/// Locked balance which has been granted as reward after a collator
	/// authored a block
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
					<Pallet<T>>::join_delegators(
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
			let (v_count, collator_staked, delegator_staked) = <Pallet<T>>::select_top_candidates(0u32);
			// Start Round 0 at Block 0
			let round: RoundInfo<T::BlockNumber> = RoundInfo::new(0u32, 0u32.into(), T::DefaultBlocksPerRound::get());
			<Round<T>>::put(round);
			// Snapshot total stake
			<Pallet<T>>::deposit_event(Event::NewRound(
				T::BlockNumber::zero(),
				0u32,
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
		pub fn set_inflation(origin: OriginFor<T>, inflation: InflationInfo) -> DispatchResult {
			frame_system::ensure_root(origin)?;

			Self::update_inflation(inflation)?;
			Ok(())
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
			Self::update_inflation(inflation)?;

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

			// Post-launch TODO: Replace with `check_collator_candidate_inclusion`.
			ensure!(
				(candidates.0.len() as u32) <= T::MaxCollatorCandidates::get(),
				Error::<T>::TooManyCollatorCandidates
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
			let when = now.saturating_add(T::BondDuration::get());
			ensure!(
				exits.insert(Bond {
					owner: collator.clone(),
					amount: when
				}),
				Error::<T>::AlreadyLeaving
			);
			state.leave_candidates(when);
			let mut candidates = <CandidatePool<T>>::get();
			if candidates.remove(&Bond::from(collator.clone())) {
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
			if candidates.remove(&Bond::from(collator.clone())) {
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

			let err_amt = T::Currency::unreserve(&collator, less);
			debug_assert!(err_amt.is_zero());
			if state.is_active() {
				Self::update_active(collator.clone(), state.total);
			}
			<CollatorState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CollatorBondedLess(collator, before, after));
			Ok(().into())
		}

		/// Join the set of delegators by delegating to a collator candidate.
		///
		/// NOTE: Expects the caller to have delegated before. Otherwise, they
		/// should call `delegate_more`.
		#[pallet::weight(0)]
		// TODO: Fix unit test panic with transactional feature enabled
		// #[transactional]
		pub fn join_delegators(
			origin: OriginFor<T>,
			// TODO: Switch to LookupSource
			collator: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			// first delegation
			ensure!(<DelegatorState<T>>::get(&acc).is_none(), Error::<T>::AlreadyDelegating);
			ensure!(amount >= T::MinDelegatorStk::get(), Error::<T>::NomBondBelowMin);
			// cannot be a collator candidate and delegator with same AccountId
			ensure!(!Self::is_candidate(&acc), Error::<T>::CandidateExists);

			// prepare update of collator state
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let delegation = Bond {
				owner: acc.clone(),
				amount,
			};
			ensure!(state.delegators.insert(delegation.clone()), Error::<T>::DelegatorExists);

			// update state and potentially kick a delegator with less bonded amount
			state = if (state.delegators.0.len() as u32) > T::MaxDelegatorsPerCollator::get() {
				Self::do_update_delegator(delegation, state)?
			} else {
				state.total = state.total.saturating_add(amount);
				state
			};
			let new_total = state.total;

			// lock bond
			// TODO: switch to lock instead of reserving
			T::Currency::reserve(&acc, amount)?;
			if state.is_active() {
				Self::update_active(collator.clone(), new_total);
			}

			// update states
			let (total_collators, total_delegators) = <Total<T>>::get();
			<Total<T>>::put((total_collators, total_delegators.saturating_add(amount)));

			// TODO: Add update of select_top_candidates
			<CollatorState<T>>::insert(&collator, state);
			<DelegatorState<T>>::insert(&acc, Delegator::new(collator.clone(), amount));
			Self::deposit_event(Event::Delegation(acc, amount, collator, new_total));

			Ok(().into())
		}

		/// Delegate to another collator candidate which results in updating the
		/// caller's delegation state.
		///
		/// NOTE: Expects the caller to have already delegated.
		#[pallet::weight(0)]
		// TODO: Fix unit test panic with transactional feature enabled
		// #[transactional]
		pub fn delegate_another_candidate(
			origin: OriginFor<T>,
			// TODO: Switch to LookupSource
			collator: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			if let Some(mut delegator) = <DelegatorState<T>>::get(&acc) {
				// delegation after first
				ensure!(amount >= T::MinDelegation::get(), Error::<T>::DelegationBelowMin);
				ensure!(
					(delegator.delegations.0.len() as u32) < T::MaxCollatorsPerDelegator::get(),
					Error::<T>::ExceedMaxCollatorsPerDelegator
				);

				// prepare new collator state
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
				ensure!(state.delegators.insert(delegation.clone()), Error::<T>::DelegatorExists);

				// update state and potentially kick a delegator with less bonded amount
				state = if (state.delegators.0.len() as u32) > T::MaxDelegatorsPerCollator::get() {
					Self::do_update_delegator(delegation, state)?
				} else {
					state.total = state.total.saturating_add(amount);
					state
				};
				let new_total = state.total;

				// lock bond
				// TODO: Switch from reserve to lock
				T::Currency::reserve(&acc, amount)?;
				if state.is_active() {
					Self::update_active(collator.clone(), new_total);
				}

				// Update states
				let (total_collators, total_delegators) = <Total<T>>::get();
				<Total<T>>::put((total_collators, total_delegators.saturating_add(amount)));

				// TODO: Add update of select_top_candidates
				<CollatorState<T>>::insert(&collator, state);
				<DelegatorState<T>>::insert(&acc, delegator);
				Self::deposit_event(Event::Delegation(acc, amount, collator, new_total));

				Ok(().into())
			} else {
				Err(Error::<T>::NotYetDelegating.into())
			}
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

			let err_amt = T::Currency::unreserve(&delegator, less);
			debug_assert!(err_amt.is_zero());
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

		/// Unlock rewards from staking which are avaible for the current block
		/// number. Checks whether the `RewardLocks` BTreeMap for the target has
		/// entries less or equal than the current block number.
		///
		/// NOTE: Unlocking automatically occurs in `note_author` after a block
		/// author has produced a block. If you assume that a specific collator
		/// candidate (and their  corresponding delegators) is an active
		/// collator for every round, it would be unnecessary to ever call
		/// `unlock_rewards` for such collator and their delegators.
		#[pallet::weight(0)]
		pub fn unlock_rewards(
			origin: OriginFor<T>,
			// TODO: Better to use Lookup?
			// target: <T::Lookup as StaticLookup>::Source
			target: T::AccountId,
		) -> DispatchResult {
			ensure_signed(origin)?;
			// let target = T::Lookup::lookup(target)?;

			let now: T::BlockNumber = <frame_system::Pallet<T>>::block_number();

			let lock = <RewardLocks<T>>::get(&target);
			ensure!(!lock.is_empty(), Error::<T>::RewardLockDNE);

			Self::do_update_reward_locks(&target, lock, now);
			Ok(())
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
			candidates.remove(&Bond::from(candidate.clone()));
			candidates.insert(Bond {
				owner: candidate,
				amount: total,
			});
			<CandidatePool<T>>::put(candidates);
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
			state.total = state.total.saturating_sub(delegator_stake);
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

		// Weight: writes(2)
		fn do_reward(who: &T::AccountId, reward: BalanceOf<T>, now: T::BlockNumber) {
			// mint
			if let Ok(imb) = T::Currency::deposit_into_existing(who, reward) {
				// set & update lock
				let mut locks = <RewardLocks<T>>::get(who);
				let unlock_block: T::BlockNumber = now.saturating_add(T::BondDuration::get().into());
				locks.insert(unlock_block, reward);
				Self::do_update_reward_locks(who, locks, now);
				// TODO: Remove?
				Self::deposit_event(Event::Rewarded(who.clone(), imb.peek()));
			}
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

			if !total_locked.is_zero() {
				<T::Currency as LockableCurrency<T::AccountId>>::set_lock(
					REWARDS_ID,
					who,
					total_locked,
					WithdrawReasons::except(WithdrawReasons::TRANSACTION_PAYMENT),
				);

				<RewardLocks<T>>::insert(who, locks);
			} else {
				// Lock is guaranteed to exist at this moment, because we ensure existance in
				// `unlock_rewards` or else have created it previously by paying rewards
				<T::Currency as LockableCurrency<T::AccountId>>::remove_lock(REWARDS_ID, who);
				<RewardLocks<T>>::remove(who);
			}
		}

		/// Attempts to add the bond to the set of delegators of a collator
		/// which already reached its maximum size by removing an already
		/// existing delegator with less bonded value. If the given bonded
		/// amount is at most the minimum bonded value of the original delegator
		/// set, an error is returned.
		fn do_update_delegator(
			bond: Bond<T::AccountId, BalanceOf<T>>,
			mut state: Collator<T::AccountId, BalanceOf<T>>,
		) -> Result<Collator<T::AccountId, BalanceOf<T>>, DispatchError> {
			// add bond & sort by amount
			let mut delegators: Vec<Bond<T::AccountId, BalanceOf<T>>> = state.delegators.0;
			// delegators.push(bond.clone());
			delegators.sort_by(|a, b| b.amount.cmp(&a.amount));

			// check whether bond is at last place
			match delegators.pop() {
				Some(Bond { amount, owner }) if amount < bond.amount => {
					state.total = state.total.saturating_sub(amount).saturating_add(bond.amount);
					state.delegators = OrderedSet::from_sorted_set(delegators);
					Self::deposit_event(Event::DelegationReplaced(
						bond.owner,
						bond.amount,
						owner,
						amount,
						state.id.clone(),
						state.total,
					));
					Ok(state)
				}
				_ => Err(Error::<T>::TooManyDelegators.into()),
			}
		}

		// Post-launch TODO: Think about Collator stake or total stake?
		// /// Attempts to add a collator candidate to the set of collator
		// /// candidates which already reached its maximum size. On success,
		// /// another collator with the minimum total stake is removed from the
		// /// set. On failure, an error is returned. removing an already existing
		// fn check_collator_candidate_inclusion(
		// 	bond: Bond<T::AccountId, BalanceOf<T>>,
		// 	mut candidates: OrderedSet<Bond<T::AccountId, BalanceOf<T>>>,
		// ) -> Result<(), DispatchError> {
		// 	todo!()
		// }
	}

	impl<T> pallet_authorship::EventHandler<T::AccountId, T::BlockNumber> for Pallet<T>
	where
		T: Config + pallet_authorship::Config,
	{
		fn note_author(author: T::AccountId) {
			let now = <Round<T>>::get().current;
			let state = <AtStake<T>>::get(now, author.clone());
			if state.bond >= T::MinCollatorStk::get() && state.total >= T::MinCollatorStk::get() {
				let block_now: T::BlockNumber = <frame_system::Pallet<T>>::block_number();
				// TODO: Do we rather want to use a snapshot of total at the start of the round?
				// --> Keep as is for now and worry about this later
				let (total_collator_stake, total_delegator_stake) = <Total<T>>::get();
				let (c_rewards, d_rewards) = Self::compute_block_issuance(total_collator_stake, total_delegator_stake);

				let amt_due_collator = c_rewards;
				let delegator_stake = state.total.saturating_sub(state.bond);
				let amt_due_delegators = d_rewards;

				// Reward collator
				Self::do_reward(&author, amt_due_collator, block_now);

				// Reward delegators
				// Reward delegators due portion
				for Bond { owner, amount } in state.delegators {
					if amount >= T::MinDelegatorStk::get() {
						// Compare this delegator's stake with the total amount of
						// delegated stake for this collator
						let percent = Perbill::from_rational(amount, delegator_stake);
						let due = percent * amt_due_delegators;
						Self::do_reward(&owner, due, block_now);
					}
				}
			}
		}
		// TODO: Does this need to be handled?
		fn note_uncle(_author: T::AccountId, _age: T::BlockNumber) {}
	}
}
