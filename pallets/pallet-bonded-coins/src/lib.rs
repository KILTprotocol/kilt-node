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

mod curves_parameters;
#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{
			fungible::{Inspect as InspectFungible, MutateHold},
			fungibles::{
				metadata::{Inspect as FungiblesInspect, Mutate as FungiblesMetadata},
				Create as CreateFungibles, Destroy as DestroyFungibles, Inspect as InspectFungibles,
				Mutate as MutateFungibles, Unbalanced,
			},
			tokens::{Fortitude, Precision, Preservation},
			AccountTouch,
		},
		Parameter,
	};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::FullCodec;
	use scale_info::TypeInfo;
	use sp_runtime::{
		traits::{Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, One, Saturating, StaticLookup, Zero},
		ArithmeticError, BoundedVec, FixedPointNumber,
	};
	use sp_std::default::Default;

	use crate::types::PoolDetails;

	type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as sp_runtime::traits::StaticLookup>::Source;

	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	pub(crate) type DepositCurrencyBalanceOf<T> =
		<<T as Config>::DepositCurrency as InspectFungible<<T as frame_system::Config>::AccountId>>::Balance;

	pub(crate) type CollateralCurrencyBalanceOf<T> =
		<<T as Config>::CollateralCurrency as InspectFungibles<<T as frame_system::Config>::AccountId>>::Balance;

	pub(crate) type FungiblesBalanceOf<T> =
		<<T as Config>::Fungibles as InspectFungibles<<T as frame_system::Config>::AccountId>>::Balance;

	type FungiblesAssetIdOf<T> =
		<<T as Config>::Fungibles as InspectFungibles<<T as frame_system::Config>::AccountId>>::AssetId;

	type CollateralAssetIdOf<T> =
		<<T as Config>::CollateralCurrency as InspectFungibles<<T as frame_system::Config>::AccountId>>::AssetId;

	type BoundedCurrencyVec<T> = BoundedVec<FungiblesAssetIdOf<T>, <T as Config>::MaxCurrencies>;

	pub(crate) type CurrencyNameOf<T> = BoundedVec<u8, <T as Config>::MaxStringLength>;

	pub(crate) type CurrencySymbolOf<T> = BoundedVec<u8, <T as Config>::MaxStringLength>;

	pub(crate) type CurveParameterTypeOf<T> = <T as Config>::CurveParameterType;

	// TODO: change CurveParameterTypeOf.
	pub(crate) type PoolDetailsOf<T> =
		PoolDetails<<T as frame_system::Config>::AccountId, CurveParameterTypeOf<T>, BoundedCurrencyVec<T>>;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The currency used for storage deposits.
		type DepositCurrency: MutateHold<Self::AccountId, Reason = Self::RuntimeHoldReason>;
		/// The currency used as collateral for minting bonded tokens.
		type CollateralCurrency: MutateFungibles<Self::AccountId>
			+ AccountTouch<CollateralAssetIdOf<Self>, Self::AccountId>
			+ FungiblesMetadata<Self::AccountId>;
		/// Implementation of creating and managing new fungibles
		type Fungibles: CreateFungibles<Self::AccountId, AssetId = Self::AssetId>
			+ DestroyFungibles<Self::AccountId>
			+ FungiblesMetadata<Self::AccountId>
			+ FungiblesInspect<Self::AccountId>
			+ MutateFungibles<Self::AccountId, Balance = CollateralCurrencyBalanceOf<Self>>;
		/// The maximum number of currencies allowed for a single pool.
		#[pallet::constant]
		type MaxCurrencies: Get<u32>;
		/// The deposit required for each bonded currency.

		#[pallet::constant]
		type MaxStringLength: Get<u32>;

		/// The deposit required for each bonded currency.
		#[pallet::constant]
		type DepositPerCurrency: Get<DepositCurrencyBalanceOf<Self>>;

		/// The base deposit required to create a new pool, primarily to cover the ED of the pool account.
		#[pallet::constant]
		type BaseDeposit: Get<DepositCurrencyBalanceOf<Self>>;

		/// The asset id of the collateral currency.
		type CollateralAssetId: Get<CollateralAssetIdOf<Self>>;
		/// Who can create new bonded currency pools.
		type PoolCreateOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
		/// The type used for pool ids
		type PoolId: Parameter + MaxEncodedLen + From<[u8; 32]> + Into<Self::AccountId>;

		/// The type used for asset ids. This is the type of the bonded currencies.
		type AssetId: Parameter + Member + FullCodec + TypeInfo + MaxEncodedLen + Saturating + One + Default;

		type RuntimeHoldReason: From<HoldReason>;

		/// The type used for the curve parameters.
		type CurveParameterType: Parameter
			+ Member
			+ FixedPointNumber<Inner = u128>
			+ TypeInfo
			+ MaxEncodedLen
			+ Zero
			+ CheckedAdd
			+ CheckedSub;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Bonded Currency Swapping Pools
	#[pallet::storage]
	#[pallet::getter(fn pools)]
	pub(crate) type Pools<T: Config> = StorageMap<_, Twox64Concat, T::PoolId, PoolDetailsOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nex_asset_id)]
	pub(crate) type NextAssetId<T: Config> = StorageValue<_, FungiblesAssetIdOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A bonded token pool has been moved to destroying state. [pool_id]
		DestructionStarted(T::PoolId),
		/// A bonded token pool has been fully destroyed and all collateral and deposits have been refunded. [pool_id]
		Destroyed(T::PoolId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The pool id is not currently registered.
		PoolUnknown,
		/// The amount to mint, burn, or swap is zero.
		ZeroAmount,
		/// The pool is already in the process of being destroyed.
		Destroying,
		/// The user is not privileged to perform the requested operation.
		Unauthorized,
		/// The pool is in use and cannot be destroyed at this point.
		InUse,
	}

	#[pallet::composite_enum]
	pub enum HoldReason {
		Deposit,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn create_pool(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn mint_into(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn burn_into(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn swap_into(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn set_lock(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn unlock(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		#[pallet::call_index(6)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn start_destroy(origin: OriginFor<T>, pool_id: T::PoolId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_start_destroy_pool(pool_id, Some(who))
		}

		#[pallet::call_index(7)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn force_start_destroy(origin: OriginFor<T>, pool_id: T::PoolId) -> DispatchResult {
			ensure_root(origin)?;

			Self::do_start_destroy_pool(pool_id, None)
		}

		#[pallet::call_index(8)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn refund_account(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			account: AccountIdLookupOf<T>,
		) -> DispatchResult {
			ensure_signed(origin)?;
			let who = T::Lookup::lookup(account)?;

			let pool_details = Pools::<T>::get(pool_id.clone()).ok_or(Error::<T>::PoolUnknown)?;

			ensure!(pool_details.state.is_destroying(), Error::<T>::InUse);

			let pool_account = pool_id.clone().into();

			let mut total_collateral_issuance = Self::get_pool_collateral(&pool_account);

			ensure!(!total_collateral_issuance.is_zero(), Error::<T>::ZeroAmount);

			let total_issuances: Vec<(FungiblesAssetIdOf<T>, FungiblesBalanceOf<T>)> = pool_details
				.bonded_currencies
				.iter()
				.map(|id| (id.clone(), T::Fungibles::total_issuance(id.clone())))
				.filter(|(_, iss)| iss.gt(&Zero::zero()))
				.collect();

			let mut sum_of_issuances: FungiblesBalanceOf<T> = total_issuances
				.iter()
				.fold(Zero::zero(), |sum, (_, x)| sum.saturating_add(*x));

			ensure!(!sum_of_issuances.is_zero(), Error::<T>::ZeroAmount);

			let mut dead_currencies = Vec::with_capacity(total_issuances.len());
			for (currency_id, token_issuance) in total_issuances.iter() {
				let burnt = T::Fungibles::decrease_balance(
					currency_id.clone(),
					&who,
					Bounded::max_value(),
					Precision::BestEffort,
					Preservation::Expendable,
					Fortitude::Force,
				)?;

				let amount = burnt
					.checked_mul(&total_collateral_issuance)
					.ok_or(ArithmeticError::Overflow)? // TODO: do we need a fallback if this fails?
					.checked_div(&sum_of_issuances)
					.unwrap_or(Zero::zero());

				T::CollateralCurrency::transfer(
					T::CollateralAssetId::get(),
					&pool_account,
					&who,
					amount,
					Preservation::Expendable,
				)?;

				total_collateral_issuance -= amount;
				sum_of_issuances -= burnt;

				// if the total issuance drops to 0 due to this burn, kill the currency
				if token_issuance <= &burnt {
					dead_currencies.push(currency_id);
				}
			}

			// destroy all active currencies if the collateral locked in the pool has dropped to 0
			if total_collateral_issuance.is_zero() || sum_of_issuances.is_zero() {
				// we assume all currencies that already had their total issuance at 0 have already been deactivated in a previous step
				for (currency_id, _) in total_issuances {
					T::Fungibles::start_destroy(currency_id, None)?;
				}
			} else {
				// deactivate all currencies whose issuance dropped to 0 in this step
				for currency_id in dead_currencies {
					T::Fungibles::start_destroy(currency_id.clone(), None)?;
				}
			}

			Ok(())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn finish_destroy(origin: OriginFor<T>, pool_id: T::PoolId) -> DispatchResult {
			ensure_signed(origin)?;

			let pool_details = Pools::<T>::get(pool_id.clone()).ok_or(Error::<T>::PoolUnknown)?;

			ensure!(pool_details.state.is_destroying(), Error::<T>::InUse);

			for currency_id in pool_details.bonded_currencies {
				if T::Fungibles::asset_exists(currency_id.clone()) {
					T::Fungibles::finish_destroy(currency_id.clone())?;
				}
			}

			let pool_account = pool_id.clone().into();

			let total_collateral_issuance = Self::get_pool_collateral(&pool_account);

			T::CollateralCurrency::transfer(
				T::CollateralAssetId::get(),
				&pool_account,
				&pool_details.manager,
				total_collateral_issuance,
				Preservation::Expendable,
			)?;

			Pools::<T>::remove(&pool_id);

			Self::deposit_event(Event::Destroyed(pool_id));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn do_start_destroy_pool(pool_id: T::PoolId, maybe_check_owner: Option<AccountIdOf<T>>) -> DispatchResult {
			let mut pool_details = Pools::<T>::get(pool_id.clone()).ok_or(Error::<T>::PoolUnknown)?;

			ensure!(!pool_details.state.is_destroying(), Error::<T>::Destroying);

			if let Some(caller) = maybe_check_owner {
				ensure!(pool_details.is_manager(&caller), Error::<T>::Unauthorized);
			}

			pool_details.state.destroy();

			Self::deposit_event(Event::DestructionStarted(pool_id.clone()));

			let total_collateral_issuance = Self::get_pool_collateral(&pool_id.clone().into());

			if total_collateral_issuance.is_zero() {
				for currency_id in pool_details.bonded_currencies.iter() {
					T::Fungibles::start_destroy(currency_id.clone(), None)?;
				}
				return Ok(());
			}

			for currency_id in pool_details.bonded_currencies.iter() {
				if T::Fungibles::total_issuance(currency_id.clone()).is_zero() {
					T::Fungibles::start_destroy(currency_id.clone(), None)?;
				}
			}

			Ok(())
		}

		fn get_pool_collateral(pool_account: &AccountIdOf<T>) -> CollateralCurrencyBalanceOf<T> {
			T::CollateralCurrency::total_balance(T::CollateralAssetId::get(), pool_account)
		}
	}
}
