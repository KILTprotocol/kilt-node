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

//! # Delegated Election Module.
//!
//! An election module based on direct delegations.
//!
//! ### Term and Round
//!
//! The election happens in _rounds_: every `N` blocks, all previous members are
//! retired and a new set is elected (which may or may not have an intersection
//! with the previous set). Each round lasts for some number of blocks defined
//! by [`Config::TermDuration`]. The words _term_ and _round_ can be used
//! interchangeably in this context.
//!
//! [`Config::TermDuration`] might change during a round. This can shorten or
//! extend the length of the round. The next election round's block number is
//! never stored but rather always checked on the fly. Based on the current
//! block number and [`Config::TermDuration`], the condition `BlockNumber %
//! TermDuration == 0` being satisfied will always trigger a new election round.
//!
//! ### Bonds and Deposits
//!
//! Both voting and being a candidate requires deposits to be taken, in exchange
//! for the data that needs to be kept on-chain. The terms *bond* and *deposit*
//! can be used interchangeably in this context.
//!
//! Bonds will be unreserved only upon adhering to the protocol laws. Failing to
//! do so will cause in the bond to slashed.
//!
//! ### Voting
//!
//! Voters can vote for a limited number of the candidates by providing a list
//! of account ids, bounded by [`MAXIMUM_VOTE`]. Invalid votes (voting for
//! non-candidates) and duplicate votes are ignored during election. Yet, a
//! voter _might_ vote for a future candidate. Voters reserve a bond
//! as they vote. Each vote defines a `value`. This amount is locked from the
//! account of the voter and indicates the weight of the vote. Voters can update
//! their votes at any time by calling `vote()` again. This can update the vote
//! targets (which might update the deposit) or update the vote's stake
//! ([`Voter::stake`]). After a round, votes are kept and might still be valid
//! for further rounds. A voter is responsible for calling `remove_voter` once
//! they are done to have their bond back and remove the lock.
//!
//! See [`Call::vote`], [`Call::remove_voter`].
//!
//! ### Defunct Voter
//!
//! A voter is defunct once all of the candidates that they have voted for are
//! not a valid candidate (as seen further below, members and runners-up are
//! also always candidates). Defunct voters can be removed via a root call
//! ([`Call::clean_defunct_voters`]). Upon being removed, their bond is
//! returned. This is an administrative operation and can be called only by the
//! root origin in the case of state bloat.
//!
//! ### Candidacy and Members
//!
//! Candidates also reserve a bond as they submit candidacy. A candidate can end
//! up in one of the below situations:
//!   - **Members**: A winner is kept as a _member_. They must still have a bond
//!     in reserve and they are automatically counted as a candidate for the
//!     next election. The number of desired members is set by
//!     [`Config::DesiredMembers`].
//!   - **Runner-up**: Runners-up are the best candidates immediately after the
//!     winners. The number of runners up to keep is set by
//!     [`Config::DesiredRunnersUp`]. Runners-up are used, in the same order as
//!     they are elected, as replacements when a candidate is kicked by
//!     [`Call::remove_member`], or when an active member renounces their
//!     candidacy. Runners are automatically counted as a candidate for the next
//!     election.
//!   - **Loser**: Any of the candidate who are not member/runner-up are left as
//!     losers. A loser might be an _outgoing member or runner-up_, meaning that
//!     they are an active member who failed to keep their spot. **An outgoing
//!     candidate/member/runner-up will always lose their bond**.
//!
//! #### Renouncing candidacy.
//!
//! All candidates, elected or not, can renounce their candidacy. A call to
//! [`Call::renounce_candidacy`] will always cause the candidacy bond to be
//! refunded.
//!
//! Note that with the members being the default candidates for the next round
//! and votes persisting in storage, the election system is entirely stable
//! given no further input. This means that if the system has a particular set
//! of candidates `C` and voters `V` that lead to a set of members
//! `M` being elected, as long as `V` and `C` don't remove their candidacy and
//! votes, `M` will keep being re-elected at the end of each round.
//!
//! ### Module Information
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Module`]

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use core::fmt::Debug;
use frame_support::{
	dispatch::WithPostDispatchInfo,
	traits::{
		ChangeMembers, Contains, ContainsLengthBound, Currency, CurrencyToVote, Get, InitializeMembers, LockIdentifier,
		LockableCurrency, OnUnbalanced, ReservableCurrency, SortedMembers, WithdrawReasons,
	},
	weights::Weight,
	BoundedVec,
};
use sp_npos_elections::{ElectionResult, ExtendedBalance};
use sp_runtime::{
	traits::{Saturating, StaticLookup, Zero},
	DispatchError, Perbill, RuntimeDebug,
};
use sp_std::{cmp::Ordering, fmt, prelude::*};

/// The current storage version.
// TODO: Enable in version > polkadot-v0.9.8
// const STORAGE_VERSION: StorageVersion = StorageVersion::new(4);

/// The maximum votes allowed per voter.
pub const MAXIMUM_VOTE: usize = 16;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

/// An indication that the renouncing account currently has which of the below
/// roles.
#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug)]
pub enum Renouncing {
	/// A member is renouncing.
	Member,
	/// A runner-up is renouncing.
	RunnerUp,
	/// A candidate is renouncing, while the given total number of candidates
	/// exists.
	Candidate(#[codec(compact)] u32),
}

// A vote.
#[derive(Encode, Decode, Debug, Clone, Default, PartialEq)]
pub struct Vote<AccountId, Balance> {
	who: AccountId,
	amount: Balance,
}

/// An active voter.
#[derive(Encode, Decode, Clone, Default, PartialEq)]
pub struct Voter<AccountId, Balance, MaxVotes: Get<u32> + Debug + Default + Clone + PartialEq> {
	/// The members being backed with their corresponding balance.
	pub votes: BoundedVec<Vote<AccountId, Balance>, MaxVotes>,
	/// The amount of deposit reserved for this vote.
	///
	/// To be unreserved upon removal.
	pub deposit: Balance,
}

// #[cfg(feature = "std")]
// impl<AccountId, Balance, MaxVotes> Debug for Voter<AccountId, Balance,
// MaxVotes> { 	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> fmt::Result
// { 		f.debug_struct("Voter")
// 			.field("votes", &self.votes)
// 			.field("deposit", &self.deposit)
// 			.finish()
// 	}
// }

// An active list of votes.
pub type Votes<T> = BoundedVec<Vote<<T as frame_system::Config>::AccountId, BalanceOf<T>>, <T as Config>::MaxVotes>;

/// A holder of a seat as either a member or a runner-up.
#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq)]
pub struct SeatHolder<AccountId, Balance> {
	/// The holder.
	pub who: AccountId,
	/// The total backing stake.
	pub stake: Balance,
	/// The amount of deposit held on-chain.
	///
	/// To be unreserved upon renouncing, or slashed upon being a loser.
	pub deposit: Balance,
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::{pallet_prelude::*, WeightInfo};
	use sp_runtime::SaturatedConversion;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Identifier for the elections-phragmen pallet's lock
		#[pallet::constant]
		type PalletId: Get<LockIdentifier>;

		/// The currency that people are electing with.
		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>
			+ ReservableCurrency<Self::AccountId>;

		/// What to do when the members change.
		type ChangeMembers: ChangeMembers<Self::AccountId>;

		/// What to do with genesis members
		type InitializeMembers: InitializeMembers<Self::AccountId>;

		/// Convert a balance into a number used for election calculation.
		/// This must fit into a `u64` but is allowed to be sensibly lossy.
		type CurrencyToVote: CurrencyToVote<BalanceOf<Self>>;

		/// How much should be locked up in order to submit one's candidacy.
		#[pallet::constant]
		type CandidacyBond: Get<BalanceOf<Self>>;

		/// Base deposit associated with voting.
		///
		/// This should be sensibly high to economically ensure the pallet
		/// cannot be attacked by creating a gigantic number of votes.
		#[pallet::constant]
		type VotingBondBase: Get<BalanceOf<Self>>;

		/// The amount of bond that need to be locked for each vote (32 bytes).
		#[pallet::constant]
		type VotingBondFactor: Get<BalanceOf<Self>>;

		/// Handler for the unbalanced reduction when a candidate has lost (and
		/// is not a runner-up)
		type LoserCandidate: OnUnbalanced<NegativeImbalanceOf<Self>>;

		/// Handler for the unbalanced reduction when a member has been kicked.
		type KickedMember: OnUnbalanced<NegativeImbalanceOf<Self>>;

		/// Number of members to elect.
		#[pallet::constant]
		type DesiredMembers: Get<u32>;

		/// Number of runners_up to keep.
		#[pallet::constant]
		type DesiredRunnersUp: Get<u32>;

		/// How long each seat is kept. This defines the next block number at
		/// which an election round will happen. If set to zero, no elections
		/// are ever triggered and the module will be in passive mode.
		#[pallet::constant]
		type TermDuration: Get<Self::BlockNumber>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		///
		type MaxVotes: Get<u32> + Debug + Default + Clone + PartialEq;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	// TODO: Enable later
	// #[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// What to do at the end of each block.
		///
		/// Checks if an election needs to happen or not.
		fn on_initialize(n: T::BlockNumber) -> Weight {
			let term_duration = T::TermDuration::get();
			if !term_duration.is_zero() && (n % term_duration).is_zero() {
				// FIXME: Replace with another algorithm
				// Self::do_phragmen()
				0
			} else {
				0
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Vote for a set of candidates for the upcoming round of election.
		/// This can be called to set the initial votes, or update already
		/// existing votes.
		///
		/// Upon initial voting, `value` units of `who`'s balance is locked and
		/// a deposit amount is reserved. The deposit is based on the number of
		/// votes and can be updated over time.
		///
		/// The `votes` should:
		///   - not be empty.
		///   - be less than the number of possible candidates. Note that all
		///     current members and runners-up are also automatically candidates
		///     for the next round.
		///
		/// If `value` is more than `who`'s total balance, then the maximum of
		/// the two is used.
		///
		/// The dispatch origin of this call must be signed.
		///
		/// ### Warning
		///
		/// It is the responsibility of the caller to **NOT** place all of their
		/// balance into the lock and keep some for further operations.
		///
		/// # <weight>
		/// We assume the maximum weight among all 3 cases: vote_equal,
		/// vote_more and vote_less. # </weight>
		// TODO: Potentially rewrite
		// #[pallet::weight(
		// 	T::WeightInfo::vote_more(votes.len() as u32)
		// 	.max(T::WeightInfo::vote_less(votes.len() as u32))
		// 	.max(T::WeightInfo::vote_equal(votes.len() as u32))
		// )]
		#[pallet::weight(1)]
		pub fn vote(
			origin: OriginFor<T>,
			votes: Votes<T>,
			#[pallet::compact] value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(!votes.len().is_zero(), Error::<T>::NoVotes);
			ensure!(
				votes.len().saturated_into::<u32>() <= T::MaxVotes::get(),
				Error::<T>::TooManyVotes
			);

			// Get sum of vote
			let value: BalanceOf<T> = votes
				.clone()
				.iter()
				.fold(BalanceOf::<T>::zero(), |acc, Vote { amount, .. }| {
					acc.saturating_add(*amount)
				});
			ensure!(value > T::Currency::minimum_balance(), Error::<T>::LowBalance);
			ensure!(
				value <= T::Currency::total_balance(&who),
				Error::<T>::InsufficientVoterFunds
			);

			// Reserve bond.
			let new_deposit = Self::deposit_of(votes.len());
			let Voter {
				deposit: old_deposit, ..
			} = <Voting<T>>::get(&who);
			match new_deposit.cmp(&old_deposit) {
				Ordering::Greater => {
					// Must reserve a bit more.
					let to_reserve = new_deposit - old_deposit;
					T::Currency::reserve(&who, to_reserve).map_err(|_| Error::<T>::UnableToPayBond)?;
				}
				Ordering::Equal => {}
				Ordering::Less => {
					// Must unreserve a bit.
					let to_unreserve = old_deposit - new_deposit;
					let _remainder = T::Currency::unreserve(&who, to_unreserve);
					debug_assert!(_remainder.is_zero());
				}
			};

			// Amount to be locked up.
			T::Currency::set_lock(T::PalletId::get(), &who, value, WithdrawReasons::all());

			Voting::<T>::insert(
				&who,
				Voter {
					votes,
					deposit: new_deposit,
				},
			);
			Ok(None.into())
		}

		/// Remove `origin` as a voter.
		///
		/// This removes the lock and returns the deposit.
		///
		/// The dispatch origin of this call must be signed and be a voter.
		// #[pallet::weight(T::WeightInfo::remove_voter())]
		#[pallet::weight(1)]
		pub fn remove_voter(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_voter(&who), Error::<T>::MustBeVoter);
			Self::do_remove_voter(&who);
			Ok(None.into())
		}

		/// Submit oneself for candidacy. A fixed amount of deposit is recorded.
		///
		/// All candidates are wiped at the end of the term. They either become
		/// a member/runner-up, or leave the system while their deposit is
		/// slashed.
		///
		/// The dispatch origin of this call must be signed.
		///
		/// ### Warning
		///
		/// Even if a candidate ends up being a member, they must call
		/// [`Call::renounce_candidacy`] to get their deposit back. Losing the
		/// spot in an election will always lead to a slash.
		///
		/// # <weight>
		/// The number of current candidates must be provided as witness data.
		/// # </weight>
		// #[pallet::weight(T::WeightInfo::submit_candidacy(*candidate_count))]
		#[pallet::weight(1)]
		pub fn submit_candidacy(
			origin: OriginFor<T>,
			#[pallet::compact] candidate_count: u32,
		) -> DispatchResultWithPostInfo {
			// TODO: Potentially rewrite
			let who = ensure_signed(origin)?;

			let actual_count = <Candidates<T>>::decode_len().unwrap_or(0);
			ensure!(actual_count as u32 <= candidate_count, Error::<T>::InvalidWitnessData);

			let index = Self::is_candidate(&who).err().ok_or(Error::<T>::DuplicatedCandidate)?;

			ensure!(!Self::is_member(&who), Error::<T>::MemberSubmit);
			ensure!(!Self::is_runner_up(&who), Error::<T>::RunnerUpSubmit);

			T::Currency::reserve(&who, T::CandidacyBond::get()).map_err(|_| Error::<T>::InsufficientCandidateFunds)?;

			<Candidates<T>>::mutate(|c| c.insert(index, (who, T::CandidacyBond::get())));
			Ok(None.into())
		}

		/// Renounce one's intention to be a candidate for the next election
		/// round. 3 potential outcomes exist:
		///
		/// - `origin` is a candidate and not elected in any set. In this case,
		///   the deposit is unreserved, returned and origin is removed as a
		///   candidate.
		/// - `origin` is a current runner-up. In this case, the deposit is
		///   unreserved, returned and origin is removed as a runner-up.
		/// - `origin` is a current member. In this case, the deposit is
		///   unreserved and origin is removed as a member, consequently not
		///   being a candidate for the next round anymore. Similar to
		///   [`remove_member`](Self::remove_member), if replacement runners
		///   exists, they are immediately used. If the prime is renouncing,
		///   then no prime will exist until the next round.
		///
		/// The dispatch origin of this call must be signed, and have one of the
		/// above roles.
		///
		/// # <weight>
		/// The type of renouncing must be provided as witness data.
		/// # </weight>
		// #[pallet::weight(match *renouncing {
		// 	Renouncing::Candidate(count) => T::WeightInfo::renounce_candidacy_candidate(count),
		// 	Renouncing::Member => T::WeightInfo::renounce_candidacy_members(),
		// 	Renouncing::RunnerUp => T::WeightInfo::renounce_candidacy_runners_up(),
		// })]
		#[pallet::weight(1)]
		pub fn renounce_candidacy(origin: OriginFor<T>, renouncing: Renouncing) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			match renouncing {
				Renouncing::Member => {
					let _ = Self::remove_and_replace_member(&who, false).map_err(|_| Error::<T>::InvalidRenouncing)?;
					Self::deposit_event(Event::Renounced(who));
				}
				Renouncing::RunnerUp => {
					<RunnersUp<T>>::try_mutate::<_, Error<T>, _>(|runners_up| {
						let index = runners_up
							.iter()
							.position(|SeatHolder { who: r, .. }| r == &who)
							.ok_or(Error::<T>::InvalidRenouncing)?;
						// can't fail anymore.
						let SeatHolder { deposit, .. } = runners_up.remove(index);
						let _remainder = T::Currency::unreserve(&who, deposit);
						debug_assert!(_remainder.is_zero());
						Self::deposit_event(Event::Renounced(who));
						Ok(())
					})?;
				}
				Renouncing::Candidate(count) => {
					<Candidates<T>>::try_mutate::<_, Error<T>, _>(|candidates| {
						ensure!(count >= candidates.len() as u32, Error::<T>::InvalidWitnessData);
						let index = candidates
							.binary_search_by(|(c, _)| c.cmp(&who))
							.map_err(|_| Error::<T>::InvalidRenouncing)?;
						let (_removed, deposit) = candidates.remove(index);
						let _remainder = T::Currency::unreserve(&who, deposit);
						debug_assert!(_remainder.is_zero());
						Self::deposit_event(Event::Renounced(who));
						Ok(())
					})?;
				}
			};
			Ok(None.into())
		}

		/// Remove a particular member from the set. This is effective
		/// immediately and the bond of the outgoing member is slashed.
		///
		/// If a runner-up is available, then the best runner-up will be removed
		/// and replaces the outgoing member. Otherwise, a new phragmen election
		/// is started.
		///
		/// The dispatch origin of this call must be root.
		///
		/// Note that this does not affect the designated block number of the
		/// next election.
		///
		/// # <weight>
		/// If we have a replacement, we use a small weight. Else, since this is
		/// a root call and will go into phragmen, we assume full block for now.
		/// # </weight>
		// #[pallet::weight(if *has_replacement {
		// 	T::WeightInfo::remove_member_with_replacement()
		// } else {
		// 	T::BlockWeights::get().max_block
		// })]
		#[pallet::weight(1)]
		pub fn remove_member(
			origin: OriginFor<T>,
			who: <T::Lookup as StaticLookup>::Source,
			has_replacement: bool,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			let who = T::Lookup::lookup(who)?;

			let will_have_replacement = <RunnersUp<T>>::decode_len().map_or(false, |l| l > 0);
			if will_have_replacement != has_replacement {
				// In both cases, we will change more weight than need. Refund and abort.
				return Err(Error::<T>::InvalidReplacement.with_weight(
					// refund. The weight value comes from a benchmark which is special to this.
					// TODO: Enable  after benchmarks
					// T::WeightInfo::remove_member_wrong_refund(),
					0,
				));
			}

			let had_replacement = Self::remove_and_replace_member(&who, true)?;
			debug_assert_eq!(has_replacement, had_replacement);
			Self::deposit_event(Event::MemberKicked(who.clone()));

			// FIXME: Replace with simple election algo
			// if !had_replacement { // 	Self::do_phragmen(); // }

			// no refund needed.
			Ok(None.into())
		}

		/// Clean all voters who are defunct (i.e. they do not serve any purpose
		/// at all). The deposit of the removed voters are returned.
		///
		/// This is an root function to be used only for cleaning the state.
		///
		/// The dispatch origin of this call must be root.
		///
		/// # <weight>
		/// The total number of voters and those that are defunct must be
		/// provided as witness data. # </weight>
		// #[pallet::weight(T::WeightInfo::clean_defunct_voters(*_num_voters, *_num_defunct))]
		#[pallet::weight(1)]
		pub fn clean_defunct_voters(
			origin: OriginFor<T>,
			_num_voters: u32,
			_num_defunct: u32,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_root(origin)?;
			<Voting<T>>::iter()
				.filter(|(_, x)| {
					let candidates = x
						.votes
						.clone()
						.into_inner()
						.into_iter()
						.map(|Vote { who, .. }| who)
						.collect::<Vec<T::AccountId>>();
					Self::is_defunct_voter(&candidates)
				})
				.for_each(|(dv, _)| Self::do_remove_voter(&dv));

			Ok(None.into())
		}
	}

	#[pallet::event]
	#[pallet::metadata(
		<T as frame_system::Config>::AccountId = "AccountId",
		BalanceOf<T> = "Balance",
		Vec<(<T as frame_system::Config>::AccountId, BalanceOf<T>)> = "Vec<(AccountId, Balance)>",
	)]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new term with \[new_members\]. This indicates that enough
		/// candidates existed to run the election, not that enough have has
		/// been elected. The inner value must be examined for this purpose. A
		/// `NewTerm(\[\])` indicates that some candidates got their bond
		/// slashed and none were elected, whilst `EmptyTerm` means that no
		/// candidates existed to begin with.
		NewTerm(Vec<(<T as frame_system::Config>::AccountId, BalanceOf<T>)>),
		/// No (or not enough) candidates existed for this round. This is
		/// different from `NewTerm(\[\])`. See the description of `NewTerm`.
		EmptyTerm,
		/// Internal error happened while trying to perform election.
		ElectionError,
		/// A \[member\] has been removed. This should always be followed by
		/// either `NewTerm` or `EmptyTerm`.
		MemberKicked(<T as frame_system::Config>::AccountId),
		/// Someone has renounced their candidacy.
		Renounced(<T as frame_system::Config>::AccountId),
		/// A \[candidate\] was slashed by \[amount\] due to failing to obtain a
		/// seat as member or runner-up.
		///
		/// Note that old members and runners-up are also candidates.
		CandidateSlashed(<T as frame_system::Config>::AccountId, BalanceOf<T>),
		/// A \[seat holder\] was slashed by \[amount\] by being forcefully
		/// removed from the set.
		SeatHolderSlashed(<T as frame_system::Config>::AccountId, BalanceOf<T>),
	}

	#[deprecated(note = "use `Event` instead")]
	pub type RawEvent<T> = Event<T>;

	#[pallet::error]
	pub enum Error<T> {
		/// Cannot vote when no candidates or members exist.
		UnableToVote,
		/// Must vote for at least one candidate.
		NoVotes,
		/// Cannot vote more than candidates.
		TooManyVotes,
		/// Cannot vote more than maximum allowed.
		MaximumVotesExceeded,
		/// Cannot vote with stake less than minimum balance.
		LowBalance,
		/// Voter can not pay voting bond.
		UnableToPayBond,
		/// Must be a voter.
		MustBeVoter,
		/// Cannot report self.
		ReportSelf,
		/// Duplicated candidate submission.
		DuplicatedCandidate,
		/// Member cannot re-submit candidacy.
		MemberSubmit,
		/// Runner cannot re-submit candidacy.
		RunnerUpSubmit,
		/// Candidate does not have enough funds.
		InsufficientCandidateFunds,
		/// Voter does not have enough funds.
		InsufficientVoterFunds,
		/// Not a member.
		NotMember,
		/// The provided count of number of candidates is incorrect.
		InvalidWitnessData,
		/// The provided count of number of votes is incorrect.
		InvalidVoteCount,
		/// The renouncing origin presented a wrong `Renouncing` parameter.
		InvalidRenouncing,
		/// Prediction regarding replacement after member removal is wrong.
		InvalidReplacement,
	}

	/// The current elected members.
	///
	/// Invariant: Always sorted based on account id.
	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T: Config> = StorageValue<_, Vec<SeatHolder<T::AccountId, BalanceOf<T>>>, ValueQuery>;

	/// The current reserved runners-up.
	///
	/// Invariant: Always sorted based on rank (worse to best). Upon removal of
	/// a member, the last (i.e. _best_) runner-up will be replaced.
	#[pallet::storage]
	#[pallet::getter(fn runners_up)]
	pub type RunnersUp<T: Config> = StorageValue<_, Vec<SeatHolder<T::AccountId, BalanceOf<T>>>, ValueQuery>;

	/// The present candidate list. A current member or runner-up can never
	/// enter this vector and is always implicitly assumed to be a candidate.
	///
	/// Second element is the deposit.
	///
	/// Invariant: Always sorted based on account id.
	#[pallet::storage]
	#[pallet::getter(fn candidates)]
	pub type Candidates<T: Config> = StorageValue<_, Vec<(T::AccountId, BalanceOf<T>)>, ValueQuery>;

	/// The total number of vote rounds that have happened, excluding the
	/// upcoming one.
	#[pallet::storage]
	#[pallet::getter(fn election_rounds)]
	pub type ElectionRounds<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Votes and locked stake of a particular voter.
	///
	/// TWOX-NOTE: SAFE as `AccountId` is a crypto hash.
	#[pallet::storage]
	#[pallet::getter(fn voting)]
	pub type Voting<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Voter<T::AccountId, BalanceOf<T>, T::MaxVotes>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub members: Vec<(T::AccountId, BalanceOf<T>)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				members: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// TODO: Potentially rewrite
			// assert!(
			// 	self.members.len() as u32 <= T::DesiredMembers::get(),
			// 	"Cannot accept more than DesiredMembers genesis member",
			// );
			// let members = self
			// 	.members
			// 	.iter()
			// 	.map(|(ref member, ref stake)| {
			// 		// make sure they have enough stake.
			// 		assert!(
			// 			T::Currency::free_balance(member) >= *stake,
			// 			"Genesis member does not have enough stake.",
			// 		);

			// 		// Note: all members will only vote for themselves, hence they
			// must be given 		// exactly their own stake as total backing. Any
			// sane election should behave as 		// such. Nonetheless, stakes will
			// be updated for term 1 onwards according to the 		// election.
			// 		Members::<T>::mutate(|members| match members.binary_search_by(|m|
			// m.who.cmp(member)) { 			Ok(_) => panic!("Duplicate member in
			// elections-phragmen genesis: {}", member), 			Err(pos) =>
			// members.insert( 				pos,
			// 				SeatHolder {
			// 					who: member.clone(),
			// 					stake: *stake,
			// 					deposit: Zero::zero(),
			// 				},
			// 			),
			// 		});

			// 		// set self-votes to make persistent. Genesis voters don't have
			// any bond, nor do 		// they have any lock. NOTE: this means that we
			// will still try to remove a lock 		// once this genesis voter is
			// removed, and for now it is okay because 		// remove_lock is noop if
			// lock is not there. 		<Voting<T>>::insert(
			// 			&member,
			// 			Voter {
			// 				votes: vec![member.clone()],
			// 				stake: *stake,
			// 				deposit: Zero::zero(),
			// 			},
			// 		);

			// 		member.clone()
			// 	})
			// 	.collect::<Vec<T::AccountId>>();

			// // report genesis members to upstream, if any.
			// T::InitializeMembers::initialize_members(&members);
		}
	}
}

impl<T: Config> Pallet<T> {
	/// The deposit value of `count` votes.
	fn deposit_of(count: usize) -> BalanceOf<T> {
		T::VotingBondBase::get().saturating_add(T::VotingBondFactor::get().saturating_mul((count as u32).into()))
	}

	/// Attempts to remove a member `who`. If a runner-up exists, it is used as
	/// the replacement.
	///
	/// Returns:
	///
	/// - `Ok(true)` if the member was removed and a replacement was found.
	/// - `Ok(false)` if the member was removed and but no replacement was
	///   found.
	/// - `Err(_)` if the member was no found.
	///
	/// Both `Members` and `RunnersUp` storage is updated accordingly.
	/// `T::ChangeMember` is called if needed. If `slash` is true, the deposit
	/// of the potentially removed member is slashed, else, it is unreserved.
	///
	/// ### Note: Prime preservation
	///
	/// This function attempts to preserve the prime. If the removed members is
	/// not the prime, it is set again via [`Config::ChangeMembers`].
	fn remove_and_replace_member(who: &T::AccountId, slash: bool) -> Result<bool, DispatchError> {
		// closure will return:
		// - `Ok(Option(replacement))` if member was removed and replacement was
		//   replaced.
		// - `Ok(None)` if member was removed but no replacement was found
		// - `Err(_)` if who is not a member.
		let maybe_replacement = <Members<T>>::try_mutate::<_, Error<T>, _>(|members| {
			let remove_index = members
				.binary_search_by(|m| m.who.cmp(who))
				.map_err(|_| Error::<T>::NotMember)?;
			// we remove the member anyhow, regardless of having a runner-up or not.
			let removed = members.remove(remove_index);

			// slash or unreserve
			if slash {
				let (imbalance, _remainder) = T::Currency::slash_reserved(who, removed.deposit);
				debug_assert!(_remainder.is_zero());
				T::LoserCandidate::on_unbalanced(imbalance);
				Self::deposit_event(Event::SeatHolderSlashed(who.clone(), removed.deposit));
			} else {
				T::Currency::unreserve(who, removed.deposit);
			}

			let maybe_next_best = <RunnersUp<T>>::mutate(|r| r.pop()).map(|next_best| {
				// defensive-only: Members and runners-up are disjoint. This will always be err
				// and give us an index to insert.
				if let Err(index) = members.binary_search_by(|m| m.who.cmp(&next_best.who)) {
					members.insert(index, next_best.clone());
				} else {
					// overlap. This can never happen. If so, it seems like our intended replacement
					// is already a member, so not much more to do.
					log::error!(
						target: "runtime::elections",
						"A member seems to also be a runner-up.",
					);
				}
				next_best
			});
			Ok(maybe_next_best)
		})?;

		let remaining_member_ids_sorted = Self::members().into_iter().map(|x| x.who.clone()).collect::<Vec<_>>();
		let outgoing = &[who.clone()];
		let maybe_current_prime = T::ChangeMembers::get_prime();
		let return_value = match maybe_replacement {
			// member ids are already sorted, other two elements have one item.
			Some(incoming) => {
				T::ChangeMembers::change_members_sorted(&[incoming.who], outgoing, &remaining_member_ids_sorted[..]);
				true
			}
			None => {
				T::ChangeMembers::change_members_sorted(&[], outgoing, &remaining_member_ids_sorted[..]);
				false
			}
		};

		// if there was a prime before and they are not the one being removed, then set
		// // them again.
		if let Some(current_prime) = maybe_current_prime {
			if &current_prime != who {
				T::ChangeMembers::set_prime(Some(current_prime));
			}
		}

		Ok(return_value)
	}

	/// Check if `who` is a candidate. It returns the insert index if the
	/// element does not exists as an error.
	fn is_candidate(who: &T::AccountId) -> Result<(), usize> {
		Self::candidates().binary_search_by(|c| c.0.cmp(who)).map(|_| ())
	}

	/// Check if `who` is a voter. It may or may not be a _current_ one.
	fn is_voter(who: &T::AccountId) -> bool {
		Voting::<T>::contains_key(who)
	}

	/// Check if `who` is currently an active member.
	fn is_member(who: &T::AccountId) -> bool {
		Self::members().binary_search_by(|m| m.who.cmp(who)).is_ok()
	}

	/// Check if `who` is currently an active runner-up.
	fn is_runner_up(who: &T::AccountId) -> bool {
		Self::runners_up().iter().position(|r| &r.who == who).is_some()
	}

	/// Get the members' account ids.
	fn members_ids() -> Vec<T::AccountId> {
		Self::members()
			.into_iter()
			.map(|m| m.who)
			.collect::<Vec<T::AccountId>>()
	}

	/// Get a concatenation of previous members and runners-up and their
	/// deposits.
	///
	/// These accounts are essentially treated as candidates.
	fn implicit_candidates_with_deposit() -> Vec<(T::AccountId, BalanceOf<T>)> {
		// invariant: these two are always without duplicates.
		Self::members()
			.into_iter()
			.map(|m| (m.who, m.deposit))
			.chain(Self::runners_up().into_iter().map(|r| (r.who, r.deposit)))
			.collect::<Vec<_>>()
	}

	/// Check if `votes` will correspond to a defunct voter. As no origin is
	/// part of the inputs, this function does not check the origin at all.
	///
	/// O(NLogM) with M candidates and `who` having voted for `N` of them.
	/// Reads Members, RunnersUp, Candidates and Voting(who) from database.
	fn is_defunct_voter(votes: &[T::AccountId]) -> bool {
		votes
			.iter()
			.all(|v| !Self::is_member(v) && !Self::is_runner_up(v) && !Self::is_candidate(v).is_ok())
	}

	/// Remove a certain someone as a voter.
	fn do_remove_voter(who: &T::AccountId) {
		let Voter { deposit, .. } = <Voting<T>>::take(who);

		// remove storage, lock and unreserve.
		T::Currency::remove_lock(T::PalletId::get(), who);

		// NOTE: we could check the deposit amount before removing and skip if zero, but
		// it will be a noop anyhow.
		let _remainder = T::Currency::unreserve(who, deposit);
		debug_assert!(_remainder.is_zero());
	}

	// FIXME: Replace with more simple version. Might involve adding more storage to
	// the pallet to efficiently calculate the top candidates (similar to staking
	// pallet).
	//
	// /// Run the phragmen
	// election with all required side processes and state 	/// updates, if election
	// succeeds. Else, it will emit an `ElectionError` 	/// event.
	// 	///
	// 	/// Calls the appropriate [`ChangeMembers`] function variant internally.
	// 	fn do_phragmen() -> Weight {
	// 		let desired_seats = T::DesiredMembers::get() as usize;
	// 		let desired_runners_up = T::DesiredRunnersUp::get() as usize;
	// 		let num_to_elect = desired_runners_up + desired_seats;

	// 		let mut candidates_and_deposit = Self::candidates();
	// 		// add all the previous members and runners-up as candidates as well.
	// 		candidates_and_deposit.append(&mut Self::implicit_candidates_with_deposit());

	// 		if candidates_and_deposit.len().is_zero() {
	// 			Self::deposit_event(Event::EmptyTerm);
	// 			return T::DbWeight::get().reads(5);
	// 		}

	// 		// All of the new winners that come out of phragmen will thus have a deposit
	// 		// recorded.
	// 		let candidate_ids = candidates_and_deposit
	// 			.iter()
	// 			.map(|(x, _)| x)
	// 			.cloned()
	// 			.collect::<Vec<_>>();

	// 		// helper closures to deal with balance/stake.
	// 		let total_issuance = T::Currency::total_issuance();
	// 		let to_votes = |b: BalanceOf<T>| T::CurrencyToVote::to_vote(b,
	// total_issuance); 		let to_balance = |e: ExtendedBalance|
	// T::CurrencyToVote::to_currency(e, total_issuance);

	// 		let mut num_edges: u32 = 0;
	// 		// used for prime election.
	// 		let voters_and_stakes = Voting::<T>::iter()
	// 			.map(|(voter, Voter { stake, votes, .. })| (voter, stake, votes))
	// 			.collect::<Vec<_>>();
	// 		// used for phragmen.
	// 		let voters_and_votes = voters_and_stakes
	// 			.iter()
	// 			.cloned()
	// 			.map(|(voter, stake, votes)| {
	// 				num_edges = num_edges.saturating_add(votes.len() as u32);
	// 				(voter, to_votes(stake), votes)
	// 			})
	// 			.collect::<Vec<_>>();

	// 		let weight_candidates = candidates_and_deposit.len() as u32;
	// 		let weight_voters = voters_and_votes.len() as u32;
	// 		let weight_edges = num_edges;
	// 		let _ = sp_npos_elections::seq_phragmen::<T::AccountId, Perbill>(
	// 			num_to_elect,
	// 			candidate_ids,
	// 			voters_and_votes.clone(),
	// 			None,
	// 		)
	// 		.map(
	// 			|ElectionResult {
	// 			     winners,
	// 			     assignments: _,
	// 			 }| {
	// 				// this is already sorted by id.
	// 				let old_members_ids_sorted = <Members<T>>::take()
	// 					.into_iter()
	// 					.map(|m| m.who)
	// 					.collect::<Vec<T::AccountId>>();
	// 				// this one needs a sort by id.
	// 				let mut old_runners_up_ids_sorted = <RunnersUp<T>>::take()
	// 					.into_iter()
	// 					.map(|r| r.who)
	// 					.collect::<Vec<T::AccountId>>();
	// 				old_runners_up_ids_sorted.sort();

	// 				// filter out those who end up with no backing stake.
	// 				let mut new_set_with_stake = winners
	// 					.into_iter()
	// 					.filter_map(|(m, b)| if b.is_zero() { None } else { Some((m, to_balance(b)))
	// }) 					.collect::<Vec<(T::AccountId, BalanceOf<T>)>>();

	// 				// OPTIMIZATION NOTE: we could bail out here if `new_set.len() == 0`. There
	// 				// isn't much left to do. Yet, re-arranging the code would require
	// duplicating 				// the slashing of exposed candidates, cleaning any previous
	// members, and so on. 				// For now, in favor of readability and veracity, we keep
	// it simple.

	// 				// split new set into winners and runners up.
	// 				let split_point = desired_seats.min(new_set_with_stake.len());
	// 				let mut new_members_sorted_by_id =
	// new_set_with_stake.drain(..split_point).collect::<Vec<_>>();
	// 				new_members_sorted_by_id.sort_by(|i, j| i.0.cmp(&j.0));

	// 				// all the rest will be runners-up
	// 				new_set_with_stake.reverse();
	// 				let new_runners_up_sorted_by_rank = new_set_with_stake;
	// 				let mut new_runners_up_ids_sorted = new_runners_up_sorted_by_rank
	// 					.iter()
	// 					.map(|(r, _)| r.clone())
	// 					.collect::<Vec<_>>();
	// 				new_runners_up_ids_sorted.sort();

	// 				// Now we select a prime member using a [Borda
	// 				// count](https://en.wikipedia.org/wiki/Borda_count). We weigh everyone's vote for
	// 				// that new member by a multiplier based on the order of the votes. i.e. the
	// 				// first person a voter votes for gets a 16x multiplier, the next person gets
	// a 				// 15x multiplier, an so on... (assuming `MAXIMUM_VOTE` = 16)
	// 				let mut prime_votes = new_members_sorted_by_id
	// 					.iter()
	// 					.map(|c| (&c.0, BalanceOf::<T>::zero()))
	// 					.collect::<Vec<_>>();
	// 				for (_, stake, votes) in voters_and_stakes.into_iter() {
	// 					for (vote_multiplier, who) in votes
	// 						.iter()
	// 						.enumerate()
	// 						.map(|(vote_position, who)| ((MAXIMUM_VOTE - vote_position) as u32, who))
	// 					{
	// 						if let Ok(i) = prime_votes.binary_search_by_key(&who, |k| k.0) {
	// 							prime_votes[i].1 = prime_votes[i]
	// 								.1
	// 								.saturating_add(stake.saturating_mul(vote_multiplier.into()));
	// 						}
	// 					}
	// 				}
	// 				// We then select the new member with the highest weighted stake. In the case
	// of 				// a tie, the last person in the list with the tied score is selected.
	// This is 				// the person with the "highest" account id based on the sort above.
	// 				let prime = prime_votes.into_iter().max_by_key(|x| x.1).map(|x| x.0.clone());

	// 				// new_members_sorted_by_id is sorted by account id.
	// 				let new_members_ids_sorted = new_members_sorted_by_id
	// 					.iter()
	// 					.map(|(m, _)| m.clone())
	// 					.collect::<Vec<T::AccountId>>();

	// 				// report member changes. We compute diff because we need the outgoing list.
	// 				let (incoming, outgoing) =
	// 					T::ChangeMembers::compute_members_diff_sorted(&new_members_ids_sorted,
	// &old_members_ids_sorted); 				T::ChangeMembers::change_members_sorted(&incoming,
	// &outgoing, &new_members_ids_sorted); 				T::ChangeMembers::set_prime(prime);

	// 				// All candidates/members/runners-up who are no longer retaining a position
	// as a 				// seat holder will lose their bond.
	// 				candidates_and_deposit.iter().for_each(|(c, d)| {
	// 					if new_members_ids_sorted.binary_search(c).is_err()
	// 						&& new_runners_up_ids_sorted.binary_search(c).is_err()
	// 					{
	// 						let (imbalance, _) = T::Currency::slash_reserved(c, *d);
	// 						T::LoserCandidate::on_unbalanced(imbalance);
	// 						Self::deposit_event(Event::CandidateSlashed(c.clone(), *d));
	// 					}
	// 				});

	// 				// write final values to storage.
	// 				let deposit_of_candidate = |x: &T::AccountId| -> BalanceOf<T> {
	// 					// defensive-only. This closure is used against the new members and new
	// 					// runners-up, both of which are phragmen winners and thus must have deposit.
	// 					candidates_and_deposit
	// 						.iter()
	// 						.find_map(|(c, d)| if c == x { Some(*d) } else { None })
	// 						.unwrap_or_default()
	// 				};
	// 				// fetch deposits from the one recorded one. This will make sure that a
	// 				// candidate who submitted candidacy before a change to candidacy deposit
	// will 				// have the correct amount recorded.
	// 				<Members<T>>::put(
	// 					new_members_sorted_by_id
	// 						.iter()
	// 						.map(|(who, stake)| SeatHolder {
	// 							deposit: deposit_of_candidate(&who),
	// 							who: who.clone(),
	// 							stake: stake.clone(),
	// 						})
	// 						.collect::<Vec<_>>(),
	// 				);
	// 				<RunnersUp<T>>::put(
	// 					new_runners_up_sorted_by_rank
	// 						.into_iter()
	// 						.map(|(who, stake)| SeatHolder {
	// 							deposit: deposit_of_candidate(&who),
	// 							who,
	// 							stake,
	// 						})
	// 						.collect::<Vec<_>>(),
	// 				);

	// 				// clean candidates.
	// 				<Candidates<T>>::kill();

	// 				Self::deposit_event(Event::NewTerm(new_members_sorted_by_id));
	// 				<ElectionRounds<T>>::mutate(|v| *v += 1);
	// 			},
	// 		)
	// 		.map_err(|e| {
	// 			log::error!(
	// 				target: "runtime::elections-phragmen",
	// 				"Failed to run election [{:?}].",
	// 				e,
	// 			);
	// 			Self::deposit_event(Event::ElectionError);
	// 		});

	// 		T::WeightInfo::election_phragmen(weight_candidates, weight_voters,
	// weight_edges) 	}
	// }
}

impl<T: Config> Contains<T::AccountId> for Pallet<T> {
	fn contains(who: &T::AccountId) -> bool {
		Self::is_member(who)
	}
}

impl<T: Config> SortedMembers<T::AccountId> for Pallet<T> {
	fn contains(who: &T::AccountId) -> bool {
		Self::is_member(who)
	}

	fn sorted_members() -> Vec<T::AccountId> {
		Self::members_ids()
	}

	// TODO: Potentially rewrite
	// A special function to populate members in this pallet for passing Origin
	// checks in runtime benchmarking.
	#[cfg(feature = "runtime-benchmarks")]
	fn add(who: &T::AccountId) {
		Members::<T>::mutate(|members| match members.binary_search_by(|m| m.who.cmp(who)) {
			Ok(_) => (),
			Err(pos) => members.insert(
				pos,
				SeatHolder {
					who: who.clone(),
					..Default::default()
				},
			),
		})
	}
}

impl<T: Config> ContainsLengthBound for Pallet<T> {
	fn min_len() -> usize {
		0
	}

	/// Implementation uses a parameter type so calling is cost-free.
	fn max_len() -> usize {
		T::DesiredMembers::get() as usize
	}
}
