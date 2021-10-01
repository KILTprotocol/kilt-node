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
//!
//! A simple staking pallet providing means of selecting a set of collators to
//! become block authors based on their total backed stake. The main difference
//! between this pallet and `frame/pallet-staking` is that this pallet uses
//! direct delegation. Delegators choose exactly who they delegate and with what
//! stake. This is different from `frame/pallet-staking` where you approval vote
//! and then run Phragmen. Moreover, this pallet rewards a collator and their
//! delegators immediately when authoring a block. Rewards are calculated
//! separately between collators and delegators.
//!
//! To join the set of candidates, an account must call `join_candidates` with
//! `MinCollatorCandidateStake` <= stake <= `MaxCollatorCandidateStake`.
//!
//! To leave the set of candidates, the collator calls `leave_candidates`. If
//! the call succeeds, the collator is removed from the pool of candidates so
//! they cannot be selected for future collator sets, but they are not unstaking
//! until executing the exit request by calling the extrinsic
//! `execute_leave_candidates` at least `ExitQueueDelay` rounds later. After
//! doing so, the collator candidate as well as their delegators are unstaked.
//! Both parties then have to wait another `StakeDuration` more blocks to be
//! able to unlock their stake.
//!
//! Candidates which requested to leave can still be in the set of authors for
//! the next round due to the design of the session pallet which at the start of
//! session s(i) chooses a set for the next session s(i+1). Thus, candidates
//! have to keep collating at least until the end of the next session (= round).
//! We extend this by delaying their execute by at least `ExitQueueDelay` many
//! sessions.
//!
//! To join the set of delegators, an account must call `join_delegators` with
//! stake >= `MinDelegatorStake`. There are also runtime methods for delegating
//! additional collators and revoking delegations.
//!
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//!
//! The KILT parachain staking pallet provides functions for:
//! - Joining the set of collator candidates of which the best
//!   `MaxSelectedCandidates` are chosen to become active collators for the next
//!   session. That makes the set of active collators the set of block authors
//!   by handing it over to the session and the authority pallet.
//! - Delegating to a collator candidate by staking for them.
//! - Increasing and reducing your stake as a collator or delegator.
//! - Revoking your delegation entirely.
//! - Requesting to leave the set of collator candidates.
//! - Withdrawing your unstaked balance after waiting for a certain number of
//!   blocks.
//!
//! ### Terminology
//!
//! - **Candidate:** A user which locks up tokens to be included into the set of
//!   authorities which author blocks and receive rewards for doing so.
//!
//! - **Collator:** A candidate that was chosen to collate this round.
//!
//! - **Delegator:** A user which locks up tokens for collators they trust. When
//!   their collator authors a block, the corresponding delegators also receive
//!   rewards.
//!
//! - **Total Stake:** A collator’s own stake + the sum of delegated stake to
//!   this collator.
//!
//! - **Total collator stake:** The sum of tokens locked for staking from all
//!   collator candidates.
//!
//! - **Total delegator stake:** The sum of tokens locked for staking from all
//!   delegators.
//!
//! - **To Stake:** Lock tokens for staking.
//!
//! - **To Unstake:** Unlock tokens from staking.
//!
//! - **Round (= Session):** A fixed number of blocks in which the set of
//!   collators does not change. We set the length of a session to the length of
//!   a staking round, thus both words are interchangeable in the context of
//!   this pallet.
//!
//! - **Lock:** A freeze on a specified amount of an account's free balance
//!   until a specified block number. Multiple locks always operate over the
//!   same funds, so they "overlay" rather than "stack"
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//! - `set_inflation` - Change the inflation configuration. Requires sudo.
//! - `set_max_selected_candidates` - Change the number of collator candidates
//!   which can be selected to be in the set of block authors. Requires sudo.
//! - `set_blocks_per_round` - Change the number of blocks of a round. Shorter
//!   rounds enable more frequent changes of the selected candidates, earlier
//!   unlockal from unstaking and earlier collator leaving. Requires sudo.
//! - `increase_max_candidate_stake_by` - Increase the maximum amount which can
//!   be staked by a collator candidate.
//! - `decrease_max_candidate_stake_by` - Decrease the maximum amount which can
//!   be staked by a collator candidate.
//! - `join_candidates` - Join the set of collator candidates by staking at
//!   least `MinCandidateStake` and at most `MaxCollatorCandidateStake`.
//! - `init_leave_candidates` - Request to leave the set of collators. Unstaking
//!   and storage clean-up is delayed until executing the exit at least
//!   ExitQueueDelay rounds later.
//! - `candidate_stake_more` - Increase your own stake as a collator candidate
//!   by the provided amount up to `MaxCollatorCandidateStake`.
//! - `candidate_stake_less` - Decrease your own stake as a collator candidate
//!   by the provided amount down to `MinCandidateStake`.
//! - `join_delegators` - Join the set of delegators by delegating to a collator
//!   candidate.
//! - `delegate_another_candidate` - Delegate to another collator candidate by
//!   staking for them.
//! - `leave_delegators` - Leave the set of delegators and revoke all
//!   delegations. Since delegators do not have to run a node and cannot be
//!   selected to become block authors, this exit is not delayed like it is for
//!   collator candidates.
//! - `revoke_delegation` - Revoke a single delegation to a collator candidate.
//! - `delegator_stake_more` - Increase your own stake as a delegator and the
//!   delegated collator candidate's total stake.
//! - `delegator_stake_less` - Decrease your own stake as a delegator and the
//!   delegated collator candidate's total stake by the provided amount down to
//!   `MinDelegatorStake`.
//! - `unlock_unstaked` - Attempt to unlock previously unstaked balance from any
//!   account. Succeeds if at least one unstake call happened at least
//!   `StakeDuration` blocks ago.
//!
//! ## Genesis config
//!
//! The KiltLaunch pallet depends on the [`GenesisConfig`].
//!
//! ## Assumptions+
//!
//! - At the start of session s(i), the set of session ids for session s(i+1)
//!   are chosen. These equal the set of selected candidates. Thus, we cannot
//!   allow collators to leave at least until the start of session s(i+2).

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
pub mod default_weights;

#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
pub(crate) mod tests;

mod inflation;
pub mod migrations;
mod set;
mod types;

use frame_support::pallet;

pub use crate::{default_weights::WeightInfo, pallet::*};
use kilt_primitives::migrations::StorageMigrator;

#[pallet]
pub mod pallet {
	use super::*;
	pub use crate::inflation::{InflationInfo, RewardRate, StakingInfo};

	use frame_support::{
		assert_ok,
		pallet_prelude::*,
		storage::bounded_btree_map::BoundedBTreeMap,
		traits::{
			Currency, EstimateNextSessionRotation, Get, Imbalance, LockIdentifier, LockableCurrency,
			ReservableCurrency, WithdrawReasons,
		},
		BoundedVec,
	};
	use frame_system::pallet_prelude::*;
	use kilt_primitives::constants::BLOCKS_PER_YEAR;
	use pallet_balances::{BalanceLock, Locks};
	use pallet_session::ShouldEndSession;
	use sp_runtime::{
		traits::{Convert, One, SaturatedConversion, Saturating, StaticLookup, Zero},
		Permill, Perquintill,
	};
	use sp_staking::SessionIndex;
	use sp_std::prelude::*;

	use crate::{
		migrations::StakingStorageVersion,
		set::OrderedSet,
		types::{
			BalanceOf, Candidate, CandidateOf, CandidateStatus, DelegationCounter, Delegator, RoundInfo, Stake,
			StakeOf, TotalStake,
		},
	};
	use sp_std::{convert::TryInto, fmt::Debug};

	/// Kilt-specific lock for staking rewards.
	pub(crate) const STAKING_ID: LockIdentifier = *b"kiltpstk";

