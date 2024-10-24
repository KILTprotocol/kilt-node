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

mod curves;
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
				Mutate as MutateFungibles,
			},
			tokens::Preservation,
			AccountTouch,
		},
		Parameter,
	};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::FullCodec;
	use scale_info::TypeInfo;
	use sp_runtime::{
		traits::{CheckedAdd, CheckedSub, One, Saturating, StaticLookup, Zero},
		BoundedVec, FixedPointNumber, SaturatedConversion,
	};
	use sp_std::default::Default;
	use substrate_fixed::types::I9F23;

	use crate::{curves::Curve, types::PoolDetails};

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

	pub(crate) type PoolDetailsOf<T> =
		PoolDetails<<T as frame_system::Config>::AccountId, Curve<CurveParameterTypeOf<T>>, BoundedCurrencyVec<T>>;

	pub(crate) type Precision = I9F23;

	pub(crate) type PassiveSupply<T> = Vec<T>;

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
		Todo,
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		IndexOutOfBounds,
		PoolUnknown,
		Locked,
		ZeroAmount,
		Slippage,
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
		pub fn mint_into(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			currency_idx: u32,
			amount_to_mint: FungiblesBalanceOf<T>,
			max_cost: CollateralCurrencyBalanceOf<T>,
			beneficiary: AccountIdLookupOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let beneficiary = T::Lookup::lookup(beneficiary)?;

			let pool_details = Pools::<T>::get(pool_id.clone()).ok_or(Error::<T>::PoolUnknown)?;

			ensure!(pool_details.is_minting_authorized(&who), Error::<T>::Locked);

			ensure!(!amount_to_mint.is_zero(), Error::<T>::ZeroAmount);

			let bonded_currencies = pool_details.bonded_currencies;

			let currency_idx: usize = currency_idx.saturated_into();

			ensure!(bonded_currencies.len() > currency_idx, Error::<T>::IndexOutOfBounds);

			let cost = Self::calculate_collateral(&pool_details, currency_idx, beneficiary, amount_to_mint)?;

			T::CollateralCurrency::transfer(
				T::CollateralAssetId::get(),
				&who,
				&pool_id.into(),
				cost,
				Preservation::Preserve,
			)?;

			// fail if cost > max_cost
			ensure!(cost <= max_cost, Error::<T>::Slippage);

			// TODO: apply lock if pool_details.transferable != true

			Ok(())
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

		// TODO: not sure if we really need that. Check that out with Raphael.
		#[pallet::call_index(6)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn start_destroy(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		#[pallet::call_index(7)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn force_start_destroy(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		// todo: check if we really need that tx.
		#[pallet::call_index(8)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn destroy_accounts(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		#[pallet::call_index(9)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn finish_destroy(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}
	}

	impl<T: Config> Pallet<T> {
		fn calculate_collateral(
			bonded_currencies: &[T::AssetId],
			currency_idx: usize,
			curve: &Curve<CurveParameterTypeOf<T>>,
			beneficiary: AccountIdOf<T>,
			amount: FungiblesBalanceOf<T>,
		) -> Result<CollateralCurrencyBalanceOf<T>, DispatchError> {
			// get id of the currency we want to mint
			// this also serves as a validation of the currency_idx parameter
			let mint_currency_id = bonded_currencies
				.get(currency_idx)
				// should never happen but better safe than sorry
				.ok_or(Error::<T>::IndexOutOfBounds)?;

			let currencies_metadata = Self::get_currencies_metadata(pool_details)?;

			let cost = Self::get_collateral_diff(DiffKind::Mint, curve, &amount, currencies_metadata, currency_idx)?;

			T::Fungibles::mint_into(mint_currency_id.clone(), &beneficiary, amount)?;

			Ok(cost)
		}

		/// save usage of currency_ids.
		pub fn get_collateral_diff(
			kind: DiffKind,
			curve: &Curve<CurveParameterTypeOf<T>>,
			amount: &FungiblesBalanceOf<T>,
			total_issuances: Vec<(FungiblesBalanceOf<T>, u128)>,
			currency_idx: usize,
		) -> Result<CollateralCurrencyBalanceOf<T>, DispatchError> {
			let denomination_normalization = CurveParameterTypeOf::<T>::DIV;
			let denomination_collateral_currency = Self::get_collateral_denomination()?;

			let denomination_bonded_currency = total_issuances.get(currency_idx).ok_or(Error::<T>::IndexOutOfBounds)?.1;

			let normalized_total_issuances = total_issuances
				.clone()
				.into_iter()
				.map(|(x, d)| convert_currency_amount::<T>(x.saturated_into::<u128>(), d, denomination_normalization))
				.collect::<Result<Vec<CurveParameterTypeOf<T>>, ArithmeticError>>()?;

			// normalize the amount to mint
			let normalized_amount = convert_currency_amount::<T>(
				amount.clone().saturated_into(),
				denomination_bonded_currency,
				denomination_normalization,
			)?;

			let (active_issuance_pre, active_issuance_post) = Self::calculate_pre_post_issuances(
				&kind,
				&normalized_amount,
				&normalized_total_issuances,
				currency_idx,
			)?;

			let passive_issuance = normalized_total_issuances
				.iter()
				.enumerate()
				.filter(|&(idx, _)| idx != currency_idx)
				.fold(CurveParameterTypeOf::<T>::zero(), |sum, (_, x)| sum.saturating_add(*x));

			let normalize_cost =
				curve.calculate_cost(active_issuance_pre, active_issuance_post, passive_issuance, kind)?;

			// transform the cost back to the target denomination of the collateral currency
			let collateral = convert_currency_amount::<T>(
				normalize_cost.into_inner(),
				denomination_normalization,
				denomination_collateral_currency,
			)?;

			Ok(collateral.into_inner().saturated_into())
		}

		fn get_fungible_supply<Fungible: FungiblesInspect<AccountIdOf<T>, AssetId = T::AssetId>>(
			asset_id: &T::AssetId,
			who: &AccountIdOf<T>,
		) -> Fungible::Balance {
			Fungible::total_balance(asset_id.to_owned(), who)
		}

		fn get_fungible_denomination<Fungible: FungiblesMetadata<AccountIdOf<T>, AssetId = T::AssetId>>(
			asset_id: &T::AssetId,
		) -> u8 {
			Fungible::decimals(asset_id.to_owned())
		}
	}
}
