use frame_benchmarking::v2::*;
use frame_support::traits::fungibles::roles::Inspect as InspectRoles;
use sp_std::{
	ops::{AddAssign, BitOrAssign, ShlAssign},
	vec::Vec,
};

use substrate_fixed::traits::{Fixed, ToFixed};

use crate::{Call, CollateralAssetIdOf, Config, CurveParameterTypeOf, FungiblesAssetIdOf, Pallet};

pub trait BenchmarkHelper<T: Config> {
	fn calculate_collateral_asset_id(seed: u32) -> CollateralAssetIdOf<T>;

	fn calculate_bonded_asset_id(seed: u32) -> FungiblesAssetIdOf<T>;
}

impl<T> BenchmarkHelper<T> for ()
where
	T: Config,
	CollateralAssetIdOf<T>: From<u32>,
	FungiblesAssetIdOf<T>: From<u32>,
{
	fn calculate_collateral_asset_id(seed: u32) -> CollateralAssetIdOf<T> {
		seed.into()
	}

	fn calculate_bonded_asset_id(seed: u32) -> FungiblesAssetIdOf<T> {
		seed.into()
	}
}

#[benchmarks(where
	<CurveParameterTypeOf<T> as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
	<T as Config>::CollateralCurrencies: Create<T::AccountId> ,
	<T as Config>::Fungibles: InspectRoles<T::AccountId> + AccountTouch<FungiblesAssetIdOf<T>, AccountIdOf<T>>,
	<T as Config>::DepositCurrency: Mutate<T::AccountId>,
	<T as Config>::CollateralCurrencies: MutateFungibles<T::AccountId>,
	AccountIdLookupOf<T>: From<T::AccountId>,
)]
mod benchmarks {
	use frame_support::traits::OriginTrait;
	use frame_support::traits::{
		fungible::{Inspect, Mutate, MutateHold},
		fungibles::{Destroy, Mutate as MutateFungibles},
	};
	use frame_support::traits::{fungibles::Create, AccountTouch, EnsureOrigin, Get};
	use sp_runtime::BoundedVec;
	use sp_runtime::SaturatedConversion;
	use sp_std::ops::Mul;

	use crate::{
		curves::Curve,
		mock::*,
		types::{Locks, PoolManagingTeam, PoolStatus},
		AccountIdLookupOf, AccountIdOf, BoundedCurrencyVec, CollateralAssetIdOf, CurveParameterInputOf, HoldReason,
		PoolDetailsOf, Pools, TokenMetaOf,
	};

	use super::*;

	fn create_collateral_asset<T: Config>(asset_id: CollateralAssetIdOf<T>)
	where
		<T as Config>::CollateralCurrencies: Create<T::AccountId>,
	{
		let pool_account = account("collateral_owner", 0, 0);
		<T as Config>::CollateralCurrencies::create(asset_id, pool_account, false, 1u128.saturated_into())
			.expect("Creating collateral asset should work");
	}

	fn create_bonded_asset<T: Config>(asset_id: T::AssetId) {
		let pool_account = account("bonded_owner", 0, 0);
		<T as Config>::Fungibles::create(asset_id, pool_account, false, 1u128.saturated_into())
			.expect("Creating bonded asset should work");
	}

	fn make_free_for_deposit<T: Config>(account: &AccountIdOf<T>)
	where
		<T as Config>::DepositCurrency: Mutate<T::AccountId>,
	{
		let balance = <T::DepositCurrency as Inspect<AccountIdOf<T>>>::minimum_balance()
			+ <T as Config>::BaseDeposit::get().mul(10u32.into())
			+ <T as Config>::DepositPerCurrency::get().mul(T::MaxCurrencies::get().into());
		<T::DepositCurrency as Mutate<AccountIdOf<T>>>::set_balance(account, balance);
	}

	fn make_free_for_collateral<T: Config>(asset_id: CollateralAssetIdOf<T>, who: &AccountIdOf<T>, amount: u128)
	where
		<T as Config>::CollateralCurrencies: MutateFungibles<T::AccountId>,
	{
		T::CollateralCurrencies::set_balance(asset_id, who, amount.saturated_into());
	}

	fn make_free_for_bonded_fungibles<T: Config>(asset_id: FungiblesAssetIdOf<T>, who: &AccountIdOf<T>, amount: u128)
	where
		<T as Config>::Fungibles: MutateFungibles<T::AccountId>,
	{
		T::Fungibles::mint_into(asset_id.clone(), who, amount.saturated_into()).unwrap();
		T::Fungibles::set_balance(asset_id, who, amount.saturated_into());
	}

	#[benchmark]
	fn create_pool_polynomial(c: Linear<1, { <T as Config>::MaxCurrencies::get() }>) {
		let collateral_id = T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX);
		create_collateral_asset::<T>(collateral_id.clone());

