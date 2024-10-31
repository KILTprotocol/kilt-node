#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod curves;
pub mod traits;
mod types;
#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{
			fungible::{Inspect as InspectFungible, MutateHold},
			fungibles::{
				metadata::{Inspect as FungiblesInspect, Mutate as FungiblesMetadata},
				roles::Inspect as FungiblesRoles,
				Create as CreateFungibles, Destroy as DestroyFungibles, Inspect as InspectFungibles,
				Mutate as MutateFungibles,
			},
			tokens::{Fortitude, Precision as WithdrawalPrecision, Preservation, Provenance},
			AccountTouch,
		},
		Hashable, Parameter,
	};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::FullCodec;
	use sp_arithmetic::ArithmeticError;
	use sp_runtime::{
		traits::{
			Bounded, CheckedDiv, CheckedMul, One, SaturatedConversion, Saturating, StaticLookup, UniqueSaturatedInto,
			Zero,
		},
		BoundedVec,
	};
	use sp_std::{
		default::Default,
		ops::{AddAssign, BitOrAssign, ShlAssign},
	};
	use substrate_fixed::{
		traits::{Fixed, FixedSigned, FixedUnsigned, ToFixed},
		types::I9F23,
	};

	use crate::{
		curves::{convert_to_fixed, BondingFunction, Curve, CurveInput},
		traits::{FreezeAccounts, ResetTeam},
		types::{PoolDetails, PoolManagingTeam, TokenMeta},
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

	type CollateralAssetIdOf<T> =
		<<T as Config>::CollateralCurrency as InspectFungibles<<T as frame_system::Config>::AccountId>>::AssetId;

	type BoundedCurrencyVec<T> = BoundedVec<FungiblesAssetIdOf<T>, <T as Config>::MaxCurrencies>;

	pub(crate) type CurrencyNameOf<T> = BoundedVec<u8, <T as Config>::MaxStringLength>;

	pub(crate) type CurrencySymbolOf<T> = BoundedVec<u8, <T as Config>::MaxStringLength>;

	pub(crate) type CurveParameterTypeOf<T> = <T as Config>::CurveParameterType;

	pub(crate) type CurveParameterInputOf<T> = <T as Config>::CurveParameterInput;

	pub(crate) type PoolDetailsOf<T> =
		PoolDetails<<T as frame_system::Config>::AccountId, Curve<CurveParameterTypeOf<T>>, BoundedCurrencyVec<T>>;

	pub(crate) type Precision = I9F23;

	pub(crate) type PassiveSupply<T> = Vec<T>;

	pub(crate) type TokenMetaOf<T> = TokenMeta<FungiblesBalanceOf<T>, CurrencyNameOf<T>, CurrencySymbolOf<T>>;

	const LOG_TARGET: &str = "runtime::pallet-bonded-coins";

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
			+ MutateFungibles<Self::AccountId, Balance = CollateralCurrencyBalanceOf<Self>>
			+ FreezeAccounts<Self::AccountId, Self::AssetId>
			+ FungiblesRoles<Self::AccountId>
			+ ResetTeam<Self::AccountId>;
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
		/// The origin for most permissionless and priviledged operations.
		type DefaultOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
		/// The dedicated origin for creating new bonded currency pools (typically permissionless).
		type PoolCreateOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
		/// The origin for permissioned operations (force_* transactions).
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// The type used for pool ids
		type PoolId: Parameter + MaxEncodedLen + From<[u8; 32]> + Into<Self::AccountId>;

		/// The type used for asset ids. This is the type of the bonded currencies.
		type AssetId: Parameter + Member + FullCodec + MaxEncodedLen + Saturating + One + Default;

		type RuntimeHoldReason: From<HoldReason>;

		/// The type used for the curve parameters.
		type CurveParameterType: Parameter
			+ Member
			+ FixedSigned
			+ MaxEncodedLen
			+ PartialOrd<Precision>
			+ From<Precision>;

		type CurveParameterInput: Parameter + FixedUnsigned + MaxEncodedLen;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn pools)]
	pub(crate) type Pools<T: Config> = StorageMap<_, Twox64Concat, T::PoolId, PoolDetailsOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nex_asset_id)]
	pub(crate) type NextAssetId<T: Config> = StorageValue<_, FungiblesAssetIdOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PoolCreated {
			id: T::PoolId,
		},
		/// A bonded token pool has been moved to refunding state.
		RefundingStarted {
			id: T::PoolId,
		},
		/// A bonded token pool has been moved to destroying state.
		DestructionStarted {
			id: T::PoolId,
		},
		/// Collateral distribution to bonded token holders has been completed for this pool - no more tokens or no more collateral to distribute.   
		RefundComplete {
			id: T::PoolId,
		},
		/// A bonded token pool has been fully destroyed and all collateral and deposits have been refunded.
		Destroyed {
			id: T::PoolId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The pool id is not currently registered.
		PoolUnknown,
		/// The pool has no associated bonded currency with the given index.
		IndexOutOfBounds,
		/// The pool does not hold collateral to be refunded, or has no remaining supply of tokens to exchange. Call start_destroy to intiate teardown.
		NothingToRefund,
		/// The user is not privileged to perform the requested operation.
		NoPermission,
		/// The pool is deactivated (i.e., in destroying or refunding state) and not available for use.
		PoolNotLive,
		/// There are active accounts associated with this pool and thus it cannot be destroyed at this point.
		LivePool,
		/// This operation can only be made when the pool is in refunding state.
		NotRefunding,
		/// The number of currencies linked to a pool exceeds the limit parameter. Thrown by transactions that require specifying the number of a pool's currencies in order to determine weight limits upfront.
		CurrencyCount,
		InvalidInput,
		Internal,
		Slippage,
	}

	#[pallet::composite_enum]
	pub enum HoldReason {
		Deposit,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<CurveParameterTypeOf<T> as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
	{
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn create_pool(
			origin: OriginFor<T>,
			curve: CurveInput<CurveParameterInputOf<T>>,
			currencies: BoundedVec<TokenMetaOf<T>, T::MaxCurrencies>,
			denomination: u8,
			transferable: bool,
		) -> DispatchResult {
			let who = T::PoolCreateOrigin::ensure_origin(origin)?;

			let currency_length = currencies.len();

			let checked_curve = curve.try_into().map_err(|_| Error::<T>::InvalidInput)?;

			let current_asset_id = NextAssetId::<T>::get();

			let (currency_ids, next_asset_id) = Self::generate_sequential_asset_ids(current_asset_id, currency_length)?;

			let pool_id = T::PoolId::from(currency_ids.blake2_256());

			// Todo: change that.
			T::DepositCurrency::hold(
				&T::RuntimeHoldReason::from(HoldReason::Deposit),
				&who,
				Self::calculate_pool_deposit(currency_length),
			)?;

			let pool_account = &pool_id.clone().into();

			currencies
				.into_iter()
				.zip(currency_ids.iter())
				.try_for_each(|(entry, asset_id)| -> DispatchResult {
					let TokenMeta {
						min_balance,
						name,
						symbol,
					} = entry;

					T::Fungibles::create(asset_id.clone(), pool_account.to_owned(), false, min_balance)?;

					// set metadata for new asset class
					T::Fungibles::set(
						asset_id.to_owned(),
						pool_account,
						name.into_inner(),
						symbol.into_inner(),
						denomination,
					)?;

					Ok(())
				})?;

			// Touch the pool account in order to be able to transfer the collateral currency to it
			T::CollateralCurrency::touch(T::CollateralAssetId::get(), pool_account, &who)?;

			Pools::<T>::set(
				&pool_id,
				Some(PoolDetails::new(
					who,
					checked_curve,
					currency_ids,
					transferable,
					denomination,
				)),
			);

			// update the storage for the next tx.
			NextAssetId::<T>::set(next_asset_id);

			Self::deposit_event(Event::PoolCreated { id: pool_id });

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn reset_team(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			team: PoolManagingTeam<AccountIdOf<T>>,
			currency_idx: u32,
		) -> DispatchResult {
			let who = T::DefaultOrigin::ensure_origin(origin)?;

			let pool_details = Pools::<T>::get(&pool_id).ok_or(Error::<T>::PoolUnknown)?;

			ensure!(pool_details.is_manager(&who), Error::<T>::NoPermission);

			let asset_id = pool_details
				.bonded_currencies
				.get(currency_idx as usize)
				.ok_or(Error::<T>::IndexOutOfBounds)?;

			let pool_id_account = pool_id.into();

			let PoolManagingTeam { freezer, admin } = team;

			T::Fungibles::reset_team(
				asset_id.to_owned(),
				pool_id_account.clone(),
				admin,
				pool_id_account,
				freezer,
			)
		}

		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn reset_manager(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			manager: Option<AccountIdOf<T>>,
		) -> DispatchResult {
			let who = T::DefaultOrigin::ensure_origin(origin)?;
			Pools::<T>::try_mutate(pool_id, |maybe_entry| -> DispatchResult {
				let entry = maybe_entry.as_mut().ok_or(Error::<T>::PoolUnknown)?;
				ensure!(entry.is_manager(&who), Error::<T>::NoPermission);
				entry.manager = manager;
				Ok(())
			})
		}

		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn mint_into(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			currency_idx: u32,
			amount_to_mint: FungiblesBalanceOf<T>,
			max_cost: CollateralCurrencyBalanceOf<T>,
			beneficiary: AccountIdLookupOf<T>,
			currency_count: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let beneficiary = T::Lookup::lookup(beneficiary)?;

			let pool_details = Pools::<T>::get(pool_id.clone()).ok_or(Error::<T>::PoolUnknown)?;

			ensure!(pool_details.is_minting_authorized(&who), Error::<T>::NoPermission);

			let bonded_currencies = pool_details.bonded_currencies;

			ensure!(
				bonded_currencies.len() <= currency_count.saturated_into::<usize>(),
				Error::<T>::CurrencyCount
			);

			let currency_idx: usize = currency_idx.saturated_into();

			let target_currency_id = bonded_currencies
				.get(currency_idx)
				.ok_or(Error::<T>::IndexOutOfBounds)?;

			let (active_pre, passive) = Self::calculate_normalized_passive_issuance(
				&bonded_currencies,
				pool_details.denomination,
				currency_idx,
			)?;

			let normalized_amount_to_mint =
				convert_to_fixed::<T>(amount_to_mint.saturated_into::<u128>(), pool_details.denomination)?;

			let active_post = active_pre
				.checked_add(normalized_amount_to_mint)
				.ok_or(ArithmeticError::Overflow)?;

			let cost = Self::calculate_collateral(active_pre, active_post, passive, &pool_details.curve)?;

			// fail if cost > max_cost
			ensure!(cost <= max_cost, Error::<T>::Slippage);

			// Transfer the collateral. We do not want to kill the minter, so this operation can fail if the account is being reaped.
			T::CollateralCurrency::transfer(
				T::CollateralAssetId::get(),
				&who,
				&pool_id.into(),
				cost,
				Preservation::Preserve,
			)?;

			T::Fungibles::mint_into(target_currency_id.clone(), &beneficiary, amount_to_mint)?;

			if !pool_details.transferable {
				T::Fungibles::freeze(target_currency_id, &beneficiary).map_err(|freeze_error| freeze_error.into())?;
			}

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn burn_into(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		#[pallet::call_index(6)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn swap_into(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		#[pallet::call_index(7)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn set_lock(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		#[pallet::call_index(8)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn unlock(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		#[pallet::call_index(9)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn start_refund(origin: OriginFor<T>, pool_id: T::PoolId, currency_count: u32) -> DispatchResult {
			let who = T::DefaultOrigin::ensure_origin(origin)?;

			Self::do_start_refund(pool_id, currency_count, Some(&who))?;

			Ok(())
		}

		#[pallet::call_index(10)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn force_start_refund(origin: OriginFor<T>, pool_id: T::PoolId, currency_count: u32) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;

			Self::do_start_refund(pool_id, currency_count, None)?;

			Ok(())
		}

		#[pallet::call_index(11)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn refund_account(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			account: AccountIdLookupOf<T>,
			asset_idx: u32,
			currency_count: u32,
		) -> DispatchResult {
			T::DefaultOrigin::ensure_origin(origin)?;
			let who = T::Lookup::lookup(account)?;

			let pool_details = Pools::<T>::get(&pool_id).ok_or(Error::<T>::PoolUnknown)?;

			ensure!(
				Self::get_currencies_number(&pool_details) <= currency_count,
				Error::<T>::CurrencyCount
			);

			ensure!(pool_details.state.is_refunding(), Error::<T>::NotRefunding);

			// get asset id from linked assets vector
			let asset_id: &FungiblesAssetIdOf<T> = pool_details
				.bonded_currencies
				.get(asset_idx.saturated_into::<usize>())
				.ok_or(Error::<T>::IndexOutOfBounds)?;

			let pool_account = pool_id.clone().into();

			// Choosing total_balance over reducible_balance to ensure that all funds are distributed fairly;
			// in case of any locks present on the pool account, this could lead to refunds failing to execute though.
			// This case would have to be resolved by governance, either by removing locks or force_destroying the pool.
			let total_collateral_issuance =
				T::CollateralCurrency::total_balance(T::CollateralAssetId::get(), &pool_account);

			// nothing to distribute; refunding is complete, user should call start_destroy
			ensure!(
				total_collateral_issuance > CollateralCurrencyBalanceOf::<T>::zero(),
				Error::<T>::NothingToRefund
			);

			// TODO: remove any existing locks on the account prior to burning

			// With amount = max_value(), this trait implementation burns the reducible balance on the account and returns the actual amount burnt
			let burnt = T::Fungibles::burn_from(
				asset_id.clone(),
				&who,
				Bounded::max_value(),
				WithdrawalPrecision::BestEffort,
				Fortitude::Force,
			)?;

			if burnt.is_zero() {
				// no funds available to be burnt on account; nothing to do here
				return Ok(());
			}

			let sum_of_issuances = pool_details
				.bonded_currencies
				.into_iter()
				.fold(FungiblesBalanceOf::<T>::zero(), |sum, id| {
					sum.saturating_add(T::Fungibles::total_issuance(id))
				});

			let amount = burnt
				.checked_mul(&total_collateral_issuance)
				.ok_or(ArithmeticError::Overflow)? // TODO: do we need a fallback if this fails?
				.checked_div(&sum_of_issuances)
				.ok_or(Error::<T>::NothingToRefund)?; // should be impossible - how would we be able to burn funds if the sum of total supplies is 0?

			if amount.is_zero()
				|| T::CollateralCurrency::can_deposit(T::CollateralAssetId::get(), &who, amount, Provenance::Extant)
					.into_result()
					.is_err()
			{
				// funds are burnt but the collateral received is not sufficient to be deposited to the account
				// this is tolerated as otherwise we could have edge cases where it's impossible to refund at least some accounts
				return Ok(());
			}

			let transferred = T::CollateralCurrency::transfer(
				T::CollateralAssetId::get(),
				&pool_account,
				&who,
				amount,
				Preservation::Expendable,
			)?; // TODO: check edge cases around existential deposit

			// if collateral or total supply drops to zero, refunding is complete -> emit event
			if sum_of_issuances <= burnt || total_collateral_issuance <= transferred {
				Self::deposit_event(Event::RefundComplete { id: pool_id });
			}

			Ok(())
		}

		#[pallet::call_index(12)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn start_destroy(origin: OriginFor<T>, pool_id: T::PoolId, currency_count: u32) -> DispatchResult {
			let who = T::DefaultOrigin::ensure_origin(origin)?;

			Self::do_start_destroy_pool(pool_id, currency_count, false, Some(&who))?;

			Ok(())
		}

		#[pallet::call_index(13)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn force_start_destroy(origin: OriginFor<T>, pool_id: T::PoolId, currency_count: u32) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;

			Self::do_start_destroy_pool(pool_id, currency_count, true, None)?;

			Ok(())
		}

		#[pallet::call_index(14)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn finish_destroy(origin: OriginFor<T>, pool_id: T::PoolId, currency_count: u32) -> DispatchResult {
			T::DefaultOrigin::ensure_origin(origin)?;

			let pool_details = Pools::<T>::get(&pool_id).ok_or(Error::<T>::PoolUnknown)?;

			let n_currencies = Self::get_currencies_number(&pool_details);

			ensure!(n_currencies <= currency_count, Error::<T>::CurrencyCount);

			ensure!(pool_details.state.is_destroying(), Error::<T>::LivePool);

			for asset_id in pool_details.bonded_currencies {
				if T::Fungibles::asset_exists(asset_id.clone()) {
					// This would fail with an LiveAsset error if there are any accounts left on any currency
					T::Fungibles::finish_destroy(asset_id)?;
				}
			}

			let pool_account = pool_id.clone().into();

			let total_collateral_issuance =
				T::CollateralCurrency::total_balance(T::CollateralAssetId::get(), &pool_account);

			if total_collateral_issuance > CollateralCurrencyBalanceOf::<T>::zero() {
				T::CollateralCurrency::transfer(
					T::CollateralAssetId::get(),
					&pool_account,
					&pool_details.owner,
					total_collateral_issuance,
					Preservation::Expendable,
				)?;
			}

			Pools::<T>::remove(&pool_id);

			T::DepositCurrency::release(
				&T::RuntimeHoldReason::from(HoldReason::Deposit),
				&pool_details.owner,
				Self::calculate_pool_deposit(n_currencies),
				WithdrawalPrecision::Exact,
			)?;

			Self::deposit_event(Event::Destroyed { id: pool_id });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T>
	where
		<CurveParameterTypeOf<T> as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
	{
		fn calculate_collateral(
			low: CurveParameterTypeOf<T>,
			high: CurveParameterTypeOf<T>,
			passive_supply: PassiveSupply<CurveParameterTypeOf<T>>,
			curve: &Curve<CurveParameterTypeOf<T>>,
		) -> Result<CollateralCurrencyBalanceOf<T>, ArithmeticError> {
			let normalized_costs = curve.calculate_costs(low, high, passive_supply)?;

			let collateral_denomination = 10u128
				.checked_pow(T::CollateralCurrency::decimals(T::CollateralAssetId::get()).into())
				.ok_or(ArithmeticError::Overflow)?;

			let real_costs = normalized_costs
				.checked_mul(CurveParameterTypeOf::<T>::from_num(collateral_denomination))
				.ok_or(ArithmeticError::Overflow)?
				// should never fail
				.checked_to_num::<u128>()
				.ok_or(ArithmeticError::Overflow)?
				.saturated_into();

			Ok(real_costs)
		}

		fn calculate_normalized_passive_issuance(
			bonded_currencies: &[FungiblesAssetIdOf<T>],
			denomination: u8,
			currency_idx: usize,
		) -> Result<(CurveParameterTypeOf<T>, PassiveSupply<CurveParameterTypeOf<T>>), DispatchError> {
			let currencies_total_supply = bonded_currencies
				.iter()
				.map(|currency_id| T::Fungibles::total_issuance(currency_id.to_owned()))
				.collect::<Vec<_>>();

			let normalized_total_issuances = currencies_total_supply
				.iter()
				.map(|x| convert_to_fixed::<T>(x.to_owned().saturated_into::<u128>(), denomination))
				.collect::<Result<Vec<CurveParameterTypeOf<T>>, ArithmeticError>>()?;

			let active_issuance = normalized_total_issuances
				.get(currency_idx)
				.ok_or(Error::<T>::IndexOutOfBounds)?
				.to_owned();

			let passive_issuance = normalized_total_issuances
				.iter()
				.enumerate()
				.filter_map(|(idx, x)| if idx != currency_idx { Some(x.to_owned()) } else { None })
				.collect();

			Ok((active_issuance, passive_issuance))
		}

		fn do_start_refund(
			pool_id: T::PoolId,
			max_currencies: u32,
			maybe_check_manager: Option<&AccountIdOf<T>>,
		) -> Result<u32, DispatchError> {
			let pool_details = Pools::<T>::get(&pool_id).ok_or(Error::<T>::PoolUnknown)?;

			let n_currencies = Self::get_currencies_number(&pool_details);

			ensure!(n_currencies <= max_currencies, Error::<T>::CurrencyCount);

			// refunding can only be triggered on a live pool
			ensure!(pool_details.state.is_live(), Error::<T>::PoolNotLive);

			if let Some(caller) = maybe_check_manager {
				// TODO: should the owner be authorized as well?
				ensure!(pool_details.is_manager(caller), Error::<T>::NoPermission);
			}

			let total_collateral_issuance =
				T::CollateralCurrency::total_balance(T::CollateralAssetId::get(), &pool_id.clone().into());
			// nothing to distribute
			ensure!(
				total_collateral_issuance > CollateralCurrencyBalanceOf::<T>::zero(),
				Error::<T>::NothingToRefund
			);

			let has_holders = pool_details
				.bonded_currencies
				.iter()
				.any(|asset_id| T::Fungibles::total_issuance(asset_id.clone()) > FungiblesBalanceOf::<T>::zero());
			// no token holders to refund
			ensure!(has_holders, Error::<T>::NothingToRefund);

			// switch pool state to refunding
			let mut new_pool_details = pool_details;
			new_pool_details.state.start_refund();
			Pools::<T>::set(&pool_id, Some(new_pool_details));

			Self::deposit_event(Event::RefundingStarted { id: pool_id });

			Ok(n_currencies)
		}

		fn do_start_destroy_pool(
			pool_id: T::PoolId,
			max_currencies: u32,
			force_skip_refund: bool,
			maybe_check_manager: Option<&AccountIdOf<T>>,
		) -> Result<u32, DispatchError> {
			let pool_details = Pools::<T>::get(&pool_id).ok_or(Error::<T>::PoolUnknown)?;

			let n_currencies = Self::get_currencies_number(&pool_details);

			ensure!(n_currencies <= max_currencies, Error::<T>::CurrencyCount);

			ensure!(
				pool_details.state.is_live() || pool_details.state.is_refunding(),
				Error::<T>::PoolNotLive
			);

			if let Some(caller) = maybe_check_manager {
				// TODO: should this be permissionless if the pool is in refunding state?
				ensure!(
					pool_details.is_owner(caller) || pool_details.is_manager(caller),
					Error::<T>::NoPermission
				);
			}

			if !force_skip_refund {
				let total_collateral_issuance =
					T::CollateralCurrency::total_balance(T::CollateralAssetId::get(), &pool_id.clone().into());

				if total_collateral_issuance > CollateralCurrencyBalanceOf::<T>::zero() {
					let has_holders = pool_details.bonded_currencies.iter().any(|asset_id| {
						T::Fungibles::total_issuance(asset_id.clone()) > FungiblesBalanceOf::<T>::zero()
					});
					// destruction is only allowed when there are no holders or no collateral to distribute
					ensure!(!has_holders, Error::<T>::LivePool);
				}
			}

			// cloning the currency ids now lets us avoid cloning the entire pool_details
			let bonded_currencies = pool_details.bonded_currencies.clone();

			// switch pool state to destroying
			let mut new_pool_details = pool_details;
			new_pool_details.state.start_destroy();
			Pools::<T>::set(&pool_id, Some(new_pool_details));

			// emit this event before the destruction started events are emitted by assets deactivation
			Self::deposit_event(Event::DestructionStarted { id: pool_id });

			// deactivate all currencies
			for asset_id in bonded_currencies {
				// Governance or other pallets using the fungibles trait can in theory destroy an asset without this pallet knowing, so we check if it's still around
				if T::Fungibles::asset_exists(asset_id.clone()) {
					T::Fungibles::start_destroy(asset_id, None)?;
				}
			}

			Ok(n_currencies)
		}

		fn generate_sequential_asset_ids(
			mut start_id: T::AssetId,
			count: usize,
		) -> Result<(BoundedCurrencyVec<T>, T::AssetId), Error<T>> {
			let mut currency_ids_vec = Vec::new();
			for _ in 0..count {
				currency_ids_vec.push(start_id.clone());
				start_id.saturating_inc();
			}

			let currency_array = BoundedVec::<FungiblesAssetIdOf<T>, T::MaxCurrencies>::try_from(currency_ids_vec)
				.map_err(|_| Error::<T>::Internal)?;

			Ok((currency_array, start_id))
		}

		fn get_currencies_number(pool_details: &PoolDetailsOf<T>) -> u32 {
			// bonded_currencies is a BoundedVec with maximum length MaxCurrencies, which is a u32; conversion to u32 must thus be lossless.
			pool_details.bonded_currencies.len().saturated_into()
		}

		fn calculate_pool_deposit<N: UniqueSaturatedInto<DepositCurrencyBalanceOf<T>>>(
			n_currencies: N,
		) -> DepositCurrencyBalanceOf<T> {
			T::BaseDeposit::get()
				.saturating_add(T::DepositPerCurrency::get().saturating_mul(n_currencies.saturated_into()))
		}
	}
}