	/// Pallet for parachain staking.
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_balances::Config + pallet_session::Config {
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
			+ From<u128>
			+ Into<<Self as pallet_balances::Config>::Balance>
			+ From<<Self as pallet_balances::Config>::Balance>;

		/// Minimum number of blocks validation rounds can last.
		#[pallet::constant]
		type MinBlocksPerRound: Get<Self::BlockNumber>;

		/// Default number of blocks validation rounds last, as set in the
		/// genesis configuration.
		#[pallet::constant]
		type DefaultBlocksPerRound: Get<Self::BlockNumber>;
		/// Number of blocks for which unstaked balance will still be locked
		/// before it can be unlocked by actively calling the extrinsic
		/// `unlock_unstaked`.
		#[pallet::constant]
		type StakeDuration: Get<Self::BlockNumber>;
		/// Number of rounds a collator has to stay active after submitting a
		/// request to leave the set of collator candidates.
		#[pallet::constant]
		type ExitQueueDelay: Get<u32>;

		/// Minimum number of collators selected from the set of candidates at
		/// every validation round.
		#[pallet::constant]
		type MinCollators: Get<u32>;

		/// Minimum number of collators which cannot leave the network if there
		/// are no others.
		#[pallet::constant]
		type MinRequiredCollators: Get<u32>;

		/// Maximum number of delegations which can be made within the same
		/// round.
		///
		/// NOTE: To prevent re-delegation-reward attacks, we should keep this
		/// to be one.
		#[pallet::constant]
		type MaxDelegationsPerRound: Get<u32>;

		/// Maximum number of delegators a single collator can have.
		#[pallet::constant]
		type MaxDelegatorsPerCollator: Get<u32> + Debug + PartialEq;

		/// Maximum number of collators a single delegator can delegate.
		#[pallet::constant]
		type MaxCollatorsPerDelegator: Get<u32> + Debug + PartialEq;

		/// Maximum size of the top candidates set.
		#[pallet::constant]
		type MaxTopCandidates: Get<u32> + Debug + PartialEq;

		/// Minimum stake required for any account to be elected as validator
		/// for a round.
		#[pallet::constant]
		type MinCollatorStake: Get<BalanceOf<Self>>;

		/// Minimum stake required for any account to be added to the set of
		/// candidates.
		#[pallet::constant]
		type MinCollatorCandidateStake: Get<BalanceOf<Self>>;

		/// Minimum stake required for any account to be able to delegate.
		#[pallet::constant]
		type MinDelegation: Get<BalanceOf<Self>>;

		/// Minimum stake required for any account to become a delegator.
		#[pallet::constant]
		type MinDelegatorStake: Get<BalanceOf<Self>>;

		/// Max number of concurrent active unstaking requests before
		/// unlocking.
		///
		/// NOTE: To protect against irremovability of a candidate or delegator,
		/// we only allow for MaxUnstakeRequests - 1 many manual unstake
		/// requests. The last one serves as a placeholder for the cases of
		/// calling either `kick_delegator`, force_remove_candidate` or
		/// `execute_leave_candidates`. Otherwise, a user could max out their
		/// unstake requests and prevent themselves from being kicked from the
		/// set of candidates/delegators until they unlock their funds.
		#[pallet::constant]
		type MaxUnstakeRequests: Get<u32>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
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
		/// The account tried to stake more or less with amount zero.
		ValStakeZero,
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
		/// The collator candidate wanted to execute the exit but has not
		/// requested to leave before by calling `init_leave_candidates`.
		NotLeaving,
		/// The collator tried to leave before waiting at least for
		/// `ExitQueueDelay` many rounds.
		CannotLeaveYet,
		/// The account has a full list of unstaking requests and needs to
		/// unlock at least one of these before being able to join (again).
		/// NOTE: Can only happen if the account was a candidate or
		/// delegator before and either got kicked or exited voluntarily.
		CannotJoinBeforeUnlocking,
		/// The account is already delegating the collator candidate.
		AlreadyDelegating,
		/// The account has not delegated any collator candidate yet, hence it
		/// is not in the set of delegators.
		NotYetDelegating,
		/// The delegator has exceeded the number of delegations per round which
		/// is equal to MaxDelegatorsPerCollator.
		///
		/// This protects against attacks in which a delegator can re-delegate
		/// from a collator who has already authored a block, to another one
		/// which has not in this round.
		DelegationsPerRoundExceeded,
		/// The collator candidate has already reached the maximum number of
		/// delegators.
		///
		/// This error is generated in case a new delegation request does not
		/// stake enough funds to replace some other existing delegation.
		TooManyDelegators,
		/// The set of collator candidates would fall below the required minimum
		/// if the collator left.
		TooFewCollatorCandidates,
		/// The collator candidate is in the process of leaving the set of
		/// candidates and cannot perform any other actions in the meantime.
		CannotStakeIfLeaving,
		/// The collator candidate is in the process of leaving the set of
		/// candidates and thus cannot be delegated to.
		CannotDelegateIfLeaving,
		/// The delegator has already delegated the maximum number of candidates
		/// allowed.
		MaxCollatorsPerDelegatorExceeded,
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
		/// Cannot unlock when Unstaked is empty.
		UnstakingIsEmpty,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new staking round has started.
		/// \[block number, round number\]
		NewRound(T::BlockNumber, SessionIndex),
		/// A new account has joined the set of top candidates.
		/// \[account\]
		EnteredTopCandidates(T::AccountId),
		/// An account was removed from the set of top candidates.
		/// \[account\]
		LeftTopCandidates(T::AccountId),
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
		/// A collator candidate has canceled the process to leave the set of
		/// candidates and was added back to the candidate pool. \[collator's
		/// account\]
		CollatorCanceledExit(T::AccountId),
		/// An account has left the set of collator candidates.
		/// \[account, amount of funds un-staked\]
		CollatorLeft(T::AccountId, BalanceOf<T>),
		/// An account was forcedly removed from the  set of collator
		/// candidates. \[account, amount of funds un-staked\]
		CollatorRemoved(T::AccountId, BalanceOf<T>),
		/// The maximum candidate stake has been changed.
		/// \[new max amount\]
		MaxCandidateStakeChanged(BalanceOf<T>),
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
		fn on_initialize(now: T::BlockNumber) -> frame_support::weights::Weight {
			let mut post_weight = <T as Config>::WeightInfo::on_initialize_no_action();
			let mut round = <Round<T>>::get();

			// check for round update
			if round.should_update(now) {
				// mutate round
				round.update(now);

				// start next round
				<Round<T>>::put(round);

				Self::deposit_event(Event::NewRound(round.first, round.current));
				post_weight = <T as Config>::WeightInfo::on_initialize_round_update();
			}
			// check for InflationInfo update
			if now > BLOCKS_PER_YEAR.saturated_into::<T::BlockNumber>() {
				post_weight = post_weight.saturating_add(Self::adjust_reward_rates(now));
			}
			post_weight
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<(), &'static str> {
			StorageMigrator::<StakingStorageVersion, T>::pre_migrate(StorageVersion::<T>::get())
		}

		fn on_runtime_upgrade() -> Weight {
			let migration_weight = StorageMigrator::<StakingStorageVersion, T>::migrate(StorageVersion::<T>::get());
			// Add one read to get the current storage version
			migration_weight.saturating_add(T::DbWeight::get().reads(1))
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade() -> Result<(), &'static str> {
			StorageMigrator::<StakingStorageVersion, T>::post_migrate(StorageVersion::<T>::get())
		}
	}

	/// True if network has been upgraded to this version.
	/// Storage version of the pallet.
	///
	/// This is set to v4 for new networks.
	#[pallet::storage]
	pub(crate) type StorageVersion<T: Config> = StorageValue<_, StakingStorageVersion, ValueQuery>;

	/// The maximum number of collator candidates selected at each round.
	#[pallet::storage]
	#[pallet::getter(fn max_selected_candidates)]
	pub(crate) type MaxSelectedCandidates<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Current round number and next round scheduled transition.
	#[pallet::storage]
	#[pallet::getter(fn round)]
	pub(crate) type Round<T: Config> = StorageValue<_, RoundInfo<T::BlockNumber>, ValueQuery>;

	/// Delegation information for the latest session in which a delegator
	/// delegated.
	///
	/// It maps from an account to the number of delegations in the last
	/// session in which they (re-)delegated.
	#[pallet::storage]
	#[pallet::getter(fn last_delegation)]
	pub(crate) type LastDelegation<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, DelegationCounter, ValueQuery>;

	/// Delegation staking information.
	///
	/// It maps from an account to its delegation details.
	#[pallet::storage]
	#[pallet::getter(fn delegator_state)]
	pub(crate) type DelegatorState<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Delegator<T::AccountId, BalanceOf<T>, T::MaxCollatorsPerDelegator>,
		OptionQuery,
	>;

	/// The staking information for a candidate.
	///
	/// It maps from an account to its information.
	#[pallet::storage]
	#[pallet::getter(fn candidate_pool)]
	pub(crate) type CandidatePool<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Candidate<T::AccountId, BalanceOf<T>, T::MaxDelegatorsPerCollator>,
		OptionQuery,
	>;

	/// The number of candidates in the pool.
	#[pallet::storage]
	#[pallet::getter(fn candidate_count)]
	pub(crate) type CandidateCount<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Total funds locked to back the currently selected collators.
	/// The sum of all collator and their delegator stakes.
	///
	/// Note: There are more funds locked by this pallet, since the backing for
	/// non collating candidates is not included in [TotalCollatorStake].
	#[pallet::storage]
	#[pallet::getter(fn total_collator_stake)]
	pub(crate) type TotalCollatorStake<T: Config> = StorageValue<_, TotalStake<BalanceOf<T>>, ValueQuery>;

	/// The collator candidates with the highest amount of stake.
	///
	/// Each time the stake of a collator is increased, it is checked whether is
	/// pushes another candidate out of the list. When the stake is reduced, it
	/// is not checked of another candidate has more stake, since this would
	/// require the iterating over the [CandidatePool].
	///
	/// There must always be more candidates than [MaxSelectedCandidates] so
	/// that a collator can drop out of the collator set by reducing his stake.
	#[pallet::storage]
	#[pallet::getter(fn top_candidates)]
	pub(crate) type TopCandidates<T: Config> =
		StorageValue<_, OrderedSet<Stake<T::AccountId, BalanceOf<T>>, T::MaxTopCandidates>, ValueQuery>;

	/// Inflation configuration.
	#[pallet::storage]
	#[pallet::getter(fn inflation_config)]
	pub(crate) type InflationConfig<T: Config> = StorageValue<_, InflationInfo, ValueQuery>;

	/// The funds waiting to be unstaked.
	///
	/// It maps from accounts to all the funds addressed to them in the future
	/// blocks.
	#[pallet::storage]
	#[pallet::getter(fn unstaking)]
	pub(crate) type Unstaking<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		BoundedBTreeMap<T::BlockNumber, BalanceOf<T>, T::MaxUnstakeRequests>,
		ValueQuery,
	>;

	/// The maximum amount a collator candidate can stake.
	#[pallet::storage]
	#[pallet::getter(fn max_candidate_stake)]
	pub(crate) type MaxCollatorCandidateStake<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// The year in which the last automatic reduction of the reward rates
	/// occurred.
	///
	/// It starts at zero at genesis and increments by one every BLOCKS_PER_YEAR
	/// many blocks.
	#[pallet::storage]
	#[pallet::getter(fn last_reward_reduction)]
	pub(crate) type LastRewardReduction<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	pub type GenesisStaker<T> = Vec<(
		<T as frame_system::Config>::AccountId,
		Option<<T as frame_system::Config>::AccountId>,
		BalanceOf<T>,
	)>;

	#[pallet::storage]
	#[pallet::getter(fn new_round_forced)]
	pub(crate) type ForceNewRound<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub stakers: GenesisStaker<T>,
		pub inflation_config: InflationInfo,
		pub max_candidate_stake: BalanceOf<T>,
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
			<MaxCollatorCandidateStake<T>>::put(self.max_candidate_stake);

			// Setup delegate & collators
			for &(ref actor, ref opt_val, balance) in &self.stakers {
				assert!(
					T::Currency::free_balance(actor) >= balance,
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
			<MaxSelectedCandidates<T>>::put(T::MinCollators::get());

			// Choose top MaxSelectedCandidates collator candidates
			<Pallet<T>>::update_total_stake();

			// Start Round 0 at Block 0
			let round: RoundInfo<T::BlockNumber> = RoundInfo::new(0u32, 0u32.into(), T::DefaultBlocksPerRound::get());
			<Round<T>>::put(round);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Forces the start of the new round in the next block.
		///
		/// The new round will be enforced via <T as
		/// ShouldEndSession<_>>::should_end_session.
		///
		/// The dispatch origin must be Root.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: [Origin Account]
		/// - Writes: ForceNewRound
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_inflation())]
		pub fn force_new_round(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;

			// set force_new_round handle which, at the start of the next block, will
			// trigger `should_end_session` in `Session::on_initialize` and update the
			// current round
			<ForceNewRound<T>>::put(true);

			Ok(())
		}

