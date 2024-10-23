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
mod traits;
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
				Create as CreateFungibles, Destroy as DestroyFungibles, Inspect as InspectFungibles,
				Mutate as MutateFungibles,
			},
			AccountTouch,
		},
		Hashable, Parameter,
	};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::FullCodec;
	use scale_info::TypeInfo;
	use sp_runtime::{
		traits::{One, Saturating},
		BoundedVec, SaturatedConversion,
	};
	use sp_std::default::Default;
	use substrate_fixed::{
		traits::{FixedSigned, FixedUnsigned},
		types::I9F23,
	};

	use crate::{
		curves::{Curve, CurveInput},
		traits::ResetTeam,
		types::{Locks, PoolDetails, PoolStatus, Team, TokenMeta},
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
			+ FixedSigned
			+ TypeInfo
			+ MaxEncodedLen
			+ TryFrom<Self::CurveParameterInput>;

		type CurveParameterInput: Parameter + FixedUnsigned + TypeInfo + MaxEncodedLen;
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
		PoolCreated(T::PoolId),
	}

	#[pallet::error]
	pub enum Error<T> {
		CurrenciesNumber,
		InvalidCoefficients,
		Internal,
	}

	#[pallet::composite_enum]
	pub enum HoldReason {
		Deposit,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn create_pool(
			origin: OriginFor<T>,
			curve: CurveInput<CurveParameterInputOf<T>>,
			currencies: BoundedVec<TokenMetaOf<T>, T::MaxCurrencies>,
			state: PoolStatus<Locks>,
			denomination: u8,
			pool_manager: AccountIdOf<T>,
			transferable: bool,
			team: Team<AccountIdOf<T>>,
		) -> DispatchResult {
			let who = T::PoolCreateOrigin::ensure_origin(origin)?;

			let currency_length = currencies.len();

			ensure!(
				(1..=(T::MaxCurrencies::get()).saturated_into()).contains(&currency_length),
				Error::<T>::CurrenciesNumber
			);

			let checked_curve = curve.try_into().map_err(|_| Error::<T>::InvalidCoefficients)?;

			let current_asset_id = NextAssetId::<T>::get();

			let (currency_ids, next_asset_id) = Self::generate_sequential_asset_ids(current_asset_id, currency_length)?;

			// update the storage for the next tx.
			NextAssetId::<T>::set(next_asset_id);

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

			currencies
				.iter()
				.enumerate()
				.try_for_each(|(idx, entry)| -> DispatchResult {
					let asset_id: &T::AssetId = currency_ids.get(idx).ok_or(Error::<T>::CurrenciesNumber)?;
					T::Fungibles::create(asset_id.clone(), pool_id.clone().into(), false, entry.min_balance)?;

					// set metadata for new asset class
					T::Fungibles::set(
						asset_id.clone(),
						&pool_id.clone().into(),
						entry.name.clone().into(),
						entry.symbol.clone().into(),
						denomination,
					)?;

					T::Fungibles::reset_team(
						asset_id.clone(),
						pool_id.clone().into(),
						team.admin.clone(),
						team.issuer.clone(),
						team.freezer.clone(),
					)?;
					Ok(())
				})?;

			// Touch the pool account in order to be able to transfer the collateral currency to it
			T::CollateralCurrency::touch(T::CollateralAssetId::get(), &pool_id.clone().into(), &who)?;

			Pools::<T>::set(
				&pool_id,
				Some(PoolDetails::new(
					pool_manager,
					checked_curve,
					currency_ids,
					transferable,
					state,
				)),
			);

			Self::deposit_event(Event::PoolCreated(pool_id));

			Ok(())
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
		fn generate_sequential_asset_ids(
			mut start_id: T::AssetId,
			count: usize,
		) -> Result<(BoundedCurrencyVec<T>, T::AssetId), Error<T>> {
			let mut currency_ids_vec = Vec::new();
			for _ in 0..count {
				currency_ids_vec.push(start_id.clone());
				start_id = start_id.saturating_plus_one();
			}

			let currency_array =
				BoundedVec::<FungiblesAssetIdOf<T>, T::MaxCurrencies>::try_from(currency_ids_vec.clone())
					.map_err(|_| Error::<T>::Internal)?;

			Ok((currency_array, start_id))
		}
	}
}
