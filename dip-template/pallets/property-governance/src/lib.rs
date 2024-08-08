#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

use frame_support::{
	sp_runtime::{traits::AccountIdConversion, Saturating},
	traits::{Currency, ExistenceRequirement::KeepAlive, OnUnbalanced, ReservableCurrency},
	PalletId,
};

use pallet_assets::Instance1;

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[cfg(feature = "runtime-benchmarks")]
	pub struct AssetHelper;

	#[cfg(feature = "runtime-benchmarks")]
	pub trait BenchmarkHelper<AssetId, T> {
		fn to_asset(i: u32) -> AssetId;
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<T: Config> BenchmarkHelper<AssetId<T>, T> for AssetHelper {
		fn to_asset(i: u32) -> AssetId<T> {
			i.into()
		}
	}

	pub type ProposalIndex = u32;
	pub type ChallengeIndex = u32;

	/// Proposal with the proposal Details.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Proposal<BlockNumber, T: Config> {
		pub proposer: AccountIdOf<T>,
		pub asset_id: u32,
		pub amount: BalanceOf<T>,
		pub created_at: BlockNumber,
		pub proposal_info: BoundedVec<u8, <T as pallet_nfts::Config>::StringLimit>,
	}

	/// Sell proposal with the proposal Details.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct SellProposal<BlockNumber, T: Config> {
		pub proposer: AccountIdOf<T>,
		pub asset_id: u32,
		pub amount: BalanceOf<T>,
		pub created_at: BlockNumber,
	}

	/// Challenge with the challenge Details.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Challenge<BlockNumber, T: Config> {
		pub proposer: AccountIdOf<T>,
		pub asset_id: u32,
		pub created_at: BlockNumber,
		pub state: ChallengeState,
	}

	/// Vote enum.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub enum Vote {
		Yes,
		No,
	}

	/// Challenge state of the challenge voting.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub enum ChallengeState {
		First,
		Second,
		Third,
		Fourth,
	}

	/// Voting stats.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct VoteStats {
		pub yes_votes: u32,
		pub no_votes: u32,
	}

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_nft_marketplace::Config
		+ pallet_property_management::Config
		+ pallet_assets::Config<Instance1>
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type representing the weight of this pallet.
		type WeightInfo: WeightInfo;

		/// The reservable currency type.
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// The amount of time given to vote for a proposal.
		type VotingTime: Get<BlockNumberFor<Self>>;

		/// The maximum amount of votes per block.
		type MaxVotesForBlock: Get<u32>;

		/// Handler for the unbalanced reduction when slashing a letting agent.
		type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

		/// The minimum amount of a letting agent that will be slashed.
		type MinSlashingAmount: Get<BalanceOf<Self>>;

		/// The maximum amount of users who can vote on an ongoing voting.
		type MaxVoter: Get<u32>;

		/// Threshold for challenge votes.
		type Threshold: Get<u32>;

		/// Threshold for high costs challenge votes.
		type HighThreshold: Get<u32>;

		#[cfg(feature = "runtime-benchmarks")]
		type Helper: crate::BenchmarkHelper<
			<Self as pallet_assets::Config<Instance1>>::AssetId,
			Self,
		>;

		/// Proposal amount to be considered a low proposal.
		type LowProposal: Get<BalanceOf<Self>>;

		/// Proposal amount to be considered a high proposal.
		type HighProposal: Get<BalanceOf<Self>>;

		/// The property governance's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Asset id type from pallet assets.
		type AssetId: IsType<<Self as pallet_assets::Config<Instance1>>::AssetId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy;

		/// Multiplier for polkadot js.
		type PolkadotJsMultiplier: Get<BalanceOf<Self>>;
	}

	pub type AssetId<T> = <T as Config>::AssetId;

	/// Number of proposals that have been made.
	#[pallet::storage]
	#[pallet::getter(fn proposal_count)]
	pub(super) type ProposalCount<T> = StorageValue<_, ProposalIndex, ValueQuery>;

	/// Number of Challenges that have been made.
	#[pallet::storage]
	#[pallet::getter(fn challenge_count)]
	pub(super) type ChallengeCount<T> = StorageValue<_, ProposalIndex, ValueQuery>;

	/// Proposals that have been made.
	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub(super) type Proposals<T> =
		StorageMap<_, Blake2_128Concat, ProposalIndex, Proposal<BlockNumberFor<T>, T>, OptionQuery>;

	/// Sell proposals that have been made.
	#[pallet::storage]
	#[pallet::getter(fn sell_proposals)]
	pub(super) type SellProposals<T> = StorageMap<
		_,
		Blake2_128Concat,
		ProposalIndex,
		SellProposal<BlockNumberFor<T>, T>,
		OptionQuery,
	>;

	/// Mapping of challenge index to the challenge info.
	#[pallet::storage]
	#[pallet::getter(fn challenges)]
	pub(super) type Challenges<T> =
		StorageMap<_, Blake2_128Concat, ChallengeIndex, Challenge<BlockNumberFor<T>, T>, OptionQuery>;

	/// Mapping of ongoing votes.
	#[pallet::storage]
	#[pallet::getter(fn ongoing_votes)]
	pub(super) type OngoingVotes<T> =
		StorageMap<_, Blake2_128Concat, ProposalIndex, VoteStats, OptionQuery>;

	/// Mapping from proposal to vector of users who voted.
	#[pallet::storage]
	#[pallet::getter(fn proposal_voter)]
	pub(super) type ProposalVoter<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		ProposalIndex,
		BoundedVec<AccountIdOf<T>, T::MaxVoter>,
		ValueQuery,
	>;

	/// Mapping of ongoing votes about challenges.
	#[pallet::storage]
	#[pallet::getter(fn ongoing_challenge_votes)]
	pub(super) type OngoingChallengeVotes<T> =
		StorageDoubleMap<_, Blake2_128Concat, ChallengeIndex, Blake2_128Concat, ChallengeState, VoteStats, OptionQuery>;

	/// Mapping from challenge to vector of users who voted.
	#[pallet::storage]
	#[pallet::getter(fn challenge_voter)]
	pub(super) type ChallengeVoter<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		ChallengeIndex,
		Blake2_128Concat,
		ChallengeState,
		BoundedVec<AccountIdOf<T>, T::MaxVoter>,
		ValueQuery,
	>;

	/// Stores the project keys and round types ending on a given block for proposal votings.
	#[pallet::storage]
	pub type RoundsExpiring<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BlockNumberFor<T>,
		BoundedVec<ProposalIndex, T::MaxVotesForBlock>,
		ValueQuery,
	>;

	/// Stores the project keys and round types ending on a given block for challenge votings.
	#[pallet::storage]
	#[pallet::getter(fn challenge_rounds_expiring)]
	pub type ChallengeRoundsExpiring<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BlockNumberFor<T>,
		BoundedVec<ChallengeIndex, T::MaxVotesForBlock>,
		ValueQuery,
	>;

	/// Stores the project keys and round types ending on a given block for sell_property votings.
	#[pallet::storage]
	#[pallet::getter(fn sell_property_rounds_expiring)]
	pub type SellPropertyRoundsExpiring<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BlockNumberFor<T>,
		BoundedVec<ChallengeIndex, T::MaxVotesForBlock>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New proposal has been created.
		Proposed { proposal_id: ProposalIndex, asset_id: u32, proposer: AccountIdOf<T> },
		/// A new challenge has been made.
		Challenge { challenge_id: ChallengeIndex, asset_id: u32, proposer: AccountIdOf<T> },
		/// Voted on proposal.
		VotedOnProposal { proposal_id: ProposalIndex, voter: AccountIdOf<T>, vote: Vote },
		/// Voted on challenge.
		VotedOnChallenge { challenge_id: ChallengeIndex, voter: AccountIdOf<T>, vote: Vote },
		/// The proposal has been executed.
		ProposalExecuted { asset_id: u32, amount: BalanceOf<T> },
		/// The agent got slashed.
		AgentSlashed { challenge_id: ChallengeIndex, amount: BalanceOf<T> },
		/// The agent has been changed.
		AgentChanged { challenge_id: ChallengeIndex, asset_id: u32 },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The user is not a property owner and has no permission to propose.
		NoPermission,
		/// There are already too many proposals in the ending block.
		TooManyProposals,
		/// The proposal is not ongoing.
		NotOngoing,
		/// Too many user voted already.
		TooManyVotes,
		/// The user already voted.
		AlreadyVoted,
		/// The assets details could not be found.
		NoAssetFound,
		/// There is no letting agent for this property.
		NoLettingAgentFound,
		/// The pallet has not enough funds.
		NotEnoughFunds,
		/// Error during converting types.
		ConversionError,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: frame_system::pallet_prelude::BlockNumberFor<T>) -> Weight {
			let mut weight = T::DbWeight::get().reads_writes(1, 1);

			let ended_votings = RoundsExpiring::<T>::take(n);
			// checks if there is a voting for a proposal ending in this block.
			ended_votings.iter().for_each(|item| {
				weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
				let voting_results = <OngoingVotes<T>>::take(item);
				let proposals = <Proposals<T>>::take(item);
				if let Some(proposal) = proposals {
					if let Some(voting_result) = voting_results {
						let required_threshold =
							if proposal.amount >= <T as Config>::HighProposal::get() {
								<T as Config>::HighThreshold::get()
							}  else {
								<T as Config>::Threshold::get()
							}; 
						let asset_details = pallet_nft_marketplace::Pallet::<T>::asset_id_details(proposal.asset_id);
						if let Some(asset_details) = asset_details {
							let yes_votes_adjusted = voting_result.yes_votes.saturating_mul(100).saturating_div(asset_details.token_amount);
							let no_votes_adjusted = voting_result.no_votes.saturating_mul(100).saturating_div(asset_details.token_amount);
	
							if yes_votes_adjusted > no_votes_adjusted
								&& required_threshold
									< yes_votes_adjusted.saturating_add(no_votes_adjusted)
							{
								let _ = Self::execute_proposal(proposal);
							}
						}
					}
				}
			});

			let ended_challenge_votings = ChallengeRoundsExpiring::<T>::take(n);
			// checks if there is a voting for an challenge ending in this block.
			ended_challenge_votings.iter().for_each(|item| {
				weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
				let challenge = Self::challenges(item);
				if let Some(mut challenge) = challenge {
					if challenge.state == ChallengeState::Second {
						challenge.state = ChallengeState::Third;
						let vote_stats = VoteStats { yes_votes: 0, no_votes: 0 };
						OngoingChallengeVotes::<T>::insert(item, challenge.state.clone(), vote_stats);
						Challenges::<T>::insert(item, challenge.clone());
						let current_block_number = <frame_system::Pallet<T>>::block_number();
						let expiry_block =
							current_block_number.saturating_add(<T as Config>::VotingTime::get());
						let _ = ChallengeRoundsExpiring::<T>::try_mutate(expiry_block, |keys| {
							keys.try_push(*item).map_err(|_| Error::<T>::TooManyProposals)?;
							Ok::<(), DispatchError>(())
						});
					} 
					else {
						let voting_results = <OngoingChallengeVotes<T>>::take(item, challenge.state.clone());
						if let Some(voting_result) = voting_results {
							let asset_details = pallet_nft_marketplace::Pallet::<T>::asset_id_details(challenge.asset_id);
							if let Some(asset_details) = asset_details {
								let yes_votes_adjusted = voting_result.yes_votes.saturating_mul(100).saturating_div(asset_details.token_amount);
								let no_votes_adjusted = voting_result.no_votes.saturating_mul(100).saturating_div(asset_details.token_amount);
	
								if yes_votes_adjusted > no_votes_adjusted
									&& <T as Config>::Threshold::get()
										< yes_votes_adjusted.saturating_add(no_votes_adjusted)
								{
									if challenge.state == ChallengeState::First {
										challenge.state = ChallengeState::Second;
										Challenges::<T>::insert(item, challenge.clone());
										let current_block_number = <frame_system::Pallet<T>>::block_number();
										let expiry_block =
											current_block_number.saturating_add(<T as Config>::VotingTime::get());
										let _ = ChallengeRoundsExpiring::<T>::try_mutate(expiry_block, |keys| {
											keys.try_push(*item).map_err(|_| Error::<T>::TooManyProposals)?;
											Ok::<(), DispatchError>(())
										});
									} 
									if challenge.state == ChallengeState::Third {
										let _ = Self::slash_letting_agent(*item);
									} 
									if challenge.state == ChallengeState::Fourth {
										let _ = Self::change_letting_agent(*item);
									} 
								} else {
									Challenges::<T>::take(*item);
								}
							}
						}
					}
					
				}
			});
			weight
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates a proposal for a real estate object.
		/// Only the letting agent can propose.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `asset_id`: The asset id of the property.
		/// - `amount`: The amount the letting agent is asking for.
		/// - `data`: The data regarding this proposal.
		///
		/// Emits `Proposed` event when succesfful.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::propose())]
		pub fn propose(
			origin: OriginFor<T>,
			asset_id: u32,
			amount: BalanceOf<T>,
			data: BoundedVec<u8, <T as pallet_nfts::Config>::StringLimit>,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			ensure!(
				pallet_property_management::Pallet::<T>::letting_storage(asset_id)
					.ok_or(Error::<T>::NoLettingAgentFound)?
					== origin.clone(),
				Error::<T>::NoPermission
			);
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let proposal = Proposal {
				proposer: origin.clone(),
				asset_id,
				amount,
				created_at: current_block_number,
				proposal_info: data,
			};

			// Check if the amount is less than LowProposal
			if amount.saturating_mul(
				<T as Config>::PolkadotJsMultiplier::get(),
			) <= <T as Config>::LowProposal::get() {
				// Execute the proposal immediately
				Self::execute_proposal(proposal)?;
				return Ok(());
			}

			let proposal_id = Self::proposal_count().saturating_add(1);
			let expiry_block =
				current_block_number.saturating_add(<T as Config>::VotingTime::get());
			RoundsExpiring::<T>::try_mutate(expiry_block, |keys| {
				keys.try_push(proposal_id).map_err(|_| Error::<T>::TooManyProposals)?;
				Ok::<(), DispatchError>(())
			})?;
			let vote_stats = VoteStats { yes_votes: 0, no_votes: 0 };

			Proposals::<T>::insert(proposal_id, proposal);
			OngoingVotes::<T>::insert(proposal_id, vote_stats);
			ProposalCount::<T>::put(proposal_id);
			Self::deposit_event(Event::Proposed { proposal_id, asset_id, proposer: origin });
			Ok(())
		}

		/// Creates an challenge against the letting agent of the real estate object.
		/// Only one of the owner of the property can propose.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `asset_id`: The asset id of the property.
		///
		/// Emits `Challenge` event when succesfful.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::challenge_against_letting_agent())]
		pub fn challenge_against_letting_agent(
			origin: OriginFor<T>,
			asset_id: u32,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let owner_list = pallet_nft_marketplace::Pallet::<T>::property_owner(asset_id);
			ensure!(owner_list.contains(&origin), Error::<T>::NoPermission);
			ensure!(pallet_property_management::Pallet::<T>::letting_storage(asset_id).is_some(), Error::<T>::NoLettingAgentFound);
			let challenge_id = Self::challenge_count().saturating_add(1);

			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let expiry_block =
				current_block_number.saturating_add(<T as Config>::VotingTime::get());
			let challenge =
				Challenge { proposer: origin.clone(), asset_id, created_at: current_block_number, state: ChallengeState::First };
			ChallengeRoundsExpiring::<T>::try_mutate(expiry_block, |keys| {
				keys.try_push(challenge_id).map_err(|_| Error::<T>::TooManyProposals)?;
				Ok::<(), DispatchError>(())
			})?;
			let vote_stats = VoteStats { yes_votes: 0, no_votes: 0 };
			OngoingChallengeVotes::<T>::insert(challenge_id, challenge.state.clone(), vote_stats);
			Challenges::<T>::insert(challenge_id, challenge);
			ChallengeCount::<T>::put(challenge_id);
			
			Self::deposit_event(Event::Challenge { challenge_id, asset_id, proposer: origin });
			Ok(())
		}

		/// Lets owner of the real estate object vote on a proposal.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `proposal_id`: The index of the proposal.
		/// - `vote`: Must be either a Yes vote or a No vote.
		///
		/// Emits `VotedOnProposal` event when succesfful.
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::vote_on_proposal())]
		pub fn vote_on_proposal(
			origin: OriginFor<T>,
			proposal_id: ProposalIndex,
			vote: Vote,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let proposal = Self::proposals(proposal_id).ok_or(Error::<T>::NotOngoing)?;
			let owner_list = pallet_nft_marketplace::Pallet::<T>::property_owner(proposal.asset_id);
			ensure!(owner_list.contains(&origin), Error::<T>::NoPermission);
			ensure!(!Self::proposal_voter(proposal_id).contains(&origin), Error::<T>::AlreadyVoted);
			let voting_power = pallet_nft_marketplace::Pallet::<T>::property_owner_token(
				proposal.asset_id,
				origin.clone(),
			);
			let mut current_vote =
				Self::ongoing_votes(proposal_id).ok_or(Error::<T>::NotOngoing)?;
			match vote {
				Vote::Yes => current_vote.yes_votes += voting_power,
				Vote::No => current_vote.no_votes += voting_power,
			}
			
			OngoingVotes::<T>::insert(proposal_id, current_vote);
			ProposalVoter::<T>::try_mutate(proposal_id, |keys| {
				keys.try_push(origin.clone()).map_err(|_| Error::<T>::TooManyVotes)?;
				Ok::<(), DispatchError>(())
			})?;
			Self::deposit_event(Event::VotedOnProposal { proposal_id, voter: origin, vote });
			Ok(())
		}

		/// Lets owner of the real estate object vote on an challenge.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `challenge_id`: The index of the challenge.
		/// - `vote`: Must be either a Yes vote or a No vote.
		///
		/// Emits `VotedOnChallenge` event when succesfful.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::vote_on_letting_agent_challenge())]
		pub fn vote_on_letting_agent_challenge(
			origin: OriginFor<T>,
			challenge_id: ChallengeIndex,
			vote: Vote,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let challenge = Self::challenges(challenge_id).ok_or(Error::<T>::NotOngoing)?;
			let owner_list = pallet_nft_marketplace::Pallet::<T>::property_owner(challenge.asset_id);
			ensure!(owner_list.contains(&origin), Error::<T>::NoPermission);
			ensure!(!Self::challenge_voter(challenge_id, challenge.state.clone()).contains(&origin), Error::<T>::AlreadyVoted);
			let voting_power = pallet_nft_marketplace::Pallet::<T>::property_owner_token(
				challenge.asset_id,
				origin.clone(),
			);
			let mut current_vote =
				Self::ongoing_challenge_votes(challenge_id, challenge.state.clone()).ok_or(Error::<T>::NotOngoing)?;
			match vote {
				Vote::Yes => current_vote.yes_votes += voting_power,
				Vote::No => current_vote.no_votes += voting_power,
			}
			OngoingChallengeVotes::<T>::insert(challenge_id, challenge.state.clone(), current_vote);
			ChallengeVoter::<T>::try_mutate(challenge_id, challenge.state, |keys| {
				keys.try_push(origin.clone()).map_err(|_| Error::<T>::TooManyVotes)?;
				Ok::<(), DispatchError>(())
			})?;
			Self::deposit_event(Event::VotedOnChallenge { challenge_id, voter: origin, vote });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> AccountIdOf<T> {
			<T as pallet::Config>::PalletId::get().into_account_truncating()
		}

		// Slashes the letting agent.
		fn slash_letting_agent(challenge_id: ChallengeIndex) -> DispatchResult {
			let mut challenge = Challenges::<T>::take(challenge_id).ok_or(Error::<T>::NotOngoing)?;
			let letting_agent =
				pallet_property_management::Pallet::<T>::letting_storage(challenge.asset_id).ok_or(Error::<T>::NoLettingAgentFound)?;
			let amount = <T as Config>::MinSlashingAmount::get();
			<T as pallet::Config>::Slash::on_unbalanced(
				<T as pallet::Config>::Currency::slash_reserved(&letting_agent, amount).0,
			);
			challenge.state = ChallengeState::Fourth;
			let vote_stats = VoteStats { yes_votes: 0, no_votes: 0 };
			OngoingChallengeVotes::<T>::insert(challenge_id, challenge.state.clone(), vote_stats);
			Challenges::<T>::insert(challenge_id, challenge.clone()); 
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let expiry_block =
				current_block_number.saturating_add(<T as Config>::VotingTime::get());
			ChallengeRoundsExpiring::<T>::try_mutate(expiry_block, |keys| {
				keys.try_push(challenge_id).map_err(|_| Error::<T>::TooManyProposals)?;
				Ok::<(), DispatchError>(())
			})?;
			Self::deposit_event(Event::AgentSlashed { challenge_id, amount });
			Ok(())
		}

		/// Changes the letting agent of a given real estate object.
		fn change_letting_agent(challenge_id: ChallengeIndex) -> DispatchResult {
			let challenge = Challenges::<T>::take(challenge_id).ok_or(Error::<T>::NotOngoing)?;
			let letting_agent =
				pallet_property_management::Pallet::<T>::letting_storage(challenge.asset_id).ok_or(Error::<T>::NoLettingAgentFound)?;
			let asset_details =
				pallet_nft_marketplace::Pallet::<T>::asset_id_details(challenge.asset_id)
					.ok_or(Error::<T>::NoAssetFound)?;
			let _ = pallet_property_management::Pallet::<T>::remove_bad_letting_agent(
				asset_details.region,
				asset_details.location.clone(),
				letting_agent,
			);
			let _ = pallet_property_management::Pallet::<T>::selects_letting_agent(
				asset_details.region,
				asset_details.location,
				challenge.asset_id,
			);
			Self::deposit_event(Event::AgentChanged { challenge_id, asset_id: challenge.asset_id });
			Ok(())
		}

		/// Executes a proposal once it passes.
		fn execute_proposal(proposal: Proposal<BlockNumberFor<T>, T>) -> DispatchResult {
			let letting_agent =
				pallet_property_management::Pallet::<T>::letting_storage(proposal.asset_id)
					.ok_or(Error::<T>::NoLettingAgentFound)?;
		
			let property_reserves_balances = pallet_property_management::Pallet::<T>::property_reserve(proposal.asset_id);
			let property_reserves: BalanceOf<T> = TryInto::<u64>::try_into(property_reserves_balances)
				.map_err(|_| Error::<T>::ConversionError)?
				.try_into()
				.map_err(|_| Error::<T>::ConversionError)?;
			let proposal_amount = proposal.amount;
		
			// Check if the property reserves cover the proposal amount
			if property_reserves >= proposal_amount {
				// Transfer the full proposal amount from the reserves
				<T as pallet::Config>::Currency::transfer(
					&Self::account_id(),
					&letting_agent,
					proposal_amount.saturating_mul(
						<T as Config>::PolkadotJsMultiplier::get(),
					),
					KeepAlive,
				).map_err(|_| Error::<T>::NotEnoughFunds)?;
		
				// Decrease the reserves by the proposal amount
				pallet_property_management::Pallet::<T>::decrease_reserves(
					proposal.asset_id,
					TryInto::<u64>::try_into(proposal_amount)
					.map_err(|_| Error::<T>::ConversionError)?
					.try_into()
					.map_err(|_| Error::<T>::ConversionError)?,
				)?;
			} else {
				// Transfer only the available property reserves
				<T as pallet::Config>::Currency::transfer(
					&Self::account_id(),
					&letting_agent,
					property_reserves.saturating_mul(
						<T as Config>::PolkadotJsMultiplier::get(),
					),
					KeepAlive,
				).map_err(|_| Error::<T>::NotEnoughFunds)?;
		
				// Calculate the remaining amount needed
				let remaining_amount = proposal_amount.saturating_sub(property_reserves);
		
				// Increase the property debts by the remaining amount
				pallet_property_management::Pallet::<T>::increase_debts(
					proposal.asset_id,
					TryInto::<u64>::try_into(remaining_amount)
					.map_err(|_| Error::<T>::ConversionError)?
					.try_into()
					.map_err(|_| Error::<T>::ConversionError)?,
				)?;
		
				// Set the reserves to zero
				pallet_property_management::Pallet::<T>::decrease_reserves(
					proposal.asset_id,
					TryInto::<u64>::try_into(property_reserves)
					.map_err(|_| Error::<T>::ConversionError)?
					.try_into()
					.map_err(|_| Error::<T>::ConversionError)?,
				)?;
			}
		
			// Emit event for proposal execution
			Self::deposit_event(Event::ProposalExecuted {
				asset_id: proposal.asset_id,
				amount: proposal.amount,
			});
		
			Ok(())
		}

		/// Converts a u64 to a balance.
		pub fn u64_to_balance_option(input: u64) -> Result<BalanceOf<T>, Error<T>> {
			input.try_into().map_err(|_| Error::<T>::ConversionError)
		}
	}
}
