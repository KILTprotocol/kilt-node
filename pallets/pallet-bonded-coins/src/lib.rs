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
		traits::{Bounded, CheckedAdd, CheckedDiv, CheckedSub, One, SaturatedConversion, Saturating, Zero},
		BoundedVec, FixedPointNumber,
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
		/// The pool is already in the process of being destroyed.
		Destroying,
		/// The user is not privileged to perform the requested operation.
		Unauthorized,
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
			let mut pool_details = Pools::<T>::get(pool_id.clone()).ok_or(Error::<T>::PoolUnknown)?;

			ensure!(pool_details.is_manager(&who), Error::<T>::Unauthorized);

			Self::do_start_destroy_pool(&mut pool_details)?;

			Self::deposit_event(Event::DestructionStarted(pool_id));
			Ok(())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn force_start_destroy(origin: OriginFor<T>, pool_id: T::PoolId) -> DispatchResult {
			ensure_root(origin)?;

			let mut pool_details = Pools::<T>::get(pool_id.clone()).ok_or(Error::<T>::PoolUnknown)?;

			Self::do_start_destroy_pool(&mut pool_details)?;

			Self::deposit_event(Event::DestructionStarted(pool_id));
			Ok(())
		}

		#[pallet::call_index(8)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn refund_accounts(origin: OriginFor<T>, pool_id: T::PoolId, max_accounts: u32) -> DispatchResult {
			ensure_signed(origin)?;

			let pool_details = Pools::<T>::get(pool_id.clone()).ok_or(Error::<T>::PoolUnknown)?;

			ensure!(pool_details.state.is_destroying(), Error::<T>::Destroying); // TODO: incorrect error

			let pool_account = pool_id.clone().into();

			let total_collateral_issuance = Self::get_pool_collateral(&pool_account);

			let total_issuances: Vec<FungiblesBalanceOf<T>> = if total_collateral_issuance.is_zero() {
				// nothing to distribute
				vec![Zero::zero(); pool_details.bonded_currencies.len()]
			} else {
				pool_details
					.bonded_currencies
					.iter()
					.map(|id| T::Fungibles::total_issuance(id.clone()))
					.collect()
			};

			let sum_of_issuances: FungiblesBalanceOf<T> = total_issuances
				.iter()
				.fold(Zero::zero(), |sum, x| sum.saturating_add(*x));
			let collateral_per_token = total_collateral_issuance
				.checked_div(&sum_of_issuances)
				.unwrap_or(Zero::zero());

			let mut remaining_max_accounts = max_accounts.clone();

			for (idx, currency_id) in pool_details.bonded_currencies.clone().iter().enumerate() {
				if !total_issuances[idx].is_zero() {
					let refunded_accounts = Self::do_refund_accounts(
						currency_id.clone(),
						collateral_per_token,
						remaining_max_accounts,
						&pool_account,
					)?;
					if remaining_max_accounts > refunded_accounts {
						remaining_max_accounts -= refunded_accounts;
					} else {
						// max_accounts reached; stop execution
						return Ok(());
					}
				}
				// issuance is zero or no more accounts; currency can be destroyed
				T::Fungibles::start_destroy(currency_id.clone(), None)?; // TODO: can be called multiple times, but can we somehow avoid it?
			}

			Ok(())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn finish_destroy(origin: OriginFor<T>, pool_id: T::PoolId) -> DispatchResult {
			ensure_signed(origin)?;

			let pool_details = Pools::<T>::get(pool_id.clone()).ok_or(Error::<T>::PoolUnknown)?;

			ensure!(pool_details.state.is_destroying(), Error::<T>::Destroying);

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
		fn do_start_destroy_pool(pool_details: &mut PoolDetailsOf<T>) -> DispatchResult {
			ensure!(!pool_details.state.is_destroying(), Error::<T>::Destroying);

			pool_details.state.destroy();

			Ok(())
		}

		fn do_refund_accounts(
			currency_id: FungiblesAssetIdOf<T>,
			collateral_per_token: CollateralCurrencyBalanceOf<T>,
			remaining_max_accounts: u32,
			collateral_account: &AccountIdOf<T>,
		) -> Result<u32, DispatchError> {
			// TODO: how do we get the accounts?
			let accounts = vec![];

			for who in accounts.iter() {
				let burnt = T::Fungibles::decrease_balance(
					currency_id.clone(),
					who,
					Bounded::max_value(),
					Precision::BestEffort,
					Preservation::Expendable,
					Fortitude::Force,
				)?;

				T::CollateralCurrency::transfer(
					T::CollateralAssetId::get(),
					collateral_account,
					who,
					burnt.saturating_mul(collateral_per_token.clone()),
					Preservation::Expendable,
				)?;
			}

			Ok(accounts.len().saturated_into())
		}

		fn get_pool_collateral(pool_account: &AccountIdOf<T>) -> CollateralCurrencyBalanceOf<T> {
			T::CollateralCurrency::total_balance(T::CollateralAssetId::get(), pool_account)
		}
	}
}
