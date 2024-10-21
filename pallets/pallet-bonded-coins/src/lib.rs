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
		ensure,
		pallet_prelude::{OptionQuery, *},
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
		Hashable, Parameter,
	};
	use frame_system::{ensure_root, pallet_prelude::*};
	use parity_scale_codec::FullCodec;
	use scale_info::TypeInfo;
	use sp_runtime::{
		traits::{Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, One, Saturating, StaticLookup, Zero},
		ArithmeticError, BoundedVec, FixedPointNumber, SaturatedConversion,
	};
	use sp_std::{default::Default, iter::Iterator, vec::Vec};

	use crate::{
		curves_parameters::{convert_currency_amount, RationalBondingFunctionParameters, SquareRoot},
		types::{Curve, DiffKind, Locks, PoolDetails, PoolStatus, TokenMeta},
	};

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

	pub(crate) type PoolDetailsOf<T> =
		PoolDetails<<T as frame_system::Config>::AccountId, Curve<CurveParameterTypeOf<T>>, BoundedCurrencyVec<T>>;

	type CollateralAssetIdOf<T> =
		<<T as Config>::CollateralCurrency as InspectFungibles<<T as frame_system::Config>::AccountId>>::AssetId;

	type BoundedCurrencyVec<T> = BoundedVec<FungiblesAssetIdOf<T>, <T as Config>::MaxCurrencies>;

	pub(crate) type CurrencyNameOf<T> = BoundedVec<u8, <T as Config>::MaxStringLength>;

	pub(crate) type CurrencySymbolOf<T> = BoundedVec<u8, <T as Config>::MaxStringLength>;

	type TokenMetaOf<T> = TokenMeta<FungiblesBalanceOf<T>, CurrencySymbolOf<T>, CurrencyNameOf<T>>;

	pub(crate) type CurveParameterTypeOf<T> = <T as Config>::CurveParameterType;

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
			+ SquareRoot
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
		/// A new bonded token pool has been initiated. [pool_id]
		PoolCreated(T::PoolId),
		/// Trading locks on a pool have been removed. [pool_id]
		Unlocked(T::PoolId),
		/// Trading locks on a pool have been set or changed. [pool_id]
		LockSet(T::PoolId),
		/// A bonded token pool has been moved to destroying state. [pool_id]
		DestructionStarted(T::PoolId),
		/// A bonded token pool has been fully destroyed and all collateral and deposits have been refunded. [pool_id]
		Destroyed(T::PoolId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The number of bonded currencies on a new pool is either lower than 1 or greater than MaxCurrencies.
		CurrenciesNumber,
		/// A token swap cannot be executed due to a lock placed on this operation.
		Locked,
		/// The pool id is not currently registered.
		PoolUnknown,
		/// The pool has no associated bonded currency with the given index.
		IndexOutOfBounds,
		/// The cost or returns for a mint, burn, or swap operation is outside the user-defined slippage tolerance.
		Slippage,
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

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn create_pool(
			origin: OriginFor<T>,
			curve: Curve<CurveParameterTypeOf<T>>,
			currencies: BoundedVec<TokenMetaOf<T>, T::MaxCurrencies>,
			tradable: bool, // Todo: make it useful.
			state: PoolStatus<Locks>,
			pool_manager: AccountIdOf<T>,
		) -> DispatchResult {
			// ensure origin is PoolCreateOrigin
			let who = T::PoolCreateOrigin::ensure_origin(origin)?;

			let currency_length = currencies.len();

			ensure!(
				(1..=T::MaxCurrencies::get().saturated_into()).contains(&currency_length),
				Error::<T>::CurrenciesNumber
			);

			let current_asset_id = NextAssetId::<T>::get();

			let (currency_ids_vec, next_asset_id) =
				Self::generate_sequential_asset_ids(current_asset_id, currency_length);

			// update the storage for the next tx.
			NextAssetId::<T>::set(next_asset_id);

			// Should never fail.
			let currency_ids =
				BoundedVec::<FungiblesAssetIdOf<T>, T::MaxCurrencies>::try_from(currency_ids_vec.clone())
					.map_err(|_| Error::<T>::CurrenciesNumber)?;

			let pool_id = T::PoolId::from(currency_ids.blake2_256());

			// Todo: change that.
			T::DepositCurrency::hold(
				&T::RuntimeHoldReason::from(HoldReason::Deposit),
				&who,
				T::BaseDeposit::get().saturating_add(
					T::DepositPerCurrency::get()
						.saturating_mul(currency_length.saturated_into())
						.saturated_into(),
				),
			)?;

			for (idx, entry) in currencies.iter().enumerate() {
				let asset_id = currency_ids_vec.get(idx).ok_or(Error::<T>::CurrenciesNumber)?;
				T::Fungibles::create(asset_id.clone(), pool_id.clone().into(), false, entry.min_balance)?;

				// set metadata for new asset class
				T::Fungibles::set(
					asset_id.clone(),
					&pool_id.clone().into(),
					entry.name.clone().into(),
					entry.symbol.clone().into(),
					entry.decimals,
				)?;

				// TODO: reset team account
			}

			// Touch the pool account in order to able to transfer the collateral currency to it
			T::CollateralCurrency::touch(T::CollateralAssetId::get(), &pool_id.clone().into(), &who)?;

			Pools::<T>::set(
				&pool_id,
				Some(PoolDetails::new(pool_manager, curve, currency_ids, tradable, state)),
			);

			Self::deposit_event(Event::PoolCreated(pool_id));

			Ok(())
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

			let currency_idx_usize: usize = currency_idx.saturated_into();

			let cost = Self::mint_pool_currency_and_calculate_collateral(
				&pool_details,
				currency_idx_usize,
				beneficiary,
				amount_to_mint,
			)?;

			// withdraw the collateral and put it in the deposit account
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
		pub fn burn_into(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			currency_idx: u32,
			amount_to_burn: FungiblesBalanceOf<T>,
			min_return: CollateralCurrencyBalanceOf<T>,
			beneficiary: AccountIdLookupOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let beneficiary = T::Lookup::lookup(beneficiary)?;

			let pool_details = Pools::<T>::get(pool_id.clone()).ok_or(Error::<T>::PoolUnknown)?;

			ensure!(pool_details.is_burning_authorized(&who), Error::<T>::Locked);

			ensure!(!amount_to_burn.is_zero(), Error::<T>::ZeroAmount);

			let currency_idx_usize: usize = currency_idx.saturated_into();

			let collateral_return = Self::burn_pool_currency_and_calculate_collateral(
				&pool_details,
				currency_idx_usize,
				who,
				amount_to_burn,
			)?;

			// withdraw collateral from deposit and transfer to beneficiary account; deposit account may be drained
			T::CollateralCurrency::transfer(
				T::CollateralAssetId::get(),
				&pool_id.into(),
				&beneficiary,
				collateral_return,
				Preservation::Expendable,
			)?;

			// get id of the currency we want to burn
			// this also serves as a validation of the currency_idx parameter

			// fail if returns < min_return
			ensure!(collateral_return >= min_return, Error::<T>::Slippage);

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn swap_into(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			from_idx: u32,
			to_idx: u32,
			amount_to_swap: FungiblesBalanceOf<T>,
			beneficiary: AccountIdLookupOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let beneficiary = T::Lookup::lookup(beneficiary)?;

			let pool_details = <Pools<T>>::get(pool_id.clone()).ok_or(Error::<T>::PoolUnknown)?;

			ensure!(pool_details.is_swapping_authorized(&who), Error::<T>::Locked);
			ensure!(!amount_to_swap.is_zero(), Error::<T>::ZeroAmount);

			let from_idx_usize: usize = from_idx.saturated_into();
			let to_idx_usize: usize = to_idx.saturated_into();

			match &pool_details.curve {
				Curve::RationalBondingFunction => {
					let collateral = Self::burn_pool_currency_and_calculate_collateral(
						&pool_details,
						from_idx_usize,
						who,
						amount_to_swap,
					)?;

					let collateral_denomination = Self::get_collateral_denomination()?;

					let currencies_metadata = Self::get_currencies_metadata(&pool_details)?;

					let raw_supply = RationalBondingFunctionParameters::<T::CurveParameterType>::process_swap::<T>(
						currencies_metadata,
						(collateral, collateral_denomination),
						to_idx_usize,
					)?;

					Self::mint_pool_currency_and_calculate_collateral(
						&pool_details,
						to_idx_usize,
						beneficiary,
						raw_supply,
					)?;
				}
				// The price for burning and minting in the pool is the same, if the bonding curve is not [RationalBondingFunction].
				_ => {
					Self::burn_pool_currency_and_calculate_collateral(
						&pool_details,
						from_idx_usize,
						who,
						amount_to_swap,
					)?;
					Self::mint_pool_currency_and_calculate_collateral(
						&pool_details,
						to_idx_usize,
						beneficiary,
						amount_to_swap,
					)?;
				}
			};

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn set_lock(origin: OriginFor<T>, pool_id: T::PoolId, lock: Locks) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Pools::<T>::try_mutate(&pool_id, |pool| -> DispatchResult {
				if let Some(pool) = pool {
					ensure!(pool.is_manager(&who), Error::<T>::Unauthorized);
					pool.state = PoolStatus::Locked(lock);
					Ok(())
				} else {
					Err(Error::<T>::PoolUnknown.into())
				}
			})?;

			Self::deposit_event(Event::LockSet(pool_id));

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn unlock(origin: OriginFor<T>, pool_id: T::PoolId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Pools::<T>::try_mutate(&pool_id, |pool| -> DispatchResult {
				if let Some(pool) = pool {
					ensure!(pool.is_manager(&who), Error::<T>::Unauthorized);
					pool.state = PoolStatus::Active;
					Ok(())
				} else {
					Err(Error::<T>::PoolUnknown.into())
				}
			})?;

			Self::deposit_event(Event::Unlocked(pool_id));

			Ok(())
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

			let total_collateral_issuance = Self::get_pool_collateral(&pool_account);

			ensure!(!total_collateral_issuance.is_zero(), Error::<T>::ZeroAmount);

			let total_issuances: Vec<FungiblesBalanceOf<T>> = pool_details
				.bonded_currencies
				.iter()
				.map(|id| T::Fungibles::total_issuance(id.clone()))
				.collect();

			let sum_of_issuances: FungiblesBalanceOf<T> = total_issuances
				.iter()
				.fold(Zero::zero(), |sum, x| sum.saturating_add(*x));

			ensure!(!sum_of_issuances.is_zero(), Error::<T>::ZeroAmount);

			for (idx, currency_id) in pool_details.bonded_currencies.iter().enumerate() {
				if !total_issuances[idx].is_zero() {
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
				}
			}

			Ok(())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn start_destroy_currencies(origin: OriginFor<T>, pool_id: T::PoolId) -> DispatchResult {
			ensure_signed(origin)?;

			let pool_details = Pools::<T>::get(pool_id.clone()).ok_or(Error::<T>::PoolUnknown)?;

			ensure!(pool_details.state.is_destroying(), Error::<T>::InUse);

			let pool_account = pool_id.clone().into();

			let total_collateral_issuance = Self::get_pool_collateral(&pool_account);

			if !total_collateral_issuance.is_zero() {
				let total_issuances: Vec<FungiblesBalanceOf<T>> = pool_details
					.bonded_currencies
					.iter()
					.map(|id| T::Fungibles::total_issuance(id.clone()))
					.collect();

				let sum_of_issuances: FungiblesBalanceOf<T> = total_issuances
					.iter()
					.fold(Zero::zero(), |sum, x| sum.saturating_add(*x));

				ensure!(!sum_of_issuances.is_zero(), Error::<T>::InUse);
			}

			for currency_id in pool_details.bonded_currencies.iter() {
				T::Fungibles::start_destroy(currency_id.clone(), None)?;
			}

			Ok(())
		}

		#[pallet::call_index(10)]
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

		fn calculate_pre_post_issuances(
			kind: &DiffKind,
			amount: &CurveParameterTypeOf<T>,
			total_issuances: &[CurveParameterTypeOf<T>],
			currency_idx: usize,
		) -> Result<(CurveParameterTypeOf<T>, CurveParameterTypeOf<T>), DispatchError> {
			let active_issuance_pre = total_issuances.get(currency_idx).ok_or(Error::<T>::IndexOutOfBounds)?;
			let active_issuance_post = match kind {
				DiffKind::Mint => active_issuance_pre
					.checked_add(amount)
					.ok_or(ArithmeticError::Overflow)?,
				DiffKind::Burn => active_issuance_pre
					.checked_sub(amount)
					.ok_or(ArithmeticError::Underflow)?,
			};
			Ok((*active_issuance_pre, active_issuance_post))
		}

		fn burn_pool_currency_and_calculate_collateral(
			pool_details: &PoolDetailsOf<T>,
			currency_idx: usize,
			payer: AccountIdOf<T>,
			amount: FungiblesBalanceOf<T>,
		) -> Result<CollateralCurrencyBalanceOf<T>, DispatchError> {
			let burn_currency_id = pool_details
				.bonded_currencies
				.get(currency_idx)
				.ok_or(Error::<T>::IndexOutOfBounds)?;

			let currencies_metadata = Self::get_currencies_metadata(pool_details)?;

			T::Fungibles::burn_from(
				burn_currency_id.clone(),
				&payer,
				amount,
				Precision::Exact,
				Fortitude::Polite,
			)?;

			//
			let returns = Self::get_collateral_diff(
				DiffKind::Burn,
				&pool_details.curve,
				&amount,
				currencies_metadata,
				currency_idx,
			)?;

			Ok(returns)
		}

		fn mint_pool_currency_and_calculate_collateral(
			pool_details: &PoolDetailsOf<T>,
			currency_idx: usize,
			beneficiary: AccountIdOf<T>,
			amount: FungiblesBalanceOf<T>,
		) -> Result<CollateralCurrencyBalanceOf<T>, DispatchError> {
			// get id of the currency we want to mint
			// this also serves as a validation of the currency_idx parameter
			let mint_currency_id = pool_details
				.bonded_currencies
				.get(currency_idx)
				.ok_or(Error::<T>::IndexOutOfBounds)?;

			let currencies_metadata = Self::get_currencies_metadata(pool_details)?;

			let cost = Self::get_collateral_diff(
				DiffKind::Mint,
				&pool_details.curve,
				&amount,
				currencies_metadata,
				currency_idx,
			)?;

			T::Fungibles::mint_into(mint_currency_id.clone(), &beneficiary, amount)?;

			Ok(cost)
		}

		fn do_start_destroy_pool(pool_details: &mut PoolDetailsOf<T>) -> DispatchResult {
			ensure!(!pool_details.state.is_destroying(), Error::<T>::Destroying);

			pool_details.state.destroy();

			Ok(())
		}

		fn generate_sequential_asset_ids(mut start_id: T::AssetId, count: usize) -> (Vec<T::AssetId>, T::AssetId) {
			let mut currency_ids_vec = Vec::new();
			for _ in 0..count {
				currency_ids_vec.push(start_id.clone());
				start_id = start_id.saturating_plus_one();
			}
			(currency_ids_vec, start_id)
		}

		fn get_currencies_metadata(
			pool_details: &PoolDetailsOf<T>,
		) -> Result<Vec<(FungiblesBalanceOf<T>, u128)>, ArithmeticError> {
			pool_details
				.bonded_currencies
				.iter()
				.map(|id| {
					let total_issuance = T::Fungibles::total_issuance(id.clone());
					let decimals = 10u128
						.checked_pow(T::Fungibles::decimals(id.clone()).into())
						.ok_or(ArithmeticError::Overflow)?;
					Ok((total_issuance, decimals))
				})
				.collect()
		}

		fn get_collateral_denomination() -> Result<u128, ArithmeticError> {
			10u128
				.checked_pow(T::CollateralCurrency::decimals(T::CollateralAssetId::get()).into())
				.ok_or(ArithmeticError::Overflow)
		}

		fn get_pool_collateral(pool_account: &AccountIdOf<T>) -> CollateralCurrencyBalanceOf<T> {
			T::CollateralCurrency::total_balance(T::CollateralAssetId::get(), pool_account)
		}
	}
}
