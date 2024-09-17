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
#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use frame_support::{
		dispatch::{DispatchResult, DispatchResultWithPostInfo},
		pallet_prelude::*,
		traits::{
			fungible::{Inspect as InspectFungible, Mutate, MutateHold},
			fungibles::{
				metadata::Inspect as FungiblesInspect, metadata::Mutate as FungiblesMetadata,
				Create as CreateFungibles, Destroy as DestroyFungibles, Inspect as InspectFungibles,
				Mutate as MutateFungibles,
			},
			tokens::{Fortitude, Precision, Preservation},
		},
		Hashable,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{
		traits::{CheckedAdd, CheckedSub, Saturating, StaticLookup, Zero},
		ArithmeticError, SaturatedConversion,
	};
	use sp_arithmetic::FixedU128;

	use crate::{types::{Curve, DiffKind, PoolDetails, PoolStatus, TokenMeta}, curves_parameters::transform_denomination_currency_amount};

	type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as sp_runtime::traits::StaticLookup>::Source;
	type DepositCurrencyBalanceOf<T> =
		<<T as Config>::DepositCurrency as InspectFungible<<T as frame_system::Config>::AccountId>>::Balance;
	type DepositCurrencyHoldReasonOf<T> =
		<<T as Config>::DepositCurrency as frame_support::traits::fungible::InspectHold<
			<T as frame_system::Config>::AccountId,
		>>::Reason;
	type CollateralCurrencyBalanceOf<T> =
		<<T as Config>::CollateralCurrency as InspectFungible<<T as frame_system::Config>::AccountId>>::Balance;
	type FungiblesBalanceOf<T> =
		<<T as Config>::Fungibles as InspectFungibles<<T as frame_system::Config>::AccountId>>::Balance;
	type FungiblesAssetIdOf<T> =
		<<T as Config>::Fungibles as InspectFungibles<<T as frame_system::Config>::AccountId>>::AssetId;

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
		type Fungibles: CreateFungibles<Self::AccountId>
			+ DestroyFungibles<Self::AccountId>
			+ FungiblesMetadata<Self::AccountId>
			+ FungiblesInspect<Self::AccountId>
			+ MutateFungibles<Self::AccountId>;
		/// The maximum number of currencies allowed for a single pool.
		#[pallet::constant]
		type MaxCurrencies: Get<u32> + TypeInfo;
		/// The deposit required for each bonded currency.
		#[pallet::constant]
		type DepositPerCurrency: Get<DepositCurrencyBalanceOf<Self>>;
		/// Who can create new bonded currency pools.
		type PoolCreateOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
		/// The type used for pool ids
		type PoolId: Parameter
			+ MaxEncodedLen
			+ From<[u8; 32]>
			+ Into<Self::AccountId>
			+ Into<DepositCurrencyHoldReasonOf<Self>>;
	}

	type CurveParameterType = FixedU128;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Bonded Currency Swapping Pools
	#[pallet::storage]
	#[pallet::getter(fn pools)]
	pub(crate) type Pools<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::PoolId,
		PoolDetails<T::AccountId, FungiblesAssetIdOf<T>, Curve<CurveParameterType>, T::MaxCurrencies>,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new bonded token pool has been initiated. [pool_id]
		PoolCreated(T::AccountId),
		/// Trading locks on a pool have been removed. [pool_id]
		Unlocked(T::AccountId),
		/// Trading locks on a pool have been set or changed. [pool_id]
		LockSet(T::AccountId),
		/// A bonded token pool has been moved to destroying state. [pool_id]
		DestructionStarted(T::AccountId),
		/// A bonded token pool has been fully destroyed and all collateral and deposits have been refunded. [pool_id]
		Destroyed(T::AccountId),
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
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		FungiblesBalanceOf<T>: TryInto<CollateralCurrencyBalanceOf<T>>,
	{
		#[pallet::call_index(0)]
		// #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))] TODO: properly configure weights
		pub fn create_pool(
			origin: OriginFor<T>,
			curve: Curve<CurveParameterType>,
			currencies: BoundedVec<TokenMeta<FungiblesBalanceOf<T>, FungiblesAssetIdOf<T>>, T::MaxCurrencies>,
			frozen: bool,
			// currency_admin: Option<AccountIdLookupOf<T>> TODO: use this to set currency admin
		) -> DispatchResultWithPostInfo {
			// ensure origin is PoolCreateOrigin
			let who = T::PoolCreateOrigin::ensure_origin(origin)?;

			ensure!(
				(1..=T::MaxCurrencies::get().saturated_into()).contains(&currencies.len()),
				Error::<T>::CurrenciesNumber
			);

			let currency_ids = BoundedVec::truncate_from(currencies.iter().map(|c| c.id.clone()).collect());

			let pool_id = T::PoolId::from(currency_ids.blake2_256());

			T::DepositCurrency::hold(
				&pool_id.clone().into(), // TODO: just assumed that you can use a pool id as hold reason, not sure that's true though
				&who,
				T::DepositPerCurrency::get()
					.saturating_mul(currencies.len().saturated_into())
					.saturated_into(),
			)?;

			for entry in currencies {
				let asset_id = entry.id.clone();

				// create new assset class; fail if it already exists
				T::Fungibles::create(asset_id.clone(), pool_id.clone().into(), false, entry.min_balance)?;

				// set metadata for new asset class
				T::Fungibles::set(
					asset_id,
					&pool_id.clone().into(),
					entry.name.clone(),
					entry.symbol.clone(),
					entry.decimals,
				)?;

				// TODO: use fungibles::roles::ResetTeam to update currency admin
			}

			<Pools<T>>::set(
				pool_id,
				Some(PoolDetails::new(who.clone(), curve, currency_ids, !frozen)),
			);

			// Emit an event.
			Self::deposit_event(Event::PoolCreated(who));
			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}

		#[pallet::call_index(1)]
		pub fn mint_into(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			currency_idx: u32,
			amount_to_mint: FungiblesBalanceOf<T>,
			max_cost: CollateralCurrencyBalanceOf<T>,
			beneficiary: AccountIdLookupOf<T>,
		) -> DispatchResultWithPostInfo {
			let signer = ensure_signed(origin)?;
			let beneficiary = T::Lookup::lookup(beneficiary)?;

			let pool_details = <Pools<T>>::get(pool_id.clone()).ok_or(Error::<T>::PoolUnknown)?;

			let mint_enabled = match pool_details.state {
				// if mint is locked, then operation is priviledged
				PoolStatus::Frozen(locks) => locks.allow_mint || signer == pool_details.creator,
				PoolStatus::Active => true,
				_ => false,
			};
			ensure!(mint_enabled, Error::<T>::Locked);

			ensure!(amount_to_mint.is_zero(), Error::<T>::ZeroAmount);

			let currency_idx_usize: usize = currency_idx.saturated_into();

			// get id of the currency we want to mint
			// this also serves as a validation of the currency_idx parameter
			let mint_currency_id = pool_details
				.bonded_currencies
				.get(currency_idx_usize)
				.ok_or(Error::<T>::IndexOutOfBounds)?;

			let currencies_metadata: Vec<(FungiblesBalanceOf<T>, u8)> = pool_details
				.bonded_currencies
				.iter()
				.map(|id| {
					(
						T::Fungibles::total_issuance(id.clone()),
						T::Fungibles::decimals(id.clone()),
					)
				})
				.collect();


			let cost = Self::get_collateral_diff(
				DiffKind::Mint,
				pool_details.curve,
				&amount_to_mint,
				currencies_metadata,
				currency_idx_usize,
			)?;

			// fail if cost > max_cost
			ensure!(!cost.gt(&max_cost), Error::<T>::Slippage);

			// withdraw the collateral and put it in the deposit account
			T::CollateralCurrency::transfer(&signer, &pool_id.into(), cost, Preservation::Preserve)?;

			// mint tokens into beneficiary account
			T::Fungibles::mint_into(mint_currency_id.clone(), &beneficiary, amount_to_mint)?;

			// TODO: apply lock if pool_details.transferable != true

			Ok(().into())
		}

		#[pallet::call_index(2)]
		pub fn burn_into(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			currency_idx: u32,
			amount_to_burn: FungiblesBalanceOf<T>,
			min_return: CollateralCurrencyBalanceOf<T>,
			beneficiary: AccountIdLookupOf<T>,
		) -> DispatchResultWithPostInfo {
			let signer = ensure_signed(origin)?;
			let beneficiary = T::Lookup::lookup(beneficiary)?;

			let pool_details = <Pools<T>>::get(pool_id.clone()).ok_or(Error::<T>::PoolUnknown)?;

			let burn_enabled = match pool_details.state {
				// if mint is locked, then operation is priviledged
				PoolStatus::Frozen(locks) => locks.allow_burn || signer == pool_details.creator,
				PoolStatus::Active => true,
				_ => false,
			};
			ensure!(burn_enabled, Error::<T>::Locked);

			ensure!(amount_to_burn.is_zero(), Error::<T>::ZeroAmount);

			let currency_idx_usize: usize = currency_idx.saturated_into();

			// get id of the currency we want to burn
			// this also serves as a validation of the currency_idx parameter
			let burn_currency_id = pool_details
				.bonded_currencies
				.get(currency_idx_usize)
				.ok_or(Error::<T>::IndexOutOfBounds)?;

			// TODO: remove lock if one exists / if pool_details.transferable != true

						let total_issuances: Vec<FungiblesBalanceOf<T>> = pool_details
				.bonded_currencies
				.iter()
				.map(|id| T::Fungibles::total_issuance(id.clone()))
				.collect();

			let burnt_amount = T::Fungibles::burn_from(
				burn_currency_id.clone(),
				&signer,
				amount_to_burn,
				Precision::Exact,
				Fortitude::Polite,
			)?;


			let total_issuances: Vec<(FungiblesBalanceOf<T>, u8)> = pool_details
				.bonded_currencies
				.iter()
				.map(|id| 	(
						T::Fungibles::total_issuance(id.clone()),
						T::Fungibles::decimals(id.clone()),
					))
				.collect();


			//
			let returns = Self::get_collateral_diff(
				DiffKind::Burn,
				pool_details.curve,
				&burnt_amount,
				total_issuances,
				currency_idx_usize,
			)?;

			// fail if returns < min_return
			ensure!(!returns.lt(&min_return), Error::<T>::Slippage);

			// withdraw collateral from deposit and transfer to beneficiary account; deposit account may be drained
			T::CollateralCurrency::transfer(&pool_id.into(), &beneficiary, returns, Preservation::Expendable)?;
			

			Ok(().into())
		}

		#[pallet::call_index(3)]
		pub fn swap_into(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			from_idx: u32,
			to_idx: u32,
			amount_to_swap: FungiblesBalanceOf<T>,
			beneficiary: AccountIdLookupOf<T>,
			min_return: FungiblesBalanceOf<T>,
		) -> DispatchResult {

			// 
			let signer = ensure_signed(origin)?;
			let beneficiary = T::Lookup::lookup(beneficiary)?;

			let pool_details = <Pools<T>>::get(pool_id.clone()).ok_or(Error::<T>::PoolUnknown)?;

			let swap_enabled = match pool_details.state {
				// if swap is locked, then operation is priviledged
				PoolStatus::Frozen(locks) => locks.allow_swap || signer == pool_details.creator,
				PoolStatus::Active => true,
				_ => false,
			};

			ensure!(swap_enabled, Error::<T>::Locked);
			ensure!(amount_to_swap.is_zero(), Error::<T>::ZeroAmount);

			let from_idx_usize: usize = from_idx.saturated_into();
			let to_idx_usize: usize = to_idx.saturated_into();

			let burn_currency_id = pool_details
				.bonded_currencies
				.get(from_idx_usize)
				.ok_or(Error::<T>::IndexOutOfBounds)?;

			let mint_currency_id = pool_details.bonded_currencies.get(to_idx_usize).ok_or(Error::<T>::IndexOutOfBounds)?;


			// 1. calculate the total issuances of the pool.
						let total_issuances: Vec<FungiblesBalanceOf<T>> = pool_details
				.bonded_currencies
				.iter()
				.map(|id| T::Fungibles::total_issuance(id.clone()))
				.collect();

			// 2. burn tokens from signer. Burned amount is used to mint new tokens.
			let burned_amount = T::Fungibles::burn_from(
				burn_currency_id.clone(),
				&signer,
				amount_to_swap,
				Precision::Exact,
				Fortitude::Polite,
			)?;

			// 3. calculate collatoral diff 

			let returns = Self::get_collateral_diff(
				DiffKind::Burn,
				pool_details.curve,
				&burned_amount,
				total_issuances,
				from_idx_usize,
			)?;


			Ok(())
		}
	}

	impl<T: Config> Pallet<T>
	where
		FungiblesBalanceOf<T>: TryInto<CollateralCurrencyBalanceOf<T>>,
		CollateralCurrencyBalanceOf<T>: TryInto<u128>,
	{

		/// save usage of currency_ids. 
		pub fn get_collateral_diff(
			kind: DiffKind,
			curve: Curve<CurveParameterType>,
			amount: &FungiblesBalanceOf<T>,
			total_issuances: Vec<(FungiblesBalanceOf<T>, u8)>,
			currency_idx: usize,
		) -> Result<CollateralCurrencyBalanceOf<T>, ArithmeticError> {

			// todo: change that. We have also to restrict the denomination of the pool currencies maybe?
			let target_denomination_normalization = 18;
			let target_denomination_costs = 10;

			

			let normalized_issuances = total_issuances
				.clone()
				.into_iter()
				.map(|(x, d)| {
					transform_denomination_currency_amount(
						x.saturated_into::<u128>(),
						d,
						target_denomination_normalization,
					)
				})
				.collect::<Result<Vec<FixedU128>, ArithmeticError>>()?;

			// normalize the amount to mint
			let normalized_amount_to_mint = transform_denomination_currency_amount(
				amount.clone().saturated_into(),
				total_issuances[currency_idx].1,
				target_denomination_normalization,
			)?;

			let (active_issuance_pre, active_issuance_post) =
				Self::calculate_pre_post_issuances(kind, &normalized_amount_to_mint, &normalized_issuances, currency_idx)?;

			let passive_issuance = normalized_issuances
				.iter()
				.enumerate()
				.filter(|&(idx, _)| idx != currency_idx)
				.fold(FixedU128::zero(), |sum, (_, x)| sum.saturating_add(*x));

 

			let normalize_cost = curve.calculate_cost(active_issuance_pre, active_issuance_post, passive_issuance)?;

			// transform the cost back to the target denomination of the collateral currency
			let real_costs = transform_denomination_currency_amount(
				normalize_cost.into_inner(),
				target_denomination_normalization,
				target_denomination_costs,
			)?;

			real_costs
				.into_inner()
				.try_into()
				.map_err(|_| ArithmeticError::Overflow)
		}

		fn calculate_pre_post_issuances(
			kind: DiffKind,
			amount: &FixedU128,
			total_issuances: &[FixedU128],
			currency_idx: usize,
		) -> Result<(FixedU128, FixedU128), ArithmeticError> {
			let active_issuance_pre = total_issuances[currency_idx];
			let active_issuance_post = match kind {
				DiffKind::Mint => active_issuance_pre
					.checked_add(amount)
					.ok_or(ArithmeticError::Overflow)?,
				DiffKind::Burn => active_issuance_pre
					.checked_sub(amount)
					.ok_or(ArithmeticError::Underflow)?,
			};
			Ok((active_issuance_pre, active_issuance_post))
		}
	}
}
