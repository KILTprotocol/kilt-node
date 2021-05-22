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
	};
	use frame_system::pallet_prelude::*;
	use pallet_balances::{BalanceLock, Locks};
	use pallet_session::ShouldEndSession;
	use sp_runtime::{
		traits::{Saturating, StaticLookup, Zero},
		Percent, Perquintill,
	};
	use sp_staking::SessionIndex;
	use sp_std::{collections::btree_map::BTreeMap, prelude::*};

	use crate::{
		set::OrderedSet,
		types::{BalanceOf, Collator, CollatorSnapshot, Delegator, RoundInfo, Stake, TotalStake},
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
		/// Default number of blocks validation rounds last, as set in the
		/// genesis configuration.
		type DefaultBlocksPerRound: Get<Self::BlockNumber>;
		/// Number of blocks for which unstaked balance will still be locked
		/// before it can be withdrawn by actively calling the extrinsic
		/// `withdraw_unstaked`.
		type StakeDuration: Get<Self::BlockNumber>;
		/// Number of rounds a collator has to stay active after submitting a
		/// request to leave the set of collator candidates.
		type ExitQueueDelay: Get<u32>;
		/// Maximum number of possible collator candidate exits per round.
		/// Requires the collators to have submitted their request to leave the
		/// set of collator candidates in advance.
		type MaxExitsPerRound: Get<usize>;
		/// Minimum number of collators selected from the set of candidates at
		/// every validation round.
		type MinSelectedCandidates: Get<u32>;
		/// Maximum number of delegators a single collator can have.
		type MaxDelegatorsPerCollator: Get<u32>;
		/// Maximum number of collators a single delegator can delegate.
		type MaxCollatorsPerDelegator: Get<u32>;
		/// Maximum size of the collator candidates set.
		type MaxCollatorCandidates: Get<u32>;
		/// Minimum stake required for any account to be elected as validator
		/// for a round.
		type MinCollatorStk: Get<BalanceOf<Self>>;
		/// Minimum stake required for any account to be added to the set of
		/// candidates.
		type MinCollatorCandidateStk: Get<BalanceOf<Self>>;
		/// Maximum stake required for any account to be added to the set of
		/// candidates.
		type MaxCollatorCandidateStk: Get<BalanceOf<Self>>;
		/// Minimum stake required for any account to be able to delegate.
		type MinDelegation: Get<BalanceOf<Self>>;
		/// Minimum stake required for any account to become a delegator.
		type MinDelegatorStk: Get<BalanceOf<Self>>;
		/// Max number of concurrent active unstaking requests before
		/// withdrawing.
		type MaxUnstakeRequests: Get<usize>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The account is not part of the delegators set.
		DelegatorNotFound,
		/// The account is not part of the collator candidates set.
		CandidateNotFound,
		/// The account is already part of the delegators set.
		DelegatorExists,
		/// The account is already part of the collator candidates set.
		CandidateExists,
		/// The account has not staked enough funds to be added to the collator
		/// candidates set.
		ValStakeBelowMin,
		/// The account has already staked the maximum amount of funds possible.
		ValStakeAboveMax,
		/// The account has not staked enough funds to become a delegator.
		NomStakeBelowMin,
		/// The account has not staked enough funds to delegate a collator
		/// candidate.
		DelegationBelowMin,
		/// The collator candidate has already trigger the process to leave the
		/// set of collator candidates.
		AlreadyLeaving,
		/// The account is already delegating the collator candidate.
		AlreadyDelegating,
		/// The account has not delegated any collator candidate yet, hence it
		/// is not in the set of delegators.
		NotYetDelegating,
		/// The collator candidate has already reached the maximum number of
		/// delegators.
		///
		/// This error is generated in cases a new delegation request does not
		/// stake enough funds to replace some other existing delegation.
		TooManyDelegators,
		/// The set of collator candidates has already reached the maximum size
		/// allowed.
		// Post-launch TODO: Update this comment when the new logic to include new collator candidates is added (by
		// using `check_collator_candidate_inclusion`).
		TooManyCollatorCandidates,
		/// The collator candidate is in the process of leaving the set of
		/// candidates and cannot perform any other actions in the meantime.
		CannotActivateIfLeaving,
		/// The delegator has already delegated the maximum number of candidates
		/// allowed.
		ExceedMaxCollatorsPerDelegator,
		/// The delegator has already previously delegated the collator
		/// candidate.
		AlreadyDelegatedCollator,
		/// The given delegation does not exist in the set of delegations.
		DelegationNotFound,
		/// The collator delegate or the delegator is trying to un-stake more
		/// funds that are currently staked.
		Underflow,
		/// The number of selected candidates or of blocks per staking round is
		/// below the minimum value allowed.
		CannotSetBelowMin,
		/// An invalid inflation configuration is trying to be set.
		InvalidSchedule,
		/// The staking reward being unlocked does not exist.
		/// Max unlocking requests reached.
		NoMoreUnstaking,
		/// Provided staked value is zero. Should never be thrown.
		StakeNotFound,
		/// Cannot withdraw when Unstaked is empty.
		UnstakingIsEmpty,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new staking round has started.
		/// \[round number, total stake of
		/// selected collators, total stake of delegators for the selected
		/// collators\]
		NewRound(T::BlockNumber, SessionIndex, BalanceOf<T>, BalanceOf<T>),
		/// A new account has joined the set of collator candidates.
		/// \[account, amount staked by the new candidate, new total stake of
		/// collator candidates\]
		JoinedCollatorCandidates(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// A collator candidate has been selected for the next validation
		/// round. \[collator's account, collator's total stake, collator's
		/// delegators' total stake\]
		CollatorChosen(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// A collator candidate has increased the amount of funds at stake.
		/// \[collator's account, previous stake, new stake\]
		CollatorStakedMore(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// A collator candidate has decreased the amount of funds at stake.
		/// \[collator's account, previous stake, new stake\]
		CollatorStakedLess(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// A collator candidate has started the process to leave the set of
		/// candidates. \[round number, collator's account, round number when
		/// the collator will be effectively removed from the set of
		/// candidates\]
		CollatorScheduledExit(SessionIndex, T::AccountId, SessionIndex),
		/// An account has left the set of collator candidates.
		/// \[account, amount of funds un-staked, new total stake of collator
		/// candidates, new total stake of delegators for the remaining
		/// collators\]
		CollatorLeft(T::AccountId, BalanceOf<T>, BalanceOf<T>, BalanceOf<T>),
		/// A delegator has increased the amount of funds at stake for a
		/// collator. \[delegator's account, collator's account, previous
		/// delegation stake, new delegation stake\]
		DelegatorStakedMore(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// A delegator has decreased the amount of funds at stake for a
		/// collator. \[delegator's account, collator's account, previous
		/// delegation stake, new delegation stake\]
		DelegatorStakedLess(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// An account has left the set of delegators.
		/// \[account, amount of funds un-staked\]
		DelegatorLeft(T::AccountId, BalanceOf<T>),
		/// An account has delegated a new collator candidate.
		/// \[account, amount of funds staked, total amount of delegators' funds
		/// staked for the collator candidate\]
		Delegation(T::AccountId, BalanceOf<T>, T::AccountId, BalanceOf<T>),
		/// A new delegation has replaced an existing one in the set of ongoing
		/// delegations for a collator candidate. \[new delegator's account,
		/// amount of funds staked in the new delegation, replaced delegator's
		/// account, amount of funds staked in the replace delegation, collator
		/// candidate's account, new total amount of delegators' funds staked
		/// for the collator candidate\]
		DelegationReplaced(
			T::AccountId,
			BalanceOf<T>,
			T::AccountId,
			BalanceOf<T>,
			T::AccountId,
			BalanceOf<T>,
		),
		/// An account has stopped delegating a collator candidate.
		/// \[account, collator candidate's account, old amount of delegators'
		/// funds staked, new amount of delegators' funds staked\]
		DelegatorLeftCollator(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// A collator or a delegator has received a reward.
		/// \[account, amount of reward\]
		Rewarded(T::AccountId, BalanceOf<T>),
		/// Inflation configuration for future validation rounds has changed.
		/// \[maximum collator's staking rate, maximum collator's reward rate,
		/// maximum delegator's staking rate, maximum delegator's reward rate\]
		RoundInflationSet(Perquintill, Perquintill, Perquintill, Perquintill),
		/// The maximum number of collator candidates selected in future
		/// validation rounds has changed. \[old value, new value\]
		MaxSelectedCandidatesSet(u32, u32),
		/// The length in blocks for future validation rounds has changed.
		/// \[round number, first block in the current round, old value, new
		/// value\]
		BlocksPerRoundSet(SessionIndex, T::BlockNumber, T::BlockNumber, T::BlockNumber),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: T::BlockNumber) -> frame_support::weights::Weight {
			let mut round = <Round<T>>::get();
			if round.should_update(n) {
				// mutate round
				round.update(n);
				// execute all delayed collator exits
				// TODO: Test live with session pallet
				// TODO: Check whether we can move this from here to an extrinsic
				Self::execute_delayed_collator_exits(round.current);

				// start next round
				<Round<T>>::put(round);

				// TODO: Check whether we want to remove the storage read if we reduce the data
				// provided by Event::NewRound
				let TotalStake {
					collators: collator_staked,
					delegators: delegator_staked,
				} = <Total<T>>::get();
				Self::deposit_event(Event::NewRound(
					round.first,
					round.current,
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
	pub(crate) type DelegatorState<T: Config> =
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
	pub(crate) type SelectedCandidates<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	/// Total funds locked by this staking pallet.
	#[pallet::storage]
	#[pallet::getter(fn total)]
	pub(crate) type Total<T: Config> = StorageValue<_, TotalStake<BalanceOf<T>>, ValueQuery>;

	/// The set of collator candidates, each with their total backing stake.
	#[pallet::storage]
	#[pallet::getter(fn candidate_pool)]
	pub(crate) type CandidatePool<T: Config> =
		StorageValue<_, OrderedSet<Stake<T::AccountId, BalanceOf<T>>>, ValueQuery>;

	/// A queue of collators waiting to be removed from the set of candidates.
	#[pallet::storage]
	#[pallet::getter(fn exit_queue)]
	pub(crate) type ExitQueue<T: Config> = StorageValue<_, OrderedSet<Stake<T::AccountId, SessionIndex>>, ValueQuery>;

	/// Snapshot of collator delegation stake.
	///
	/// NOTE: We don't care about the round index here because unstaking/staking
	/// less comes with an unstaking duration. Thus, we allow delegators to
	/// "snipe" rewards by delegating (more) to a collator which will soon
	/// author a block. This will increase the delegator's rewards but they
	/// cannot immediately withdraw their (additional) stake to the collator and
	/// have to wait for at least `StakeDuration` many blocks until they can do
	/// so. Plus, they have to pay transaction fees for both unstaking and
	/// withdrawing.
	///
	/// All in all, we don't think this can be an attack scenario
	/// due to the unstaking time and the fact that you have to actively
	/// withdraw a previously unstaked amount.
	#[pallet::storage]
	#[pallet::getter(fn at_stake)]
	pub type AtStake<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, CollatorSnapshot<T::AccountId, BalanceOf<T>>, ValueQuery>;

	/// Inflation configuration.
	#[pallet::storage]
	#[pallet::getter(fn inflation_config)]
	pub type InflationConfig<T: Config> = StorageValue<_, InflationInfo, ValueQuery>;

	/// The funds waiting to be unstaked.
	///
	/// It maps from accounts to all the funds addressed to them in the future
	/// blocks.
	#[pallet::storage]
	#[pallet::getter(fn unstaking)]
	pub type Unstaking<T: Config> =
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
					"Account does not have enough balance to stake."
				);
				if let Some(delegated_val) = opt_val {
					assert_ok!(<Pallet<T>>::join_delegators(
						T::Origin::from(Some(actor.clone()).into()),
						T::Lookup::unlookup(delegated_val.clone()),
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
			let (_, collator_staked, delegator_staked) = <Pallet<T>>::select_top_candidates();

			// Start Round 0 at Block 0

			let round: RoundInfo<T::BlockNumber> = RoundInfo::new(0u32, 0u32.into(), T::DefaultBlocksPerRound::get());
			<Round<T>>::put(round);
			// Snapshot total stake

			Pallet::<T>::deposit_event(Event::NewRound(
				T::BlockNumber::zero(),
				0u32,
				collator_staked,
				delegator_staked,
			));
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set the annual inflation rate to derive per-round inflation.
		///
		/// The inflation details are considered valid if the annual reward rate
		/// is approximately the per-block reward rate multiplied by the
		/// estimated* total number of blocks per year.
		///
		/// *The estimated average block time is six seconds.
		///
		/// The dispatch origin must be Root.
		///
		/// Emits `RoundInflationSet`.
		#[pallet::weight(100_000_000)]
		pub fn set_inflation(origin: OriginFor<T>, inflation: InflationInfo) -> DispatchResult {
			frame_system::ensure_root(origin)?;

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

		/// Set the maximum number of collator candidates that can be selected
		/// at the beginning of each validation round.
		///
		/// Changes are not applied until the start of the next round.
		///
		/// The new value must be higher than the minimum allowed as set in the
		/// pallet's configuration.
		///
		/// The dispatch origin must be Root.
		///
		/// Emits `MaxSelectedCandidatesSet`.
		#[pallet::weight(100_000_000)]
		pub fn set_max_selected_candidates(origin: OriginFor<T>, new: u32) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			ensure!(new >= T::MinSelectedCandidates::get(), Error::<T>::CannotSetBelowMin);
			let old = <TotalSelected<T>>::get();
			<TotalSelected<T>>::put(new);

			// update candidates for next round
			Self::select_top_candidates();

			Self::deposit_event(Event::MaxSelectedCandidatesSet(old, new));
			Ok(().into())
		}

		/// Set the number of blocks each validation round lasts.
		///
		/// If the new value is less than the length of the current round, the
		/// system will immediately move to the next round in the next block.
		///
		/// The new value must be higher than the minimum allowed as set in the
		/// pallet's configuration.
		///
		/// The dispatch origin must be Root.
		///
		/// Emits `BlocksPerRoundSet`.
		#[pallet::weight(100_000_000)]
		pub fn set_blocks_per_round(origin: OriginFor<T>, new: T::BlockNumber) -> DispatchResultWithPostInfo {
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
		/// In the next blocks, if the collator candidate has enough funds
		/// staked to be included in any of the top `TotalSelected` positions,
		/// it will be included in the set of potential authors that will be
		/// selected by the stake-weighted random selection function.
		///
		/// The staked funds of the new collator candidate are added to the
		/// total stake of the system.
		///
		/// The total amount of funds staked must be within the allowed range as
		/// set in the pallet's configuration.
		///
		/// The dispatch origin must not be already part of the collator
		/// candidates nor of the delegators set.
		///
		/// Emits `JoinedCollatorCandidates`.
		#[pallet::weight(100_000_000)]
		pub fn join_candidates(origin: OriginFor<T>, stake: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			ensure!(!Self::is_candidate(&acc), Error::<T>::CandidateExists);
			ensure!(!Self::is_delegator(&acc), Error::<T>::DelegatorExists);
			ensure!(stake >= T::MinCollatorCandidateStk::get(), Error::<T>::ValStakeBelowMin);
			ensure!(stake <= T::MaxCollatorCandidateStk::get(), Error::<T>::ValStakeAboveMax);

			let mut candidates = <CandidatePool<T>>::get();
			// should never fail but let's be safe
			ensure!(
				candidates.insert(Stake {
					owner: acc.clone(),
					amount: stake
				}),
				Error::<T>::CandidateExists
			);

			// Post-launch TODO: Replace with `check_collator_candidate_inclusion`.
			ensure!(
				(candidates.len() as u32) <= T::MaxCollatorCandidates::get(),
				Error::<T>::TooManyCollatorCandidates
			);
			Self::increase_lock(&acc, stake, BalanceOf::<T>::zero())?;

			let candidate = Collator::new(acc.clone(), stake);
			let TotalStake {
				collators: total_collators,
				delegators: total_delegators,
			} = <Total<T>>::get();
			let total_collators = total_collators.saturating_add(stake);
			<Total<T>>::put(TotalStake {
				collators: total_collators,
				delegators: total_delegators,
			});
			<CollatorState<T>>::insert(&acc, candidate);
			<CandidatePool<T>>::put(candidates);

			// update candidates for next round
			Self::select_top_candidates();

			Self::deposit_event(Event::JoinedCollatorCandidates(acc, stake, total_collators));
			Ok(().into())
		}

		/// Request to leave the set of collator candidates.
		///
		/// On success, the account is immediately removed from the candidate
		/// pool to prevent selection as a collator in future validation rounds,
		/// but unstaking of the funds is executed with a delay of
		/// `StakeDuration` rounds.
		///
		/// The total stake of the pallet is not affected by this operation
		/// until the funds are released after `StakeDuration` rounds.
		///
		/// NOTE: Upon starting a new session_i in `new_session`, the current
		/// top candidates are selected to be block authors for session_i+1. Any
		/// changes to the top candidates afterwards do not effect the set of
		/// authors for session_i+1.
		/// Thus, we have to make sure none of these collators can
		/// leave before session_i+1 ends by keeping them in the ExitQueue for
		/// at least 2 sessions (= 2 rounds), e.g., the current (i) and the next
		/// one (i+1).
		///
		/// Emits `CollatorScheduledExit`.
		#[pallet::weight(100_000_000)]
		pub fn leave_candidates(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateNotFound)?;
			ensure!(!state.is_leaving(), Error::<T>::AlreadyLeaving);
			let mut exits = <ExitQueue<T>>::get();
			let now = <Round<T>>::get().current;
			let when = now.saturating_add(T::ExitQueueDelay::get());
			ensure!(
				exits.insert(Stake {
					owner: collator.clone(),
					amount: when
				}),
				Error::<T>::AlreadyLeaving
			);
			state.leave_candidates(when);
			let mut candidates = <CandidatePool<T>>::get();
			if candidates.remove_by(|stake| stake.owner.cmp(&collator)).is_some() {
				<CandidatePool<T>>::put(candidates);
			}
			<ExitQueue<T>>::put(exits);
			<CollatorState<T>>::insert(&collator, state);

			// update candidates for next round
			Self::select_top_candidates();

			Self::deposit_event(Event::CollatorScheduledExit(now, collator, when));
			Ok(().into())
		}

		/// Stake more funds for a collator candidate.
		///
		/// If not in the set of candidates, staking enough funds allows the
		/// account to be added to it. The larger amount of funds, the higher
		/// chances to be selected as the author of the next block.
		///
		/// This operation affects the pallet's total stake amount.
		///
		/// The resulting total amount of funds staked must be within the
		/// allowed range as set in the pallet's configuration.
		///
		/// Emits `CollatorStakedMore`.
		#[pallet::weight(100_000_000)]
		pub fn candidate_stake_more(origin: OriginFor<T>, more: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;

			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateNotFound)?;
			ensure!(!state.is_leaving(), Error::<T>::CannotActivateIfLeaving);

			let before = state.stake;
			state.stake_more(more);
			let after = state.stake;
			ensure!(after <= T::MaxCollatorCandidateStk::get(), Error::<T>::ValStakeAboveMax);

			Self::increase_lock(&collator, after, more)?;

			if state.is_active() {
				Self::update(collator.clone(), state.total);
			}
			<CollatorState<T>>::insert(&collator, state);
			Total::<T>::mutate(|old| {
				old.collators = old.collators.saturating_add(more);
			});

			// update candidates for next round
			Self::select_top_candidates();

			Self::deposit_event(Event::CollatorStakedMore(collator, before, after));
			Ok(().into())
		}

		/// Stake less funds for a collator candidate.
		///
		/// If the new amound of staked fund is not large enough, the account
		/// could be removed from the set of collator candidates and not be
		/// considered for authoring the next blocks.
		///
		/// This operation affects the pallet's total stake amount.
		///
		/// The unstaked funds are not release immediately to the account, but
		/// they will be available after `StakeDuration` rounds.
		///
		/// The resulting total amount of funds staked must be within the
		/// allowed range as set in the pallet's configuration.
		///
		/// Emits `CollatorStakedLess`.
		#[pallet::weight(100_000_000)]
		pub fn candidate_stake_less(origin: OriginFor<T>, less: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateNotFound)?;
			ensure!(!state.is_leaving(), Error::<T>::CannotActivateIfLeaving);
			let before = state.stake;
			let after = state.stake_less(less).ok_or(Error::<T>::Underflow)?;
			ensure!(after >= T::MinCollatorCandidateStk::get(), Error::<T>::ValStakeBelowMin);

			// we don't unlock immediately
			Self::prep_unstake(&collator, less)?;

			if state.is_active() {
				Self::update(collator.clone(), state.total);
			}
			<CollatorState<T>>::insert(&collator, state);
			Total::<T>::mutate(|old| {
				old.collators = old.collators.saturating_sub(less);
			});

			// update candidates for next round
			Self::select_top_candidates();

			Self::deposit_event(Event::CollatorStakedLess(collator, before, after));
			Ok(().into())
		}

		/// Join the set of delegators by delegating to a collator candidate.
		///
		/// The account that wants to delegate cannot be part of the collator
		/// candidates set as well.
		///
		/// The caller must _not_ have delegated before. Otherwise,
		/// `delegate_another_candidate` should be called.
		///
		/// The amount staked must be larger than the minimum required to become
		/// a delegator as set in the pallet's configuration.
		///
		/// As only `MaxDelegatorsPerCollator` are allowed to delegate a given
		/// collator, the amount staked must be larger than the lowest one in
		/// the current set of delegator for the operation to be meaningful.
		///
		/// The collator's total stake as well as the pallet's total stake are
		/// increased accordingly.
		///
		/// Emits `Delegation`.
		#[pallet::weight(100_000_000)]
		pub fn join_delegators(
			origin: OriginFor<T>,
			collator: <T::Lookup as StaticLookup>::Source,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			let collator = T::Lookup::lookup(collator)?;
			// first delegation
			ensure!(<DelegatorState<T>>::get(&acc).is_none(), Error::<T>::AlreadyDelegating);
			ensure!(amount >= T::MinDelegatorStk::get(), Error::<T>::NomStakeBelowMin);
			// cannot be a collator candidate and delegator with same AccountId
			ensure!(!Self::is_candidate(&acc), Error::<T>::CandidateExists);

			// prepare update of collator state
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateNotFound)?;
			let delegation = Stake {
				owner: acc.clone(),
				amount,
			};
			// should never fail but let's be safe
			ensure!(state.delegators.insert(delegation.clone()), Error::<T>::DelegatorExists);

			// update state and potentially kick a delegator with less staked amount
			state = if (state.delegators.len() as u32) > T::MaxDelegatorsPerCollator::get() {
				let (new_state, replaced_delegation) = Self::do_update_delegator(delegation.clone(), state)?;
				Self::deposit_event(Event::DelegationReplaced(
					delegation.owner,
					delegation.amount,
					replaced_delegation.owner,
					replaced_delegation.amount,
					new_state.id.clone(),
					new_state.total,
				));
				new_state
			} else {
				state.total = state.total.saturating_add(amount);
				state
			};
			let new_total = state.total;

			// lock stake
			Self::increase_lock(&acc, amount, BalanceOf::<T>::zero())?;
			if state.is_active() {
				Self::update(collator.clone(), new_total);
			}

			// update states
			Total::<T>::mutate(|old| {
				old.delegators = old.delegators.saturating_add(amount);
			});
			<CollatorState<T>>::insert(&collator, state);
			<DelegatorState<T>>::insert(&acc, Delegator::new(collator.clone(), amount));

			// update candidates for next round
			Self::select_top_candidates();

			Self::deposit_event(Event::Delegation(acc, amount, collator, new_total));
			Ok(().into())
		}

		/// Delegate another collator's candidate by staking some funds and
		/// increasing the pallet's as well as the collator's total stake.
		///
		/// The account that wants to delegate cannot be part of the collator
		/// candidates set as well.
		///
		/// The caller _must_ have delegated before. Otherwise,
		/// `join_delegators` should be called.
		///
		/// If the delegator has already delegated the maximum number of
		/// collator candidates, this operation will fail.
		///
		/// The amount staked must be larger than the minimum required to become
		/// a delegator as set in the pallet's configuration.
		///
		/// As only `MaxDelegatorsPerCollator` are allowed to delegate a given
		/// collator, the amount staked must be larger than the lowest one in
		/// the current set of delegator for the operation to be meaningful.
		///
		/// The collator's total stake as well as the pallet's total stake are
		/// increased accordingly.
		///
		/// Emits `Delegation`.
		#[pallet::weight(100_000_000)]
		pub fn delegate_another_candidate(
			origin: OriginFor<T>,
			collator: <T::Lookup as StaticLookup>::Source,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			let collator = T::Lookup::lookup(collator)?;
			let mut delegator = <DelegatorState<T>>::get(&acc).ok_or(Error::<T>::NotYetDelegating)?;
			// delegation after first
			ensure!(amount >= T::MinDelegation::get(), Error::<T>::DelegationBelowMin);
			ensure!(
				(delegator.delegations.len() as u32) < T::MaxCollatorsPerDelegator::get(),
				Error::<T>::ExceedMaxCollatorsPerDelegator
			);

			// prepare new collator state
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateNotFound)?;
			ensure!(
				delegator.add_delegation(Stake {
					owner: collator.clone(),
					amount
				}),
				Error::<T>::AlreadyDelegatedCollator
			);
			let delegation = Stake {
				owner: acc.clone(),
				amount,
			};
			// should never fail but let's be safe
			ensure!(state.delegators.insert(delegation.clone()), Error::<T>::DelegatorExists);

			// update state and potentially kick a delegator with less staked amount
			state = if (state.delegators.len() as u32) > T::MaxDelegatorsPerCollator::get() {
				let (new_state, replaced_delegation) = Self::do_update_delegator(delegation.clone(), state)?;
				Self::deposit_event(Event::DelegationReplaced(
					delegation.owner,
					delegation.amount,
					replaced_delegation.owner,
					replaced_delegation.amount,
					new_state.id.clone(),
					new_state.total,
				));
				new_state
			} else {
				state.total = state.total.saturating_add(amount);
				state
			};
			let new_total = state.total;

			// lock stake
			Self::increase_lock(&acc, delegator.total, amount)?;
			if state.is_active() {
				Self::update(collator.clone(), new_total);
			}

			// Update states
			Total::<T>::mutate(|old| {
				old.delegators = old.delegators.saturating_add(amount);
			});
			<CollatorState<T>>::insert(&collator, state);
			<DelegatorState<T>>::insert(&acc, delegator);

			// update candidates for next round
			Self::select_top_candidates();

			Self::deposit_event(Event::Delegation(acc, amount, collator, new_total));
			Ok(().into())
		}

		/// Leave the set of delegators and, by implication, revoke all ongoing
		/// delegations.
		///
		/// All staked funds are not unlocked immediately, but they are added to
		/// the queue of pending unstaking, and will effectively be released
		/// after `StakeDuration` rounds from the moment the delegator leaves.
		///
		/// This operation reduces the total stake of the pallet as well as the
		/// stakes of all collators that were delegated, pontentially affecting
		/// their chances to be included in the set of candidates in the next
		/// rounds.
		///
		/// Emits `DelegatorLeft`.
		#[pallet::weight(100_000_000)]
		pub fn leave_delegators(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			let delegator = <DelegatorState<T>>::get(&acc).ok_or(Error::<T>::DelegatorNotFound)?;
			for stake in delegator.delegations.into_iter() {
				Self::delegator_leaves_collator(acc.clone(), stake.owner.clone())?;
			}
			<DelegatorState<T>>::remove(&acc);

			// update candidates for next round
			Self::select_top_candidates();

			Self::deposit_event(Event::DelegatorLeft(acc, delegator.total));
			Ok(().into())
		}

		/// Terminates an ongoing delegation for a given collator candidate.
		///
		/// The staked funds are not unlocked immediately, but they are added to
		/// the queue of pending unstaking, and will effectively be released
		/// after `StakeDuration` rounds from the moment the delegation is
		/// terminated.
		///
		/// This operation reduces the total stake of the pallet as well as the
		/// stakes of the collator involved, pontentially affecting its chances
		/// to be included in the set of candidates in the next rounds.
		///
		/// Emits `DelegatorLeft`.
		/// NOTE:: update candidates for next round in
		/// `delegator_revokes_collator`
		#[pallet::weight(100_000_000)]
		pub fn revoke_delegation(
			origin: OriginFor<T>,
			collator: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let collator = T::Lookup::lookup(collator)?;
			let delegator = ensure_signed(origin)?;
			Self::delegator_revokes_collator(delegator, collator)
		}

		/// Increase the stake for delegating a collator candidate.
		///
		/// If not in the set of candidates, staking enough funds allows the
		/// collator candidate to be added to it.
		///
		/// Emits `DelegatorStakedMore`.
		#[pallet::weight(100_000_000)]
		pub fn delegator_stake_more(
			origin: OriginFor<T>,
			candidate: <T::Lookup as StaticLookup>::Source,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			let candidate = T::Lookup::lookup(candidate)?;
			let mut delegations = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorNotFound)?;
			let mut collator = <CollatorState<T>>::get(&candidate).ok_or(Error::<T>::CandidateNotFound)?;
			let delegator_total = delegations
				.inc_delegation(candidate.clone(), more)
				.ok_or(Error::<T>::DelegationNotFound)?;

			// update lock
			Self::increase_lock(&delegator, delegator_total, more)?;
			let before = collator.total;
			collator.inc_delegator(delegator.clone(), more);
			let after = collator.total;

			if collator.is_active() {
				Self::update(candidate.clone(), collator.total);
			}
			Total::<T>::mutate(|old| {
				old.delegators = old.delegators.saturating_add(more);
			});
			<CollatorState<T>>::insert(&candidate, collator);
			<DelegatorState<T>>::insert(&delegator, delegations);

			// update candidates for next round
			Self::select_top_candidates();

			Self::deposit_event(Event::DelegatorStakedMore(delegator, candidate, before, after));
			Ok(().into())
		}

		/// Reduce the stake for delegating a collator candidate.
		///
		/// If the new amound of staked fund is not large enough, the collator
		/// could be removed from the set of collator candidates and not be
		/// considered for authoring the next blocks.
		///
		/// The unstaked funds are not release immediately to the account, but
		/// they will be available after `StakeDuration` rounds.
		///
		/// The remaining staked funds must still be larger than the minimum
		/// required by this pallet to maintain the status of delegator.
		///
		/// The resulting total amount of funds staked must be within the
		/// allowed range as set in the pallet's configuration.
		///
		/// Emits `DelegatorStakedLess`.
		#[pallet::weight(100_000_000)]
		pub fn delegator_stake_less(
			origin: OriginFor<T>,
			candidate: <T::Lookup as StaticLookup>::Source,
			less: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			let candidate = T::Lookup::lookup(candidate)?;
			let mut delegations = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorNotFound)?;
			let mut collator = <CollatorState<T>>::get(&candidate).ok_or(Error::<T>::CandidateNotFound)?;
			let remaining = delegations
				.dec_delegation(candidate.clone(), less)
				.ok_or(Error::<T>::DelegationNotFound)?
				.ok_or(Error::<T>::Underflow)?;

			ensure!(remaining >= T::MinDelegation::get(), Error::<T>::DelegationBelowMin);
			ensure!(
				delegations.total >= T::MinDelegatorStk::get(),
				Error::<T>::NomStakeBelowMin
			);

			Self::prep_unstake(&delegator, less)?;

			let before = collator.total;
			collator.dec_delegator(delegator.clone(), less);
			let after = collator.total;
			if collator.is_active() {
				Self::update(candidate.clone(), collator.total);
			}
			Total::<T>::mutate(|old| {
				old.delegators = old.delegators.saturating_sub(less);
			});
			<CollatorState<T>>::insert(&candidate, collator);
			<DelegatorState<T>>::insert(&delegator, delegations);

			// update candidates for next round
			Self::select_top_candidates();

			Self::deposit_event(Event::DelegatorStakedLess(delegator, candidate, before, after));
			Ok(().into())
		}

		/// Withdraw all previously staked funds that are now available for
		/// wihtdrawal by the origin account after `StakeDuration` rounds have
		/// elapsed.
		///
		/// The withdrawn funds will be fully available to the account, minus
		/// the transaction fees, for all other operations.
		#[pallet::weight(100_000_000)]
		pub fn withdraw_unstaked(origin: OriginFor<T>, target: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
			ensure_signed(origin)?;
			let target = T::Lookup::lookup(target)?;

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

		/// Check whether an account is currently among the selected collator
		/// candidates for the current validation round.
		pub fn is_selected_candidate(acc: &T::AccountId) -> bool {
			<SelectedCandidates<T>>::get().binary_search(acc).is_ok()
		}

		/// Update the staking information for an active collator candidate.
		///
		/// NOTE: it is assumed that the calling context checks whether the
		/// collator candidate is currently active before calling this function.
		fn update(candidate: T::AccountId, total: BalanceOf<T>) {
			let mut candidates = <CandidatePool<T>>::get();
			candidates.upsert(Stake {
				owner: candidate,
				amount: total,
			});

			<CandidatePool<T>>::put(candidates);
		}

		/// Compute block production coinbase rewards based on the current
		/// inflation configuration.
		fn compute_block_issuance(
			collator_stake: BalanceOf<T>,
			delegator_stake: BalanceOf<T>,
		) -> (BalanceOf<T>, BalanceOf<T>) {
			let config = <InflationConfig<T>>::get();
			config.block_issuance::<T>(collator_stake, delegator_stake)
		}

		/// Update the delegator's state by removing the collator candidate from
		/// the set of ongoing delegations.
		fn delegator_revokes_collator(acc: T::AccountId, collator: T::AccountId) -> DispatchResultWithPostInfo {
			let mut delegator = <DelegatorState<T>>::get(&acc).ok_or(Error::<T>::DelegatorNotFound)?;
			let old_total = delegator.total;
			let remaining = delegator
				.rm_delegation(collator.clone())
				.ok_or(Error::<T>::DelegationNotFound)?;
			// edge case; if no delegations remaining, leave set of delegators
			if delegator.delegations.is_empty() {
				// leave the set of delegators because no delegations left
				Self::delegator_leaves_collator(acc.clone(), collator)?;
				<DelegatorState<T>>::remove(&acc);
				Self::deposit_event(Event::DelegatorLeft(acc, old_total));
				// update candidates for next round
				Self::select_top_candidates();
				return Ok(().into());
			}
			// can never fail iff MinDelegatorStk == MinDelegation
			ensure!(remaining >= T::MinDelegatorStk::get(), Error::<T>::NomStakeBelowMin);
			Self::delegator_leaves_collator(acc.clone(), collator)?;
			<DelegatorState<T>>::insert(&acc, delegator);

			// update candidates for next round
			Self::select_top_candidates();

			Ok(().into())
		}

		/// Update the collator's state by removing the delegator's stake and
		/// starting the process to unlock the delegator's staked funds.
		///
		/// This operation affects the pallet's total stake.
		fn delegator_leaves_collator(delegator: T::AccountId, collator: T::AccountId) -> DispatchResultWithPostInfo {
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateNotFound)?;

			let delegator_stake = state
				.delegators
				.remove_by(|nom| nom.owner.cmp(&delegator))
				.map(|nom| nom.amount)
				.ok_or(Error::<T>::DelegatorNotFound)?;

			state.total = state.total.saturating_sub(delegator_stake);

			// we don't unlock immediately
			Self::prep_unstake(&delegator, delegator_stake)?;

			if state.is_active() {
				Self::update(collator.clone(), state.total);
			}
			Total::<T>::mutate(|old| {
				old.delegators = old.delegators.saturating_sub(delegator_stake);
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

		/// Process all the queued operations regarding collators' unstaking
		/// requests.
		///
		/// This round processes exit requests for stakes that have an amount of
		/// locked funds lower than the current round number.
		///
		/// This implies that the higher a collator's stake upon request to
		/// leave the set of candidates, the longer it will be possible for the
		/// collator to unlock the staked funds.
		fn execute_delayed_collator_exits(next: SessionIndex) {
			let mut maybe_exits = <ExitQueue<T>>::get().into_vec();
			let split_index = T::MaxExitsPerRound::get().min(maybe_exits.len());

			// early bail if exit queue is empty
			if split_index < 1 {
				return;
			}

			// only iterate over at most `MaxExitsPerRound` potentially leaving candidates
			// to defend against exceeding the PoV size
			let remain_exits = maybe_exits.split_off(split_index);
			maybe_exits = maybe_exits
				.into_iter()
				.filter(|x| {
					if x.amount > next {
						true
					} else {
						if let Some(state) = <CollatorState<T>>::get(&x.owner.clone()) {
							for stake in state.delegators.into_iter() {
								// prepare unstaking of delegator
								Self::prep_unstake_exit_queue(&stake.owner, stake.amount);
								// remove delegation from delegator state
								if let Some(mut delegator) = <DelegatorState<T>>::get(&stake.owner.clone()) {
									if let Some(remaining) = delegator.rm_delegation(x.owner.clone()) {
										if remaining.is_zero() {
											<DelegatorState<T>>::remove(&stake.owner);
										} else {
											<DelegatorState<T>>::insert(&stake.owner, delegator);
										}
									}
								}
							}
							// prepare unstaking of collator candidate
							Self::prep_unstake_exit_queue(&state.id, state.stake);

							let TotalStake {
								collators: total_collators,
								delegators: total_delegators,
							} = <Total<T>>::get();
							let total_collators = total_collators.saturating_sub(state.stake);
							// safe because stake <= total at all times
							let total_delegators = total_delegators.saturating_sub(state.total - state.stake);
							<Total<T>>::put(TotalStake {
								collators: total_collators,
								delegators: total_delegators,
							});

							<CollatorState<T>>::remove(&x.owner);
							Self::deposit_event(Event::CollatorLeft(
								x.owner.clone(),
								state.total,
								total_collators,
								total_delegators,
							));
						}
						false
					}
				})
				.collect::<Vec<Stake<T::AccountId, SessionIndex>>>();

			// append back the remaining exits
			maybe_exits.extend_from_slice(&remain_exits);

			<ExitQueue<T>>::put(OrderedSet::from(maybe_exits));
		}

		/// Select the top `n` collators in terms of cumulated stake (self +
		/// from delegators) from the CandidatePool to become block authors for
		/// the next round. The number of candidates selected can be `n` or
		/// lower in case that are less candidates available.
		///
		/// We do not want to execute this function in `on_initialize` or
		/// `new_session` because we could exceed the PoV size limit and brick
		/// our chain.
		/// Instead we execute this function in every extrinsic which mutates
		/// the amount at stake (collator or delegator). This will heavily
		/// increase the weight of each of these transactions it enables us to
		/// do a simple storage read to get the top candidates when a session
		/// starts in `new_session.
		fn select_top_candidates() -> (u32, BalanceOf<T>, BalanceOf<T>) {
			let (mut all_collators, mut total_collators, mut total_delegators) =
				(0u32, BalanceOf::<T>::zero(), BalanceOf::<T>::zero());
			log::trace!("Selecting collators");
			let mut candidates = <CandidatePool<T>>::get().into_vec();
			let top_n = <TotalSelected<T>>::get() as usize;

			log::trace!("{} Candidates for {} Collator seats", candidates.len(), top_n);

			// Order candidates by their total stake
			candidates.sort_by(|a, b| a.amount.cmp(&b.amount));
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
				let amount_collator = state.stake;
				let amount_delegators = state.total.saturating_sub(amount_collator);
				let exposure: CollatorSnapshot<T::AccountId, BalanceOf<T>> = state.into();
				<AtStake<T>>::insert(account, exposure);
				all_collators = all_collators.saturating_add(1u32);
				total_collators = total_collators.saturating_add(amount_collator);
				total_delegators = total_delegators.saturating_add(amount_delegators);
				Self::deposit_event(Event::CollatorChosen(
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

		/// Process the coinbase rewards for the production of a new block.
		// Weight: reads_writes(2, 2) + deposit_into_existing
		fn do_reward(who: &T::AccountId, reward: BalanceOf<T>) {
			// mint
			if let Ok(imb) = T::Currency::deposit_into_existing(who, reward) {
				Self::deposit_event(Event::Rewarded(who.clone(), imb.peek()));
			}
		}

		/// Attempts to add the stake to the set of delegators of a collator
		/// which already reached its maximum size by removing an already
		/// existing delegator with less staked value. If the given staked
		/// amount is at most the minimum staked value of the original delegator
		/// set, an error is returned.
		/// Returns the old delegation that is updated, if any.
		fn do_update_delegator(
			stake: Stake<T::AccountId, BalanceOf<T>>,
			mut state: Collator<T::AccountId, BalanceOf<T>>,
		) -> Result<(Collator<T::AccountId, BalanceOf<T>>, Stake<T::AccountId, BalanceOf<T>>), DispatchError> {
			// add stake & sort by amount
			let mut delegators: Vec<Stake<T::AccountId, BalanceOf<T>>> = state.delegators.into();
			// delegators.push(stake.clone());
			delegators.sort_by(|a, b| b.amount.cmp(&a.amount));

			// check whether stake is at last place
			match delegators.pop() {
				Some(stake_to_remove) if stake_to_remove.amount < stake.amount => {
					state.total = state
						.total
						.saturating_sub(stake_to_remove.amount)
						.saturating_add(stake.amount);
					state.delegators = OrderedSet::from_sorted_set(delegators);
					Ok((state, stake_to_remove))
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
		// 	stake: Stake<T::AccountId, BalanceOf<T>>,
		// 	mut candidates: OrderedSet<Stake<T::AccountId, BalanceOf<T>>>,
		// ) -> Result<(), DispatchError> {
		// 	todo!()
		// }

		/// Either set or increase the BalanceLock of target account to
		/// amount.
		///
		/// Consumes unstaked balance which can be withdrawn in the future up to
		/// amount and updates `Unstaking` storage accordingly.
		fn increase_lock(who: &T::AccountId, amount: BalanceOf<T>, more: BalanceOf<T>) -> Result<(), DispatchError> {
			ensure!(
				pallet_balances::Pallet::<T>::free_balance(who) >= amount.into(),
				pallet_balances::Error::<T>::InsufficientBalance
			);

			// update Unstaking by consuming up to {amount | more} and sum up balance locked
			// in unstaking in case that unstaking.sum > amount
			let mut total_locked: BalanceOf<T> = Zero::zero();
			<Unstaking<T>>::mutate(who, |unstaking| {
				// reduce {amount | more} by unstaking until either {amount | more} is zero or
				// no unstaking is left
				// if more is set, we only want to reduce by more to achieve 100 - 40 + 30 = 90
				// locked
				let mut amt_consuming_unstaking = if more.is_zero() { amount } else { more };
				for (block_number, locked_balance) in unstaking.clone() {
					// append to total_locked if amount is not reducable anymore
					if amt_consuming_unstaking.is_zero() {
						total_locked = total_locked.saturating_add(locked_balance);
					} else if locked_balance > amt_consuming_unstaking {
						// amount is only reducable by locked_balance - amt_consuming_unstaking
						let delta = locked_balance.saturating_sub(amt_consuming_unstaking);
						// replace old entry with delta
						unstaking.insert(block_number, delta);
						amt_consuming_unstaking = Zero::zero();
						total_locked = total_locked.saturating_add(locked_balance);
					} else {
						// amount is either still reducable or reached
						amt_consuming_unstaking = amt_consuming_unstaking.saturating_sub(locked_balance);
						unstaking.remove(&block_number);
					}
				}
			});

			// Handle case of collator/delegator decreasing their stake and increasing
			// afterwards which results in amount != locked
			//
			// Example: if delegator has 100 staked and decreases by 30 and then increases
			// by 20, 80 have been delegated to the collator but
			// amount = 80, more = 30, locked = 100.
			//
			// This would immediately unlock 20 for the delegator
			let amount: BalanceOf<T> = if let Some(BalanceLock { amount: locked, .. }) =
				Locks::<T>::get(who).iter().find(|l| l.id == STAKING_ID)
			{
				BalanceOf::<T>::from(*locked).max(amount)
			} else {
				amount
			};
			T::Currency::set_lock(STAKING_ID, who, amount, WithdrawReasons::all());

			Ok(())
		}

		/// Set the unlocking block for the account and corresponding amount
		/// which can be withdrawn via `withdraw_unstaked` after waiting at
		/// least for `StakeDuration` many rounds.
		///
		/// Throws if the amount is zero (unlikely) or if active unlocking
		/// requests exceed limit. The latter defends against stake reduction
		/// spamming.
		///
		/// Should never be called in `execute_delayed_exit_queue`!
		fn prep_unstake(who: &T::AccountId, amount: BalanceOf<T>) -> Result<(), DispatchError> {
			// should never occur but let's be safe
			ensure!(!amount.is_zero(), Error::<T>::StakeNotFound);

			let now = <frame_system::Pallet<T>>::block_number();
			let unlock_block = now.saturating_add(T::StakeDuration::get());
			let mut unstaking = <Unstaking<T>>::get(who);

			ensure!(
				unstaking.len() <= T::MaxUnstakeRequests::get(),
				Error::<T>::NoMoreUnstaking
			);

			// if existent, we have to add the current amount of same unlock_block, because
			// insert overwrites the current value
			let amount = amount.saturating_add(*unstaking.get(&unlock_block).unwrap_or(&BalanceOf::<T>::zero()));
			unstaking.insert(unlock_block, amount);
			<Unstaking<T>>::insert(who, unstaking);
			Ok(())
		}

		/// Prepare unstaking without checking for exceeding the unstake request
		/// limit. Same as `prep_unstake` but without checking for errors.
		///
		/// That way, we defend against a stagnating exit queue if all first
		/// `MaxExitsPerRound` candidates have reached their maximum unstake
		/// limit such that the queue would never shrink in case we executed
		/// `prep_unstake` instead of `prep_unstake_exit_queue`.
		///
		/// Should only be called in `execute_delayed_exit_queue`!
		fn prep_unstake_exit_queue(who: &T::AccountId, amount: BalanceOf<T>) {
			let now = <frame_system::Pallet<T>>::block_number();
			let unlock_block = now.saturating_add(T::StakeDuration::get());
			let mut unstaking = <Unstaking<T>>::get(who);
			unstaking.insert(unlock_block, amount);
			<Unstaking<T>>::insert(who, unstaking);
		}

		/// Withdraw all staked currency which was unstaked at least
		/// `StakeDuration` rounds ago.
		fn do_withdraw(who: &T::AccountId) -> Result<(), DispatchError> {
			let now = <frame_system::Pallet<T>>::block_number();
			let mut unstaking = <Unstaking<T>>::get(who);
			ensure!(!unstaking.is_empty(), Error::<T>::UnstakingIsEmpty);

			let mut total_unlocked: BalanceOf<T> = Zero::zero();
			let mut total_locked: BalanceOf<T> = Zero::zero();
			let mut expired = Vec::new();

			// check potential unlocks
			for (block_number, locked_balance) in &unstaking {
				if block_number <= &now {
					expired.push(*block_number);
					total_unlocked = total_unlocked.saturating_add(*locked_balance);
				} else {
					total_locked = total_locked.saturating_add(*locked_balance);
				}
			}
			for block_number in expired {
				unstaking.remove(&block_number);
			}

			// iterate balance locks to retrieve amount of locked balance
			let locks = Locks::<T>::get(who);
			total_locked = if let Some(BalanceLock { amount, .. }) = locks.iter().find(|l| l.id == STAKING_ID) {
				amount.saturating_sub(total_unlocked.into()).into()
			} else {
				// should never fail to find the lock since we checked whether unstaking is not
				// empty but let's be safe
				Zero::zero()
			};

			if total_locked.is_zero() {
				T::Currency::remove_lock(STAKING_ID, who);
				<Unstaking<T>>::remove(who);
			} else {
				T::Currency::set_lock(STAKING_ID, who, total_locked, WithdrawReasons::all());
				<Unstaking<T>>::insert(who, unstaking);
			}

			Ok(())
		}
	}

	impl<T> pallet_authorship::EventHandler<T::AccountId, T::BlockNumber> for Pallet<T>
	where
		T: Config + pallet_authorship::Config,
	{
		/// Compute coinbase rewawrds for block production and distribute it to
		/// collator's (block producer) and its delegators according to their
		/// stake.
		fn note_author(author: T::AccountId) {
			let state = <AtStake<T>>::get(author.clone());
			if state.stake >= T::MinCollatorStk::get() && state.total >= T::MinCollatorStk::get() {
				let TotalStake {
					collators: total_collators,
					delegators: total_delegators,
				} = <Total<T>>::get();
				let (c_rewards, d_rewards) = Self::compute_block_issuance(total_collators, total_delegators);

				let amt_due_collator = c_rewards;
				let delegator_stake = state.total.saturating_sub(state.stake);
				let amt_due_delegators = d_rewards;

				// Reward collator
				Self::do_reward(&author, amt_due_collator);

				// Reward delegators
				// Reward delegators due portion
				for Stake { owner, amount } in state.delegators {
					if amount >= T::MinDelegatorStk::get() {
						// Compare this delegator's stake with the total amount of
						// delegated stake for this collator
						// multiplication with perbill cannot overflow
						let percent = Perquintill::from_rational(amount, delegator_stake);
						let due = percent * amt_due_delegators;
						Self::do_reward(&owner, due);
					}
				}
			}
		}

		fn note_uncle(_author: T::AccountId, _age: T::BlockNumber) {
			// we too are not caring.
		}
	}

	impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
		/// This is where the magic happens!
		///
		/// See NOTE of `leave_candidates` for details.
		fn new_session(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
			log::info!(
				"assembling new collators for new session {} at #{:?}",
				new_index,
				<frame_system::Pallet<T>>::block_number(),
			);

			frame_system::Pallet::<T>::register_extra_weight_unchecked(
				0, // TODO: T::WeightInfo::new_session(candidates_len_before as u32, removed as u32),
				DispatchClass::Mandatory,
			);

			// get top collator candidates which are updated in any transaction which
			// affects either the stake of collators or delegators, see
			// `select_top_candidates` for details
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
			<Round<T>>::get().length
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
