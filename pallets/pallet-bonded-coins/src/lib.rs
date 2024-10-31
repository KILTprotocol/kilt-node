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
			tokens::{Fortitude, Precision as TokenPrecision, Preservation},
			AccountTouch,
		},
		Hashable, Parameter,
	};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::FullCodec;
	use sp_arithmetic::ArithmeticError;
	use sp_runtime::{
		traits::{One, Saturating, StaticLookup},
		BoundedVec, SaturatedConversion,
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
		PoolCreated { id: T::PoolId },
	}

	#[pallet::error]
	pub enum Error<T> {
		IndexOutOfBounds,
		PoolUnknown,
		NoPermission,
		Slippage,
		Internal,
		CurrencyCount,
		InvalidInput,
		UnknownPool,
		NoPermission,
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
				T::BaseDeposit::get().saturating_add(
					T::DepositPerCurrency::get()
						.saturating_mul(currency_length.saturated_into())
						.saturated_into(),
				),
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
						&pool_account,
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

			let pool_details = Pools::<T>::get(&pool_id).ok_or(Error::<T>::UnknownPool)?;

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
				let entry = maybe_entry.as_mut().ok_or(Error::<T>::UnknownPool)?;
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

			let pool_details = Pools::<T>::get(&pool_id).ok_or(Error::<T>::PoolUnknown)?;

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

			ensure!(pool_details.is_minting_authorized(&who), Error::<T>::Locked);

			let bonded_currencies = pool_details.bonded_currencies;

			let currency_idx: usize = currency_idx.saturated_into();

			let target_currency_id = bonded_currencies
				.get(currency_idx)
				.ok_or(Error::<T>::IndexOutOfBounds)?;

			let (high, passive) = Self::calculate_normalized_passive_issuance(
				&bonded_currencies,
				pool_details.denomination,
				currency_idx,
			)?;

			let normalized_amount_to_burn =
				convert_to_fixed::<T>(amount_to_burn.saturated_into::<u128>(), pool_details.denomination)?;

			let low = high
				.checked_sub(normalized_amount_to_burn)
				.ok_or(ArithmeticError::Overflow)?;

			let collateral_return = Self::calculate_collateral(low, high, passive, &pool_details.curve)?;

			ensure!(collateral_return >= min_return, Error::<T>::Slippage);

			T::CollateralCurrency::transfer(
				T::CollateralAssetId::get(),
				&pool_id.into(),
				&beneficiary,
				collateral_return,
				Preservation::Expendable,
			)?;

			// we act on behalf of the admin.
			let admin = T::Fungibles::admin(target_currency_id.clone())
				// Should never fail. Either the admin has been updated or it is the pool id.
				.ok_or_else(|| {
					log::error!(
						target: LOG_TARGET,
						"Admin not found for currency id: {:?}",
						target_currency_id
					);
					Error::<T>::Internal
				})?;

			// just remove any locks, if existing.
			T::Fungibles::thaw(&admin, &beneficiary, target_currency_id).map_err(|freeze_error| freeze_error.into())?;

			T::Fungibles::burn_from(
				target_currency_id.clone(),
				&beneficiary,
				amount_to_burn,
				TokenPrecision::Exact,
				Fortitude::Polite,
			)?;

			if !pool_details.transferable {
				T::Fungibles::freeze(target_currency_id, &beneficiary).map_err(|freeze_error| freeze_error.into())?;
			}

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn burn_into(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			currency_idx: u32,
			amount_to_burn: FungiblesBalanceOf<T>,
			min_return: CollateralCurrencyBalanceOf<T>,
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

			let (high, passive) = Self::calculate_normalized_passive_issuance(
				&bonded_currencies,
				pool_details.denomination,
				currency_idx,
			)?;

			let normalized_amount_to_burn =
				convert_to_fixed::<T>(amount_to_burn.saturated_into::<u128>(), pool_details.denomination)?;

			let low = high
				.checked_sub(normalized_amount_to_burn)
				.ok_or(ArithmeticError::Underflow)?;

			let collateral_return = Self::calculate_collateral(low, high, passive, &pool_details.curve)?;

			ensure!(collateral_return >= min_return, Error::<T>::Slippage);

			T::CollateralCurrency::transfer(
				T::CollateralAssetId::get(),
				&pool_id.into(),
				&beneficiary,
				collateral_return,
				Preservation::Expendable,
			)?;

			// just remove any locks, if existing.
			T::Fungibles::thaw(target_currency_id, &beneficiary).map_err(|freeze_error| freeze_error.into())?;

			T::Fungibles::burn_from(
				target_currency_id.clone(),
				&beneficiary,
				amount_to_burn,
				TokenPrecision::Exact,
				Fortitude::Force,
			)?;

			if !pool_details.transferable {
				// Restore locks. Act on behalf of the freezer.
				let freezer = T::Fungibles::freezer(target_currency_id.clone())
					// Should never fail. Either the freezer has been updated or it is the pool id.
					.ok_or_else(|| {
						log::error!(
							target: LOG_TARGET,
							"Freezer not found for currency id: {:?}",
							target_currency_id
						);
						Error::<T>::Internal
					})?;

				T::Fungibles::freeze(&freezer, &beneficiary, target_currency_id)
					.map_err(|freeze_error| freeze_error.into())?;
			}

			Ok(())
		}

		#[pallet::call_index(6)]
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

			let pool_details = Pools::<T>::get(&pool_id).ok_or(Error::<T>::PoolUnknown)?;

			ensure!(pool_details.is_swapping_authorized(&who), Error::<T>::Locked);

			let from_idx: usize = from_idx.saturated_into();
			let to_idx: usize = to_idx.saturated_into();

			match &pool_details.curve {
				Curve::LMSR(params) => {
					let (high, passive) = Self::calculate_normalized_passive_issuance(
						&pool_details.bonded_currencies,
						pool_details.denomination,
						from_idx,
					)?;

					let normalized_amount_to_burn =
						convert_to_fixed::<T>(amount_to_swap.saturated_into::<u128>(), pool_details.denomination)?;

					let low = high
						.checked_sub(normalized_amount_to_burn)
						.ok_or(ArithmeticError::Overflow)?;

					let collateral_return =
						Self::calculate_collateral(low, high, passive.clone(), &pool_details.curve)?;

					let from_currency_id = pool_details
						.bonded_currencies
						.get(from_idx)
						.ok_or(Error::<T>::IndexOutOfBounds)?;

					T::Fungibles::burn_from(
						from_currency_id.to_owned(),
						&beneficiary,
						amount_to_swap,
						TokenPrecision::Exact,
						Fortitude::Polite,
					)?;

					let normalized_collateral =
						convert_to_fixed::<T>(collateral_return.saturated_into::<u128>(), pool_details.denomination)?;

					let share_to_mint = params.calculate_shares_from_collateral(normalized_collateral, passive, to_idx);
				}
				// The price for burning and minting in the pool is the same, if the bonding curve is not [LMSR].
				_ => {
					let from_currency_id = pool_details
						.bonded_currencies
						.get(from_idx)
						.ok_or(Error::<T>::IndexOutOfBounds)?;

					let to_currency_id = pool_details
						.bonded_currencies
						.get(to_idx)
						.ok_or(Error::<T>::IndexOutOfBounds)?;

					T::Fungibles::burn_from(
						from_currency_id.clone(),
						&beneficiary,
						amount_to_swap,
						TokenPrecision::Exact,
						Fortitude::Polite,
					)?;
					T::Fungibles::mint_into(to_currency_id.clone(), &beneficiary, amount_to_swap)?;
				}
			};

			Ok(())
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

		// TODO: not sure if we really need that. Check that out with Raphael.
		#[pallet::call_index(9)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn start_destroy(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		#[pallet::call_index(10)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn force_start_destroy(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		// todo: check if we really need that tx.
		#[pallet::call_index(11)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn destroy_accounts(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		#[pallet::call_index(12)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn finish_destroy(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
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
	}
}
