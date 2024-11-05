use frame_benchmarking::v2::*;
use pallet_assets::BenchmarkHelper;
use sp_std::ops::{AddAssign, BitOrAssign, ShlAssign};
use substrate_fixed::traits::{Fixed, ToFixed};

use crate::{curves::CurveInput, Call, Config, CurveParameterTypeOf, FungiblesAssetIdOf, Pallet};

#[benchmarks(where
	<CurveParameterTypeOf<T> as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
	<T as Config>::CollateralCurrencies: Create<T::AccountId> + BenchmarkHelper<CollateralAssetIdOf<T>>,
	<T as Config>::Fungibles: BenchmarkHelper<FungiblesAssetIdOf<T>>
)]
mod benchmarks {
	use frame_support::traits::{fungibles::Create, EnsureOrigin, Get};
	use sp_runtime::BoundedVec;
	use sp_runtime::SaturatedConversion;

	use crate::{
		curves::{
			polynomial::{PolynomialParameters, PolynomialParametersInput},
			Curve,
		},
		types::PoolStatus,
		CollateralAssetIdOf, CurveParameterInputOf, PoolDetailsOf, Pools, TokenMetaOf,
	};

	use super::*;

	fn calculate_collateral_asset_id<T: Config>(seed: u32) -> CollateralAssetIdOf<T>
	where
		<T as Config>::CollateralCurrencies: BenchmarkHelper<CollateralAssetIdOf<T>>,
	{
		<T as Config>::CollateralCurrencies::create_asset_id_parameter(seed)
	}

	fn create_collateral_asset<T: Config>(asset_id: CollateralAssetIdOf<T>)
	where
		<T as Config>::CollateralCurrencies: Create<T::AccountId>,
	{
		let pool_account = account("collateral_owner", 0, 0);
		<T as Config>::CollateralCurrencies::create(asset_id, pool_account, false, 1u128.saturated_into())
			.expect("Creating collateral asset should work");
	}

	fn calculate_bonded_asset_id<T: Config>(seed: u32) -> FungiblesAssetIdOf<T>
	where
		<T as Config>::Fungibles: BenchmarkHelper<FungiblesAssetIdOf<T>>,
	{
		<T as Config>::Fungibles::create_asset_id_parameter(seed)
	}

	fn create_bonded_asset<T: Config>(asset_id: T::AssetId) {
		let pool_account = account("bonded_owner", 0, 0);
		<T as Config>::Fungibles::create(asset_id, pool_account, false, 1u128.saturated_into())
			.expect("Creating bonded asset should work");
	}

	#[benchmark]
	fn create_pool() {
		let collateral_id = calculate_collateral_asset_id::<T>(u32::MAX);
		create_collateral_asset::<T>(collateral_id.clone());

		let curve_parameters = CurveParameterInputOf::<T>::from_num(0);

		let curve = CurveInput::Polynomial(PolynomialParametersInput {
			m: curve_parameters,
			n: curve_parameters,
			o: curve_parameters,
		});

		let mut token_meta = vec![];

		for _ in 0..<T as Config>::MaxCurrencies::get() {
			token_meta.push(TokenMetaOf::<T> {
				min_balance: 1u128.saturated_into(),
				name: BoundedVec::try_from(vec![]).expect("Failed to create BoundedVec"),
				symbol: BoundedVec::try_from(vec![]).expect("Failed to create BoundedVec"),
			})
		}

		let currencies = BoundedVec::try_from(token_meta).expect("Failed to create BoundedVec");
		let origin = T::PoolCreateOrigin::try_successful_origin().unwrap();

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, curve, collateral_id, currencies, 10, true);

		// Verify
		assert_eq!(Pools::<T>::iter().count(), 1);
	}

	#[benchmark]
	fn reset_team() {
		let collateral_id = calculate_collateral_asset_id::<T>(u32::MAX);
		let bonded_coin_id = calculate_bonded_asset_id::<T>(0);

		create_bonded_asset::<T>(bonded_coin_id.clone());

		let curve_parameters = CurveParameterTypeOf::<T>::from_num(0);

		let curve = Curve::Polynomial(PolynomialParameters {
			m: curve_parameters,
			n: curve_parameters,
			o: curve_parameters,
		});

		let pool_details = PoolDetailsOf::<T> {
			curve,
			manager: None,
			transferable: true,
			bonded_currencies: BoundedVec::truncate_from(vec![bonded_coin_id]),
			state: PoolStatus::Active,
			collateral_id,
			denomination: 10,
			owner: account("owner", 0, 0),
		};

		let origin = T::DefaultOrigin::try_successful_origin().unwrap();

		Pools::<T>::insert(calculate_pool_id(&[collateral_id]), pool_details);

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, curve, collateral_id, currencies);

		// Verify
		assert_eq!(Pools::<T>::iter().count(), 1);
	}
}
