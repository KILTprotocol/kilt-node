#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;

#[frame_support::pallet]
pub mod pallet {
	use crate::types::EventData;
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::{ValueQuery, *},
		traits::{
			fungible::{Inspect as InspectFungible, Mutate, MutateHold},
			fungibles::{
				metadata::Mutate as FungiblesMetadata, Create as CreateFungibles, Destroy as DestroyFungibles,
			},
			tokens::{AssetId, Balance},
		},
	};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::EncodeLike;
	use sp_runtime::{
		traits::{EnsureAddAssign, One, Saturating},
		SaturatedConversion,
	};

	pub type BalanceOf<T, A> = <T as InspectFungible<A>>::Balance;
	pub type DepositCurrencyHoldReasonOf<T> =
		<<T as Config>::DepositCurrency as frame_support::traits::fungible::InspectHold<
			<T as frame_system::Config>::AccountId,
		>>::Reason;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The currency used for storage deposits.
		type DepositCurrency: MutateHold<Self::AccountId>;
		/// The currency used as collateral for minting bonded tokens.
		type CollateralCurrency: Mutate<Self::AccountId>;
		/// Implementation of creating and managing new fungibles
		type Fungibles: CreateFungibles<Self::AccountId, Balance = Self::Balance, AssetId = Self::AssetId>
			+ DestroyFungibles<Self::AccountId, Balance = Self::Balance, AssetId = Self::AssetId>
			+ FungiblesMetadata<Self::AccountId, Balance = Self::Balance, AssetId = Self::AssetId>;
		/// The maximum amount that a user can place on a bet.
		#[pallet::constant]
		type MaxStake: Get<BalanceOf<Self::CollateralCurrency, Self::AccountId>>;
		/// The maximum length allowed for an event name.
		#[pallet::constant]
		type MaxNameLength: Get<u32> + TypeInfo; // TODO: is there a better type for a length?
		/// The maximum number of outcomes allowed for an event.
		#[pallet::constant]
		type MaxOutcomes: Get<u32> + TypeInfo;
		/// The deposit required for each outcome currency.
		#[pallet::constant]
		type DepositPerOutcome: Get<BalanceOf<Self::DepositCurrency, Self::AccountId>>;
		/// Who can create new events.
		type EventCreateOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
		/// Who can mint coins.
		type WagerOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
		/// The type used for event ids
		type EventId: EncodeLike
			+ Decode
			+ TypeInfo
			+ Clone
			+ MaxEncodedLen
			+ From<Self::AssetId>
			+ Into<Self::AccountId>
			+ Into<DepositCurrencyHoldReasonOf<Self>>;
		/// Type of an asset id in the Fungibles implementation
		type AssetId: AssetId + Default + EnsureAddAssign + One;
		/// The balance of assets in the Fungibles implementation
		type Balance: Balance;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Predictable Events
	#[pallet::storage]
	#[pallet::getter(fn events)]
	pub(crate) type Events<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::EventId,
		EventData<T::AccountId, T::AssetId, T::MaxNameLength, T::MaxOutcomes>,
		OptionQuery,
	>;

	/// The asset id to be used for the next event creation.
	#[pallet::storage]
	#[pallet::getter(fn next_asset_id)]
	pub(crate) type NextAssetId<T: Config> = StorageValue<_, T::AssetId, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new predictable event has been initiated. [event_id]
		EventCreated(T::AccountId),
		/// Prediction registration for a predictable event has been paused. [event_id]
		EventPaused(T::AccountId),
		/// Prediction registration for a predictable event has been resumed. [event_id]
		EventResumed(T::AccountId),
		/// Prediction registration for a predictable event has been resumed. [event_id, selected_outcome]
		EventDecided(T::AccountId, u32), // TODO: outcome index type should be configurable
		/// A predictable event has fully deleted. [event_id]
		EventDestroyed(T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The number of outcomes is either lower than 2 or greater than MaxOutcomes.
		OutcomesLength,
		/// A wager cannot be placed or modified on a predictable event whose status is not Active.
		Inactive,
		/// The event id is not currently registered.
		EventUnknown,
		/// The event has no outcome with the given index.
		OutcomeUnknown,
		/// An account has exceeded the maximum allowed wager value.
		MaxStakeExceeded,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[derive(Debug, Encode, Decode, Clone, PartialEq, TypeInfo)]
	pub struct TokenMeta<Balance> {
		pub name: Vec<u8>,
		pub symbol: Vec<u8>,
		pub decimals: u8,
		pub min_balance: Balance,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))] // TODO: properly configure weights
		pub fn create_event(
			origin: OriginFor<T>,
			event_name: BoundedVec<u8, T::MaxNameLength>,
			outcomes: BoundedVec<TokenMeta<T::Balance>, T::MaxOutcomes>,
		) -> DispatchResultWithPostInfo {
			// ensure origin is EventCreateOrigin
			let who = T::EventCreateOrigin::ensure_origin(origin)?;

			ensure!(
				(2..=T::MaxOutcomes::get().saturated_into()).contains(&outcomes.len()),
				Error::<T>::OutcomesLength
			);

			let mut asset_id = Self::next_asset_id();
			let event_id = T::EventId::from(asset_id.clone());

			T::DepositCurrency::hold(
				&event_id.clone().into(), // TODO: just assumed that you can use an event id as hold reason, not sure that's true though
				&who,
				T::DepositPerOutcome::get()
					.saturating_mul(outcomes.len().saturated_into())
					.saturated_into(),
			)?;

			let mut currency_ids = BoundedVec::with_bounded_capacity(outcomes.len());

			for (idx, entry) in outcomes.iter().enumerate() {
				T::Fungibles::create(asset_id.clone(), event_id.clone().into(), true, entry.min_balance)?;
				currency_ids[idx] = asset_id.clone();
				asset_id.ensure_add_assign(T::AssetId::one())?;

				T::Fungibles::set(
					asset_id.clone(),
					&event_id.clone().into(),
					entry.name.clone(),
					entry.symbol.clone(),
					entry.decimals,
				)?;
			}

			<NextAssetId<T>>::put(asset_id);

			<Events<T>>::set(event_id, Some(EventData::new(who.clone(), event_name, currency_ids)));

			// Emit an event.
			Self::deposit_event(Event::EventCreated(who));
			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}
	}
}
