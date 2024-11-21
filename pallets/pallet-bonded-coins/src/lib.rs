#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(any(test, feature = "runtime-benchmarks"))]
mod mock;
#[cfg(test)]
mod tests;

mod curves;
mod default_weights;
pub mod traits;
mod types;
pub use default_weights::WeightInfo;

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
			tokens::{Fortitude, Precision as WithdrawalPrecision, Preservation, Provenance},
			AccountTouch,
		},
		Hashable, Parameter,
	};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::FullCodec;
	use sp_arithmetic::ArithmeticError;
	use sp_core::U256;
	use sp_runtime::{
		traits::{
			Bounded, CheckedConversion, One, SaturatedConversion, Saturating, StaticLookup, UniqueSaturatedInto, Zero,
		},
		BoundedVec,
	};
	use sp_std::{
		default::Default,
		ops::{AddAssign, BitOrAssign, ShlAssign},
		prelude::*,
		vec::Vec,
	};
	use substrate_fixed::{
		traits::{Fixed, FixedSigned, FixedUnsigned, ToFixed},
		types::I9F23,
	};

	use crate::{
		curves::{convert_to_fixed, BondingFunction, Curve, CurveInput},
		traits::{FreezeAccounts, ResetTeam},
		types::{Locks, PoolDetails, PoolManagingTeam, PoolStatus, TokenMeta},
		WeightInfo,
	};

	pub(crate) type AccountIdLookupOf<T> =
		<<T as frame_system::Config>::Lookup as sp_runtime::traits::StaticLookup>::Source;

	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	pub(crate) type DepositCurrencyBalanceOf<T> =
		<<T as Config>::DepositCurrency as InspectFungible<<T as frame_system::Config>::AccountId>>::Balance;

	pub(crate) type CollateralCurrenciesBalanceOf<T> =
		<<T as Config>::CollateralCurrencies as InspectFungibles<<T as frame_system::Config>::AccountId>>::Balance;

	pub(crate) type FungiblesBalanceOf<T> =
		<<T as Config>::Fungibles as InspectFungibles<<T as frame_system::Config>::AccountId>>::Balance;

	pub(crate) type FungiblesAssetIdOf<T> =
		<<T as Config>::Fungibles as InspectFungibles<<T as frame_system::Config>::AccountId>>::AssetId;

	pub(crate) type CollateralAssetIdOf<T> =
		<<T as Config>::CollateralCurrencies as InspectFungibles<<T as frame_system::Config>::AccountId>>::AssetId;

	pub(crate) type BoundedCurrencyVec<T> = BoundedVec<FungiblesAssetIdOf<T>, <T as Config>::MaxCurrencies>;

	pub(crate) type CurrencyNameOf<T> = BoundedVec<u8, <T as Config>::MaxStringLength>;

	pub(crate) type CurrencySymbolOf<T> = BoundedVec<u8, <T as Config>::MaxStringLength>;

	pub(crate) type CurveParameterTypeOf<T> = <T as Config>::CurveParameterType;

	pub(crate) type CurveParameterInputOf<T> = <T as Config>::CurveParameterInput;

	pub(crate) type PoolDetailsOf<T> = PoolDetails<
		<T as frame_system::Config>::AccountId,
		Curve<CurveParameterTypeOf<T>>,
		BoundedCurrencyVec<T>,
		CollateralAssetIdOf<T>,
	>;

	pub(crate) type Precision = I9F23;

	pub(crate) type PassiveSupply<T> = Vec<T>;

	pub(crate) type TokenMetaOf<T> = TokenMeta<FungiblesBalanceOf<T>, CurrencyNameOf<T>, CurrencySymbolOf<T>>;

	pub(crate) const LOG_TARGET: &str = "runtime::pallet-bonded-coins";

	/// Configure the pallet by specifying the parameters and types on which it
	/// depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's
		/// definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The currency used for storage deposits.
		type DepositCurrency: MutateHold<Self::AccountId, Reason = Self::RuntimeHoldReason>;
		/// A fungibles trait implementation to interact with currencies which
		/// can be used as collateral for minting bonded tokens.
		type CollateralCurrencies: MutateFungibles<Self::AccountId>
			+ AccountTouch<CollateralAssetIdOf<Self>, Self::AccountId>
			+ FungiblesMetadata<Self::AccountId>;
		/// Implementation of creating and managing new fungibles
		type Fungibles: CreateFungibles<Self::AccountId, AssetId = Self::AssetId>
			+ DestroyFungibles<Self::AccountId>
			+ FungiblesMetadata<Self::AccountId>
			+ FungiblesInspect<Self::AccountId>
			+ MutateFungibles<Self::AccountId, Balance = CollateralCurrenciesBalanceOf<Self>>
			+ FreezeAccounts<Self::AccountId, Self::AssetId>
			+ ResetTeam<Self::AccountId>;
		/// The maximum number of currencies allowed for a single pool.
		#[pallet::constant]
		type MaxCurrencies: Get<u32>;

		#[pallet::constant]
		type MaxStringLength: Get<u32>;

		/// The maximum denomination that bonded currencies can use. This should
		/// be configured so that
		/// 10^MaxDenomination < 2^CurveParameterType::frac_nbits()
		/// as larger denominations could result in truncation.
		#[pallet::constant]
		type MaxDenomination: Get<u8>;

		/// The deposit required for each bonded currency.
		#[pallet::constant]
		type DepositPerCurrency: Get<DepositCurrencyBalanceOf<Self>>;

		/// The base deposit required to create a new pool, primarily to cover
		/// the ED of the pool account.
		#[pallet::constant]
		type BaseDeposit: Get<DepositCurrencyBalanceOf<Self>>;

		/// The origin for most permissionless and priviledged operations.
		type DefaultOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
		/// The dedicated origin for creating new bonded currency pools
		/// (typically permissionless).
		type PoolCreateOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
		/// The origin for permissioned operations (force_* transactions).
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// The type used for pool ids
		type PoolId: Parameter + MaxEncodedLen + From<[u8; 32]> + Into<Self::AccountId>;

		/// The type used for asset ids. This is the type of the bonded
		/// currencies.
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

		type WeightInfo: WeightInfo;

		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper: crate::benchmarking::BenchmarkHelper<Self>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn integrity_test() {
			let scaling_factor = U256::from(10).checked_pow(T::MaxDenomination::get().into()).expect(
				"`MaxDenomination` is set so high that the resulting scaling factor cannot be represented. /
				Any attempt to mint or burn on a pool where `10^denomination > 2^256` _WILL_ fail.",
			);

			assert!(
				U256::from(2).pow(T::CurveParameterType::frac_nbits().into()) > scaling_factor,
				"In order to prevent truncation of balances, `MaxDenomination` should be configured such \
				that the maximum scaling factor `10^MaxDenomination` is smaller than the fractional \
				capacity `2^frac_nbits` of `CurveParameterType`",
			);
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn pools)]
	pub(crate) type Pools<T: Config> = StorageMap<_, Twox64Concat, T::PoolId, PoolDetailsOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nex_asset_id)]
	pub(crate) type NextAssetId<T: Config> = StorageValue<_, FungiblesAssetIdOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		LockSet {
			id: T::PoolId,
			lock: Locks,
		},
		Unlocked {
			id: T::PoolId,
		},
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
		/// Collateral distribution to bonded token holders has been completed
		/// for this pool (no more tokens or no more collateral to distribute).
		RefundComplete {
			id: T::PoolId,
		},
		/// A bonded token pool has been fully destroyed and all collateral and
		/// deposits have been refunded.
		Destroyed {
			id: T::PoolId,
		},
		/// The manager of a pool has been updated.
		ManagerUpdated {
			id: T::PoolId,
			manager: Option<T::AccountId>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The pool id is not currently registered.
		PoolUnknown,
		/// The pool has no associated bonded currency with the given index.
		IndexOutOfBounds,
		/// The pool does not hold collateral to be refunded, or has no
		/// remaining supply of tokens to exchange. Call start_destroy to
		/// intiate teardown.
		NothingToRefund,
		/// The user is not privileged to perform the requested operation.
		NoPermission,
		/// The pool is deactivated (i.e., in destroying or refunding state) and
		/// not available for use.
		PoolNotLive,
		/// There are active accounts associated with this pool and thus it
		/// cannot be destroyed at this point.
		LivePool,
		/// This operation can only be made when the pool is in refunding state.
		NotRefunding,
		/// The number of currencies linked to a pool exceeds the limit
		/// parameter. Thrown by transactions that require specifying the number
		/// of a pool's currencies in order to determine weight limits upfront.
		CurrencyCount,
		InvalidInput,
		Internal,
		Slippage,
		/// The user has no bonded currencies.
		InsufficientBalance,
	}

	#[pallet::composite_enum]
	pub enum HoldReason {
		Deposit,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<CurveParameterTypeOf<T> as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign + TryFrom<U256>,
		CollateralCurrenciesBalanceOf<T>: Into<U256> + TryFrom<U256>, // TODO: make large integer type configurable
	{
		#[pallet::call_index(0)]
		#[pallet::weight({
			let currency_length = currencies.len().saturated_into();
			let weight_polynomial = T::WeightInfo::create_pool_polynomial(currency_length);
			let weight_square_root = T::WeightInfo::create_pool_square_root(currency_length);
			let weight_lmsr = T::WeightInfo::create_pool_lmsr(currency_length);
			weight_polynomial.max(weight_square_root).max(weight_lmsr)
		})]
		pub fn create_pool(
			origin: OriginFor<T>,
			curve: CurveInput<CurveParameterInputOf<T>>,
			collateral_id: CollateralAssetIdOf<T>,
			currencies: BoundedVec<TokenMetaOf<T>, T::MaxCurrencies>,
			denomination: u8,
			transferable: bool,
		) -> DispatchResult {
			let who = T::PoolCreateOrigin::ensure_origin(origin)?;

			ensure!(denomination <= T::MaxDenomination::get(), Error::<T>::InvalidInput);

			let checked_curve: Curve<CurveParameterTypeOf<T>> =
				curve.try_into().map_err(|_| Error::<T>::InvalidInput)?;

			let currency_length = currencies.len();

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

			// Touch the pool account in order to be able to transfer the collateral
			// currency to it. This should also verify that the currency actually exists.
			T::CollateralCurrencies::touch(collateral_id.clone(), pool_account, &who)?;

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

			Pools::<T>::set(
				&pool_id,
				Some(PoolDetails::new(
					who,
					checked_curve,
					collateral_id,
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

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::reset_team())]
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
				.get(currency_idx.saturated_into::<usize>())
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

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::reset_manager())]
		pub fn reset_manager(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			new_manager: Option<AccountIdOf<T>>,
		) -> DispatchResult {
			let who = T::DefaultOrigin::ensure_origin(origin)?;
			Pools::<T>::try_mutate(&pool_id, |maybe_entry| -> DispatchResult {
				let entry = maybe_entry.as_mut().ok_or(Error::<T>::PoolUnknown)?;
				ensure!(entry.is_manager(&who), Error::<T>::NoPermission);
				entry.manager = new_manager.clone();

				Ok(())
			})?;

			Self::deposit_event(Event::ManagerUpdated {
				id: pool_id,
				manager: new_manager,
			});

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::set_lock())]
		pub fn set_lock(origin: OriginFor<T>, pool_id: T::PoolId, lock: Locks) -> DispatchResult {
			let who = T::DefaultOrigin::ensure_origin(origin)?;

			Pools::<T>::try_mutate(&pool_id, |pool| -> DispatchResult {
				let entry = pool.as_mut().ok_or(Error::<T>::PoolUnknown)?;
				ensure!(entry.is_manager(&who), Error::<T>::NoPermission);
				ensure!(entry.state.is_live(), Error::<T>::PoolNotLive);

				entry.state = PoolStatus::Locked(lock.clone());

				Ok(())
			})?;

			Self::deposit_event(Event::LockSet { id: pool_id, lock });

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::unlock())]
		pub fn unlock(origin: OriginFor<T>, pool_id: T::PoolId) -> DispatchResult {
			let who = T::DefaultOrigin::ensure_origin(origin)?;

			Pools::<T>::try_mutate(&pool_id, |pool| -> DispatchResult {
				let entry = pool.as_mut().ok_or(Error::<T>::PoolUnknown)?;
				ensure!(entry.is_manager(&who), Error::<T>::NoPermission);
				ensure!(entry.state.is_live(), Error::<T>::PoolNotLive);
				entry.state = PoolStatus::Active;

				Ok(())
			})?;

			Self::deposit_event(Event::Unlocked { id: pool_id });

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight({
			let weight_polynomial = T::WeightInfo::mint_into_polynomial(currency_count.to_owned());
			let weight_square_root = T::WeightInfo::mint_into_square_root(currency_count.to_owned());
			let weight_lmsr = T::WeightInfo::mint_into_lmsr(currency_count.to_owned());
			weight_polynomial.max(weight_square_root).max(weight_lmsr)
		})]
		pub fn mint_into(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			currency_idx: u32,
			beneficiary: AccountIdLookupOf<T>,
			amount_to_mint: FungiblesBalanceOf<T>,
			max_cost: CollateralCurrenciesBalanceOf<T>,
			currency_count: u32,
		) -> DispatchResultWithPostInfo {
			let who = T::DefaultOrigin::ensure_origin(origin)?;
			let beneficiary = T::Lookup::lookup(beneficiary)?;

			let pool_details = Pools::<T>::get(&pool_id).ok_or(Error::<T>::PoolUnknown)?;

			ensure!(pool_details.can_mint(&who), Error::<T>::NoPermission);

			let number_of_currencies = Self::get_currencies_number(&pool_details);
			ensure!(number_of_currencies <= currency_count, Error::<T>::CurrencyCount);

			let bonded_currencies = pool_details.bonded_currencies;

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

			let cost = Self::calculate_collateral(
				active_pre,
				active_post,
				passive,
				&pool_details.curve,
				pool_details.collateral_id.clone(),
			)?;

			// fail if cost > max_cost
			ensure!(cost <= max_cost, Error::<T>::Slippage);

			// Transfer the collateral. We do not want to kill the minter, so this operation
			// can fail if the account is being reaped.
			T::CollateralCurrencies::transfer(
				pool_details.collateral_id,
				&who,
				&pool_id.into(),
				cost,
				Preservation::Preserve,
			)?;

			T::Fungibles::mint_into(target_currency_id.clone(), &beneficiary, amount_to_mint)?;

			if !pool_details.transferable {
				T::Fungibles::freeze(target_currency_id, &beneficiary).map_err(|freeze_error| freeze_error.into())?;
			}

			Ok(Some(match pool_details.curve {
				Curve::Polynomial(_) => T::WeightInfo::mint_into_polynomial(number_of_currencies),
				Curve::SquareRoot(_) => T::WeightInfo::mint_into_square_root(number_of_currencies),
				Curve::Lmsr(_) => T::WeightInfo::mint_into_lmsr(number_of_currencies),
			})
			.into())
		}

		#[pallet::call_index(6)]
		#[pallet::weight({
			let weight_polynomial = T::WeightInfo::burn_into_polynomial(currency_count.to_owned());
			let weight_square_root = T::WeightInfo::burn_into_square_root(currency_count.to_owned());
			let weight_lmsr = T::WeightInfo::burn_into_lmsr(currency_count.to_owned());
			weight_polynomial.max(weight_square_root).max(weight_lmsr)
		})]
		pub fn burn_into(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			currency_idx: u32,
			beneficiary: AccountIdLookupOf<T>,
			amount_to_burn: FungiblesBalanceOf<T>,
			min_return: CollateralCurrenciesBalanceOf<T>,
			currency_count: u32,
		) -> DispatchResultWithPostInfo {
			let who = T::DefaultOrigin::ensure_origin(origin)?;
			let beneficiary = T::Lookup::lookup(beneficiary)?;

			let pool_details = Pools::<T>::get(&pool_id).ok_or(Error::<T>::PoolUnknown)?;

			ensure!(pool_details.can_burn(&who), Error::<T>::NoPermission);

			let number_of_currencies = Self::get_currencies_number(&pool_details);
			ensure!(number_of_currencies <= currency_count, Error::<T>::CurrencyCount);

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
				.ok_or(ArithmeticError::Underflow)?;

			let collateral_return = Self::calculate_collateral(
				low,
				high,
				passive,
				&pool_details.curve,
				pool_details.collateral_id.clone(),
			)?;

			ensure!(collateral_return >= min_return, Error::<T>::Slippage);

			T::CollateralCurrencies::transfer(
				pool_details.collateral_id,
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
				WithdrawalPrecision::Exact,
				Fortitude::Force,
			)?;

			if !pool_details.transferable {
				// Restore locks.
				T::Fungibles::freeze(target_currency_id, &beneficiary).map_err(|freeze_error| freeze_error.into())?;
			}

			Ok(Some(match pool_details.curve {
				Curve::Polynomial(_) => T::WeightInfo::burn_into_polynomial(number_of_currencies),
				Curve::SquareRoot(_) => T::WeightInfo::burn_into_square_root(number_of_currencies),
				Curve::Lmsr(_) => T::WeightInfo::burn_into_lmsr(number_of_currencies),
			})
			.into())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn swap_into(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::start_refund(currency_count.to_owned()))]
		pub fn start_refund(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			currency_count: u32,
		) -> DispatchResultWithPostInfo {
			let who = T::DefaultOrigin::ensure_origin(origin)?;

			let actual_currency_count = Self::do_start_refund(pool_id, currency_count, Some(&who))?;

			Ok(Some(T::WeightInfo::start_refund(actual_currency_count)).into())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(T::WeightInfo::force_start_refund(currency_count.to_owned()))]
		pub fn force_start_refund(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			currency_count: u32,
		) -> DispatchResultWithPostInfo {
			T::ForceOrigin::ensure_origin(origin)?;

			let actual_currency_count = Self::do_start_refund(pool_id, currency_count, None)?;

			Ok(Some(T::WeightInfo::force_start_refund(actual_currency_count)).into())
		}

		#[pallet::call_index(10)]
		#[pallet::weight(T::WeightInfo::refund_account(currency_count.to_owned()))]
		pub fn refund_account(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			account: AccountIdLookupOf<T>,
			asset_idx: u32,
			currency_count: u32,
		) -> DispatchResultWithPostInfo {
			T::DefaultOrigin::ensure_origin(origin)?;
			let who = T::Lookup::lookup(account)?;

			let pool_details = Pools::<T>::get(&pool_id).ok_or(Error::<T>::PoolUnknown)?;

			let number_of_currencies = Self::get_currencies_number(&pool_details);

			ensure!(number_of_currencies <= currency_count, Error::<T>::CurrencyCount);

			ensure!(pool_details.state.is_refunding(), Error::<T>::NotRefunding);

			// get asset id from linked assets vector
			let asset_id: &FungiblesAssetIdOf<T> = pool_details
				.bonded_currencies
				.get(asset_idx.saturated_into::<usize>())
				.ok_or(Error::<T>::IndexOutOfBounds)?;

			let pool_account = pool_id.clone().into();

			// Choosing total_balance over reducible_balance to ensure that all funds are
			// distributed fairly; in case of any locks present on the pool account, this
			// could lead to refunds failing to execute. This case would have to be
			// resolved by governance, either by removing locks or force_destroying the
			// pool.
			let total_collateral_issuance =
				T::CollateralCurrencies::total_balance(pool_details.collateral_id.clone(), &pool_account);

			// nothing to distribute; refunding is complete, user should call start_destroy
			ensure!(
				total_collateral_issuance > CollateralCurrenciesBalanceOf::<T>::zero(),
				Error::<T>::NothingToRefund
			);

			//  remove any existing locks on the account prior to burning
			T::Fungibles::thaw(asset_id, &who).map_err(|freeze_error| freeze_error.into())?;

			// With amount = max_value(), this trait implementation burns the reducible
			// balance on the account and returns the actual amount burnt
			let burnt: U256 = T::Fungibles::burn_from(
				asset_id.clone(),
				&who,
				Bounded::max_value(),
				WithdrawalPrecision::BestEffort,
				Fortitude::Force,
			)?
			.into();

			ensure!(!burnt.is_zero(), Error::<T>::InsufficientBalance);

			let sum_of_issuances = pool_details
				.bonded_currencies
				.into_iter()
				.fold(U256::from(0), |sum, id| {
					sum.saturating_add(T::Fungibles::total_issuance(id).into())
				})
				// Add the burnt amount back to the sum of total supplies
				.checked_add(burnt)
				.ok_or(ArithmeticError::Overflow)?;

			defensive_assert!(
				sum_of_issuances >= burnt,
				"burnt amount exceeds the total supply of all bonded currencies"
			);

			let amount: CollateralCurrenciesBalanceOf<T> = burnt
				.checked_mul(total_collateral_issuance.into())
				// As long as the balance type is half the size of a U256, this won't overflow.
				.ok_or(ArithmeticError::Overflow)?
				.checked_div(sum_of_issuances)
				// Because sum_of_issuances >= burnt > 0, this is theoretically impossible
				.ok_or(Error::<T>::Internal)?
				.checked_into()
				// Also theoretically impossible, as the result must be <= total_collateral_issuance
				// if burnt <= sum_of_issuances, which should always hold true
				.ok_or(Error::<T>::Internal)?;

			if amount.is_zero()
				|| T::CollateralCurrencies::can_deposit(
					pool_details.collateral_id.clone(),
					&who,
					amount,
					Provenance::Extant,
				)
				.into_result()
				.is_err()
			{
				// Funds are burnt but the collateral received is not sufficient to be deposited
				// to the account. This is tolerated as otherwise we could have edge cases where
				// it's impossible to refund at least some accounts.
				return Ok(Some(T::WeightInfo::refund_account(currency_count.to_owned())).into());
			}

			let transferred = T::CollateralCurrencies::transfer(
				pool_details.collateral_id,
				&pool_account,
				&who,
				amount,
				Preservation::Expendable,
			)?; // TODO: check edge cases around existential deposit

			// if collateral or total supply drops to zero, refunding is complete
			// -> emit event
			if sum_of_issuances <= burnt || total_collateral_issuance <= transferred {
				Self::deposit_event(Event::RefundComplete { id: pool_id });
			}

			Ok(Some(T::WeightInfo::refund_account(currency_count.to_owned())).into())
		}

		#[pallet::call_index(11)]
		#[pallet::weight(T::WeightInfo::start_destroy(currency_count.to_owned()))]
		pub fn start_destroy(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			currency_count: u32,
		) -> DispatchResultWithPostInfo {
			let who = T::DefaultOrigin::ensure_origin(origin)?;

			let actual_currency_count = Self::do_start_destroy_pool(pool_id, currency_count, false, Some(&who))?;

			Ok(Some(T::WeightInfo::start_destroy(actual_currency_count)).into())
		}

		#[pallet::call_index(12)]
		#[pallet::weight(T::WeightInfo::force_start_destroy(currency_count.to_owned()))]
		pub fn force_start_destroy(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			currency_count: u32,
		) -> DispatchResultWithPostInfo {
			T::ForceOrigin::ensure_origin(origin)?;

			let actual_currency_count = Self::do_start_destroy_pool(pool_id, currency_count, true, None)?;

			Ok(Some(T::WeightInfo::force_start_destroy(actual_currency_count)).into())
		}

		#[pallet::call_index(13)]
		#[pallet::weight(T::WeightInfo::finish_destroy(currency_count.to_owned()))]
		pub fn finish_destroy(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			currency_count: u32,
		) -> DispatchResultWithPostInfo {
			T::DefaultOrigin::ensure_origin(origin)?;

			let pool_details = Pools::<T>::get(&pool_id).ok_or(Error::<T>::PoolUnknown)?;

			let n_currencies = Self::get_currencies_number(&pool_details);

			ensure!(n_currencies <= currency_count, Error::<T>::CurrencyCount);

			ensure!(pool_details.state.is_destroying(), Error::<T>::LivePool);

			for asset_id in pool_details.bonded_currencies {
				if T::Fungibles::asset_exists(asset_id.clone()) {
					// This would fail with an LiveAsset error if there are any accounts left on any
					// currency
					T::Fungibles::finish_destroy(asset_id)?;
				}
			}

			let pool_account = pool_id.clone().into();

			let total_collateral_issuance =
				T::CollateralCurrencies::total_balance(pool_details.collateral_id.clone(), &pool_account);

			if total_collateral_issuance > CollateralCurrenciesBalanceOf::<T>::zero() {
				T::CollateralCurrencies::transfer(
					pool_details.collateral_id,
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

			Ok(Some(T::WeightInfo::finish_destroy(n_currencies)).into())
		}
	}

	impl<T: Config> Pallet<T>
	where
		<CurveParameterTypeOf<T> as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign + TryFrom<U256>,
	{
		fn calculate_collateral(
			low: CurveParameterTypeOf<T>,
			high: CurveParameterTypeOf<T>,
			passive_supply: PassiveSupply<CurveParameterTypeOf<T>>,
			curve: &Curve<CurveParameterTypeOf<T>>,
			collateral_currency_id: CollateralAssetIdOf<T>,
		) -> Result<CollateralCurrenciesBalanceOf<T>, ArithmeticError> {
			let normalized_costs = curve.calculate_costs(low, high, passive_supply)?;

			let collateral_denomination = 10u128
				.checked_pow(T::CollateralCurrencies::decimals(collateral_currency_id).into())
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

			let mut normalized_total_issuances = currencies_total_supply
				.into_iter()
				.map(|x| convert_to_fixed::<T>(x.saturated_into::<u128>(), denomination))
				.collect::<Result<Vec<CurveParameterTypeOf<T>>, ArithmeticError>>()?;

			let active_issuance = normalized_total_issuances.swap_remove(currency_idx);

			Ok((active_issuance, normalized_total_issuances))
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
				T::CollateralCurrencies::total_balance(pool_details.collateral_id.clone(), &pool_id.clone().into());
			// nothing to distribute
			ensure!(
				total_collateral_issuance > CollateralCurrenciesBalanceOf::<T>::zero(),
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
					T::CollateralCurrencies::total_balance(pool_details.collateral_id.clone(), &pool_id.clone().into());

				if total_collateral_issuance > CollateralCurrenciesBalanceOf::<T>::zero() {
					let has_holders = pool_details.bonded_currencies.iter().any(|asset_id| {
						T::Fungibles::total_issuance(asset_id.clone()) > FungiblesBalanceOf::<T>::zero()
					});
					// destruction is only allowed when there are no holders or no collateral to
					// distribute
					ensure!(!has_holders, Error::<T>::LivePool);
				}
			}

			// cloning the currency ids now lets us avoid cloning the entire pool_details
			let bonded_currencies = pool_details.bonded_currencies.clone();

			// switch pool state to destroying
			let mut new_pool_details = pool_details;
			new_pool_details.state.start_destroy();
			Pools::<T>::set(&pool_id, Some(new_pool_details));

			// emit this event before the destruction started events are emitted by assets
			// deactivation
			Self::deposit_event(Event::DestructionStarted { id: pool_id });

			for asset_id in bonded_currencies {
				// Governance or other pallets using the fungibles trait can in theory destroy
				// an asset without this pallet knowing, so we check if it's still around
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

		pub(crate) fn get_currencies_number(pool_details: &PoolDetailsOf<T>) -> u32 {
			// bonded_currencies is a BoundedVec with maximum length MaxCurrencies, which is
			// a u32; conversion to u32 must thus be lossless.
			pool_details.bonded_currencies.len().saturated_into()
		}

		pub(crate) fn calculate_pool_deposit<N: UniqueSaturatedInto<DepositCurrencyBalanceOf<T>>>(
			n_currencies: N,
		) -> DepositCurrencyBalanceOf<T> {
			T::BaseDeposit::get()
				.saturating_add(T::DepositPerCurrency::get().saturating_mul(n_currencies.saturated_into()))
		}
	}
}