		/// Set the annual inflation rate to derive per-round inflation.
		///
		/// The inflation details are considered valid if the annual reward rate
		/// is approximately the per-block reward rate multiplied by the
		/// estimated* total number of blocks per year.
		///
		/// The estimated average block time is twelve seconds.
		///
		/// The dispatch origin must be Root.
		///
		/// Emits `RoundInflationSet`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: [Origin Account]
		/// - Writes: InflationConfig
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_inflation())]
		pub fn set_inflation(
			origin: OriginFor<T>,
			collator_max_rate_percentage: Perquintill,
			collator_annual_reward_rate_percentage: Perquintill,
			delegator_max_rate_percentage: Perquintill,
			delegator_annual_reward_rate_percentage: Perquintill,
		) -> DispatchResult {
			ensure_root(origin)?;

			let inflation = InflationInfo::new(
				collator_max_rate_percentage,
				collator_annual_reward_rate_percentage,
				delegator_max_rate_percentage,
				delegator_annual_reward_rate_percentage,
			);

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
		///
		///
		/// # <weight>
		/// - The transaction's complexity is mainly dependent on updating the
		///   `SelectedCandidates` storage in `select_top_candidates` which in
		///   return depends on the number of `MaxSelectedCandidates` (N).
		/// - For each N, we read `CollatorState` from the storage.
		/// ---------
		/// Weight: O(N) where N is `MaxSelectedCandidates` bounded by
		/// `MaxTopCandidates`
		/// - Reads: MaxSelectedCandidates, TopCandidates, N * CollatorState
		/// - Writes: MaxSelectedCandidates
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_max_selected_candidates(*new, (*new).saturating_mul(T::MaxDelegatorsPerCollator::get())))]
		pub fn set_max_selected_candidates(origin: OriginFor<T>, new: u32) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			ensure!(new >= T::MinCollators::get(), Error::<T>::CannotSetBelowMin);
			let old = <MaxSelectedCandidates<T>>::get();
			<MaxSelectedCandidates<T>>::put(new);

			// update candidates for next round
			let (num_collators, num_delegators, _, _) = Self::update_total_stake();

			Self::deposit_event(Event::MaxSelectedCandidatesSet(old, new));

			Ok(Some(<T as pallet::Config>::WeightInfo::set_max_selected_candidates(
				num_collators,
				num_delegators,
			))
			.into())
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
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: [Origin Account], Round
		/// - Writes: Round
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_blocks_per_round())]
		pub fn set_blocks_per_round(origin: OriginFor<T>, new: T::BlockNumber) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(new >= T::MinBlocksPerRound::get(), Error::<T>::CannotSetBelowMin);

			let old_round = <Round<T>>::get();
			<Round<T>>::put(RoundInfo {
				length: new,
				..old_round
			});

			Self::deposit_event(Event::BlocksPerRoundSet(
				old_round.current,
				old_round.first,
				old_round.length,
				new,
			));
			Ok(())
		}

		/// Set the maximal amount a collator can stake. Existing stakes are not
		/// changed.
		///
		/// The dispatch origin must be Root.
		///
		/// Emits `MaxCandidateStakeChanged`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: [Origin Account], MaxCollatorCandidateStake
		/// - Writes: Round
		/// # </weight>
		#[pallet::weight(<T as Config>::WeightInfo::set_max_candidate_stake())]
		pub fn set_max_candidate_stake(origin: OriginFor<T>, new: BalanceOf<T>) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(
				new >= T::MinCollatorCandidateStake::get(),
				Error::<T>::CannotSetBelowMin
			);

			MaxCollatorCandidateStake::<T>::put(new);

			Self::deposit_event(Event::MaxCandidateStakeChanged(new));
			Ok(())
		}

		/// Forcedly removes a collator candidate from the TopCandidates and
		/// clears all associated storage for the candidate and their
		/// delegators.
		///
		/// Prepares unstaking of the candidates and their delegators stake
		/// which can be unlocked via `unlock_unstaked` after waiting at
		/// least `StakeDuration` many blocks.
		///
		/// Emits `CandidateRemoved`.
		///
		/// # <weight>
		/// - The transaction's complexity is mainly dependent on updating the
		///   `SelectedCandidates` storage in `select_top_candidates` which in
		///   return depends on the number of `MaxSelectedCandidates` (N).
		/// - For each N, we read `CollatorState` from the storage.
		/// ---------
		/// Weight: O(N + D) where N is `MaxSelectedCandidates` bounded by
		/// `MaxTopCandidates` and D is the number of delegators of the
		/// collator candidate bounded by `MaxDelegatorsPerCollator`
		/// - Reads: MaxCollatorCandidateStake, 2 * N * CollatorState,
		///   TopCandidates, BlockNumber, D * DelegatorState, D * Unstaking
		/// - Writes: MaxCollatorCandidateStake, N * CollatorState,
		///   SelectedCandidates, D * DelegatorState, (D + 1) * Unstaking
		/// - Kills: CollatorState, DelegatorState for all delegators which only
		///   delegated to the candidate
		/// # </weight>
		#[pallet::weight(<T as Config>::WeightInfo::force_remove_candidate(T::MaxTopCandidates::get(), T::MaxTopCandidates::get().saturating_mul(T::MaxDelegatorsPerCollator::get())))]
		pub fn force_remove_candidate(
			origin: OriginFor<T>,
			collator: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			let collator = T::Lookup::lookup(collator)?;
			let state = <CandidatePool<T>>::get(&collator).ok_or(Error::<T>::CandidateNotFound)?;
			let total_amount = state.total;

			let mut candidates = <TopCandidates<T>>::get();
			ensure!(
				candidates.len().saturated_into::<u32>() > T::MinRequiredCollators::get(),
				Error::<T>::TooFewCollatorCandidates
			);

			Self::remove_candidate(&collator, &state)?;

			if candidates
				.remove(&Stake {
					owner: collator.clone(),
					amount: state.total,
				})
				.is_some()
			{
				TopCandidates::<T>::put(candidates);
			}

			// update candidates for next round
			let (num_collators, num_delegators, _, _) = Self::update_total_stake();

			Self::deposit_event(Event::CollatorRemoved(collator, total_amount));

			Ok(Some(<T as Config>::WeightInfo::force_remove_candidate(
				num_collators,
				num_delegators,
			))
			.into())
		}

		/// Join the set of collator candidates.
		///
		/// In the next blocks, if the collator candidate has enough funds
		/// staked to be included in any of the top `MaxSelectedCandidates`
		/// positions, it will be included in the set of potential authors that
		/// will be selected by the stake-weighted random selection function.
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
		///
		/// # <weight>
		/// - The transaction's complexity is mainly dependent on updating the
		///   `SelectedCandidates` storage in `select_top_candidates` which in
		///   return depends on the number of `MaxSelectedCandidates` (N).
		/// - For each N, we read `CollatorState` from the storage.
		/// ---------
		/// Weight: O(N) + O(C) where N is `MaxSelectedCandidates` bounded by
		/// `MaxTopCandidates` and C the size of the TopCandidates (bounded
		/// by MaxTopCandidates)
		/// - Reads: [Origin Account], DelegatorState,
		///   MaxCollatorCandidateStake, Locks, TotalStake, TopCandidates,
		///   MaxSelectedCandidates, (N + 1) * CollatorState, CandidateCount
		/// - Writes: Locks, TotalStake, CollatorState, TopCandidates,
		///   SelectedCandidates, CandidateCount
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::join_candidates(T::MaxTopCandidates::get(), T::MaxTopCandidates::get().saturating_mul(T::MaxDelegatorsPerCollator::get())))]
		pub fn join_candidates(origin: OriginFor<T>, stake: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			if let Some(is_active_candidate) = Self::is_active_candidate(&sender) {
				ensure!(is_active_candidate, Error::<T>::AlreadyLeaving);
				ensure!(!is_active_candidate, Error::<T>::CandidateExists);
			}
			ensure!(!Self::is_delegator(&sender), Error::<T>::DelegatorExists);
			ensure!(
				stake >= T::MinCollatorCandidateStake::get(),
				Error::<T>::ValStakeBelowMin
			);
			ensure!(
				stake <= <MaxCollatorCandidateStake<T>>::get(),
				Error::<T>::ValStakeAboveMax
			);
			ensure!(
				Unstaking::<T>::get(&sender).len().saturated_into::<u32>() < T::MaxUnstakeRequests::get(),
				Error::<T>::CannotJoinBeforeUnlocking
			);

			Self::increase_lock(&sender, stake, BalanceOf::<T>::zero())?;

			let candidate = Candidate::new(sender.clone(), stake);
			CandidatePool::<T>::insert(&sender, candidate);
			Self::update_top_candidates(sender.clone(), BalanceOf::<T>::zero(), stake);

			CandidateCount::<T>::mutate(|count| {
				*count = count.saturating_add(1);
			});

			// update candidates for next round
			let (num_collators, num_delegators, total_collators, _) = Self::update_total_stake();

			Self::deposit_event(Event::JoinedCollatorCandidates(sender, stake, total_collators));
			Ok(Some(<T as pallet::Config>::WeightInfo::join_candidates(
				num_collators,
				num_delegators,
			))
			.into())
		}

		/// Request to leave the set of collator candidates.
		///
		/// On success, the account is immediately removed from the candidate
		/// pool to prevent selection as a collator in future validation rounds,
		/// but unstaking of the funds is executed with a delay of
		/// `StakeDuration` blocks.
		///
		/// The exit request can be reversed by calling
		/// `cancel_leave_candidates`.
		///
		/// The total stake of the pallet is not affected by this operation
		/// until the funds are released after `StakeDuration` blocks.
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
		///
		/// # <weight>
		/// - The transaction's complexity is mainly dependent on updating the
		///   `SelectedCandidates` storage in `select_top_candidates` which in
		///   return depends on the number of `MaxSelectedCandidates` (N).
		/// - For each N, we read `CollatorState` from the storage.
		/// ---------
		/// Weight: O(N) where N is `MaxSelectedCandidates` bounded by
		/// `MaxTopCandidates`
		/// - Reads: [Origin Account], TopCandidates, ExitQueue, (N + 1) *
		///   CollatorState * N
		/// - Writes: CollatorState, TopCandidates, ExitQueue,
		///   SelectedCandidates
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::init_leave_candidates(
			T::MaxTopCandidates::get(),
			T::MaxTopCandidates::get().saturating_mul(T::MaxDelegatorsPerCollator::get())
		))]
		pub fn init_leave_candidates(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidatePool<T>>::get(&collator).ok_or(Error::<T>::CandidateNotFound)?;
			ensure!(!state.is_leaving(), Error::<T>::AlreadyLeaving);
			let mut candidates = <TopCandidates<T>>::get();
			ensure!(
				candidates.len().saturated_into::<u32>() > T::MinRequiredCollators::get(),
				Error::<T>::TooFewCollatorCandidates
			);

			let now = <Round<T>>::get().current;
			let when = now.saturating_add(T::ExitQueueDelay::get());
			state.leave_candidates(when);
			if candidates
				.remove(&Stake {
					owner: collator.clone(),
					amount: state.total,
				})
				.is_some()
			{
				<TopCandidates<T>>::put(candidates);
				Self::deposit_event(Event::LeftTopCandidates(collator.clone()))
			}
			<CandidatePool<T>>::insert(&collator, state);

			// update candidates for next round
			let (num_collators, num_delegators, _, _) = Self::update_total_stake();

			Self::deposit_event(Event::CollatorScheduledExit(now, collator, when));
			Ok(Some(<T as pallet::Config>::WeightInfo::init_leave_candidates(
				num_collators,
				num_delegators,
			))
			.into())
		}

		/// Execute the network exit of a candidate who requested to leave at
		/// least `ExitQueueDelay` rounds ago. Prepares unstaking of the
		/// candidates and their delegators stake which can be unlocked via
		/// `unlock_unstaked` after waiting at least `StakeDuration` many
		/// blocks.
		///
		/// Requires the candidate to previously have called
		/// `init_leave_candidates`.
		///
		/// The exit request can be reversed by calling
		/// `cancel_leave_candidates`.
		///
		/// Emits `CollatorLeft`.
		///
		/// # <weight>
		/// Weight: O(D) where D is the number of delegators of the collator
		/// candidate bounded by `MaxDelegatorsPerCollator`
		/// - Reads: CollatorState, Round, D * DelegatorState, D
		///   * BlockNumber, D * Unstaking
		/// - Writes: D * Unstaking, D * DelegatorState, Total
		/// - Kills: CollatorState, DelegatorState
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::execute_leave_candidates(
			T::MaxTopCandidates::get(),
			T::MaxDelegatorsPerCollator::get(),
			T::MaxUnstakeRequests::get()
		))]
		pub fn execute_leave_candidates(
			origin: OriginFor<T>,
			collator: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			let collator = T::Lookup::lookup(collator)?;
			let state = <CandidatePool<T>>::get(&collator).ok_or(Error::<T>::CandidateNotFound)?;
			ensure!(state.is_leaving(), Error::<T>::NotLeaving);
			ensure!(state.can_exit(<Round<T>>::get().current), Error::<T>::CannotLeaveYet);

			let num_delegators = state.delegators.len().saturated_into::<u32>();
			let total_amount = state.total;
			Self::remove_candidate(&collator, &state)?;

			Self::deposit_event(Event::CollatorLeft(collator, total_amount));

			Ok(Some(<T as pallet::Config>::WeightInfo::execute_leave_candidates(
				T::MaxTopCandidates::get(),
				num_delegators,
				T::MaxUnstakeRequests::get(),
			))
			.into())
		}

		/// Revert the previously requested exit of the network of a collator
		/// candidate. On success, adds back the candidate to the TopCandidates
		/// and updates the SelectedCandidates.
		///
		/// Requires the candidate to previously have called
		/// `init_leave_candidates`.
		///
		/// Emits `CollatorCanceledExit`.
		///
		/// # <weight>
		/// - The transaction's complexity is mainly dependent on updating the
		///   `SelectedCandidates` storage in `select_top_candidates` which in
		///   return depends on the number of `MaxSelectedCandidates` (N).
		/// - For each N, we read `CollatorState` from the storage.
		/// ---------
		/// Weight: O(N) + O(C) where N is `MaxSelectedCandidates` bounded by
		/// `MaxTopCandidates` and C the size of the TopCandidates (bounded
		/// by MaxTopCandidates)
		/// - Reads: [Origin Account], Total, TopCandidates,
		///   MaxSelectedCandidates, (N + 1) * CollatorState
		/// - Writes: Total, CollatorState, TopCandidates, SelectedCandidates
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::cancel_leave_candidates(
			T::MaxTopCandidates::get(),
			T::MaxDelegatorsPerCollator::get(),
		))]
		pub fn cancel_leave_candidates(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let candidate = ensure_signed(origin)?;
			let mut state = <CandidatePool<T>>::get(&candidate).ok_or(Error::<T>::CandidateNotFound)?;
			ensure!(state.is_leaving(), Error::<T>::NotLeaving);

			// revert leaving state
			state.revert_leaving();

			Self::update_top_candidates(candidate.clone(), state.total, state.total);

			// update candidates for next round
			<CandidatePool<T>>::insert(&candidate, state);
			let (num_collators, num_delegators, _, _) = Self::update_total_stake();

			Self::deposit_event(Event::CollatorCanceledExit(candidate));
			Ok(Some(<T as pallet::Config>::WeightInfo::cancel_leave_candidates(
				num_collators,
				num_delegators,
			))
			.into())
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
		///
		/// # <weight>
		/// - The transaction's complexity is mainly dependent on updating the
		///   `SelectedCandidates` storage in `select_top_candidates` which in
		///   return depends on the number of `MaxSelectedCandidates` (N).
		/// - For each N, we read `CollatorState` from the storage.
		/// ---------
		/// Weight: O(N) where N is `MaxSelectedCandidates` bounded by
		/// `MaxTopCandidates`
		/// - Reads: [Origin Account], Locks, TotalStake,
		///   MaxCollatorCandidateStake, TopCandidates, (N + 1)
		///   * CollatorState
		/// - Writes: Locks, TotalStake, CollatorState, TopCandidates,
		///   SelectedCandidates
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::candidate_stake_more(T::MaxTopCandidates::get(), T::MaxTopCandidates::get().saturating_mul(T::MaxDelegatorsPerCollator::get()), T::MaxUnstakeRequests::get().saturated_into::<u32>()))]
		pub fn candidate_stake_more(origin: OriginFor<T>, more: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;

			ensure!(!more.is_zero(), Error::<T>::ValStakeZero);
			let mut state = <CandidatePool<T>>::get(&collator).ok_or(Error::<T>::CandidateNotFound)?;
			ensure!(!state.is_leaving(), Error::<T>::CannotStakeIfLeaving);

			let before = state.stake;
			state.stake_more(more);
			let after = state.stake;
			ensure!(
				after <= <MaxCollatorCandidateStake<T>>::get(),
				Error::<T>::ValStakeAboveMax
			);

			let unstaking_len = Self::increase_lock(&collator, after, more)?;

			if state.is_active() {
				Self::update_top_candidates(collator.clone(), before, state.total);
			}
			<CandidatePool<T>>::insert(&collator, state);

			// update candidates for next round
			let (num_collators, num_delegators, _, _) = Self::update_total_stake();

			Self::deposit_event(Event::CollatorStakedMore(collator, before, after));
			Ok(Some(<T as pallet::Config>::WeightInfo::candidate_stake_more(
				num_collators,
				num_delegators,
				unstaking_len,
			))
			.into())
		}

		/// Stake less funds for a collator candidate.
		///
		/// If the new amount of staked fund is not large enough, the account
		/// could be removed from the set of collator candidates and not be
		/// considered for authoring the next blocks.
		///
		/// This operation affects the pallet's total stake amount.
		///
		/// The unstaked funds are not release immediately to the account, but
		/// they will be available after `StakeDuration` blocks.
		///
		/// The resulting total amount of funds staked must be within the
		/// allowed range as set in the pallet's configuration.
		///
		/// Emits `CollatorStakedLess`.
		///
		/// # <weight>
		/// - The transaction's complexity is mainly dependent on updating the
		///   `SelectedCandidates` storage in `select_top_candidates` which in
		///   return depends on the number of `MaxSelectedCandidates` (N).
		/// - For each N, we read `CollatorState` from the storage.
		/// ---------
		/// Weight: O(N) where N is `MaxSelectedCandidates` bounded by
		/// `MaxTopCandidates`
		/// - Reads: [Origin Account], Unstaking, TopCandidates,
		///   MaxSelectedCandidates, N * CollatorState
		/// - Writes: Unstaking, CollatorState, Total, SelectedCandidates
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::candidate_stake_less(T::MaxTopCandidates::get(), T::MaxTopCandidates::get().saturating_mul(T::MaxDelegatorsPerCollator::get())))]
		pub fn candidate_stake_less(origin: OriginFor<T>, less: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			ensure!(!less.is_zero(), Error::<T>::ValStakeZero);

			let mut state = <CandidatePool<T>>::get(&collator).ok_or(Error::<T>::CandidateNotFound)?;
			ensure!(!state.is_leaving(), Error::<T>::CannotStakeIfLeaving);
			let before = state.stake;
			let after = state.stake_less(less).ok_or(Error::<T>::Underflow)?;
			ensure!(
				after >= T::MinCollatorCandidateStake::get(),
				Error::<T>::ValStakeBelowMin
			);

			// we don't unlock immediately
			Self::prep_unstake(&collator, less, false)?;

			if state.is_active() {
				Self::update_top_candidates(collator.clone(), before, state.total);
			}
			<CandidatePool<T>>::insert(&collator, state);

			// update candidates for next round
			let (num_collators, num_delegators, _, _) = Self::update_total_stake();

			Self::deposit_event(Event::CollatorStakedLess(collator, before, after));
			Ok(Some(<T as pallet::Config>::WeightInfo::candidate_stake_less(
				num_collators,
				num_delegators,
			))
			.into())
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
		/// Emits `DelegationReplaced` if the candidate has
		/// `MaxDelegatorsPerCollator` many delegations but this delegator
		/// staked more than one of the other delegators of this candidate.
		///
		/// # <weight>
		/// - The transaction's complexity is mainly dependent on updating the
		///   `SelectedCandidates` storage in `select_top_candidates` which in
		///   return depends on the number of `MaxSelectedCandidates` (N).
		/// - For each N, we read `CollatorState` from the storage.
		/// ---------
		/// Weight: O(N) + O(D) where N is `MaxSelectedCandidates` bounded by
		/// `MaxTopCandidates` and D is the number of delegators for this
		/// collator bounded by `MaxDelegatorsPerCollator`.
		/// - Reads: [Origin Account], DelegatorState, TopCandidates,
		///   MaxSelectedCandidates, (N + 2) * CollatorState, LastDelegation,
		///   Round
		/// - Writes: Locks, CollatorState, DelegatorState, Total,
		///   SelectedCandidates, LastDelegation
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::join_delegators(T::MaxTopCandidates::get(), T::MaxTopCandidates::get().saturating_mul(T::MaxDelegatorsPerCollator::get())))]
		pub fn join_delegators(
			origin: OriginFor<T>,
			collator: <T::Lookup as StaticLookup>::Source,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			let collator = T::Lookup::lookup(collator)?;
			// first delegation
			ensure!(<DelegatorState<T>>::get(&acc).is_none(), Error::<T>::AlreadyDelegating);
			ensure!(amount >= T::MinDelegatorStake::get(), Error::<T>::NomStakeBelowMin);
			// cannot be a collator candidate and delegator with same AccountId
			ensure!(!Self::is_active_candidate(&acc).is_some(), Error::<T>::CandidateExists);
			ensure!(
				Unstaking::<T>::get(&acc).len().saturated_into::<u32>() < T::MaxUnstakeRequests::get(),
				Error::<T>::CannotJoinBeforeUnlocking
			);
			// cannot delegate if number of delegations in this round exceeds
			// MaxDelegationsPerRound
			let delegation_counter = Self::get_delegation_counter(&acc)?;

			// prepare update of collator state
			let mut state = <CandidatePool<T>>::get(&collator).ok_or(Error::<T>::CandidateNotFound)?;
			let num_delegations_pre_insertion: u32 = state.delegators.len().saturated_into();

			ensure!(!state.is_leaving(), Error::<T>::CannotDelegateIfLeaving);
			let delegation = Stake {
				owner: acc.clone(),
				amount,
			};

			// attempt to insert delegator and check for uniqueness
			// NOTE: excess is handled below because we support replacing a delegator with
			// fewer stake
			let insert_delegator = state
				.delegators
				// we handle TooManyDelegators error below in do_update_delegator
				.try_insert(delegation.clone())
				.unwrap_or(true);
			// should never fail but let's be safe
			ensure!(insert_delegator, Error::<T>::DelegatorExists);

			// can only throw if MaxCollatorsPerDelegator is set to 0 which should never
			// occur in practice, even if the delegator rewards are set to 0
			let delegator_state = Delegator::try_new(collator.clone(), amount)
				.map_err(|_| Error::<T>::MaxCollatorsPerDelegatorExceeded)?;

			let old_total = state.total;
			// update state and potentially kick a delegator with less staked amount
			state = if num_delegations_pre_insertion == T::MaxDelegatorsPerCollator::get() {
				Self::do_update_delegator(delegation, state)?
			} else {
				state.total = state.total.saturating_add(amount);
				state
			};
			let new_total = state.total;

			// lock stake
			Self::increase_lock(&acc, amount, BalanceOf::<T>::zero())?;
			if state.is_active() {
				Self::update_top_candidates(collator.clone(), old_total, new_total);
			}

			// update states
			<CandidatePool<T>>::insert(&collator, state);
			<DelegatorState<T>>::insert(&acc, delegator_state);
			<LastDelegation<T>>::insert(&acc, delegation_counter);

			// update candidates for next round
			let (num_collators, num_delegators, _, _) = Self::update_total_stake();

			Self::deposit_event(Event::Delegation(acc, amount, collator, new_total));
			Ok(Some(<T as pallet::Config>::WeightInfo::join_delegators(
				num_collators,
				num_delegators,
			))
			.into())
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
		/// NOTE: This transaction is expected to throw until we increase
		/// `MaxCollatorsPerDelegator` by at least one, since it is currently
		/// set to one.
		///
		/// Emits `Delegation`.
		/// Emits `DelegationReplaced` if the candidate has
		/// `MaxDelegatorsPerCollator` many delegations but this delegator
		/// staked more than one of the other delegators of this candidate.
		///
		/// # <weight>
		/// - The transaction's complexity is mainly dependent on updating the
		///   `SelectedCandidates` storage in `select_top_candidates` which in
		///   return depends on the number of `MaxSelectedCandidates` (N).
		/// - For each N, we read `CollatorState` from the storage.
		/// ---------
		/// Weight: O(N) + O(D) where N is `MaxSelectedCandidates` bounded by
		/// `MaxTopCandidates` and D is the number of delegators for this
		/// collator bounded by `MaxDelegatorsPerCollator`.
		/// - Reads: [Origin Account], DelegatorState, TopCandidates,
		///   MaxSelectedCandidates, (N + 1) * CollatorState, LastDelegation,
		///   Round
		/// - Writes: Locks, CollatorState, DelegatorState, Total,
		///   SelectedCandidates, LastDelegation
		/// # </weight>
		//
		// We can't benchmark this extrinsic until we have increased `MaxCollatorsPerDelegator` by at least 1, thus we
		// use the closest weight we can get.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::join_delegators(T::MaxTopCandidates::get(), T::MaxTopCandidates::get().saturating_mul(T::MaxDelegatorsPerCollator::get())))]
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
				(delegator.delegations.len().saturated_into::<u32>()) < T::MaxCollatorsPerDelegator::get(),
				Error::<T>::MaxCollatorsPerDelegatorExceeded
			);
			// cannot delegate if number of delegations in this round exceeds
			// MaxDelegationsPerRound
			let delegation_counter = Self::get_delegation_counter(&acc)?;

			// prepare new collator state
			let mut state = <CandidatePool<T>>::get(&collator).ok_or(Error::<T>::CandidateNotFound)?;
			let num_delegations_pre_insertion: u32 = state.delegators.len().saturated_into();
			ensure!(!state.is_leaving(), Error::<T>::CannotDelegateIfLeaving);

			// attempt to insert delegator and check for uniqueness
			// NOTE: excess is handled below because we support replacing a delegator with
			// fewer stake
			ensure!(
				delegator
					.add_delegation(Stake {
						owner: collator.clone(),
						amount
					})
					.unwrap_or(true),
				Error::<T>::AlreadyDelegatedCollator
			);
			let delegation = Stake {
				owner: acc.clone(),
				amount,
			};

			// throws if delegation insertion exceeds bounded vec limit which we will handle
			// below in Self::do_update_delegator
			ensure!(
				state.delegators.try_insert(delegation.clone()).unwrap_or(true),
				Error::<T>::DelegatorExists
			);

			let old_total = state.total;

			// update state and potentially kick a delegator with less staked amount
			state = if num_delegations_pre_insertion == T::MaxDelegatorsPerCollator::get() {
				Self::do_update_delegator(delegation, state)?
			} else {
				state.total = state.total.saturating_add(amount);
				state
			};
			let new_total = state.total;

			// lock stake
			Self::increase_lock(&acc, delegator.total, amount)?;
			if state.is_active() {
				Self::update_top_candidates(collator.clone(), old_total, new_total);
			}

			// Update states
			<CandidatePool<T>>::insert(&collator, state);
			<DelegatorState<T>>::insert(&acc, delegator);
			<LastDelegation<T>>::insert(&acc, delegation_counter);

			// update candidates for next round
			let (num_collators, num_delegators, _, _) = Self::update_total_stake();

			Self::deposit_event(Event::Delegation(acc, amount, collator, new_total));
			Ok(Some(<T as pallet::Config>::WeightInfo::join_delegators(
				num_collators,
				num_delegators,
			))
			.into())
		}

		/// Leave the set of delegators and, by implication, revoke all ongoing
		/// delegations.
		///
		/// All staked funds are not unlocked immediately, but they are added to
		/// the queue of pending unstaking, and will effectively be released
		/// after `StakeDuration` blocks from the moment the delegator leaves.
		///
		/// This operation reduces the total stake of the pallet as well as the
		/// stakes of all collators that were delegated, potentially affecting
		/// their chances to be included in the set of candidates in the next
		/// rounds.
		///
		/// Emits `DelegatorLeft`.
		///
		/// # <weight>
		/// - The transaction's complexity is mainly dependent on updating the
		///   `SelectedCandidates` storage in `select_top_candidates` which in
		///   return depends on the number of `MaxSelectedCandidates` (N).
		/// - For each N, we read `CollatorState` from the storage.
		/// - If the numbers of delegators per collator (1 at genesis) and
		///   collators per delegator (25 at genesis) increased from the initial
		///   config at some point, the O(C * D) could weigh in more at that
		///   point.
		/// ---------
		/// Weight: O(N) + O(C * D) where N is `MaxSelectedCandidates` bounded
		/// by `MaxTopCandidates`, C the number collators for this
		/// delegator bounded by `MaxCollatorsPerDelegator` and D the number of
		/// total delegators for each C bounded by `MaxCollatorsPerDelegator`.
		/// - Reads: [Origin Account], DelegatorState, BlockNumber, Unstaking,
		///   TopCandidates, MaxSelectedCandidates, (N + 1) * CollatorState,
		///   CandidateCount
		/// - Writes: Unstaking, CollatorState, Total, SelectedCandidates,
		///   CandidateCount
		/// - Kills: DelegatorState
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::leave_delegators(T::MaxTopCandidates::get(), T::MaxTopCandidates::get().saturating_mul(T::MaxDelegatorsPerCollator::get())))]
		pub fn leave_delegators(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			let delegator = <DelegatorState<T>>::get(&acc).ok_or(Error::<T>::DelegatorNotFound)?;
			for stake in delegator.delegations.into_iter() {
				Self::delegator_leaves_collator(acc.clone(), stake.owner.clone())?;
			}
			<DelegatorState<T>>::remove(&acc);

			// update candidates for next round
			let (num_collators, num_delegators, _, _) = Self::update_total_stake();

			Self::deposit_event(Event::DelegatorLeft(acc, delegator.total));
			Ok(Some(<T as pallet::Config>::WeightInfo::leave_delegators(
				num_collators,
				num_delegators,
			))
			.into())
		}

		/// Terminates an ongoing delegation for a given collator candidate.
		///
		/// The staked funds are not unlocked immediately, but they are added to
		/// the queue of pending unstaking, and will effectively be released
		/// after `StakeDuration` blocks from the moment the delegation is
		/// terminated.
		///
		/// This operation reduces the total stake of the pallet as well as the
		/// stakes of the collator involved, potentially affecting its chances
		/// to be included in the set of candidates in the next rounds.
		///
		/// Emits `DelegatorLeft`.
		///
		/// # <weight>
		/// - The transaction's complexity is mainly dependent on updating the
		///   `SelectedCandidates` storage in `select_top_candidates` which in
		///   return depends on the number of `MaxSelectedCandidates` (N).
		/// - For each N, we read `CollatorState` from the storage.
		/// ---------
		/// Weight: O(N) + O(D) where N is `MaxSelectedCandidates` bounded
		/// by `MaxTopCandidates` and D the number of total delegators for
		/// this collator bounded by `MaxCollatorsPerDelegator`.
		/// - Reads: [Origin Account], DelegatorState, BlockNumber, Unstaking,
		///   Locks, TopCandidates, (N + 1) * CollatorState,
		///   MaxSelectedCandidates
		/// - Writes: Unstaking, Locks, DelegatorState, CollatorState, Total,
		///   SelectedCandidates
		/// - Kills: DelegatorState if the delegator has not delegated to
		///   another collator
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::revoke_delegation(T::MaxTopCandidates::get(), T::MaxTopCandidates::get().saturating_mul(T::MaxDelegatorsPerCollator::get())))]
		pub fn revoke_delegation(
			origin: OriginFor<T>,
			collator: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let collator = T::Lookup::lookup(collator)?;
			let delegator = ensure_signed(origin)?;
			Self::delegator_revokes_collator(delegator, collator)?;

			// update candidates for next round
			// TODO: Only need to be updated if collator is not leaving!
			let (num_collators, num_delegators, _, _) = Self::update_total_stake();

			Ok(Some(<T as pallet::Config>::WeightInfo::revoke_delegation(
				num_collators,
				num_delegators,
			))
			.into())
		}

		/// Increase the stake for delegating a collator candidate.
		///
		/// If not in the set of candidates, staking enough funds allows the
		/// collator candidate to be added to it.
		///
		/// Emits `DelegatorStakedMore`.
		///
		/// # <weight>
		/// - The transaction's complexity is mainly dependent on updating the
		///   `SelectedCandidates` storage in `select_top_candidates` which in
		///   return depends on the number of `MaxSelectedCandidates` (N).
		/// - For each N, we read `CollatorState` from the storage.
		/// ---------
		/// Weight: O(N) + O(D) where N is `MaxSelectedCandidates` bounded
		/// by `MaxTopCandidates` and D the number of total delegators for
		/// this collator bounded by `MaxCollatorsPerDelegator`.
		/// - Reads: [Origin Account], DelegatorState, BlockNumber, Unstaking,
		///   Locks, TopCandidates, (N + 1) * CollatorState,
		///   MaxSelectedCandidates
		/// - Writes: Unstaking, Locks, DelegatorState, CollatorState, Total,
		///   SelectedCandidates
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::candidate_stake_more(T::MaxTopCandidates::get(), T::MaxTopCandidates::get().saturating_mul(T::MaxDelegatorsPerCollator::get()), T::MaxUnstakeRequests::get().saturated_into::<u32>()))]
		pub fn delegator_stake_more(
			origin: OriginFor<T>,
			candidate: <T::Lookup as StaticLookup>::Source,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			ensure!(!more.is_zero(), Error::<T>::ValStakeZero);

			let candidate = T::Lookup::lookup(candidate)?;
			let mut delegations = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorNotFound)?;
			let mut collator = <CandidatePool<T>>::get(&candidate).ok_or(Error::<T>::CandidateNotFound)?;
			ensure!(!collator.is_leaving(), Error::<T>::CannotDelegateIfLeaving);
			let delegator_total = delegations
				.inc_delegation(candidate.clone(), more)
				.ok_or(Error::<T>::DelegationNotFound)?;

			// update lock
			let unstaking_len = Self::increase_lock(&delegator, delegator_total, more)?;
			let before = collator.total;
			collator.inc_delegator(delegator.clone(), more);
			let after = collator.total;

			if collator.is_active() {
				Self::update_top_candidates(candidate.clone(), before, collator.total);
			}
			<CandidatePool<T>>::insert(&candidate, collator);
			<DelegatorState<T>>::insert(&delegator, delegations);

			// update candidates for next round
			let (num_collators, num_delegators, _, _) = Self::update_total_stake();

			Self::deposit_event(Event::DelegatorStakedMore(delegator, candidate, before, after));
			Ok(Some(<T as pallet::Config>::WeightInfo::delegator_stake_more(
				num_collators,
				num_delegators,
				unstaking_len,
			))
			.into())
		}

		/// Reduce the stake for delegating a collator candidate.
		///
		/// If the new amount of staked fund is not large enough, the collator
		/// could be removed from the set of collator candidates and not be
		/// considered for authoring the next blocks.
		///
		/// The unstaked funds are not release immediately to the account, but
		/// they will be available after `StakeDuration` blocks.
		///
		/// The remaining staked funds must still be larger than the minimum
		/// required by this pallet to maintain the status of delegator.
		///
		/// The resulting total amount of funds staked must be within the
		/// allowed range as set in the pallet's configuration.
		///
		/// Emits `DelegatorStakedLess`.
		///
		/// # <weight>
		/// - The transaction's complexity is mainly dependent on updating the
		///   `SelectedCandidates` storage in `select_top_candidates` which in
		///   return depends on the number of `MaxSelectedCandidates` (N).
		/// - For each N, we read `CollatorState` from the storage.
		/// ---------
		/// Weight: O(N) + O(D) where N is `MaxSelectedCandidates` bounded
		/// by `MaxTopCandidates` and D the number of total delegators for
		/// this collator bounded by `MaxCollatorsPerDelegator`.
		/// - Reads: [Origin Account], DelegatorState, BlockNumber, Unstaking,
		///   TopCandidates, (N + 1) * CollatorState, MaxSelectedCandidates
		/// - Writes: Unstaking, DelegatorState, CollatorState, Total,
		///   SelectedCandidates
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::delegator_stake_less(T::MaxTopCandidates::get(), T::MaxTopCandidates::get().saturating_mul(T::MaxDelegatorsPerCollator::get())))]
		pub fn delegator_stake_less(
			origin: OriginFor<T>,
			candidate: <T::Lookup as StaticLookup>::Source,
			less: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			ensure!(!less.is_zero(), Error::<T>::ValStakeZero);

			let candidate = T::Lookup::lookup(candidate)?;
			let mut delegations = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorNotFound)?;
			let mut collator = <CandidatePool<T>>::get(&candidate).ok_or(Error::<T>::CandidateNotFound)?;
			ensure!(!collator.is_leaving(), Error::<T>::CannotDelegateIfLeaving);
			let remaining = delegations
				.dec_delegation(candidate.clone(), less)
				.ok_or(Error::<T>::DelegationNotFound)?
				.ok_or(Error::<T>::Underflow)?;

			ensure!(remaining >= T::MinDelegation::get(), Error::<T>::DelegationBelowMin);
			ensure!(
				delegations.total >= T::MinDelegatorStake::get(),
				Error::<T>::NomStakeBelowMin
			);

			Self::prep_unstake(&delegator, less, false)?;

			let before = collator.total;
			collator.dec_delegator(delegator.clone(), less);
			let after = collator.total;
			if collator.is_active() {
				Self::update_top_candidates(candidate.clone(), before, collator.total);
			}
			<CandidatePool<T>>::insert(&candidate, collator);
			<DelegatorState<T>>::insert(&delegator, delegations);

			// update candidates for next round
			let (num_collators, num_delegators, _, _) = Self::update_total_stake();

			Self::deposit_event(Event::DelegatorStakedLess(delegator, candidate, before, after));
			Ok(Some(<T as pallet::Config>::WeightInfo::delegator_stake_less(
				num_collators,
				num_delegators,
			))
			.into())
		}

		/// Unlock all previously staked funds that are now available for
		/// unlocking by the origin account after `StakeDuration` blocks have
		/// elapsed.
		///
		/// Weight: O(U) where U is the number of locked unstaking requests
		/// bounded by `MaxUnstakeRequests`.
		/// - Reads: [Origin Account], Unstaking, Locks
		/// - Writes: Unstaking, Locks
		/// - Kills: Unstaking & Locks if no balance is locked anymore
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::unlock_unstaked(T::MaxUnstakeRequests::get().saturated_into::<u32>()))]
		pub fn unlock_unstaked(
			origin: OriginFor<T>,
			target: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			let target = T::Lookup::lookup(target)?;

			let unstaking_len = Self::do_unlock(&target)?;

			Ok(Some(<T as pallet::Config>::WeightInfo::unlock_unstaked(unstaking_len)).into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Check whether an account is currently delegating.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: DelegatorState
		/// # </weight>
		pub fn is_delegator(acc: &T::AccountId) -> bool {
			<DelegatorState<T>>::get(acc).is_some()
		}

		/// Check whether an account is currently a collator candidate and
		/// whether their state is CollatorStatus::Active.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: CollatorState
		/// # </weight>
		pub fn is_active_candidate(acc: &T::AccountId) -> Option<bool> {
			if let Some(state) = <CandidatePool<T>>::get(acc) {
				Some(state.status == CandidateStatus::Active)
			} else {
				None
			}
		}

		/// Update the staking information for an active collator candidate.
		///
		/// NOTE: it is assumed that the calling context checks whether the
		/// collator candidate is currently active before calling this function.
		///
		/// # <weight>
		/// Weight: O(D) where D is the number of top candidates.
		/// - Reads: TopCandidates
		/// - Writes: TopCandidates
		/// # </weight>
		fn update_top_candidates(candidate: T::AccountId, old_total: BalanceOf<T>, new_total: BalanceOf<T>) {
			// check if candidate is in top_candidates (and update)
			// or if candidate will ascend into top_candidates
			let mut top_candidates = <TopCandidates<T>>::get();
			let old_stake = Stake {
				owner: candidate.clone(),
				amount: old_total,
			};
			if let Ok(i) = top_candidates.linear_search(&old_stake) {
				top_candidates.mutate(|vec| {
					if let Some(stake) = vec.get_mut(i) {
						stake.amount = new_total;
					}
				});
				TopCandidates::<T>::put(top_candidates);
			} else if let Ok(drop_out) = top_candidates.try_insert_replace(Stake {
				owner: candidate.clone(),
				amount: new_total,
			}) {
				if let Some(drop_out) = drop_out {
					Self::deposit_event(Event::LeftTopCandidates(drop_out.owner));
				}
				Self::deposit_event(Event::EnteredTopCandidates(candidate));
				TopCandidates::<T>::put(top_candidates);
			}
		}

		/// Update the delegator's state by removing the collator candidate from
		/// the set of ongoing delegations.
		///
		/// # <weight>
		/// - The transaction's complexity is mainly dependent on updating the
		///   `SelectedCandidates` storage in `select_top_candidates` which in
		///   return depends on the number of `MaxSelectedCandidates` (N).
		/// - For each N, we read `CollatorState` from the storage.
		/// ---------
		/// Weight: O(N) + O(D) where N is `MaxSelectedCandidates` bounded
		/// by `MaxTopCandidates` and D the number of total delegators for
		/// this collator bounded by `MaxCollatorsPerDelegator`.
		/// - Reads: [Origin Account], DelegatorState, BlockNumber, Unstaking,
		///   Locks, TopCandidates, (N + 1) * CollatorState,
		///   MaxSelectedCandidates
		/// - Writes: Unstaking, Locks, DelegatorState, CollatorState, Total,
		///   SelectedCandidates
		/// - Kills: DelegatorState if the delegator has not delegated to
		///   another collator
		/// # </weight>
		fn delegator_revokes_collator(acc: T::AccountId, collator: T::AccountId) -> DispatchResult {
			let mut delegator = <DelegatorState<T>>::get(&acc).ok_or(Error::<T>::DelegatorNotFound)?;
			let old_total = delegator.total;
			let remaining = delegator
				.rm_delegation(&collator)
				.ok_or(Error::<T>::DelegationNotFound)?;

			// edge case; if no delegations remaining, leave set of delegators
			if delegator.delegations.is_empty() {
				// leave the set of delegators because no delegations left
				Self::delegator_leaves_collator(acc.clone(), collator)?;
				<DelegatorState<T>>::remove(&acc);
				Self::deposit_event(Event::DelegatorLeft(acc, old_total));
			} else {
				// can never fail iff MinDelegatorStake == MinDelegation
				ensure!(remaining >= T::MinDelegatorStake::get(), Error::<T>::NomStakeBelowMin);
				Self::delegator_leaves_collator(acc.clone(), collator)?;
				<DelegatorState<T>>::insert(&acc, delegator);
			}

			Ok(())
		}

		/// Update the collator's state by removing the delegator's stake and
		/// starting the process to unlock the delegator's staked funds.
		///
		/// This operation affects the pallet's total stake.
		///
		/// # <weight>
		/// Weight: O(D) where D is the number of delegators for this
		/// collator bounded by `MaxDelegatorsPerCollator`.
		/// - Reads: CollatorState, BlockNumber, Unstaking
		/// - Writes: Unstaking, Total, CollatorState
		/// # </weight>
		fn delegator_leaves_collator(delegator: T::AccountId, collator: T::AccountId) -> DispatchResult {
			let mut state = <CandidatePool<T>>::get(&collator).ok_or(Error::<T>::CandidateNotFound)?;

			let delegator_stake = state
				.delegators
				.remove(&Stake {
					owner: delegator.clone(),
					// amount is irrelevant for removal
					amount: BalanceOf::<T>::one(),
				})
				.map(|nom| nom.amount)
				.ok_or(Error::<T>::DelegatorNotFound)?;

			let old_total = state.total;
			state.total = state.total.saturating_sub(delegator_stake);

			// we don't unlock immediately
			Self::prep_unstake(&delegator, delegator_stake, false)?;

			if state.is_active() {
				Self::update_top_candidates(collator.clone(), old_total, state.total);
			}
			let new_total = state.total;
			<CandidatePool<T>>::insert(&collator, state);

			Self::deposit_event(Event::DelegatorLeftCollator(
				delegator,
				collator,
				delegator_stake,
				new_total,
			));
			Ok(())
		}

		fn kick_delegator(delegation: &StakeOf<T>, collator: &T::AccountId) -> DispatchResult {
			let mut state = <DelegatorState<T>>::get(&delegation.owner).ok_or(Error::<T>::DelegatorNotFound)?;
			state.rm_delegation(collator);
			// we don't unlock immediately
			Self::prep_unstake(&delegation.owner, delegation.amount, true)?;

			// clear storage if no delegations are remaining
			if state.delegations.is_empty() {
				<DelegatorState<T>>::remove(&delegation.owner);
			} else {
				<DelegatorState<T>>::insert(&delegation.owner, state);
			}
			Ok(())
		}

		/// Select the top `MaxSelectedCandidates` many collators in terms of
		/// cumulated stake (self + from delegators) from the TopCandidates to
		/// become block authors for the next round. The number of candidates
		/// selected can be `n` or lower in case there are less candidates
		/// available.
		///
		/// We do not want to execute this function in `on_initialize` or
		/// `new_session` because we could exceed the PoV size limit and brick
		/// our chain.
		/// Instead we execute this function in every extrinsic which mutates
		/// the amount at stake (collator or delegator). This will heavily
		/// increase the weight of each of these transactions it enables us to
		/// do a simple storage read to get the top candidates when a session
		/// starts in `new_session.
		///
		/// # <weight>
		/// Weight: O(N) where N is `MaxSelectedCandidates` bounded by
		/// `MaxTopCandidates`
		/// - Reads: TopCandidates, MaxSelectedCandidates, N * CollatorState
		/// - Writes: SelectedCandidates
		/// # </weight>
		fn update_total_stake() -> (u32, u32, BalanceOf<T>, BalanceOf<T>) {
			let mut num_of_delegators = 0u32;
			let mut collator_stake = BalanceOf::<T>::zero();
			let mut delegator_stake = BalanceOf::<T>::zero();

			let collators = Self::selected_candidates();

			// Snapshot exposure for round for weighting reward distribution
			for account in collators.iter() {
				let state =
					<CandidatePool<T>>::get(&account).expect("all members of TopCandidates must be candidates q.e.d");
				num_of_delegators = num_of_delegators.saturating_add(state.delegators.len().saturated_into::<u32>());

				// sum up total stake and amount of collators, delegators
				let amount_collator = state.stake;
				collator_stake = collator_stake.saturating_add(state.stake);
				// safe to subtract because total >= stake
				let amount_delegators = state.total - amount_collator;
				delegator_stake = delegator_stake.saturating_add(amount_delegators);

				Self::deposit_event(Event::CollatorChosen(
					account.clone(),
					amount_collator,
					amount_delegators,
				));
			}

			<TotalCollatorStake<T>>::mutate(|total| {
				total.collators = collator_stake;
				total.delegators = delegator_stake;
			});

			// return number of selected candidates and the corresponding number of their
			// delegators for post-weight correction
			(
				collators.len().saturated_into(),
				num_of_delegators,
				collator_stake,
				delegator_stake,
			)
		}

		/// Return the best `MaxSelectedCandidates` many candidates.
		///
		/// In case a collator from last round was replaced by a candidate with
		/// the same total stake during sorting, we revert this swap to
		/// prioritize collators over candidates.
		///
		/// # <weight>
		/// Weight: O(MaxSelectedCandidates)
		/// - Reads: TopCandidates, MaxSelectedCandidates
		/// # </weight>
		pub fn selected_candidates() -> BoundedVec<T::AccountId, T::MaxTopCandidates> {
			let candidates = <TopCandidates<T>>::get();

			// Should never fail since WASM usize are 32bits and native are either 32 or 64
			let top_n = MaxSelectedCandidates::<T>::get().saturated_into::<usize>();

			log::trace!("{} Candidates for {} Collator seats", candidates.len(), top_n);

			// Choose the top MaxSelectedCandidates qualified candidates
			let collators = candidates
				.into_iter()
				.take(top_n)
				.filter(|x| x.amount >= T::MinCollatorStake::get())
				.map(|x| x.owner)
				.collect::<Vec<T::AccountId>>();

			collators.try_into().expect("Did not extend Collators q.e.d.")
		}

		/// Attempts to add the stake to the set of delegators of a collator
		/// which already reached its maximum size by removing an already
		/// existing delegator with less staked value. If the given staked
		/// amount is at most the minimum staked value of the original delegator
		/// set, an error is returned.
		///
		/// Returns the old delegation that is updated, if any.
		///
		/// Emits `DelegationReplaced` if the stake exceeds one of the current
		/// delegations.
		///
		/// # <weight>
		/// Weight: O(D) where D is the number of delegators for this collator
		/// bounded by `MaxDelegatorsPerCollator`.
		/// - Reads/Writes: 0
		/// # </weight>
		fn do_update_delegator(
			stake: Stake<T::AccountId, BalanceOf<T>>,
			mut state: Candidate<T::AccountId, BalanceOf<T>, T::MaxDelegatorsPerCollator>,
		) -> Result<CandidateOf<T, T::MaxDelegatorsPerCollator>, DispatchError> {
			// attempt to replace the last element of the set
			let stake_to_remove = state
				.delegators
				.try_insert_replace(stake.clone())
				.map_err(|err_too_many| {
					if err_too_many {
						Error::<T>::TooManyDelegators
					} else {
						// should never occur because we previously check this case, but let's be sure
						Error::<T>::AlreadyDelegating
					}
				})?;

			state.total = state.total.saturating_add(stake.amount);

			if let Some(stake_to_remove) = stake_to_remove {
				// update total stake
				state.total = state.total.saturating_sub(stake_to_remove.amount);

				// update storage of kicked delegator
				Self::kick_delegator(&stake_to_remove, &state.id)?;

				Self::deposit_event(Event::DelegationReplaced(
					stake.owner,
					stake.amount,
					stake_to_remove.owner,
					stake_to_remove.amount,
					state.id.clone(),
					state.total,
				));
			}

			Ok(state)
		}

		/// Either set or increase the BalanceLock of target account to
		/// amount.
		///
		/// Consumes unstaked balance which can be unlocked in the future up to
		/// amount and updates `Unstaking` storage accordingly.
		///
		/// # <weight>
		/// Weight: O(U) where U is the number of locked unstaking requests
		/// bounded by `MaxUnstakeRequests`.
		/// - Reads: Unstaking, Locks
		/// - Writes: Unstaking, Locks
		/// # </weight>
		fn increase_lock(who: &T::AccountId, amount: BalanceOf<T>, more: BalanceOf<T>) -> Result<u32, DispatchError> {
			ensure!(
				pallet_balances::Pallet::<T>::free_balance(who) >= amount.into(),
				pallet_balances::Error::<T>::InsufficientBalance
			);

			let mut unstaking_len = 0u32;

			// update Unstaking by consuming up to {amount | more}
			<Unstaking<T>>::try_mutate(who, |unstaking| -> DispatchResult {
				// reduce {amount | more} by unstaking until either {amount | more} is zero or
				// no unstaking is left
				// if more is set, we only want to reduce by more to achieve 100 - 40 + 30 = 90
				// locked
				let mut amt_consuming_unstaking = if more.is_zero() { amount } else { more };
				unstaking_len = unstaking.len().saturated_into();
				for (block_number, locked_balance) in unstaking.clone() {
					if amt_consuming_unstaking.is_zero() {
						break;
					} else if locked_balance > amt_consuming_unstaking {
						// amount is only reducible by locked_balance - amt_consuming_unstaking
						let delta = locked_balance.saturating_sub(amt_consuming_unstaking);
						// replace old entry with delta
						unstaking
							.try_insert(block_number, delta)
							.map_err(|_| Error::<T>::NoMoreUnstaking)?;
						amt_consuming_unstaking = Zero::zero();
					} else {
						// amount is either still reducible or reached
						amt_consuming_unstaking = amt_consuming_unstaking.saturating_sub(locked_balance);
						unstaking.remove(&block_number);
					}
				}
				Ok(())
			})?;

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

			Ok(unstaking_len)
		}

		/// Set the unlocking block for the account and corresponding amount
		/// which can be unlocked via `unlock_unstaked` after waiting at
		/// least for `StakeDuration` many blocks.
		///
		/// Throws if the amount is zero (unlikely) or if active unlocking
		/// requests exceed limit. The latter defends against stake reduction
		/// spamming.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: BlockNumber, Unstaking
		/// - Writes: Unstaking
		/// # </weight>
		fn prep_unstake(who: &T::AccountId, amount: BalanceOf<T>, is_removal: bool) -> DispatchResult {
			// should never occur but let's be safe
			ensure!(!amount.is_zero(), Error::<T>::StakeNotFound);

			let now = <frame_system::Pallet<T>>::block_number();
			let unlock_block = now.saturating_add(T::StakeDuration::get());
			let mut unstaking = <Unstaking<T>>::get(who);

			let allowed_unstakings = if is_removal {
				// the account was forcedly removed and we allow to fill all unstake requests
				T::MaxUnstakeRequests::get()
			} else {
				// we need to reserve a free slot for a forced removal of the account
				T::MaxUnstakeRequests::get().saturating_sub(1)
			};
			ensure!(
				unstaking.len().saturated_into::<u32>() < allowed_unstakings,
				Error::<T>::NoMoreUnstaking,
			);

			// if existent, we have to add the current amount of same unlock_block, because
			// insert overwrites the current value
			let amount = amount.saturating_add(*unstaking.get(&unlock_block).unwrap_or(&BalanceOf::<T>::zero()));
			unstaking
				.try_insert(unlock_block, amount)
				.map_err(|_| Error::<T>::NoMoreUnstaking)?;
			<Unstaking<T>>::insert(who, unstaking);
			Ok(())
		}

		/// Clear the CollatorState of the candidate and remove all delegations
		/// to the candidate. Moreover, prepare unstaking for the candidate and
		/// their former delegations.
		///
		/// # <weight>
		/// Weight: O(D) where D is the number of delegators of the collator
		/// candidate bounded by `MaxDelegatorsPerCollator`
		/// - Reads: BlockNumber, D * DelegatorState, D * Unstaking
		/// - Writes: D * DelegatorState, (D + 1) * Unstaking
		/// - Kills: CollatorState, DelegatorState for all delegators which only
		///   delegated to the candidate
		/// # </weight>
		fn remove_candidate(
			collator: &T::AccountId,
			state: &CandidateOf<T, T::MaxDelegatorsPerCollator>,
		) -> DispatchResult {
			// iterate over delegators
			for stake in &state.delegators[..] {
				// prepare unstaking of delegator
				Self::prep_unstake(&stake.owner, stake.amount, true)?;
				// remove delegation from delegator state
				if let Some(mut delegator) = <DelegatorState<T>>::get(&stake.owner) {
					if let Some(remaining) = delegator.rm_delegation(collator) {
						if remaining.is_zero() {
							<DelegatorState<T>>::remove(&stake.owner);
						} else {
							<DelegatorState<T>>::insert(&stake.owner, delegator);
						}
					}
				}
			}
			// prepare unstaking of collator candidate
			Self::prep_unstake(&state.id, state.stake, true)?;

			// disable validator for next session if they were in the set of validators
			pallet_session::Pallet::<T>::validators()
				.into_iter()
				.enumerate()
				.find_map(|(i, id)| {
					if <T as pallet_session::Config>::ValidatorIdOf::convert(collator.clone()) == Some(id) {
						Some(i)
					} else {
						None
					}
				})
				// FIXME: Does not prevent the collator from being able to author a block in this (or potentially the next) session. See https://github.com/paritytech/substrate/issues/8004
				.map(pallet_session::Pallet::<T>::disable_index);

			<CandidatePool<T>>::remove(&collator);
			CandidateCount::<T>::mutate(|count| *count = count.saturating_sub(1));
			Ok(())
		}

		/// Withdraw all staked currency which was unstaked at least
		/// `StakeDuration` blocks ago.
		///
		/// # <weight>
		/// Weight: O(U) where U is the number of locked unstaking
		/// requests bounded by `MaxUnstakeRequests`.
		/// - Reads: Unstaking, Locks
		/// - Writes: Unstaking, Locks
		/// - Kills: Unstaking & Locks if no balance is locked anymore
		/// # </weight>
		fn do_unlock(who: &T::AccountId) -> Result<u32, DispatchError> {
			let now = <frame_system::Pallet<T>>::block_number();
			let mut unstaking = <Unstaking<T>>::get(who);
			let unstaking_len = unstaking.len().saturated_into::<u32>();
			ensure!(!unstaking.is_empty(), Error::<T>::UnstakingIsEmpty);

			let mut total_unlocked: BalanceOf<T> = Zero::zero();
			let mut total_locked: BalanceOf<T> = Zero::zero();
			let mut expired = Vec::new();

			// check potential unlocks
			for (block_number, locked_balance) in unstaking.clone().into_iter() {
				if block_number <= now {
					expired.push(block_number);
					total_unlocked = total_unlocked.saturating_add(locked_balance);
				} else {
					total_locked = total_locked.saturating_add(locked_balance);
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

			Ok(unstaking_len)
		}

		/// Process the coinbase rewards for the production of a new block.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: Balance
		/// - Writes: Balance
		/// # </weight>
		fn do_reward(who: &T::AccountId, reward: BalanceOf<T>) {
			// mint
			if let Ok(imb) = T::Currency::deposit_into_existing(who, reward) {
				Self::deposit_event(Event::Rewarded(who.clone(), imb.peek()));
			}
		}

		/// Annually reduce the reward rates for collators and delegators.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: LastRewardReduction, InflationConfig
		/// - Writes: LastRewardReduction, InflationConfig
		/// # </weight>
		fn adjust_reward_rates(now: T::BlockNumber) -> Weight {
			let year = now / BLOCKS_PER_YEAR.saturated_into::<T::BlockNumber>();
			let last_update = <LastRewardReduction<T>>::get();
			if year > last_update {
				let inflation = <InflationConfig<T>>::get();
				// collator reward rate decreases by 2% of the previous one per year
				let c_reward_rate = inflation.collator.reward_rate.annual * Perquintill::from_percent(98);
				// delegator reward rate should be 6% in 2nd year and 0% afterwards
				let d_reward_rate = if year == T::BlockNumber::one() {
					Perquintill::from_percent(6)
				} else {
					Perquintill::zero()
				};

				let new_inflation = InflationInfo::new(
					inflation.collator.max_rate,
					c_reward_rate,
					inflation.delegator.max_rate,
					d_reward_rate,
				);
				<InflationConfig<T>>::put(new_inflation.clone());
				<LastRewardReduction<T>>::put(year);
				Self::deposit_event(Event::RoundInflationSet(
					new_inflation.collator.max_rate,
					new_inflation.collator.reward_rate.per_block,
					new_inflation.delegator.max_rate,
					new_inflation.delegator.reward_rate.per_block,
				));
				<T as Config>::WeightInfo::on_initialize_new_year();
			}
			T::DbWeight::get().reads(1)
		}

		/// Checks whether a delegator can still delegate in this round, e.g.,
		/// if they have not delegated MaxDelegationsPerRound many times
		/// already in this round.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: LastDelegation, Round
		/// # </weight>
		fn get_delegation_counter(delegator: &T::AccountId) -> Result<DelegationCounter, DispatchError> {
			let last_delegation = <LastDelegation<T>>::get(delegator);
			let round = <Round<T>>::get();

			let counter = if last_delegation.round < round.current {
				0u32
			} else {
				last_delegation.counter
			};

			ensure!(
				T::MaxDelegationsPerRound::get() > counter,
				Error::<T>::DelegationsPerRoundExceeded
			);

			Ok(DelegationCounter {
				round: round.current,
				counter: counter.saturating_add(1),
			})
		}

		// [Post-launch TODO] Think about Collator stake or total stake?
		// /// Attempts to add a collator candidate to the set of collator
		// /// candidates which already reached its maximum size. On success,
		// /// another collator with the minimum total stake is removed from the
		// /// set. On failure, an error is returned. removing an already existing
		// fn check_collator_candidate_inclusion(
		// 	stake: Stake<T::AccountId, BalanceOf<T>>,
		// 	mut candidates: OrderedSet<Stake<T::AccountId, BalanceOf<T>>,
		// T::MaxTopCandidates>, ) -> Result<(), DispatchError> {
		// 	todo!()
		// }
	}

	impl<T> pallet_authorship::EventHandler<T::AccountId, T::BlockNumber> for Pallet<T>
	where
		T: Config + pallet_authorship::Config + pallet_session::Config,
	{
		/// Compute coinbase rewards for block production and distribute it to
		/// collator's (block producer) and its delegators according to their
		/// stake and the current InflationInfo.
		///
		/// The rewards are split between collators and delegators with
		/// different reward rates and maximum staking rates. The latter is
		/// required to have at most our targeted inflation because rewards are
		/// minted. Rewards are immediately available without any restrictions
		/// after minting.
		///
		/// If the current staking rate is below the maximum, each collator and
		/// delegator receives the corresponding `reward_rate * stake /
		/// blocks_per_year`. Since a collator can only author blocks every
		/// `MaxSelectedCandidates` many rounds, we multiply the reward with
		/// this number. As a result, a collator who has been in the set of
		/// selected candidates, eventually receives `reward_rate * stake` after
		/// one year.
		///
		/// However, if the current staking rate exceeds the max staking rate,
		/// the reward will be reduced by `max_rate / current_rate`. E.g., if
		/// the current rate is at 50% and the max rate at 40%, the reward is
		/// reduced by 20%.
		///
		/// # <weight>
		/// Weight: O(D) where D is the number of delegators of this collator
		/// block author bounded by `MaxDelegatorsPerCollator`.
		/// - Reads: CollatorState, Total, Balance, InflationConfig,
		///   MaxSelectedCandidates, Validators, DisabledValidators
		/// - Writes: (D + 1) * Balance
		/// # </weight>
		fn note_author(author: T::AccountId) {
			let mut reads = Weight::one();
			let mut writes = Weight::zero();
			log::info!(
				"Noting author {:#?} in block {:?} with starting balance {:?}",
				&author,
				<frame_system::Pallet<T>>::block_number(),
				T::Currency::free_balance(&author)
			);
			// should always include state except if the collator has been forcedly removed
			// via `force_remove_candidate` in the current or previous round
			if let Some(state) = <CandidatePool<T>>::get(author.clone()) {
				let total_issuance = T::Currency::total_issuance();
				let TotalStake {
					collators: total_collators,
					delegators: total_delegators,
				} = <TotalCollatorStake<T>>::get();
				let c_staking_rate = Perquintill::from_rational(total_collators, total_issuance);
				let d_staking_rate = Perquintill::from_rational(total_delegators, total_issuance);
				let inflation_config = <InflationConfig<T>>::get();
				let authors = pallet_session::Pallet::<T>::validators();
				let authors_per_round = <BalanceOf<T>>::from(authors.len().saturated_into::<u128>());

				// Reward collator
				let amt_due_collator =
					inflation_config
						.collator
						.compute_reward::<T>(state.stake, c_staking_rate, authors_per_round);
				Self::do_reward(&author, amt_due_collator);
				writes = writes.saturating_add(Weight::one());

				// Reward delegators
				for Stake { owner, amount } in state.delegators {
					log::info!(
						"Noting delegator {:#?} in block {:?} with starting balance {:?}",
						&owner,
						<frame_system::Pallet<T>>::block_number(),
						T::Currency::free_balance(&owner)
					);
					if amount >= T::MinDelegatorStake::get() {
						let due =
							inflation_config
								.delegator
								.compute_reward::<T>(amount, d_staking_rate, authors_per_round);
						Self::do_reward(&owner, due);
						writes = writes.saturating_add(Weight::one());
					}
				}
				reads = reads.saturating_add(4);
			}

			frame_system::Pallet::<T>::register_extra_weight_unchecked(
				T::DbWeight::get().reads_writes(reads, writes),
				DispatchClass::Mandatory,
			);
		}

		fn note_uncle(_author: T::AccountId, _age: T::BlockNumber) {
			// we too are not caring.
		}
	}

	impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
		/// 1. A new session starts.
		/// 2. In hook new_session: Read the current top n candidates from the
		///    TopCandidates and assign this set to author blocks for the next
		///    session.
		/// 3. AuRa queries the authorities from the session pallet for
		///    this session and picks authors on round-robin-basis from list of
		///    authorities.
		fn new_session(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
			log::debug!(
				"assembling new collators for new session {} at #{:?}",
				new_index,
				<frame_system::Pallet<T>>::block_number(),
			);

			frame_system::Pallet::<T>::register_extra_weight_unchecked(
				T::DbWeight::get().reads(2),
				DispatchClass::Mandatory,
			);

			let collators = Pallet::<T>::selected_candidates().to_vec();
			if collators.is_empty() {
				// we never want to pass an empty set of collators. This would brick the chain.
				log::error!("💥 keeping old session because of empty collator set!");
				None
			} else {
				Some(collators)
			}
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
			frame_system::Pallet::<T>::register_extra_weight_unchecked(
				T::DbWeight::get().reads(2),
				DispatchClass::Mandatory,
			);

			let mut round = <Round<T>>::get();
			// always update when a new round should start
			if round.should_update(now) {
				true
			} else if <ForceNewRound<T>>::get() {
				frame_system::Pallet::<T>::register_extra_weight_unchecked(
					T::DbWeight::get().writes(2),
					DispatchClass::Mandatory,
				);
				// check for forced new round
				<ForceNewRound<T>>::put(false);
				round.update(now);
				<Round<T>>::put(round);
				Self::deposit_event(Event::NewRound(round.first, round.current));
				true
			} else {
				false
			}
		}
	}

	impl<T: Config> EstimateNextSessionRotation<T::BlockNumber> for Pallet<T> {
		fn average_session_length() -> T::BlockNumber {
			<Round<T>>::get().length
		}

		fn estimate_current_session_progress(now: T::BlockNumber) -> (Option<Permill>, Weight) {
			let round = <Round<T>>::get();
			let passed_blocks = now.saturating_sub(round.first);

			(
				Some(Permill::from_rational(passed_blocks, round.length)),
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