		let curve = get_linear_bonding_curve_input::<CurveParameterInputOf<T>>();

		let mut token_meta = Vec::new();
		for _ in 0..c {
			token_meta.push(TokenMetaOf::<T> {
				min_balance: 1u128.saturated_into(),
				name: BoundedVec::try_from(b"BTC".to_vec()).expect("Failed to create BoundedVec"),
				symbol: BoundedVec::try_from(b"BTC".to_vec()).expect("Failed to create BoundedVec"),
			})
		}

		let currencies = BoundedVec::try_from(token_meta).expect("Failed to create BoundedVec");
		let origin = T::PoolCreateOrigin::try_successful_origin().unwrap();

		let account_origin = origin.clone().into_signer().unwrap();
		make_free_for_deposit::<T>(&account_origin);

		#[extrinsic_call]
		create_pool(origin as T::RuntimeOrigin, curve, collateral_id, currencies, 10, true);

		// Verify
		let (id, pool) = Pools::<T>::iter().next().expect("Pool should exist");
		let expected_pool_id: T::PoolId = calculate_pool_id(&pool.bonded_currencies.into_inner());
		match pool.curve {
			Curve::Polynomial(_) => {
				assert_eq!(id, expected_pool_id);
			}
			_ => panic!("pool.curve is not a Polynomial function"),
		}
	}

	#[benchmark]
	fn create_pool_square_root() {
		let collateral_id = T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX);
		create_collateral_asset::<T>(collateral_id.clone());

		let curve = get_square_root_curve_input::<CurveParameterInputOf<T>>();

		let c = 0..<T as Config>::MaxCurrencies::get();
		let mut token_meta = Vec::new();
		for _ in c {
			token_meta.push(TokenMetaOf::<T> {
				min_balance: 1u128.saturated_into(),
				name: BoundedVec::try_from(b"BTC".to_vec()).expect("Failed to create BoundedVec"),
				symbol: BoundedVec::try_from(b"BTC".to_vec()).expect("Failed to create BoundedVec"),
			})
		}

		let currencies = BoundedVec::try_from(token_meta).expect("Failed to create BoundedVec");

		let origin = T::PoolCreateOrigin::try_successful_origin().unwrap();

		let account_origin = origin.clone().into_signer().unwrap();
		make_free_for_deposit::<T>(&account_origin);

		make_free_for_deposit::<T>(&account_origin);
		#[extrinsic_call]
		create_pool(origin as T::RuntimeOrigin, curve, collateral_id, currencies, 10, true);

		// Verify
		let (id, pool) = Pools::<T>::iter().next().expect("Pool should exist");
		let expected_pool_id: T::PoolId = calculate_pool_id(&pool.bonded_currencies.into_inner());
		match pool.curve {
			Curve::SquareRoot(_) => {
				assert_eq!(id, expected_pool_id);
			}
			_ => panic!("pool.curve is not a Polynomial function"),
		}
	}

	#[benchmark]
	fn create_pool_lmsr(c: Linear<1, { <T as Config>::MaxCurrencies::get() }>) {
		let collateral_id = T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX);
		create_collateral_asset::<T>(collateral_id.clone());

		let curve = get_lmsr_curve_input::<CurveParameterInputOf<T>>();

		let mut token_meta = Vec::new();
		for _ in 0..c {
			token_meta.push(TokenMetaOf::<T> {
				min_balance: 1u128.saturated_into(),
				name: BoundedVec::try_from(b"BTC".to_vec()).expect("Failed to create BoundedVec"),
				symbol: BoundedVec::try_from(b"BTC".to_vec()).expect("Failed to create BoundedVec"),
			})
		}

		let currencies = BoundedVec::try_from(token_meta).expect("Failed to create BoundedVec");
		let origin = T::PoolCreateOrigin::try_successful_origin().unwrap();

		let account_origin = origin.clone().into_signer().unwrap();
		make_free_for_deposit::<T>(&account_origin);

		#[extrinsic_call]
		create_pool(origin as T::RuntimeOrigin, curve, collateral_id, currencies, 10, true);

		// Verify
		let (id, pool) = Pools::<T>::iter().next().expect("Pool should exist");
		let expected_pool_id: T::PoolId = calculate_pool_id(&pool.bonded_currencies.into_inner());
		match pool.curve {
			Curve::Lmsr(_) => {
				assert_eq!(id, expected_pool_id);
			}
			_ => panic!("pool.curve is not a Polynomial function"),
		}
	}

	#[benchmark]
	fn reset_team() {
		let origin = T::DefaultOrigin::try_successful_origin().unwrap();
		let account_origin = origin.clone().into_signer().unwrap();
		make_free_for_deposit::<T>(&account_origin);

		let collateral_id = T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX);
		let bonded_coin_id = T::BenchmarkHelper::calculate_bonded_asset_id(0);

		create_bonded_asset::<T>(bonded_coin_id.clone());
		let curve = get_linear_bonding_curve::<CurveParameterTypeOf<T>>();
		let pool_details = PoolDetailsOf::<T> {
			curve,
			manager: Some(account_origin),
			transferable: true,
			bonded_currencies: BoundedVec::truncate_from([bonded_coin_id.clone()].to_vec()),
			state: PoolStatus::Active,
			collateral_id,
			denomination: 10,
			owner: account("owner", 0, 0),
		};

		let pool_id: T::PoolId = calculate_pool_id(&[bonded_coin_id.clone()]);

		let admin: AccountIdOf<T> = account("admin", 0, 0);
		let freezer: AccountIdOf<T> = account("freezer", 0, 0);
		let fungibles_team = PoolManagingTeam {
			admin: admin.clone(),
			freezer: freezer.clone(),
		};

		Pools::<T>::insert(&pool_id, pool_details);

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, pool_id, fungibles_team, 0);

		// Verify
		assert_eq!(T::Fungibles::admin(bonded_coin_id.clone()), Some(admin));
		assert_eq!(T::Fungibles::freezer(bonded_coin_id), Some(freezer));
	}

	#[benchmark]
	fn reset_manager() {
		let origin = T::DefaultOrigin::try_successful_origin().unwrap();
		let account_origin = origin.clone().into_signer().unwrap();
		make_free_for_deposit::<T>(&account_origin);

		let collateral_id = T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX);
		let bonded_coin_id = T::BenchmarkHelper::calculate_bonded_asset_id(0);

		create_bonded_asset::<T>(bonded_coin_id.clone());
		let curve = get_linear_bonding_curve::<CurveParameterTypeOf<T>>();
		let pool_details = PoolDetailsOf::<T> {
			curve,
			manager: Some(account_origin),
			transferable: true,
			bonded_currencies: BoundedVec::truncate_from([bonded_coin_id.clone()].to_vec()),
			state: PoolStatus::Active,
			collateral_id,
			denomination: 10,
			owner: account("owner", 0, 0),
		};

		let pool_id: T::PoolId = calculate_pool_id(&[bonded_coin_id.clone()]);

		Pools::<T>::insert(&pool_id, pool_details);

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, pool_id, None);
		// Verify
		let (_, pool) = Pools::<T>::iter().next().expect("Pool should exist");
		assert_eq!(pool.manager, None);
	}

	#[benchmark]
	fn set_lock() {
		let origin = T::DefaultOrigin::try_successful_origin().unwrap();
		let account_origin = origin.clone().into_signer().unwrap();
		make_free_for_deposit::<T>(&account_origin);

		let collateral_id = T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX);
		let bonded_coin_id = T::BenchmarkHelper::calculate_bonded_asset_id(0);

		create_bonded_asset::<T>(bonded_coin_id.clone());
		let curve = get_linear_bonding_curve::<CurveParameterTypeOf<T>>();
		let pool_details = PoolDetailsOf::<T> {
			curve,
			manager: Some(account_origin),
			transferable: true,
			bonded_currencies: BoundedVec::truncate_from([bonded_coin_id.clone()].to_vec()),
			state: PoolStatus::Active,
			collateral_id,
			denomination: 10,
			owner: account("owner", 0, 0),
		};

		let pool_id: T::PoolId = calculate_pool_id(&[bonded_coin_id.clone()]);

		Pools::<T>::insert(&pool_id, pool_details);

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, pool_id, Locks::default());
		// Verify
		let (_, pool) = Pools::<T>::iter().next().expect("Pool should exist");
		assert_eq!(pool.state, PoolStatus::Locked(Locks::default()));
	}

	#[benchmark]
	fn unlock() {
		let origin = T::DefaultOrigin::try_successful_origin().unwrap();
		let account_origin = origin.clone().into_signer().unwrap();
		make_free_for_deposit::<T>(&account_origin);

		let collateral_id = T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX);
		let bonded_coin_id = T::BenchmarkHelper::calculate_bonded_asset_id(0);

		create_bonded_asset::<T>(bonded_coin_id.clone());
		let curve = get_linear_bonding_curve::<CurveParameterTypeOf<T>>();
		let pool_details = PoolDetailsOf::<T> {
			curve,
			manager: Some(account_origin),
			transferable: true,
			bonded_currencies: BoundedVec::truncate_from([bonded_coin_id.clone()].to_vec()),
			state: PoolStatus::Locked(Locks::default()),
			collateral_id,
			denomination: 10,
			owner: account("owner", 0, 0),
		};

		let pool_id: T::PoolId = calculate_pool_id(&[bonded_coin_id.clone()]);

		Pools::<T>::insert(&pool_id, pool_details);

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, pool_id);
		// Verify
		let (_, pool) = Pools::<T>::iter().next().expect("Pool should exist");
		assert_eq!(pool.state, PoolStatus::Active);
	}

	#[benchmark]
	fn mint_into_polynomial(c: Linear<1, { <T as Config>::MaxCurrencies::get() }>) {
		let origin = T::PoolCreateOrigin::try_successful_origin().unwrap();
		let account_origin = origin.clone().into_signer().unwrap();
		make_free_for_deposit::<T>(&account_origin);

		let collateral_id = T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX);
		create_collateral_asset::<T>(collateral_id.clone());

		let mut bonded_currencies = Vec::new();
		for i in 0..c {
			let asset_id = T::BenchmarkHelper::calculate_bonded_asset_id(i);
			bonded_currencies.push(asset_id.clone());
			create_bonded_asset::<T>(T::BenchmarkHelper::calculate_bonded_asset_id(i));
		}

		let curve = get_linear_bonding_curve::<CurveParameterTypeOf<T>>();

		let pool_details = PoolDetailsOf::<T> {
			curve,
			manager: Some(account_origin.clone()),
			transferable: false,
			bonded_currencies: BoundedCurrencyVec::<T>::try_from(bonded_currencies.clone())
				.expect("Failed to create BoundedVec"),
			state: PoolStatus::Active,
			collateral_id: collateral_id.clone(),
			denomination: 10,
			owner: account("owner", 0, 0),
		};

		let pool_id: T::PoolId = calculate_pool_id(&bonded_currencies);

		T::CollateralCurrencies::touch(collateral_id, &pool_id.clone().into(), &account_origin)
			.expect("Touching should work");

		Pools::<T>::insert(&pool_id, pool_details);

		let beneficiary = AccountIdLookupOf::<T>::from(account_origin);

		#[extrinsic_call]
		mint_into(
			origin as T::RuntimeOrigin,
			pool_id,
			0,
			beneficiary,
			10u128.saturated_into(),
			100u128.saturated_into(),
			T::MaxCurrencies::get(),
		);

		// Verify
	}

	#[benchmark]
	fn mint_into_square_root(c: Linear<1, { <T as Config>::MaxCurrencies::get() }>) {
		let origin = T::PoolCreateOrigin::try_successful_origin().unwrap();
		let account_origin = origin.clone().into_signer().unwrap();
		make_free_for_deposit::<T>(&account_origin);

		let collateral_id = T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX);
		create_collateral_asset::<T>(collateral_id.clone());

		let mut bonded_currencies = Vec::new();
		for i in 0..c {
			let asset_id = T::BenchmarkHelper::calculate_bonded_asset_id(i);
			bonded_currencies.push(asset_id.clone());
			create_bonded_asset::<T>(T::BenchmarkHelper::calculate_bonded_asset_id(i));
		}

		let curve = get_square_root_curve::<CurveParameterTypeOf<T>>();

		let pool_details = PoolDetailsOf::<T> {
			curve,
			manager: Some(account_origin.clone()),
			transferable: false,
			bonded_currencies: BoundedCurrencyVec::<T>::try_from(bonded_currencies.clone())
				.expect("Failed to create BoundedVec"),
			state: PoolStatus::Active,
			collateral_id: collateral_id.clone(),
			denomination: 10,
			owner: account("owner", 0, 0),
		};

		let pool_id: T::PoolId = calculate_pool_id(&bonded_currencies);

		T::CollateralCurrencies::touch(collateral_id, &pool_id.clone().into(), &account_origin)
			.expect("Touching should work");

		Pools::<T>::insert(&pool_id, pool_details);

		let beneficiary = AccountIdLookupOf::<T>::from(account_origin);

		#[extrinsic_call]
		mint_into(
			origin as T::RuntimeOrigin,
			pool_id,
			0,
			beneficiary,
			10u128.saturated_into(),
			100u128.saturated_into(),
			T::MaxCurrencies::get(),
		);

		// Verify
	}

	#[benchmark]
	fn mint_into_lmsr(c: Linear<1, { <T as Config>::MaxCurrencies::get() }>) {
		let origin = T::PoolCreateOrigin::try_successful_origin().unwrap();
		let account_origin = origin.clone().into_signer().unwrap();
		make_free_for_deposit::<T>(&account_origin);

		let collateral_id = T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX);
		create_collateral_asset::<T>(collateral_id.clone());

		let mut bonded_currencies = Vec::new();
		for i in 0..c {
			let asset_id = T::BenchmarkHelper::calculate_bonded_asset_id(i);
			bonded_currencies.push(asset_id.clone());
			create_bonded_asset::<T>(T::BenchmarkHelper::calculate_bonded_asset_id(i));
		}

		let curve = get_lmsr_curve::<CurveParameterTypeOf<T>>();

		let pool_details = PoolDetailsOf::<T> {
			curve,
			manager: Some(account_origin.clone()),
			transferable: false,
			bonded_currencies: BoundedCurrencyVec::<T>::try_from(bonded_currencies.clone())
				.expect("Failed to create BoundedVec"),
			state: PoolStatus::Active,
			collateral_id: collateral_id.clone(),
			denomination: 10,
			owner: account("owner", 0, 0),
		};

		let pool_id: T::PoolId = calculate_pool_id(&bonded_currencies);

		T::CollateralCurrencies::touch(collateral_id, &pool_id.clone().into(), &account_origin)
			.expect("Touching should work");

		Pools::<T>::insert(&pool_id, pool_details);

		let beneficiary = AccountIdLookupOf::<T>::from(account_origin);

		#[extrinsic_call]
		mint_into(
			origin as T::RuntimeOrigin,
			pool_id,
			0,
			beneficiary,
			10u128.saturated_into(),
			100u128.saturated_into(),
			T::MaxCurrencies::get(),
		);

		// Verify
	}

	#[benchmark]
	fn burn_into_polynomial(c: Linear<1, { <T as Config>::MaxCurrencies::get() }>) {
		let collateral_id = T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX);
		create_collateral_asset::<T>(collateral_id.clone());

		let origin = T::PoolCreateOrigin::try_successful_origin().unwrap();
		let account_origin = origin.clone().into_signer().unwrap();
		make_free_for_deposit::<T>(&account_origin);

		let mut bonded_currencies = Vec::new();
		for i in 0..c {
			let asset_id = T::BenchmarkHelper::calculate_bonded_asset_id(i);
			bonded_currencies.push(asset_id.clone());
			create_bonded_asset::<T>(T::BenchmarkHelper::calculate_bonded_asset_id(i));
		}

		make_free_for_bonded_fungibles::<T>(
			T::BenchmarkHelper::calculate_bonded_asset_id(0),
			&account_origin,
			100u128,
		);

		let curve = get_linear_bonding_curve::<CurveParameterTypeOf<T>>();

		let pool_details = PoolDetailsOf::<T> {
			curve,
			manager: Some(account_origin.clone()),
			transferable: false,
			bonded_currencies: BoundedCurrencyVec::<T>::try_from(bonded_currencies.clone())
				.expect("Failed to create BoundedVec"),
			state: PoolStatus::Active,
			collateral_id: collateral_id.clone(),
			denomination: 0,
			owner: account("owner", 0, 0),
		};

		let pool_id: T::PoolId = calculate_pool_id(&bonded_currencies);
		let pool_account = pool_id.clone().into();

		T::CollateralCurrencies::touch(collateral_id.clone(), &pool_account, &account_origin)
			.expect("Touching should work");

		make_free_for_collateral::<T>(collateral_id, &pool_account, 10000u128);

		Pools::<T>::insert(&pool_id, pool_details);

		let beneficiary = AccountIdLookupOf::<T>::from(account_origin);

		#[extrinsic_call]
		burn_into(
			origin as T::RuntimeOrigin,
			pool_id,
			0,
			beneficiary,
			10u128.saturated_into(),
			0u128.saturated_into(),
			T::MaxCurrencies::get(),
		);

		// Verify
	}

	#[benchmark]
	fn burn_into_square_root(c: Linear<1, { <T as Config>::MaxCurrencies::get() }>) {
		let collateral_id = T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX);
		create_collateral_asset::<T>(collateral_id.clone());

		let origin = T::PoolCreateOrigin::try_successful_origin().unwrap();
		let account_origin = origin.clone().into_signer().unwrap();
		make_free_for_deposit::<T>(&account_origin);

		let mut bonded_currencies = Vec::new();
		for i in 0..c {
			let asset_id = T::BenchmarkHelper::calculate_bonded_asset_id(i);
			bonded_currencies.push(asset_id.clone());
			create_bonded_asset::<T>(T::BenchmarkHelper::calculate_bonded_asset_id(i));
		}

		make_free_for_bonded_fungibles::<T>(
			T::BenchmarkHelper::calculate_bonded_asset_id(0),
			&account_origin,
			100u128,
		);

		let curve = get_square_root_curve::<CurveParameterTypeOf<T>>();

		let pool_details = PoolDetailsOf::<T> {
			curve,
			manager: Some(account_origin.clone()),
			transferable: false,
			bonded_currencies: BoundedCurrencyVec::<T>::try_from(bonded_currencies.clone())
				.expect("Failed to create BoundedVec"),
			state: PoolStatus::Active,
			collateral_id: collateral_id.clone(),
			denomination: 0,
			owner: account("owner", 0, 0),
		};

		let pool_id: T::PoolId = calculate_pool_id(&bonded_currencies);
		let pool_account = pool_id.clone().into();

		T::CollateralCurrencies::touch(collateral_id.clone(), &pool_account, &account_origin)
			.expect("Touching should work");

		make_free_for_collateral::<T>(collateral_id, &pool_account, 10000u128);

		Pools::<T>::insert(&pool_id, pool_details);

		let beneficiary = AccountIdLookupOf::<T>::from(account_origin);

		#[extrinsic_call]
		burn_into(
			origin as T::RuntimeOrigin,
			pool_id,
			0,
			beneficiary,
			10u128.saturated_into(),
			0u128.saturated_into(),
			T::MaxCurrencies::get(),
		);

		// Verify
	}

	#[benchmark]
	fn burn_into_lsmr(c: Linear<1, { <T as Config>::MaxCurrencies::get() }>) {
		let collateral_id = T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX);
		create_collateral_asset::<T>(collateral_id.clone());

		let origin = T::PoolCreateOrigin::try_successful_origin().unwrap();
		let account_origin = origin.clone().into_signer().unwrap();
		make_free_for_deposit::<T>(&account_origin);

		let mut bonded_currencies = Vec::new();
		for i in 0..c {
			let asset_id = T::BenchmarkHelper::calculate_bonded_asset_id(i);
			bonded_currencies.push(asset_id.clone());
			create_bonded_asset::<T>(T::BenchmarkHelper::calculate_bonded_asset_id(i));
		}

		make_free_for_bonded_fungibles::<T>(
			T::BenchmarkHelper::calculate_bonded_asset_id(0),
			&account_origin,
			100u128,
		);

		let curve = get_lmsr_curve::<CurveParameterTypeOf<T>>();

		let pool_details = PoolDetailsOf::<T> {
			curve,
			manager: Some(account_origin.clone()),
			transferable: false,
			bonded_currencies: BoundedCurrencyVec::<T>::try_from(bonded_currencies.clone())
				.expect("Failed to create BoundedVec"),
			state: PoolStatus::Active,
			collateral_id: collateral_id.clone(),
			denomination: 0,
			owner: account("owner", 0, 0),
		};

		let pool_id: T::PoolId = calculate_pool_id(&bonded_currencies);
		let pool_account = pool_id.clone().into();

		T::CollateralCurrencies::touch(collateral_id.clone(), &pool_account, &account_origin)
			.expect("Touching should work");

		make_free_for_collateral::<T>(collateral_id, &pool_account, 10000u128);

		Pools::<T>::insert(&pool_id, pool_details);

		let beneficiary = AccountIdLookupOf::<T>::from(account_origin);

		#[extrinsic_call]
		burn_into(
			origin as T::RuntimeOrigin,
			pool_id,
			0,
			beneficiary,
			10u128.saturated_into(),
			0u128.saturated_into(),
			T::MaxCurrencies::get(),
		);

		// Verify
	}

	#[benchmark]
	fn start_destroy(c: Linear<1, { <T as Config>::MaxCurrencies::get() }>) {
		let collateral_id = T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX);
		create_collateral_asset::<T>(collateral_id.clone());

		let origin = T::DefaultOrigin::try_successful_origin().unwrap();
		let account_origin = origin.clone().into_signer().unwrap();

		let mut bonded_currencies = Vec::new();
		for i in 0..c {
			let asset_id = T::BenchmarkHelper::calculate_bonded_asset_id(i);
			bonded_currencies.push(asset_id.clone());
			create_bonded_asset::<T>(T::BenchmarkHelper::calculate_bonded_asset_id(i));
		}

		let curve = get_lmsr_curve::<CurveParameterTypeOf<T>>();

		let pool_details = PoolDetailsOf::<T> {
			curve,
			manager: Some(account_origin.clone()),
			transferable: false,
			bonded_currencies: BoundedCurrencyVec::<T>::try_from(bonded_currencies.clone())
				.expect("Failed to create BoundedVec"),
			state: PoolStatus::Active,
			collateral_id: collateral_id.clone(),
			denomination: 0,
			owner: account("owner", 0, 0),
		};

		let pool_id: T::PoolId = calculate_pool_id(&bonded_currencies);

		Pools::<T>::insert(&pool_id, pool_details);

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, pool_id, T::MaxCurrencies::get());

		// Verify
	}

	#[benchmark]
	fn force_start_destroy(c: Linear<1, { <T as Config>::MaxCurrencies::get() }>) {
		let collateral_id = T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX);
		create_collateral_asset::<T>(collateral_id.clone());

		let mut bonded_currencies = Vec::new();
		for i in 0..c {
			let asset_id = T::BenchmarkHelper::calculate_bonded_asset_id(i);
			bonded_currencies.push(asset_id.clone());
			create_bonded_asset::<T>(T::BenchmarkHelper::calculate_bonded_asset_id(i));
		}

		let curve = get_lmsr_curve::<CurveParameterTypeOf<T>>();

		let pool_details = PoolDetailsOf::<T> {
			curve,
			manager: Some(account("manager", 0, 0)),
			transferable: false,
			bonded_currencies: BoundedCurrencyVec::<T>::try_from(bonded_currencies.clone())
				.expect("Failed to create BoundedVec"),
			state: PoolStatus::Active,
			collateral_id: collateral_id.clone(),
			denomination: 0,
			owner: account("owner", 0, 0),
		};

		let pool_id: T::PoolId = calculate_pool_id(&bonded_currencies);

		Pools::<T>::insert(&pool_id, pool_details);

		let origin = T::ForceOrigin::try_successful_origin().unwrap();

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, pool_id, T::MaxCurrencies::get());

		// Verify
	}

	#[benchmark]
	fn finish_destroy(c: Linear<1, { <T as Config>::MaxCurrencies::get() }>) {
		let collateral_id = T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX);
		create_collateral_asset::<T>(collateral_id.clone());

		let mut bonded_currencies = Vec::new();
		for i in 0..c {
			let asset_id = T::BenchmarkHelper::calculate_bonded_asset_id(i);
			bonded_currencies.push(asset_id.clone());
			create_bonded_asset::<T>(T::BenchmarkHelper::calculate_bonded_asset_id(i));
			T::Fungibles::start_destroy(asset_id, None).unwrap();
		}

		let curve = get_lmsr_curve::<CurveParameterTypeOf<T>>();

		let owner: T::AccountId = account("owner", 0, 0);

		let pool_details = PoolDetailsOf::<T> {
			curve,
			manager: Some(account("manager", 0, 0)),
			transferable: false,
			bonded_currencies: BoundedCurrencyVec::<T>::try_from(bonded_currencies.clone())
				.expect("Failed to create BoundedVec"),
			state: PoolStatus::Destroying,
			collateral_id: collateral_id.clone(),
			denomination: 0,
			owner: owner.clone(),
		};

		make_free_for_deposit::<T>(&owner);

		T::DepositCurrency::hold(
			&T::RuntimeHoldReason::from(HoldReason::Deposit),
			&owner,
			Pallet::<T>::calculate_pool_deposit(bonded_currencies.len()),
		)
		.unwrap();

		let pool_id: T::PoolId = calculate_pool_id(&bonded_currencies);

		Pools::<T>::insert(&pool_id, pool_details);

		let origin = T::DefaultOrigin::try_successful_origin().unwrap();

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, pool_id, T::MaxCurrencies::get());

		// Verify
	}

	#[benchmark]
	fn start_refund(c: Linear<1, { <T as Config>::MaxCurrencies::get() }>) {
		let collateral_id = T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX);
		create_collateral_asset::<T>(collateral_id.clone());

		let origin = T::DefaultOrigin::try_successful_origin().unwrap();
		let account_origin = origin.clone().into_signer().unwrap();

		let mut bonded_currencies = Vec::new();
		for i in 0..c {
			let asset_id = T::BenchmarkHelper::calculate_bonded_asset_id(i);
			bonded_currencies.push(asset_id.clone());
			create_bonded_asset::<T>(asset_id);
		}

		let curve = get_lmsr_curve::<CurveParameterTypeOf<T>>();

		let pool_details = PoolDetailsOf::<T> {
			curve,
			manager: Some(account_origin.clone()),
			transferable: false,
			bonded_currencies: BoundedCurrencyVec::<T>::try_from(bonded_currencies.clone())
				.expect("Failed to create BoundedVec"),
			state: PoolStatus::Active,
			collateral_id: collateral_id.clone(),
			denomination: 0,
			owner: account("owner", 0, 0),
		};

		let pool_id: T::PoolId = calculate_pool_id(&bonded_currencies);
		let pool_account = pool_id.clone().into();
		Pools::<T>::insert(&pool_id, pool_details);

		T::CollateralCurrencies::touch(collateral_id.clone(), &pool_id.clone().into(), &account_origin)
			.expect("Touching should work");

		make_free_for_collateral::<T>(collateral_id, &pool_account, 10000u128);

		let asset_id = T::BenchmarkHelper::calculate_bonded_asset_id(0);
		let holder: T::AccountId = account("holder", 0, 0);
		T::Fungibles::touch(asset_id.clone(), &holder, &account_origin).expect("Touching should work");
		make_free_for_bonded_fungibles::<T>(asset_id, &holder, 10000u128);

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, pool_id, T::MaxCurrencies::get());

		// Verify
	}

	#[benchmark]
	fn force_start_refund(c: Linear<1, { <T as Config>::MaxCurrencies::get() }>) {
		let collateral_id = T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX);
		create_collateral_asset::<T>(collateral_id.clone());

		let mut bonded_currencies = Vec::new();
		for i in 0..c {
			let asset_id = T::BenchmarkHelper::calculate_bonded_asset_id(i);
			bonded_currencies.push(asset_id.clone());
			create_bonded_asset::<T>(T::BenchmarkHelper::calculate_bonded_asset_id(i));
		}

		let curve = get_lmsr_curve::<CurveParameterTypeOf<T>>();

		let pool_details = PoolDetailsOf::<T> {
			curve,
			manager: Some(account("manager", 0, 0)),
			transferable: false,
			bonded_currencies: BoundedCurrencyVec::<T>::try_from(bonded_currencies.clone())
				.expect("Failed to create BoundedVec"),
			state: PoolStatus::Active,
			collateral_id: collateral_id.clone(),
			denomination: 0,
			owner: account("owner", 0, 0),
		};
		let pool_id: T::PoolId = calculate_pool_id(&bonded_currencies);
		let pool_account = pool_id.clone().into();
		T::CollateralCurrencies::touch(collateral_id.clone(), &pool_id.clone().into(), &pool_account)
			.expect("Touching should work");
		make_free_for_collateral::<T>(collateral_id.clone(), &pool_account, 10000u128);
		Pools::<T>::insert(&pool_id, pool_details);

		let asset_id = T::BenchmarkHelper::calculate_bonded_asset_id(0);

		let holder: T::AccountId = account("holder", 0, 0);

		make_free_for_deposit::<T>(&holder);

		make_free_for_bonded_fungibles::<T>(asset_id, &holder, 10000u128);

		make_free_for_collateral::<T>(collateral_id, &pool_account, 10000u128);

		let origin = T::ForceOrigin::try_successful_origin().unwrap();

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, pool_id, T::MaxCurrencies::get());

		// Verify
	}

	#[benchmark]
	fn refund_account(c: Linear<1, { <T as Config>::MaxCurrencies::get() }>) {
		let collateral_id = T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX);
		create_collateral_asset::<T>(collateral_id.clone());

		let origin = T::DefaultOrigin::try_successful_origin().unwrap();
		let account_origin = origin.clone().into_signer().unwrap();
		make_free_for_deposit::<T>(&account_origin);

		let mut bonded_currencies = Vec::new();
		for i in 0..c {
			let asset_id = T::BenchmarkHelper::calculate_bonded_asset_id(i);
			bonded_currencies.push(asset_id.clone());
			create_bonded_asset::<T>(asset_id);
		}

		let curve = get_lmsr_curve::<CurveParameterTypeOf<T>>();

		let pool_details = PoolDetailsOf::<T> {
			curve,
			manager: Some(account_origin.clone()),
			transferable: false,
			bonded_currencies: BoundedCurrencyVec::<T>::try_from(bonded_currencies.clone())
				.expect("Failed to create BoundedVec"),
			state: PoolStatus::Refunding,
			collateral_id: collateral_id.clone(),
			denomination: 0,
			owner: account("owner", 0, 0),
		};

		let pool_id: T::PoolId = calculate_pool_id(&bonded_currencies);
		let pool_account = pool_id.clone().into();
		Pools::<T>::insert(&pool_id, pool_details);

		T::CollateralCurrencies::touch(collateral_id.clone(), &pool_id.clone().into(), &account_origin)
			.expect("Touching should work");

		make_free_for_collateral::<T>(collateral_id, &pool_account, 10000u128);

		let asset_id = T::BenchmarkHelper::calculate_bonded_asset_id(0);

		T::Fungibles::touch(asset_id.clone(), &account_origin, &account_origin).expect("Touching should work");
		make_free_for_bonded_fungibles::<T>(asset_id.clone(), &account_origin, 100000u128);

		let beneficiary = AccountIdLookupOf::<T>::from(account_origin);

		#[extrinsic_call]
		_(
			origin as T::RuntimeOrigin,
			pool_id,
			beneficiary,
			0,
			T::MaxCurrencies::get(),
		);

		// Verify
	}
	#[cfg(test)]
	mod benchmark_tests {
		use crate::Pallet;

		frame_benchmarking::v2::impl_benchmark_test_suite!(
			Pallet,
			crate::mock::runtime::ExtBuilder::default().build_with_keystore(),
			crate::mock::runtime::Test
		);
	}
}
