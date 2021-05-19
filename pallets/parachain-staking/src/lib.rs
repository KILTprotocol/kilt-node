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
//! * issuance is distributed to collators for `StakeDuration` rounds ago
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
//! until `StakeDuration` rounds later. The exit request is stored in the
//! `ExitQueue` and processed `StakeDuration` rounds later to unstake the
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
		assert_ok,
		pallet_prelude::*,
		traits::{
			Currency, EstimateNextSessionRotation, Get, Imbalance, LockIdentifier, LockableCurrency,
			ReservableCurrency, WithdrawReasons,
		},
		transactional,
	};
	use frame_system::pallet_prelude::*;
	use pallet_session::ShouldEndSession;
	// TODO: Use ORML one once they point to Substrate master
	// use orml_utilities::OrderedSet;
	use pallet_balances::{BalanceLock, Locks};
	use sp_runtime::{
		traits::{Saturating, Zero},
		Percent, Perquintill,
	};
	use sp_staking::SessionIndex;
	use sp_std::{collections::btree_map::BTreeMap, prelude::*};

	use crate::{
		set::OrderedSet,
		types::{BalanceOf, Bond, Collator, CollatorSnapshot, Delegator, RoundIndex, RoundInfo, TotalStake},
	};

	/// Kilt-specific lock for staking rewards.
	pub(crate) const STAKING_ID: LockIdentifier = *b"kiltpstk";

	/// Pallet for parachain staking.
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_balances::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		// FIXME: Remove Currency and CurrencyBalance types. Problem: Need to restrict
		// pallet_balances::Config::Balance with From<u64> for usage with Perquintill
		// multiplication
		/// The currency type
		/// Note: Declaration of Balance taken from pallet_gilt
		type Currency: Currency<Self::AccountId, Balance = Self::CurrencyBalance>
			+ ReservableCurrency<Self::AccountId, Balance = Self::CurrencyBalance>
			+ LockableCurrency<Self::AccountId, Balance = Self::CurrencyBalance>
			+ Eq;

		// TODO: Check whether this type is still needed after restricting Config to
		// pallet_balances::Config
		/// Just the `Currency::Balance` type; we have this item to allow us to
		/// constrain it to `From<u64>`.
		/// Note: Definition taken from pallet_gilt
		type CurrencyBalance: sp_runtime::traits::AtLeast32BitUnsigned
			+ parity_scale_codec::FullCodec
			+ Copy
			+ MaybeSerializeDeserialize
			+ sp_std::fmt::Debug
			+ Default
			+ From<u64>
			+ Into<<Self as pallet_balances::Config>::Balance>
			+ From<<Self as pallet_balances::Config>::Balance>;

		/// Minimum number of blocks validation rounds can last.
		type MinBlocksPerRound: Get<Self::BlockNumber>;
		/// Default number of blocks validation rounds last, as set in the genesis configuration.
		type DefaultBlocksPerRound: Get<Self::BlockNumber>;
		/// Number of rounds a collator's funds remain staked after the collator submits a request to leave the set of collator candidates.
		// TODO: Split into `StakeDuration` and `ExitQueueDelay`
		type BondDuration: Get<RoundIndex>;
		/// Minimum number of collators selected from the set of candidates at every validation round.
		type MinSelectedCandidates: Get<u32>;
		/// Maximum number of delegators a single collator can have.
		type MaxDelegatorsPerCollator: Get<u32>;
		/// Maximum number of collators a single delegator can delegate.
		type MaxCollatorsPerDelegator: Get<u32>;
		/// Maximum size of the collator candidates set.
		type MaxCollatorCandidates: Get<u32>;
		/// Minimum stake required for any account to be elected as validator for a round.
		type MinCollatorStk: Get<BalanceOf<Self>>;
		/// Minimum stake required for any account to be added to the set of candidates.
		type MinCollatorCandidateStk: Get<BalanceOf<Self>>;
		/// Maximum stake required for any account to be added to the set of candidates.
		type MaxCollatorCandidateStk: Get<BalanceOf<Self>>;
		/// Minimum stake required for any account to be able to delegate.
		type MinDelegation: Get<BalanceOf<Self>>;
		/// Minimum stake required for any account to become a delegator.
		type MinDelegatorStk: Get<BalanceOf<Self>>;
		/// Max number of concurrent active unbonding requests before
		/// withdrawing.
		type MaxUnbondRequests: Get<usize>;
	}

	#[pallet::error]
	// TODO: Add documentation
	pub enum Error<T> {
		/// The account is not part of the delegators set.
		DelegatorDNE,
		/// The account is not part of the collator candidates set.
		CandidateDNE,
		/// The account is already part of the delegators set.
		DelegatorExists,
		/// The account is already part of the collator candidates set.
		CandidateExists,
		/// The account has not staked enough funds to be added to the collator candidates set.
		ValBondBelowMin,
		/// The account has already staked the maximum amount of funds possible.
		ValBondAboveMax,
		/// The account has not staked enough funds to become a delegator.
		NomBondBelowMin,
		/// The account has not staked enough funds to delegate a collator candidate.
		DelegationBelowMin,
		/// The collator candidate has already left the set of active collator candidates.
		AlreadyOffline,
		/// The collator candidate is already actively actively participating in the set of collator candidates.
		AlreadyActive,
		/// The collator candidate has already trigger the process to leave the set of collator candidates.
		AlreadyLeaving,
		/// The account is already delegating the collator candidate.
		AlreadyDelegating,
		/// The account has not delegated any collator candidate yet, hence it is not in the set of delegators.
		NotYetDelegating,
		/// The collator candidate has already reached the maximum number of delegators.
		///
		/// This error is generated in cases a new delegation request does not stake enough funds to replace some other existing delegation.
		TooManyDelegators,
		/// The set of collator candidates has already reached the maximum size allowed.
		// TODO: Update this comment when the new logic to include new collator candidates is added (by using `check_collator_candidate_inclusion`).
		TooManyCollatorCandidates,
		/// The collator candidate is in the process of leaving the set of candidates and cannot perform any other actions in the meantime.
		CannotActivateIfLeaving,
		/// The delegator has already delegated the maximum number of candidates allowed.
		ExceedMaxCollatorsPerDelegator,
		/// The delegator has already previously delegated the collator candidate.
		AlreadyDelegatedCollator,
		/// The given delegation does not exist in the set of delegations.
		DelegationDNE,
		/// The collator delegate or the delegator is trying to un-stake more funds that are currently staked.
		Underflow,
		/// The number of selected candidates or of blocks per staking round is below the minimum value allowed.
		CannotSetBelowMin,
		/// An invalid inflation configuration is trying to be set.
		InvalidSchedule,
		/// The staking reward being unlocked does not exist.
		RewardLockDNE,
		/// Max unlocking requests reached.
		NoMoreUnbonding,
		/// Provided bonded value is zero.
		BondDNE,
		/// Cannot withdraw when Unbonded is empty.
		UnbondingIsEmpty,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new staking round has started.
		/// \[round starting block, round number, number of collators selected, total stake of selected collators, total stake of delegators for the selected collators\]
		NewRound(T::BlockNumber, RoundIndex, u32, BalanceOf<T>, BalanceOf<T>),
		/// A new account has joined the set of collator candidates.
		/// \[account, amount staked by the new candidate, new total stake of collator candidates\]
		JoinedCollatorCandidates(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// A collator candidate has been selected for the next validation round.
		/// \[round number, collator's account, collator's total stake, collator's delegators' total stake\]
		CollatorChosen(RoundIndex, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// A collator candidate has increased the amount of funds at stake.
		/// \[collator's account, previous stake, new stake\]
		CollatorBondedMore(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// A collator candidate has decreased the amount of funds at stake.
		/// \[collator's account, previous stake, new stake\]
		CollatorBondedLess(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// A collator candidate has gone from active to idle.
		/// \[round number, collator's account\]
		CollatorWentOffline(RoundIndex, T::AccountId),
		/// A collator candidate has returned to an active state.
		/// \[round number, collator's account\]
		CollatorBackOnline(RoundIndex, T::AccountId),
		/// A collator candidate has started the process to leave the set of candidates.
		/// \[round number, collator's account, round number when the collator will be effectively removed from the set of candidates\]
		CollatorScheduledExit(RoundIndex, T::AccountId, RoundIndex),
		/// An account has left the set of collator candidates.
		/// \[account, amount of funds un-staked, new total stake of collator candidates, new total stake of delegators for the remaining collators\]
		CollatorLeft(T::AccountId, BalanceOf<T>, BalanceOf<T>, BalanceOf<T>),
		/// A delegator has increased the amount of funds at stake for a collator.
		/// \[delegator's account, collator's account, previous delegation stake, new delegation stake\]
		DelegationIncreased(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// A delegator has decreased the amount of funds at stake for a collator.
		/// \[delegator's account, collator's account, previous delegation stake, new delegation stake\]
		DelegationDecreased(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// An account has left the set of delegators.
		/// \[account, amount of funds un-staked\]
		DelegatorLeft(T::AccountId, BalanceOf<T>),
		/// An account has delegated a new collator candidate.
		/// \[account, amount of funds staked, total amount of delegators' funds staked for the collator candidate\]
		Delegation(T::AccountId, BalanceOf<T>, T::AccountId, BalanceOf<T>),
		/// A new delegation has replaced an existing one in the set of ongoing delegations for a collator candidate.
		/// \[new delegator's account, amount of funds staked in the new delegation, replaced delegator's account, amount of funds staked in the replace delegation, collator candidate's account, new total amount of delegators' funds staked for the collator candidate\]
		DelegationReplaced(
			T::AccountId,
			BalanceOf<T>,
			T::AccountId,
			BalanceOf<T>,
			T::AccountId,
			BalanceOf<T>,
		),
		/// An account has stopped delegating a collator candidate.
		/// \[account, collator candidate's account, old amount of delegators' funds staked, new amount of delegators' funds staked\]
		DelegatorLeftCollator(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// A collator or a delegator has received a reward.
		/// \[account, amount of reward\]
		Rewarded(T::AccountId, BalanceOf<T>),
		/// Inflation configuration for future validation rounds has changed.
		/// \[maximum collator's staking rate, maximum collator's reward rate, maximum delegator's staking rate, maximum delegator's reward rate\]
		RoundInflationSet(Perquintill, Perquintill, Perquintill, Perquintill),
		/// The maximum number of collator candidates selected in future validation rounds has changed.
		/// \[old value, new value\]
		TotalSelectedSet(u32, u32),
		/// The length in blocks for future validation rounds has changed.
		/// \[round number, first block in the current round, old value, new value\]
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
				let (collator_count, collator_staked, delegator_staked) =
				Self::select_top_candidates(round.current); 		// start next round
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

	/// The maximum number of collator candidates selected at each round.
	#[pallet::storage]
	#[pallet::getter(fn total_selected)]
	//TODO: Should be renamed to something like MaxSelected
	type TotalSelected<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Current round number and next round scheduled transition.
	#[pallet::storage]
	#[pallet::getter(fn round)]
	pub type Round<T: Config> = StorageValue<_, RoundInfo<T::BlockNumber>, ValueQuery>;

	/// Delegation staking information.
	///
	/// It maps from an account to its delegation details.
	#[pallet::storage]
	#[pallet::getter(fn delegator_state)]
	type DelegatorState<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Delegator<T::AccountId, BalanceOf<T>>, OptionQuery>;

	/// Collator candidates staking information.
	///
	/// It maps from an account to its collator details.
	#[pallet::storage]
	#[pallet::getter(fn collator_state)]
	pub(crate) type CollatorState<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Collator<T::AccountId, BalanceOf<T>>, OptionQuery>;

	/// The collator candidates selected for the latest validation round.
	#[pallet::storage]
	#[pallet::getter(fn selected_candidates)]
	type SelectedCandidates<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	/// Total funds locked by this staking pallet.
	#[pallet::storage]
	#[pallet::getter(fn total)]
	type Total<T: Config> = StorageValue<_, TotalStake<BalanceOf<T>>, ValueQuery>;

	/// The set of collator candidates, each with their total backing stake.
	#[pallet::storage]
	#[pallet::getter(fn candidate_pool)]
	type CandidatePool<T: Config> = StorageValue<_, OrderedSet<Bond<T::AccountId, BalanceOf<T>>>, ValueQuery>;

	/// A queue of collators waiting to be removed from the set of candidates.
	#[pallet::storage]
	#[pallet::getter(fn exit_queue)]
	type ExitQueue<T: Config> = StorageValue<_, OrderedSet<Bond<T::AccountId, RoundIndex>>, ValueQuery>;

	/// Snapshot of collator delegation stake at the start of the round.
	///
	/// It maps from the combination of round number and account to the collator snapshot for that account.
	// TODO: Try to reduce storage footprint
	#[pallet::storage]
	#[pallet::getter(fn at_stake)]
	pub type AtStake<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		RoundIndex,
		Twox64Concat,
		T::AccountId,
		CollatorSnapshot<T::AccountId, BalanceOf<T>>,
		ValueQuery,
	>;

	/// Inflation configuration.
	#[pallet::storage]
	#[pallet::getter(fn inflation_config)]
	pub type InflationConfig<T: Config> = StorageValue<_, InflationInfo, ValueQuery>;

	/// The funds waiting to be unstaked.
	///
	/// It maps from accounts to all the funds addressed to them in the future blocks.
	#[pallet::storage]
	#[pallet::getter(fn unbonding)]
	pub type Unbonding<T: Config> =
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
			assert!(
				self.stakers.iter().find(|s| s.1.is_none()).is_some(),
				"at least one collator in genesis config"
			);

			<InflationConfig<T>>::put(self.inflation_config.clone());

			for &(ref actor, ref opt_val, balance) in &self.stakers {
				assert!(
					T::Currency::free_balance(&actor) >= balance,
					"Account does not have enough balance to bond."
				);
				if let Some(delegated_val) = opt_val {
					assert_ok!(<Pallet<T>>::join_delegators(
						T::Origin::from(Some(actor.clone()).into()),
						delegated_val.clone(),
						balance,
					));
				} else {
					assert_ok!(<Pallet<T>>::join_candidates(
						T::Origin::from(Some(actor.clone()).into()),
						balance
					));
				}
			}
			// Set total selected candidates to minimum config
			<TotalSelected<T>>::put(T::MinSelectedCandidates::get());
			// Choose top TotalSelected collator candidates
			let (v_count, collator_staked, delegator_staked) = <Pallet<T>>::select_top_candidates(0u32);
			assert!(!v_count.is_zero());
			assert!(!<SelectedCandidates<T>>::get().is_empty());

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

		/// Set the annual inflation rate to derive per-round inflation.
		///
		/// The inflation details are considered valid if the annual reward rate is approximately the per-block reward rate multiplied by the estimated* total number of blocks per year.
		///
		/// *The estimated average block time is six seconds.
		///
		/// The dispatch origin must be Root.
		///
		/// Emits `RoundInflationSet`.
		#[pallet::weight(0)]
		pub fn set_inflation(origin: OriginFor<T>, inflation: InflationInfo) -> DispatchResult {
			frame_system::ensure_root(origin)?;

			Self::update_inflation(inflation)?;
			Ok(())
		}

		/// Set the maximum number of collator candidates that can be selected at the beginning of each validation round.
		///
		/// Changes are not applied until the start of the next round.
		///
		/// The new value must be higher than the minimum allowed as set in the pallet's configuration.
		///
		/// The dispatch origin must be Root.
		///
		/// Emits `TotalSelectedSet`.
		//TODO: Should be changed to something like set_maximum_selected. Same for the Event name and all related stuff.
		#[pallet::weight(0)]
		pub fn set_total_selected(origin: OriginFor<T>, new: u32) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			ensure!(new >= T::MinSelectedCandidates::get(), Error::<T>::CannotSetBelowMin);
			let old = <TotalSelected<T>>::get();
			<TotalSelected<T>>::put(new);
			Self::deposit_event(Event::TotalSelectedSet(old, new));
			Ok(().into())
		}

		/// Set the number of blocks each validation round lasts.
		///
		/// If the new value is less than the length of the current round, the system will immediately move to the next round in the next block.
		///
		/// The new value must be higher than the minimum allowed as set in the pallet's configuration.
		///
		/// The dispatch origin must be Root.
		///
		/// Emits `BlocksPerRoundSet`.
		#[pallet::weight(0)]
		pub fn set_blocks_per_round(origin: OriginFor<T>, new: u32) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			ensure!(new >= T::MinBlocksPerRound::get(), Error::<T>::CannotSetBelowMin);

			let mut round = <Round<T>>::get();
			let (now, first, old) = (round.current, round.first, round.length);
			round.length = new;
			<Round<T>>::put(round);

			Self::deposit_event(Event::BlocksPerRoundSet(now, first, old, new));
			Ok(().into())
		}

		/// Join the set of collator candidates.
		///
		/// In the next blocks, if the collator candidate has enough funds staked to be included in any of the top `TotalSelected` positions, it will be included in the set of potential authors that will be selected by the stake-weighted random selection function.
		///
		/// The staked funds of the new collator candidate are added to the total stake of the system.
		///
		/// The total amount of funds staked must be within the allowed range as set in the pallet's configuration.
		///
		/// The dispatch origin must not be already part of the collator candidates nor of the delegators set.
		///
		/// Emits `JoinedCollatorCandidates`.
		//TODO: Change bond to stake, in both signature and in the comment.
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
				(candidates.len() as u32) <= T::MaxCollatorCandidates::get(),
				Error::<T>::TooManyCollatorCandidates
			);
			Self::increase_lock(&acc, bond)?;

			let candidate = Collator::new(acc.clone(), bond);
			let TotalStake {
				collators: total_collators,
				delegators: total_delegators,
			} = <Total<T>>::get();
			let total_collators = total_collators.saturating_add(bond);
			<Total<T>>::put(TotalStake {
				collators: total_collators,
				delegators: total_delegators,
			});
			<CollatorState<T>>::insert(&acc, candidate);
			<CandidatePool<T>>::put(candidates);

			Self::deposit_event(Event::JoinedCollatorCandidates(acc, bond, total_collators));
			Ok(().into())
		}

		/// Request to leave the set of collator candidates.
		///
		/// If successful, the account is immediately removed from the candidate pool to prevent selection as a collator in future validation rounds,
		/// but unstaking of the funds is executed with a delay of `BondDuration` rounds.
		///
		/// The total stake of the pallet is not affected by this operation until the funds are released after `BondDuration` rounds.
		///
		/// Emits `CollatorScheduledExit`.
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
			if candidates.remove_by(|bond| bond.owner.cmp(&collator)).is_some() {
				<CandidatePool<T>>::put(candidates);
			}
			<ExitQueue<T>>::put(exits);
			<CollatorState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CollatorScheduledExit(now, collator, when));
			Ok(().into())
		}

		/// Temporarily leave the set of collator candidates without unstaking.
		///
		/// For as long as offline, the collator will not be selected anymore to produce new blocks, but its staked funds will not scheduled to be released.
		///
		/// The total stake of the system, including validators' stakes, is left unchanged.
		///
		/// Emits `CollatorWentOffline`.
		#[pallet::weight(0)]
		pub fn go_offline(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;

			ensure!(!state.is_leaving(), Error::<T>::CannotActivateIfLeaving);
			ensure!(state.is_active(), Error::<T>::AlreadyOffline);
			state.go_offline();
			let mut candidates = <CandidatePool<T>>::get();
			if candidates.remove_by(|bond| bond.owner.cmp(&collator)).is_some() {
				<CandidatePool<T>>::put(candidates);
			}
			<CollatorState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CollatorWentOffline(<Round<T>>::get().current, collator));
			Ok(().into())
		}

		/// Rejoin the set of collator candidates with the previously staked funds.
		///
		/// From the moment of re-inclusion in the set, the collator candidate will be re-considered for block production if enough funds are staked to rank in any of the top `TotalSelected` positions.
		///
		/// The total stake of the pallet is not affected by this operation.
		///
		/// Emits `CollatorBackOnline`.
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

		/// Stake more funds for a collator candidate.
		///
		/// If not in the set of candidates, staking enough funds allows the account to be added to it.
		/// The larger amount of funds, the higher chances to be selected as the author of the next block.
		///
		/// This operation affects the pallet's total stake amount.
		///
		/// The resulting total amount of funds staked must be within the allowed range as set in the pallet's configuration.
		///
		/// Emits `CollatorBondedMore`.
		#[pallet::weight(0)]
		// TODO: Make transactional
		pub fn candidate_bond_more(origin: OriginFor<T>, more: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;

			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_leaving(), Error::<T>::CannotActivateIfLeaving);

			let before = state.bond;
			state.bond_more(more);
			let after = state.bond;
			ensure!(after <= T::MaxCollatorCandidateStk::get(), Error::<T>::ValBondAboveMax);

			Self::increase_lock(&collator, after)?;

			if state.is_active() {
				Self::update_active(collator.clone(), state.total);
			}
			<CollatorState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CollatorBondedMore(collator, before, after));
			Ok(().into())
		}

		/// Stake less funds for a collator candidate.
		///
		/// If the new amound of staked fund is not large enough, the account could be removed from the set of collator candidates and not be considered for authoring the next blocks.
		///
		/// This operation affects the pallet's total stake amount.
		///
		/// The unstaked funds are not release immediately to the account, but they will be available after `BondDuration` rounds.
		///
		/// The resulting total amount of funds staked must be within the allowed range as set in the pallet's configuration.
		///
		/// Emits `CollatorBondedLess`.
		#[pallet::weight(0)]
		pub fn candidate_bond_less(origin: OriginFor<T>, less: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_leaving(), Error::<T>::CannotActivateIfLeaving);
			let before = state.bond;
			let after = state.bond_less(less).ok_or(Error::<T>::Underflow)?;
			ensure!(after >= T::MinCollatorCandidateStk::get(), Error::<T>::ValBondBelowMin);

			// we don't unlock immediately
			Self::prepare_withdraw(&collator, less)?;

			if state.is_active() {
				Self::update_active(collator.clone(), state.total);
			}
			<CollatorState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CollatorBondedLess(collator, before, after));
			Ok(().into())
		}

		/// Join the set of delegators by delegating to a collator candidate.
		///
		/// The account that wants to delegate cannot be part of the collator candidates set as well.
		///
		/// The caller must _not_ have delegated before. Otherwise, `delegate_another_candidate` should be called.
		///
		/// The amount staked must be larger than the minimum required to become a delegator as set in the pallet's configuration.
		///
		/// As only `MaxDelegatorsPerCollator` are allowed to delegate a given collator, the amount staked must be larger than the lowest one in the current set of delegator for the operation to be meaningful.
		///
		/// The collator's total stake as well as the pallet's total stake are increased accordingly.
		///
		/// Emits `Delegation`.
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
			state = if (state.delegators.len() as u32) > T::MaxDelegatorsPerCollator::get() {
				Self::do_update_delegator(delegation, state)?
			} else {
				state.total = state.total.saturating_add(amount);
				state
			};
			let new_total = state.total;

			// lock bond
			Self::increase_lock(&acc, amount)?;
			if state.is_active() {
				Self::update_active(collator.clone(), new_total);
			}

			// update states
			let TotalStake {
				collators: total_collators,
				delegators: total_delegators,
			} = <Total<T>>::get();
			<Total<T>>::put(TotalStake {
				collators: total_collators,
				delegators: total_delegators.saturating_add(amount),
			});

			// TODO: Add update of select_top_candidates
			<CollatorState<T>>::insert(&collator, state);
			<DelegatorState<T>>::insert(&acc, Delegator::new(collator.clone(), amount));
			Self::deposit_event(Event::Delegation(acc, amount, collator, new_total));

			Ok(().into())
		}

		/// Delegate another collator's candidate by staking some funds and increasing the pallet's as well as the collator's total stake.
		///
		/// The account that wants to delegate cannot be part of the collator candidates set as well.
		///
		/// The caller _must_ have delegated before. Otherwise, `join_delegators` should be called.
		///
		/// If the delegator has already delegated the maximum number of collator candidates, this operation will fail.
		///
		/// The amount staked must be larger than the minimum required to become a delegator as set in the pallet's configuration.
		///
		/// As only `MaxDelegatorsPerCollator` are allowed to delegate a given collator, the amount staked must be larger than the lowest one in the current set of delegator for the operation to be meaningful.
		///
		/// The collator's total stake as well as the pallet's total stake are increased accordingly.
		///
		/// Emits `Delegation`.
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
			let mut delegator = <DelegatorState<T>>::get(&acc).ok_or(Error::<T>::NotYetDelegating)?;
			// delegation after first
			ensure!(amount >= T::MinDelegation::get(), Error::<T>::DelegationBelowMin);
			ensure!(
				(delegator.delegations.len() as u32) < T::MaxCollatorsPerDelegator::get(),
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
			state = if (state.delegators.len() as u32) > T::MaxDelegatorsPerCollator::get() {
				Self::do_update_delegator(delegation, state)?
			} else {
				state.total = state.total.saturating_add(amount);
				state
			};
			let new_total = state.total;

			// lock bond
			Self::increase_lock(&acc, delegator.total)?;
			if state.is_active() {
				Self::update_active(collator.clone(), new_total);
			}

			// Update states
			Total::<T>::mutate(|old| {
				old.delegators = old.delegators.saturating_add(amount);
			});

			// TODO: Add update of select_top_candidates
			<CollatorState<T>>::insert(&collator, state);
			<DelegatorState<T>>::insert(&acc, delegator);
			Self::deposit_event(Event::Delegation(acc, amount, collator, new_total));

			Ok(().into())
		}

		/// Leave the set of delegators and, by implication, revoke all ongoing delegations.
		///
		/// All staked funds are not unlocked immediately, but they are added to the queue of pending unstaking, and will effectively be released after `BondDuration` rounds from the moment the delegator leaves.
		///
		/// This operation reduces the total stake of the pallet as well as the stakes of all collators that were delegated, pontentially affecting their chances to be included in the set of candidates in the next rounds.
		///
		/// Emits `DelegatorLeft`.
		#[pallet::weight(0)]
		pub fn leave_delegators(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			let delegator = <DelegatorState<T>>::get(&acc).ok_or(Error::<T>::DelegatorDNE)?;
			for bond in delegator.delegations.into_iter() {
				Self::delegator_leaves_collator(acc.clone(), bond.owner.clone())?;
			}
			<DelegatorState<T>>::remove(&acc);
			Self::deposit_event(Event::DelegatorLeft(acc, delegator.total));
			Ok(().into())
		}

		/// Terminates an ongoing delegation for a given collator candidate.
		///
		/// The staked funds are not unlocked immediately, but they are added to the queue of pending unstaking, and will effectively be released after `BondDuration` rounds from the moment the delegation is terminated.
		///
		/// This operation reduces the total stake of the pallet as well as the stakes of the collator involved, pontentially affecting its chances to be included in the set of candidates in the next rounds.
		///
		/// Emits `DelegatorLeft`.
		#[pallet::weight(0)]
		pub fn revoke_delegation(origin: OriginFor<T>, collator: T::AccountId) -> DispatchResultWithPostInfo {
			Self::delegator_revokes_collator(ensure_signed(origin)?, collator)
		}

		/// Increase the stake for delegating a collator candidate.
		///
		/// If not in the set of candidates, staking enough funds allows the collator candidate to be added to it.
		///
		/// Emits `DelegationIncreased`.
		#[pallet::weight(0)]
		pub fn delegator_bond_more(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			let mut delegations = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorDNE)?;
			let mut collator = <CollatorState<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
			let delegator_total = delegations
				.inc_delegation(candidate.clone(), more)
				.ok_or(Error::<T>::DelegationDNE)?;

			// update lock
			Self::increase_lock(&delegator, delegator_total)?;
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

		/// Reduce the stake for delegating a collator candidate.
		///
		/// If the new amound of staked fund is not large enough, the collator could be removed from the set of collator candidates and not be considered for authoring the next blocks.
		///
		/// The unstaked funds are not release immediately to the account, but they will be available after `BondDuration` rounds.
		///
		/// The remaining staked funds must still be larger than the minimum required by this pallet to maintain the status of delegator.
		///
		/// The resulting total amount of funds staked must be within the allowed range as set in the pallet's configuration.
		///
		/// Emits `DelegationDecreased`.
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

			Self::prepare_withdraw(&delegator, remaining)?;

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

		/// Withdraw all previously staked funds that are now available for wihtdrawal by the origin account after `BondDuration` rounds have elapsed.
		///
		/// The withdrawn funds will be fully available to the account, minus the transaction fees, for all other operations.
		#[pallet::weight(0)]
		pub fn withdraw_unbonded(
			origin: OriginFor<T>,
			// TODO: Switch to Lookup
			// target: <T::Lookup as StaticLookup>::Source
			target: T::AccountId,
		) -> DispatchResult {
			ensure_signed(origin)?;
			// let target = T::Lookup::lookup(target)?;

			Self::do_withdraw(&target)
		}
	}

	impl<T: Config> Pallet<T> {
		/// Check whether an account is currently delegating.
		pub fn is_delegator(acc: &T::AccountId) -> bool {
			<DelegatorState<T>>::get(acc).is_some()
		}

		/// Check whether an account is currently a collator candidate.
		pub fn is_candidate(acc: &T::AccountId) -> bool {
			<CollatorState<T>>::get(acc).is_some()
		}

		/// Check whether an account is currently among the selected collator candidates for the current validation round.
		pub fn is_selected_candidate(acc: &T::AccountId) -> bool {
			<SelectedCandidates<T>>::get().binary_search(acc).is_ok()
		}

		/// Update the staking information for an active collator candidate.
		///
		/// NOTE: it is assumed that the calling context checks whether the collator candidate is currently active before calling this function.
		fn update_active(candidate: T::AccountId, total: BalanceOf<T>) {
			let mut candidates = <CandidatePool<T>>::get();
			candidates.upsert(Bond {
				owner: candidate,
				amount: total,
			});

			<CandidatePool<T>>::put(candidates);
		}

		/// Compute block production coinbase rewards based on the current inflation configuration.
		fn compute_block_issuance(
			collator_stake: BalanceOf<T>,
			delegator_stake: BalanceOf<T>,
		) -> (BalanceOf<T>, BalanceOf<T>) {
			let config = <InflationConfig<T>>::get();
			config.block_issuance::<T>(collator_stake, delegator_stake)
		}

		/// Update the delegator's state by removing the collator candidate from the set of ongoing delegations.
		fn delegator_revokes_collator(acc: T::AccountId, collator: T::AccountId) -> DispatchResultWithPostInfo {
			let mut delegator = <DelegatorState<T>>::get(&acc).ok_or(Error::<T>::DelegatorDNE)?;
			let old_total = delegator.total;
			let remaining = delegator
				.rm_delegation(collator.clone())
				.ok_or(Error::<T>::DelegationDNE)?;
			// edge case; if no delegations remaining, leave set of delegators
			if delegator.delegations.is_empty() {
				// leave the set of delegators because no delegations left
				Self::delegator_leaves_collator(acc.clone(), collator)?;
				<DelegatorState<T>>::remove(&acc);
				Self::deposit_event(Event::DelegatorLeft(acc, old_total));
				return Ok(().into());
			}
			// can never fail iff MinDelegatorStk == MinDelegation
			ensure!(remaining >= T::MinDelegatorStk::get(), Error::<T>::NomBondBelowMin);
			Self::delegator_leaves_collator(acc.clone(), collator)?;
			<DelegatorState<T>>::insert(&acc, delegator);
			Ok(().into())
		}

		/// Update the collator's state by removing the delegator's stake and starting the process to unlock the delegator's staked funds.
		///
		/// This operation affects the pallet's total stake.
		fn delegator_leaves_collator(delegator: T::AccountId, collator: T::AccountId) -> DispatchResultWithPostInfo {
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;

			let delegator_stake = state
				.delegators
				.remove_by(|nom| nom.owner.cmp(&delegator))
				.map(|nom| nom.amount)
				.ok_or(Error::<T>::DelegatorDNE)?;

			state.total = state.total.saturating_sub(delegator_stake);

			// we don't unlock immediately
			Self::prepare_withdraw(&delegator, delegator_stake)?;

			if state.is_active() {
				Self::update_active(collator.clone(), state.total);
			}
			Total::<T>::mutate(|old| {
				old.delegators = old.delegators.saturating_add(delegator_stake);
			});
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

		/// Process all the queued operations regarding collators' unstaking requests.
		///
		/// This round processes exit requests for stakes that have an amount of locked funds lower than the current round number.
		///
		/// This implies that the higher a collator's stake upon request to leave the set of candidates, the longer it will be possible for the collator to unlock the staked funds.
		// TODO: Replace with function without delay
		fn execute_delayed_collator_exits(next: RoundIndex) {
			let remain_exits = <ExitQueue<T>>::get()
				.into_iter()
				.filter_map(|x| {
					if x.amount > next {
						Some(x)
					} else {
						if let Some(state) = <CollatorState<T>>::get(&x.owner) {
							for bond in state.delegators.into_iter() {
								// return stake to delegator
								// NOTE: Since this function will be removed anyway, the below is okay for now
								Self::prepare_withdraw(&bond.owner, bond.amount).ok()?;
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
							// NOTE: Since this function will be removed anyway, the below is okay for now
							Self::prepare_withdraw(&state.id, state.bond).ok()?;

							let TotalStake {
								collators: total_collators,
								delegators: total_delegators,
							} = <Total<T>>::get();
							let total_collators = total_collators.saturating_sub(state.bond);
							// safe because bond <= total at all times
							let total_delegators = total_delegators.saturating_sub(state.total - state.bond);
							<Total<T>>::put(TotalStake {
								collators: total_collators,
								delegators: total_delegators,
							});

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

		/// Select the top `n` collators in terms of total stake (self +
		/// from delegators) from the pool of candidates to get the possibility to author blocks during
		/// the next validation round.
		///
		/// The number of candidates selected can be `n` or lower in case that are less candidates available.
		fn select_top_candidates(next: RoundIndex) -> (u32, BalanceOf<T>, BalanceOf<T>) {
			let (mut all_collators, mut total_collators, mut total_delegators) =
				(0u32, BalanceOf::<T>::zero(), BalanceOf::<T>::zero());
			log::trace!("Select collators for round {}", next);
			let mut candidates = <CandidatePool<T>>::get().to_vec();
			let top_n = <TotalSelected<T>>::get() as usize;

			log::trace!("{} Candidates for {} Collator seats", candidates.len(), top_n);

			// Order candidates by their total stake
			candidates.sort_unstable_by(|a, b| a.amount.cmp(&b.amount));

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
				all_collators = all_collators.saturating_add(1u32);
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

			log::trace!("Selected {} collators", collators.len());
			// store canonical collator set
			<SelectedCandidates<T>>::put(collators);
			(all_collators, total_collators, total_delegators)
		}

		/// Update the staking inflation information.
		fn update_inflation(inflation: InflationInfo) -> Result<(), DispatchError> {
			ensure!(inflation.is_valid(), Error::<T>::InvalidSchedule);
			Self::deposit_event(Event::RoundInflationSet(
				inflation.collator.max_rate,
				inflation.collator.reward_rate.per_block,
				inflation.delegator.max_rate,
				inflation.delegator.reward_rate.per_block,
			));
			<InflationConfig<T>>::put(inflation);
			Ok(())
		}

		/// Process the coinbase rewards for the production of a new block.
		// Weight: reads_writes(2, 2) + deposit_into_existing
		fn do_reward(who: &T::AccountId, reward: BalanceOf<T>, now: T::BlockNumber) {
			// mint
			if let Ok(imb) = T::Currency::deposit_into_existing(who, reward) {
				Self::deposit_event(Event::Rewarded(who.clone(), imb.peek()));
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
			let mut delegators: Vec<Bond<T::AccountId, BalanceOf<T>>> = state.delegators.into();
			// delegators.push(bond.clone());
			delegators.sort_by(|a, b| b.amount.cmp(&a.amount));

			// check whether bond is at last place
			match delegators.pop() {
				Some(Bond { amount, owner }) if amount < bond.amount => {
					state.total = state.total.saturating_sub(amount).saturating_add(bond.amount);
					state.delegators = OrderedSet::from_sorted_set(delegators);
					// TODO: Might want to remove this, if extrinis cannot be made transactional
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

		/// Either set or increase the BalanceLock of target account to
		/// amount.
		fn increase_lock(who: &T::AccountId, amount: BalanceOf<T>) -> Result<(), DispatchError> {
			// println!(
			// 	"who {:?} balance {:?}, amount {:?}",
			// 	who,
			// 	pallet_balances::Pallet::<T>::free_balance(who),
			// 	amount
			// );
			ensure!(
				pallet_balances::Pallet::<T>::free_balance(who) >= amount.into(),
				pallet_balances::Error::<T>::InsufficientBalance
			);

			T::Currency::set_lock(STAKING_ID, who, amount, WithdrawReasons::all());
			Ok(())
		}

		/// Set the unlocking block for the account and corresponding amount
		/// which can be withdrawn via `withdraw_unbonded` after waiting at
		/// least for `BondDuration` many rounds.
		fn prepare_withdraw(who: &T::AccountId, amount: BalanceOf<T>) -> Result<(), DispatchError> {
			ensure!(!amount.is_zero(), Error::<T>::BondDNE);

			let now = <frame_system::Pallet<T>>::block_number();
			let unlock_block = now.saturating_add(T::BondDuration::get().into());
			let mut unbonding = <Unbonding<T>>::get(who);

			ensure!(
				unbonding.len() < T::MaxUnbondRequests::get(),
				Error::<T>::NoMoreUnbonding
			);

			unbonding.insert(unlock_block, amount);
			<Unbonding<T>>::insert(who, unbonding);
			Ok(())
		}

		/// Withdraw all bonded currency which was unbonded at least
		/// `BondDuration` rounds ago.
		fn do_withdraw(who: &T::AccountId) -> Result<(), DispatchError> {
			let now = <frame_system::Pallet<T>>::block_number();
			let mut unbonding = <Unbonding<T>>::get(who);
			ensure!(!unbonding.is_empty(), Error::<T>::UnbondingIsEmpty);

			let mut total_unlocked: BalanceOf<T> = Zero::zero();
			let mut expired = Vec::new();

			// check potential unlocks
			for (block_number, locked_balance) in &unbonding {
				if block_number <= &now {
					expired.push(*block_number);
					total_unlocked = total_unlocked.saturating_add(*locked_balance);
				}
			}
			for block_number in expired {
				unbonding.remove(&block_number);
			}

			// iterate balance locks to retrieve amount of locked balance
			let locks = Locks::<T>::get(who);
			// should always find the lock
			let total_locked: BalanceOf<T> =
				if let Some(BalanceLock { amount, .. }) = locks.iter().find(|l| l.id == STAKING_ID) {
					amount.saturating_sub(total_unlocked.into()).into()
				} else {
					Zero::zero()
				};

			if total_locked.is_zero() {
				T::Currency::remove_lock(STAKING_ID, who);
				<Unbonding<T>>::remove(who);
			} else {
				T::Currency::set_lock(STAKING_ID, who, total_locked, WithdrawReasons::all());
				<Unbonding<T>>::insert(who, unbonding);
			}

			Ok(())
		}
	}

	impl<T> pallet_authorship::EventHandler<T::AccountId, T::BlockNumber> for Pallet<T>
	where
		T: Config + pallet_authorship::Config,
	{
		/// Compute coinbase rewawrds for block production and distribute it to collator's (block producer) and its delegators according to their stake.
		fn note_author(author: T::AccountId) {
			let now = <Round<T>>::get().current;
			let state = <AtStake<T>>::get(now, author.clone());
			if state.bond >= T::MinCollatorStk::get() && state.total >= T::MinCollatorStk::get() {
				let block_now: T::BlockNumber = <frame_system::Pallet<T>>::block_number();
				// TODO: Do we rather want to use a snapshot of total at the start of the round?
				// --> Keep as is for now and worry about this later
				let TotalStake {
					collators: total_collators,
					delegators: total_delegators,
				} = <Total<T>>::get();
				let (c_rewards, d_rewards) = Self::compute_block_issuance(total_collators, total_delegators);

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
						// multiplication with perbill cannot overflow
						let percent = Perquintill::from_rational(amount, delegator_stake);
						let due = percent * amt_due_delegators;
						Self::do_reward(&owner, due, block_now);
					}
				}
			}
		}
		// TODO: Does this need to be handled?
		fn note_uncle(_author: T::AccountId, _age: T::BlockNumber) {}
	}

	impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
		fn new_session(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
			log::info!(
				"assembling new collators for new session {} at #{:?}",
				new_index,
				<frame_system::Pallet<T>>::block_number(),
			);

			Self::select_top_candidates(new_index);

			frame_system::Pallet::<T>::register_extra_weight_unchecked(
				0, // TODO: T::WeightInfo::new_session(candidates_len_before as u32, removed as u32),
				DispatchClass::Mandatory,
			);

			Some(<SelectedCandidates<T>>::get())
		}

		fn end_session(_end_index: SessionIndex) {
			// we too are not caring.
		}

		fn start_session(_start_index: SessionIndex) {
			// we too are not caring.
		}
	}

	impl<T: Config> ShouldEndSession<T::BlockNumber> for Pallet<T> {
		fn should_end_session(now: T::BlockNumber) -> bool {
			<Round<T>>::get().should_update(now)
		}
	}

	impl<T: Config> EstimateNextSessionRotation<T::BlockNumber> for Pallet<T> {
		fn average_session_length() -> T::BlockNumber {
			<Round<T>>::get().length.into()
		}

		fn estimate_current_session_progress(now: T::BlockNumber) -> (Option<Percent>, Weight) {
			let round = <Round<T>>::get();
			let passed_blocks = now.saturating_sub(round.first);

			(
				Some(Percent::from_rational(passed_blocks, round.length)),
				// One read for the round info, blocknumber is read free
				T::DbWeight::get().reads(1),
			)
		}

		fn estimate_next_session_rotation(_now: T::BlockNumber) -> (Option<T::BlockNumber>, Weight) {
			let round = <Round<T>>::get();

			(
				Some(round.first + round.length),
				// One read for the round info, blocknumber is read free
				T::DbWeight::get().reads(1),
			)
		}
	}
}
