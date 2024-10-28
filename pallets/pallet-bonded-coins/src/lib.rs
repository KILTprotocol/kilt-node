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
				Mutate as MutateFungibles,
			},
			tokens::{Fortitude, Precision, Preservation, Provenance},
			AccountTouch,
		},
		Parameter,
	};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::FullCodec;
	use scale_info::TypeInfo;
	use sp_runtime::{
		traits::{
			Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, One, SaturatedConversion, Saturating,
			StaticLookup, Zero,
		},
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
		/// A bonded token pool has been moved to refunding state.
		RefundingStarted { id: T::PoolId },
		/// A bonded token pool has been moved to destroying state.
		DestructionStarted { id: T::PoolId },
		/// Collateral distribution to bonded token holders has been completed for this pool - no more tokens or no more collateral to distribute.   
		RefundComplete { id: T::PoolId },
		/// A bonded token pool has been fully destroyed and all collateral and deposits have been refunded.
		Destroyed { id: T::PoolId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The pool id is not currently registered.
		PoolUnknown,
		/// The pool has no associated bonded currency with the given index.
		IndexOutOfBounds,
		/// The pool does not hold collateral to be refunded, or has no remaining supply of tokens to exchange. Call start_destroy to intiate teardown.
		NothingToRefund,
		/// The user is not privileged to perform the requested operation.
		Unauthorized,
		/// The pool is deactivated (i.e., in destroying or refunding state) and not available for use.
		PoolNotLive,
		/// There are active accounts associated with this pool and thus it cannot be destroyed at this point.
		LivePool,
		/// This operation can only be made when the pool is in refunding state.
		NotRefunding,
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
		pub fn start_refund(origin: OriginFor<T>, pool_id: T::PoolId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_start_refund(pool_id, Some(&who))
		}

		#[pallet::call_index(7)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn force_start_refund(origin: OriginFor<T>, pool_id: T::PoolId) -> DispatchResult {
			ensure_root(origin)?;

			Self::do_start_refund(pool_id, None)
		}

		#[pallet::call_index(8)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn refund_account(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			account: AccountIdLookupOf<T>,
			asset_idx: u32,
		) -> DispatchResult {
			ensure_signed(origin)?;
			let who = T::Lookup::lookup(account)?;

			let pool_details = Pools::<T>::get(&pool_id).ok_or(Error::<T>::PoolUnknown)?;

			ensure!(pool_details.state.is_refunding(), Error::<T>::NotRefunding);

			// get asset id from linked assets vector
			let asset_id: &FungiblesAssetIdOf<T> = pool_details
				.bonded_currencies
				.get(asset_idx.saturated_into::<usize>())
				.ok_or(Error::<T>::IndexOutOfBounds)?;

			let pool_account = pool_id.clone().into();

			let total_collateral_issuance = Self::get_pool_collateral(&pool_account);

			// nothing to distribute; refunding is complete, user should call start_destroy
			ensure!(!total_collateral_issuance.is_zero(), Error::<T>::NothingToRefund);

			// TODO: remove any existing locks on the account prior to burning

			// With amount = max_value(), this trait implementation burns the reducible balance on the account and returns the actual amount burnt
			let burnt = T::Fungibles::burn_from(
				asset_id.clone(),
				&who,
				Bounded::max_value(),
				Precision::BestEffort,
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

		#[pallet::call_index(9)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn start_destroy(origin: OriginFor<T>, pool_id: T::PoolId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_start_destroy_pool(pool_id, Some(&who), Fortitude::Polite)
		}

		#[pallet::call_index(10)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn force_start_destroy(origin: OriginFor<T>, pool_id: T::PoolId) -> DispatchResult {
			ensure_root(origin)?;

			Self::do_start_destroy_pool(pool_id, None, Fortitude::Force)
		}

		#[pallet::call_index(11)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn finish_destroy(origin: OriginFor<T>, pool_id: T::PoolId) -> DispatchResult {
			ensure_signed(origin)?;

			let pool_details = Pools::<T>::get(pool_id.clone()).ok_or(Error::<T>::PoolUnknown)?;

			ensure!(pool_details.state.is_destroying(), Error::<T>::LivePool);

			for asset_id in pool_details.bonded_currencies {
				if T::Fungibles::asset_exists(asset_id.clone()) {
					// This would fail with an LiveAsset error if there are any accounts left on any currency
					T::Fungibles::finish_destroy(asset_id)?;
				}
			}

			let pool_account = pool_id.clone().into();

			let total_collateral_issuance = Self::get_pool_collateral(&pool_account);

			if !total_collateral_issuance.is_zero() {
				T::CollateralCurrency::transfer(
					T::CollateralAssetId::get(),
					&pool_account,
					&pool_details.manager,
					total_collateral_issuance,
					Preservation::Expendable,
				)?;
			}

			Pools::<T>::remove(&pool_id);

			// TODO: refund deposit

			Self::deposit_event(Event::Destroyed { id: pool_id });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn do_start_refund(pool_id: T::PoolId, maybe_check_manager: Option<&AccountIdOf<T>>) -> DispatchResult {
			let mut pool_details = Pools::<T>::get(&pool_id).ok_or(Error::<T>::PoolUnknown)?;

			// refunding can only be triggered on a live pool
			ensure!(pool_details.state.is_live(), Error::<T>::PoolNotLive);

			if let Some(caller) = maybe_check_manager {
				ensure!(pool_details.is_manager(caller), Error::<T>::Unauthorized);
			}

			let total_collateral_issuance = Self::get_pool_collateral(&pool_id.clone().into());
			// nothing to distribute
			ensure!(!total_collateral_issuance.is_zero(), Error::<T>::NothingToRefund);

			let has_holders = pool_details
				.bonded_currencies
				.iter()
				.any(|asset_id| !T::Fungibles::total_issuance(asset_id.clone()).is_zero());
			// no token holders to refund
			ensure!(has_holders, Error::<T>::NothingToRefund);

			// move pool state to refunding
			pool_details.state.refunding();
			Pools::<T>::set(&pool_id, Some(pool_details));

			Self::deposit_event(Event::RefundingStarted { id: pool_id });

			Ok(())
		}

		fn do_start_destroy_pool(
			pool_id: T::PoolId,
			maybe_check_manager: Option<&AccountIdOf<T>>,
			force_skip_refund: Fortitude, // TODO: enum or boolean flag?
		) -> DispatchResult {
			let mut pool_details = Pools::<T>::get(&pool_id).ok_or(Error::<T>::PoolUnknown)?;

			ensure!(
				pool_details.state.is_live() || pool_details.state.is_refunding(),
				Error::<T>::PoolNotLive
			);

			if let Some(caller) = maybe_check_manager {
				ensure!(pool_details.is_manager(caller), Error::<T>::Unauthorized); // TODO: should this be permissionless if the pool is in refunding state?
			}

			if force_skip_refund != Fortitude::Force {
				let total_collateral_issuance = Self::get_pool_collateral(&pool_id.clone().into());
				if !total_collateral_issuance.is_zero() {
					let has_holders = pool_details
						.bonded_currencies
						.iter()
						.any(|asset_id| !T::Fungibles::total_issuance(asset_id.clone()).is_zero());
					// destruction is only allowed when there are no holders or no collateral to distribute
					ensure!(!has_holders, Error::<T>::LivePool);
				}
			}

			// move to destroying state
			pool_details.state.destroy();
			Pools::<T>::set(&pool_id, Some(pool_details.clone()));

			// emit this event before the destruction started events are emitted by assets deactivation
			Self::deposit_event(Event::DestructionStarted { id: pool_id });

			// deactivate all currencies
			for asset_id in pool_details.bonded_currencies.iter() {
				// Governance or other pallets using the fungibles trait can in theory destroy an asset without this pallet knowing, so we check if it's still around
				if T::Fungibles::asset_exists(asset_id.clone()) {
					T::Fungibles::start_destroy(asset_id.clone(), None)?;
				}
			}

			Ok(())
		}

		fn get_pool_collateral(pool_account: &AccountIdOf<T>) -> CollateralCurrencyBalanceOf<T> {
			T::CollateralCurrency::total_balance(T::CollateralAssetId::get(), pool_account)
		}
	}
}
