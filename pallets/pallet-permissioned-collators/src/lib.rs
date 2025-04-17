#![cfg_attr(not(feature = "std"), no_std)]

use sp_staking::SessionIndex;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod types;
pub mod weights;

mod migrations;
pub use migrations::MigrateV0ToV1;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::{pallet_prelude::*, traits::EstimateNextSessionRotation};
use frame_system::pallet_prelude::*;
use pallet_session::{SessionManager, ShouldEndSession};
use sp_runtime::{Permill, Saturating};
use sp_std::vec::Vec;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, sp_runtime::traits::SaturatedConversion};
	use frame_system::pallet_prelude::*;
	use sp_staking::SessionIndex;

	use crate::types::RoundInfo;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type MaxCollators: Get<u32>;

		type MinBlocksPerRound: Get<BlockNumberFor<Self>>;
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type Collators<T: Config> = StorageValue<_, BoundedVec<T::AccountId, T::MaxCollators>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn round)]
	pub(crate) type Round<T: Config> = StorageValue<_, RoundInfo<BlockNumberFor<T>>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SomethingStored,
		CollatorAdded(T::AccountId),
		CollatorRemoved(T::AccountId),
		BlocksPerRoundSet(RoundInfo<BlockNumberFor<T>>),
		NewRound {
			block_number: BlockNumberFor<T>,
			session_index: SessionIndex,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		MaxCollatorExceeded,
		StorageOverflow,
		IndexOutOfBounds,
		CannotSetBelowMin,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: BlockNumberFor<T>) -> frame_support::weights::Weight {
			let post_weight = Weight::from_parts(10_000, 0) + T::DbWeight::get().reads(1);

			let mut round = Round::<T>::get();

			// check for round update
			if round.should_update(now) {
				// mutate round
				round.update(now);
				// start next round
				Round::<T>::put(round);

				Self::deposit_event(Event::NewRound {
					block_number: round.first,
					session_index: round.current,
				});
			}

			post_weight
		}

		#[cfg(feature = "try-runtime")]
		fn try_state(_n: BlockNumberFor<T>) -> Result<(), sp_runtime::TryRuntimeError> {
			crate::try_state::do_try_state::<T>()
		}
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn add_collator(origin: OriginFor<T>, collator: T::AccountId) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let mut collators = Collators::<T>::get();
			collators
				.try_push(collator.clone())
				.map_err(|_| Error::<T>::MaxCollatorExceeded)?;

			Collators::<T>::put(collators);

			Self::deposit_event(Event::CollatorAdded(collator));

			Ok(().into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
		pub fn remove_collator(origin: OriginFor<T>, index: u32) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			let mut collators = Collators::<T>::get();

			ensure!(index >= collators.len().saturated_into(), Error::<T>::IndexOutOfBounds);

			let collator = collators.swap_remove(index.saturated_into());
			Collators::<T>::put(collators);

			Self::deposit_event(Event::CollatorRemoved(collator));
			Ok(().into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
		pub fn set_blocks_per_round(origin: OriginFor<T>, new: BlockNumberFor<T>) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(new >= T::MinBlocksPerRound::get(), Error::<T>::CannotSetBelowMin);

			let old_round = Round::<T>::get();

			let new_round = RoundInfo {
				length: new,
				..old_round
			};

			Round::<T>::put(new_round.clone());

			Self::deposit_event(Event::BlocksPerRoundSet(new_round));
			Ok(())
		}
	}
}

impl<T: Config> SessionManager<T::AccountId> for Pallet<T> {
	fn new_session(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		log::debug!(
			"assembling new collators for new session {} at #{:?}",
			new_index,
			frame_system::Pallet::<T>::block_number(),
		);

		let collators = Collators::<T>::get().to_vec();
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

impl<T: Config> ShouldEndSession<BlockNumberFor<T>> for Pallet<T> {
	fn should_end_session(now: BlockNumberFor<T>) -> bool {
		frame_system::Pallet::<T>::register_extra_weight_unchecked(
			T::DbWeight::get().reads(1),
			DispatchClass::Mandatory,
		);
		let round = Round::<T>::get();
		round.should_update(now)
	}
}

impl<T: Config> EstimateNextSessionRotation<BlockNumberFor<T>> for Pallet<T> {
	fn average_session_length() -> BlockNumberFor<T> {
		Round::<T>::get().length
	}

	fn estimate_current_session_progress(now: BlockNumberFor<T>) -> (Option<Permill>, Weight) {
		let round = Round::<T>::get();
		let passed_blocks = now.saturating_sub(round.first);

		(
			Some(Permill::from_rational(passed_blocks, round.length)),
			// One read for the round info, blocknumber is read free
			T::DbWeight::get().reads(1),
		)
	}

	fn estimate_next_session_rotation(_now: BlockNumberFor<T>) -> (Option<BlockNumberFor<T>>, Weight) {
		let round = Round::<T>::get();

		(
			Some(round.first + round.length),
			// One read for the round info, blocknumber is read free
			T::DbWeight::get().reads(1),
		)
	}
}
