use frame_benchmarking::v2::*;
use frame_support::traits::fungibles::roles::Inspect as InspectRoles;
use sp_core::U256;
use sp_std::{
	ops::{AddAssign, BitOrAssign, ShlAssign},
	vec::Vec,
};
use substrate_fixed::traits::{Fixed, FixedSigned, FixedUnsigned, ToFixed};

use crate::{
	curves::{
		lmsr::{LMSRParameters, LMSRParametersInput},
		square_root::{SquareRootParameters, SquareRootParametersInput},
		Curve, CurveInput,
	},
	Call, CollateralAssetIdOf, CollateralCurrenciesBalanceOf, Config, CurveParameterTypeOf, FungiblesAssetIdOf,
	FungiblesBalanceOf, Pallet,
};

/// Helper trait to calculate asset ids for collateral and bonded assets used in
/// benchmarks.
pub trait BenchmarkHelper<T: Config> {
	/// Calculate the asset id for the collateral asset.
	fn calculate_collateral_asset_id(seed: u32) -> CollateralAssetIdOf<T>;

	/// Calculate the asset id for the bonded asset.
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

fn get_square_root_curve<Float: FixedSigned>() -> Curve<Float> {
	let m = Float::from_num(3);
	let n = Float::from_num(2);
	Curve::SquareRoot(SquareRootParameters { m, n })
}

fn get_square_root_curve_input<Float: FixedUnsigned>() -> CurveInput<Float> {
	let m = Float::from_num(3);
	let n = Float::from_num(2);
	CurveInput::SquareRoot(SquareRootParametersInput { m, n })
}

fn get_lmsr_curve<Float: FixedSigned>() -> Curve<Float> {
	let m = Float::from_num(3);
	Curve::Lmsr(LMSRParameters { m })
}

fn get_lmsr_curve_input<Float: FixedUnsigned>() -> CurveInput<Float> {
	let m = Float::from_num(3);
	CurveInput::Lmsr(LMSRParametersInput { m })
}

#[benchmarks(where
	<CurveParameterTypeOf<T> as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign + TryFrom<U256> + TryInto<U256>,
	CollateralCurrenciesBalanceOf<T>: Into<U256> + TryFrom<U256>,
	FungiblesBalanceOf<T>: Into<U256> + TryFrom<U256>,
	T::CollateralCurrencies: Create<T::AccountId> ,
	T::Fungibles: InspectRoles<T::AccountId> + AccountTouch<FungiblesAssetIdOf<T>, AccountIdOf<T>>,
	T::DepositCurrency: Mutate<T::AccountId>,
	T::CollateralCurrencies: MutateFungibles<T::AccountId>,
	AccountIdLookupOf<T>: From<T::AccountId>,
)]
mod benchmarks {
	use frame_support::traits::{
		fungible::{Inspect, Mutate, MutateHold},
		fungibles::{Create, Destroy, Inspect as InspectFungibles, Mutate as MutateFungibles},
		AccountTouch, EnsureOrigin, Get, OriginTrait,
	};
	use sp_runtime::{traits::Zero, BoundedVec, SaturatedConversion};
	use sp_std::ops::Mul;

	use crate::{
		curves::Curve,
		mock::*,
		types::{Locks, PoolManagingTeam, PoolStatus},
		AccountIdLookupOf, AccountIdOf, CollateralAssetIdOf, CurveParameterInputOf, HoldReason, PoolDetailsOf, Pools,
		TokenMetaOf,
	};

	use super::*;

	// helper functions
	// collateral currencies
	fn create_collateral_asset<T: Config>(asset_id: CollateralAssetIdOf<T>)
	where
		T::CollateralCurrencies: Create<T::AccountId>,
	{
		let pool_account = account("collateral_owner", 0, 0);
		T::CollateralCurrencies::create(asset_id.clone(), pool_account, false, 1u128.saturated_into())
			.expect("Creating collateral asset should work");
		assert!(T::CollateralCurrencies::asset_exists(asset_id));
	}

	fn calculate_default_collateral_asset_id<T: Config>() -> CollateralAssetIdOf<T> {
		T::BenchmarkHelper::calculate_collateral_asset_id(u32::MAX)
	}

	fn create_default_collateral_asset<T: Config>() -> CollateralAssetIdOf<T>
	where
		T::CollateralCurrencies: Create<T::AccountId>,
	{
		let collateral_id = calculate_default_collateral_asset_id::<T>();
		create_collateral_asset::<T>(collateral_id.clone());
		collateral_id
	}

	fn set_collateral_balance<T: Config>(asset_id: CollateralAssetIdOf<T>, who: &AccountIdOf<T>, amount: u128)
	where
		T::CollateralCurrencies: MutateFungibles<T::AccountId>,
	{
		T::CollateralCurrencies::set_balance(asset_id.clone(), who, amount.saturated_into());
		let balance = T::CollateralCurrencies::balance(asset_id, who);
		assert_eq!(balance, amount.saturated_into());
	}

	// bonded currencies

	fn create_bonded_asset<T: Config>(asset_id: FungiblesAssetIdOf<T>) {
		let pool_account = account("bonded_owner", 0, 0);
		T::Fungibles::create(asset_id.clone(), pool_account, false, 1u128.saturated_into())
			.expect("Creating bonded asset should work");
		assert!(T::Fungibles::asset_exists(asset_id));
	}

	fn create_bonded_currencies_in_range<T: Config>(c: u32, is_destroying: bool) -> Vec<FungiblesAssetIdOf<T>> {
		let mut asset_ids = Vec::new();
		for i in 1..=c {
			let asset_id = T::BenchmarkHelper::calculate_bonded_asset_id(i);
			asset_ids.push(asset_id.clone());
			create_bonded_asset::<T>(asset_id.clone());
			if is_destroying {
				T::Fungibles::start_destroy(asset_id, None).expect("Destroying should work");
			}
		}

		asset_ids
	}

	fn set_fungible_balance<T: Config>(asset_id: FungiblesAssetIdOf<T>, who: &AccountIdOf<T>, amount: u128)
	where
		T::Fungibles: MutateFungibles<T::AccountId>,
	{
		T::Fungibles::set_balance(asset_id.clone(), who, amount.saturated_into());
		let balance = T::Fungibles::balance(asset_id, who);
		assert_eq!(balance, amount.saturated_into());
	}

	// native currency

	fn make_free_for_deposit<T: Config>(account: &AccountIdOf<T>)
	where
		T::DepositCurrency: Mutate<T::AccountId>,
	{
		let balance = <T::DepositCurrency as Inspect<AccountIdOf<T>>>::minimum_balance()
			+ T::BaseDeposit::get().mul(1000u32.into())
			+ T::DepositPerCurrency::get().mul(T::MaxCurrencies::get().into());
		set_native_balance::<T>(account, balance.saturated_into());
	}

	fn set_native_balance<T: Config>(account: &AccountIdOf<T>, amount: u128)
	where
		T::DepositCurrency: Mutate<T::AccountId>,
	{
		<T::DepositCurrency as Mutate<AccountIdOf<T>>>::set_balance(account, amount.saturated_into());
		let balance = <T::DepositCurrency as Inspect<AccountIdOf<T>>>::balance(account);
		assert_eq!(balance, amount.saturated_into());
	}

	// Storage

	fn create_pool<T: Config>(
		curve: Curve<CurveParameterTypeOf<T>>,
		bonded_coin_ids: Vec<FungiblesAssetIdOf<T>>,
		manager: Option<AccountIdOf<T>>,
		state: Option<PoolStatus<Locks>>,
		denomination: Option<u8>,
	) -> T::PoolId {
		let owner = account("owner", 0, 0);
		let state = state.unwrap_or(PoolStatus::Active);
		let collateral_id = calculate_default_collateral_asset_id::<T>();
		let denomination = denomination.unwrap_or(10);

		let pool_id: T::PoolId = calculate_pool_id(&bonded_coin_ids);
		let pool_details = PoolDetailsOf::<T> {
			curve,
			manager,
			owner,
			state,
			collateral_id,
			denomination,
			bonded_currencies: BoundedVec::truncate_from(bonded_coin_ids),
			transferable: true,
			min_operation_balance: 1u128.saturated_into(),
		};
		Pools::<T>::insert(&pool_id, pool_details);

		pool_id
	}

	fn generate_token_metadata<T: Config>(c: u32) -> BoundedVec<TokenMetaOf<T>, T::MaxCurrencies> {
		let mut token_meta = Vec::new();
		for _ in 1..=c {
			token_meta.push(TokenMetaOf::<T> {
				min_balance: 1u128.saturated_into(),
				name: BoundedVec::try_from(b"BTC".to_vec()).expect("Failed to create BoundedVec"),
				symbol: BoundedVec::try_from(b"BTC".to_vec()).expect("Failed to create BoundedVec"),
			})
		}
		BoundedVec::try_from(token_meta).expect("creating bounded Vec should not fail")
	}

	#[benchmark]
	fn create_pool_polynomial(c: Linear<1, { T::MaxCurrencies::get() }>) {
		let collateral_id = create_default_collateral_asset::<T>();
		let curve = get_linear_bonding_curve_input::<CurveParameterInputOf<T>>();

		let currencies = generate_token_metadata::<T>(c);
		let origin = T::PoolCreateOrigin::try_successful_origin().expect("creating origin should not fail");

		let account_origin = origin
			.clone()
			.into_signer()
			.expect("generating account_id from origin should not fail");
		make_free_for_deposit::<T>(&account_origin);

		#[extrinsic_call]
		create_pool(
			origin as T::RuntimeOrigin,
			curve,
			collateral_id,
			currencies,
			10,
			true,
			1,
		);

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
	fn create_pool_square_root(c: Linear<1, { T::MaxCurrencies::get() }>) {
		let collateral_id = create_default_collateral_asset::<T>();

		let curve = get_square_root_curve_input::<CurveParameterInputOf<T>>();

		let currencies = generate_token_metadata::<T>(c);

		let origin = T::PoolCreateOrigin::try_successful_origin().expect("creating origin should not fail");

		let account_origin = origin
			.clone()
			.into_signer()
			.expect("generating account_id from origin should not fail");
		make_free_for_deposit::<T>(&account_origin);

		#[extrinsic_call]
		create_pool(
			origin as T::RuntimeOrigin,
			curve,
			collateral_id,
			currencies,
			10,
			true,
			1,
		);

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
	fn create_pool_lmsr(c: Linear<1, { T::MaxCurrencies::get() }>) {
		let collateral_id = create_default_collateral_asset::<T>();

		let curve = get_lmsr_curve_input::<CurveParameterInputOf<T>>();
		let currencies = generate_token_metadata::<T>(c);

		let origin = T::PoolCreateOrigin::try_successful_origin().expect("creating origin should not fail");

		let account_origin = origin
			.clone()
			.into_signer()
			.expect("generating account_id from origin should not fail");
		make_free_for_deposit::<T>(&account_origin);

		#[extrinsic_call]
		create_pool(
			origin as T::RuntimeOrigin,
			curve,
			collateral_id,
			currencies,
			10,
			true,
			1,
		);

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
		let origin = T::DefaultOrigin::try_successful_origin().expect("creating origin should not fail");
		let account_origin = origin
			.clone()
			.into_signer()
			.expect("generating account_id from origin should not fail");
		make_free_for_deposit::<T>(&account_origin);

		let bonded_coin_id = T::BenchmarkHelper::calculate_bonded_asset_id(0);
		create_bonded_asset::<T>(bonded_coin_id.clone());

		let curve = get_linear_bonding_curve::<CurveParameterTypeOf<T>>();
		let pool_id = create_pool::<T>(
			curve,
			[bonded_coin_id.clone()].to_vec(),
			Some(account_origin),
			None,
			None,
		);

		let admin: AccountIdOf<T> = account("admin", 0, 0);
		let freezer: AccountIdOf<T> = account("freezer", 0, 0);
		let fungibles_team = PoolManagingTeam {
			admin: admin.clone(),
			freezer: freezer.clone(),
		};

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, pool_id, fungibles_team, 0);

		// Verify
		assert_eq!(T::Fungibles::admin(bonded_coin_id.clone()), Some(admin));
		assert_eq!(T::Fungibles::freezer(bonded_coin_id), Some(freezer));
	}

	#[benchmark]
	fn reset_manager() {
		let origin = T::DefaultOrigin::try_successful_origin().expect("creating origin should not fail");
		let account_origin = origin
			.clone()
			.into_signer()
			.expect("generating account_id from origin should not fail");
		make_free_for_deposit::<T>(&account_origin);

		let bonded_coin_id = T::BenchmarkHelper::calculate_bonded_asset_id(0);
		create_bonded_asset::<T>(bonded_coin_id.clone());

		let curve = get_linear_bonding_curve::<CurveParameterTypeOf<T>>();
		let pool_id = create_pool::<T>(curve, [bonded_coin_id].to_vec(), Some(account_origin), None, None);

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, pool_id, None);
		// Verify
		let (_, pool) = Pools::<T>::iter().next().expect("Pool should exist");
		assert_eq!(pool.manager, None);
	}

	#[benchmark]
	fn set_lock() {
		let origin = T::DefaultOrigin::try_successful_origin().expect("creating origin should not fail");
		let account_origin = origin
			.clone()
			.into_signer()
			.expect("generating account_id from origin should not fail");
		make_free_for_deposit::<T>(&account_origin);

		let bonded_coin_id = T::BenchmarkHelper::calculate_bonded_asset_id(0);
		create_bonded_asset::<T>(bonded_coin_id.clone());

		let curve = get_linear_bonding_curve::<CurveParameterTypeOf<T>>();
		let pool_id = create_pool::<T>(curve, [bonded_coin_id].to_vec(), Some(account_origin), None, None);

		let locks = Locks::default();

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, pool_id, locks);
		// Verify
		let (_, pool) = Pools::<T>::iter().next().expect("Pool should exist");
		assert_eq!(pool.state, PoolStatus::Locked(Locks::default()));
	}

	#[benchmark]
	fn unlock() {
		let origin = T::DefaultOrigin::try_successful_origin().expect("creating origin should not fail");
		let account_origin = origin
			.clone()
			.into_signer()
			.expect("generating account_id from origin should not fail");
		make_free_for_deposit::<T>(&account_origin);

		let bonded_coin_id = T::BenchmarkHelper::calculate_bonded_asset_id(0);
		create_bonded_asset::<T>(bonded_coin_id.clone());

		let curve = get_linear_bonding_curve::<CurveParameterTypeOf<T>>();
		let pool_id = create_pool::<T>(
			curve,
			[bonded_coin_id].to_vec(),
			Some(account_origin),
			Some(PoolStatus::Locked(Locks::default())),
			None,
		);

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, pool_id);
		// Verify
		let (_, pool) = Pools::<T>::iter().next().expect("Pool should exist");
		assert_eq!(pool.state, PoolStatus::Active);
	}

	#[benchmark]
	fn mint_into_polynomial(c: Linear<1, { T::MaxCurrencies::get() }>) {
		let collateral_id = create_default_collateral_asset::<T>();
		let origin = T::PoolCreateOrigin::try_successful_origin().expect("creating origin should not fail");
		let account_origin = origin
			.clone()
			.into_signer()
			.expect("generating account_id from origin should not fail");
		make_free_for_deposit::<T>(&account_origin);
		set_collateral_balance::<T>(collateral_id.clone(), &account_origin, 10000u128);

		let curve = get_linear_bonding_curve::<CurveParameterTypeOf<T>>();
		let bonded_currencies = create_bonded_currencies_in_range::<T>(c, false);

		let pool_id = create_pool::<T>(curve, bonded_currencies.clone(), None, None, None);

		T::CollateralCurrencies::touch(collateral_id, &pool_id.clone().into(), &account_origin)
			.expect("Touching should work");

		let beneficiary = AccountIdLookupOf::<T>::from(account_origin.clone());
		let amount_to_mint = 10u128.saturated_into();
		let max_costs = 100000u128.saturated_into();
		let max_currencies = T::MaxCurrencies::get();

		#[extrinsic_call]
		mint_into(
			origin as T::RuntimeOrigin,
			pool_id,
			0,
			beneficiary,
			amount_to_mint,
			max_costs,
			max_currencies,
		);

		// Verify

		let target_asset_id = bonded_currencies[0].clone();

		let balance = T::Fungibles::balance(target_asset_id, &account_origin);
		assert_eq!(balance, amount_to_mint.saturated_into());
	}

	#[benchmark]
	fn mint_into_square_root(c: Linear<1, { T::MaxCurrencies::get() }>) {
		let collateral_id = create_default_collateral_asset::<T>();
		let origin = T::PoolCreateOrigin::try_successful_origin().expect("creating origin should not fail");
		let account_origin = origin
			.clone()
			.into_signer()
			.expect("generating account_id from origin should not fail");
		make_free_for_deposit::<T>(&account_origin);
		set_collateral_balance::<T>(collateral_id.clone(), &account_origin, 10000u128);

		let curve = get_square_root_curve::<CurveParameterTypeOf<T>>();
		let bonded_currencies = create_bonded_currencies_in_range::<T>(c, false);

		let pool_id = create_pool::<T>(curve, bonded_currencies.clone(), None, None, None);

		T::CollateralCurrencies::touch(collateral_id, &pool_id.clone().into(), &account_origin)
			.expect("Touching should work");

		let beneficiary = AccountIdLookupOf::<T>::from(account_origin.clone());
		let amount_to_mint = 10u128.saturated_into();
		let max_costs = 100000u128.saturated_into();
		let max_currencies = T::MaxCurrencies::get();

		#[extrinsic_call]
		mint_into(
			origin as T::RuntimeOrigin,
			pool_id,
			0,
			beneficiary,
			amount_to_mint,
			max_costs,
			max_currencies,
		);

		// Verify

		let target_asset_id = bonded_currencies[0].clone();

		let balance = T::Fungibles::balance(target_asset_id, &account_origin);
		assert_eq!(balance, amount_to_mint.saturated_into());
	}

	#[benchmark]
	fn mint_into_lmsr(c: Linear<1, { T::MaxCurrencies::get() }>) {
		let collateral_id = create_default_collateral_asset::<T>();
		let origin = T::PoolCreateOrigin::try_successful_origin().expect("creating origin should not fail");
		let account_origin = origin
			.clone()
			.into_signer()
			.expect("generating account_id from origin should not fail");
		make_free_for_deposit::<T>(&account_origin);
		set_collateral_balance::<T>(collateral_id.clone(), &account_origin, 10000u128);

		let curve = get_lmsr_curve::<CurveParameterTypeOf<T>>();

		let bonded_currencies = create_bonded_currencies_in_range::<T>(c, false);

		let pool_id = create_pool::<T>(curve, bonded_currencies.clone(), None, None, None);

		T::CollateralCurrencies::touch(collateral_id, &pool_id.clone().into(), &account_origin)
			.expect("Touching should work");

		let beneficiary = AccountIdLookupOf::<T>::from(account_origin.clone());
		let amount_to_mint = 10u128.saturated_into();
		let max_costs = 100000u128.saturated_into();
		let max_currencies = T::MaxCurrencies::get();

		#[extrinsic_call]
		mint_into(
			origin as T::RuntimeOrigin,
			pool_id,
			0,
			beneficiary,
			amount_to_mint,
			max_costs,
			max_currencies,
		);

		// Verify
		let target_asset_id = bonded_currencies[0].clone();
		let balance = T::Fungibles::balance(target_asset_id, &account_origin);
		assert_eq!(balance, amount_to_mint.saturated_into());
	}

	#[benchmark]
	fn burn_into_polynomial(c: Linear<1, { T::MaxCurrencies::get() }>) {
		let origin = T::PoolCreateOrigin::try_successful_origin().expect("creating origin should not fail");
		let account_origin = origin
			.clone()
			.into_signer()
			.expect("generating account_id from origin should not fail");
		make_free_for_deposit::<T>(&account_origin);

		let collateral_id = create_default_collateral_asset::<T>();
		let bonded_currencies = create_bonded_currencies_in_range::<T>(c, false);
		let target_asset_id = bonded_currencies[0].clone();

		let start_balance = 100u128;
		set_fungible_balance::<T>(target_asset_id.clone(), &account_origin, start_balance);

		let curve = get_linear_bonding_curve::<CurveParameterTypeOf<T>>();

		let pool_id = create_pool::<T>(curve, bonded_currencies, None, None, Some(0));
		let pool_account = pool_id.clone().into();

		T::CollateralCurrencies::touch(collateral_id.clone(), &pool_account, &account_origin)
			.expect("Touching should work");

		set_collateral_balance::<T>(collateral_id, &pool_account, 10000u128);

		let beneficiary = AccountIdLookupOf::<T>::from(account_origin.clone());
		let amount_to_burn = 10u128.saturated_into();
		let min_return = 0u128.saturated_into();
		let max_currencies = T::MaxCurrencies::get();

		#[extrinsic_call]
		burn_into(
			origin as T::RuntimeOrigin,
			pool_id,
			0,
			beneficiary,
			amount_to_burn,
			min_return,
			max_currencies,
		);

		let balance = T::Fungibles::balance(target_asset_id, &account_origin);
		assert_eq!(
			balance,
			(start_balance - amount_to_burn.saturated_into::<u128>()).saturated_into()
		);
	}

	#[benchmark]
	fn burn_into_square_root(c: Linear<1, { T::MaxCurrencies::get() }>) {
		let origin = T::PoolCreateOrigin::try_successful_origin().expect("creating origin should not fail");
		let account_origin = origin
			.clone()
			.into_signer()
			.expect("generating account_id from origin should not fail");
		make_free_for_deposit::<T>(&account_origin);

		let collateral_id = create_default_collateral_asset::<T>();
		let bonded_currencies = create_bonded_currencies_in_range::<T>(c, false);
		let target_asset_id = bonded_currencies[0].clone();

		let start_balance = 100u128;
		set_fungible_balance::<T>(target_asset_id.clone(), &account_origin, start_balance);

		let curve = get_square_root_curve::<CurveParameterTypeOf<T>>();
		let pool_id = create_pool::<T>(curve, bonded_currencies, None, None, Some(0));
		let pool_account = pool_id.clone().into();

		T::CollateralCurrencies::touch(collateral_id.clone(), &pool_account, &account_origin)
			.expect("Touching should work");

		set_collateral_balance::<T>(collateral_id, &pool_account, 10000u128);

		let beneficiary = AccountIdLookupOf::<T>::from(account_origin.clone());
		let amount_to_burn = 10u128.saturated_into();
		let min_return = 0u128.saturated_into();
		let max_currencies = T::MaxCurrencies::get();

		#[extrinsic_call]
		burn_into(
			origin as T::RuntimeOrigin,
			pool_id,
			0,
			beneficiary,
			amount_to_burn,
			min_return,
			max_currencies,
		);

		let balance = T::Fungibles::balance(target_asset_id, &account_origin);
		assert_eq!(
			balance,
			(start_balance - amount_to_burn.saturated_into::<u128>()).saturated_into()
		);
	}

	#[benchmark]

	fn burn_into_lmsr(c: Linear<1, { T::MaxCurrencies::get() }>) {
		let origin = T::PoolCreateOrigin::try_successful_origin().expect("creating origin should not fail");
		let account_origin = origin
			.clone()
			.into_signer()
			.expect("generating account_id from origin should not fail");
		make_free_for_deposit::<T>(&account_origin);

		let collateral_id = create_default_collateral_asset::<T>();
		let bonded_currencies = create_bonded_currencies_in_range::<T>(c, false);
		let target_asset_id = bonded_currencies[0].clone();

		let start_balance = 100u128;
		set_fungible_balance::<T>(target_asset_id.clone(), &account_origin, start_balance);

		let curve = get_lmsr_curve::<CurveParameterTypeOf<T>>();
		let pool_id = create_pool::<T>(curve, bonded_currencies, None, None, Some(0));
		let pool_account = pool_id.clone().into();

		T::CollateralCurrencies::touch(collateral_id.clone(), &pool_account, &account_origin)
			.expect("Touching should work");

		set_collateral_balance::<T>(collateral_id, &pool_account, 10000u128);

		let beneficiary = AccountIdLookupOf::<T>::from(account_origin.clone());
		let amount_to_burn = 10u128.saturated_into();
		let min_return = 0u128.saturated_into();
		let max_currencies = T::MaxCurrencies::get();

		#[extrinsic_call]
		burn_into(
			origin as T::RuntimeOrigin,
			pool_id,
			0,
			beneficiary,
			amount_to_burn,
			min_return,
			max_currencies,
		);

		let balance = T::Fungibles::balance(target_asset_id, &account_origin);
		assert_eq!(
			balance,
			(start_balance - amount_to_burn.saturated_into::<u128>()).saturated_into()
		);
	}

	#[benchmark]
	fn start_destroy(c: Linear<1, { T::MaxCurrencies::get() }>) {
		let origin = T::DefaultOrigin::try_successful_origin().expect("creating origin should not fail");
		let account_origin = origin
			.clone()
			.into_signer()
			.expect("generating account_id from origin should not fail");

		let bonded_currencies = create_bonded_currencies_in_range::<T>(c, false);
		let curve = get_lmsr_curve::<CurveParameterTypeOf<T>>();

		let pool_id = create_pool::<T>(curve, bonded_currencies, Some(account_origin), None, None);
		let pool_id_clone = pool_id.clone();
		let max_currencies = T::MaxCurrencies::get();

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, pool_id_clone, max_currencies);

		// Verify

		let pool_details = Pools::<T>::get(&pool_id).expect("Pool should exist");
		assert_eq!(pool_details.state, PoolStatus::Destroying);
	}

	#[benchmark]
	fn force_start_destroy(c: Linear<1, { T::MaxCurrencies::get() }>) {
		let bonded_currencies = create_bonded_currencies_in_range::<T>(c, false);
		let curve = get_lmsr_curve::<CurveParameterTypeOf<T>>();

		let pool_id = create_pool::<T>(curve, bonded_currencies, None, None, None);

		let origin = T::ForceOrigin::try_successful_origin().expect("creating origin should not fail");
		let pool_id_clone = pool_id.clone();
		let max_currencies = T::MaxCurrencies::get();

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, pool_id_clone, max_currencies);

		// Verify

		let pool_details = Pools::<T>::get(&pool_id).expect("Pool should exist");
		assert_eq!(pool_details.state, PoolStatus::Destroying);
	}

	#[benchmark]
	fn finish_destroy(c: Linear<1, { T::MaxCurrencies::get() }>) {
		let bonded_currencies = create_bonded_currencies_in_range::<T>(c, true);
		let curve = get_lmsr_curve::<CurveParameterTypeOf<T>>();

		let owner: T::AccountId = account("owner", 0, 0);
		let pool_id = create_pool::<T>(
			curve,
			bonded_currencies.clone(),
			None,
			Some(PoolStatus::Destroying),
			None,
		);

		make_free_for_deposit::<T>(&owner);

		T::DepositCurrency::hold(
			&T::RuntimeHoldReason::from(HoldReason::Deposit),
			&owner,
			Pallet::<T>::calculate_pool_deposit(bonded_currencies.len()),
		)
		.expect("Generating Hold should not fail");

		let origin = T::DefaultOrigin::try_successful_origin().expect("creating origin should not fail");
		let pool_id_clone = pool_id.clone();
		let max_currencies = T::MaxCurrencies::get();

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, pool_id_clone, max_currencies);

		// Verify
		let pool_details = Pools::<T>::get(&pool_id);
		assert!(pool_details.is_none());
	}

	#[benchmark]
	fn start_refund(c: Linear<1, { T::MaxCurrencies::get() }>) {
		let origin = T::DefaultOrigin::try_successful_origin().expect("creating origin should not fail");
		let account_origin = origin
			.clone()
			.into_signer()
			.expect("generating account_id from origin should not fail");

		let bonded_currencies = create_bonded_currencies_in_range::<T>(c, false);
		let target_asset_id = bonded_currencies[0].clone();

		let curve = get_lmsr_curve::<CurveParameterTypeOf<T>>();

		let pool_id = create_pool::<T>(curve, bonded_currencies, Some(account_origin.clone()), None, None);

		let pool_account = pool_id.clone().into();

		let collateral_id = create_default_collateral_asset::<T>();
		T::CollateralCurrencies::touch(collateral_id.clone(), &pool_id.clone().into(), &account_origin)
			.expect("Touching should work");
		set_collateral_balance::<T>(collateral_id, &pool_account, 10000u128);

		let holder: T::AccountId = account("holder", 0, 1);
		T::Fungibles::touch(target_asset_id.clone(), &holder, &account_origin).expect("Touching should work");
		set_fungible_balance::<T>(target_asset_id, &holder, 10000u128);

		let pool_id_clone = pool_id.clone();
		let max_currencies = T::MaxCurrencies::get();

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, pool_id_clone, max_currencies);

		// Verify

		let pool_details = Pools::<T>::get(&pool_id).expect("Pool should exist");
		assert_eq!(pool_details.state, PoolStatus::Refunding);
	}

	#[benchmark]
	fn force_start_refund(c: Linear<1, { T::MaxCurrencies::get() }>) {
		let bonded_currencies = create_bonded_currencies_in_range::<T>(c, false);
		let target_asset_id = bonded_currencies[0].clone();

		let curve = get_lmsr_curve::<CurveParameterTypeOf<T>>();

		let pool_id = create_pool::<T>(curve, bonded_currencies, None, None, None);

		let pool_account = pool_id.clone().into();

		// give the pool account some funds.
		make_free_for_deposit::<T>(&pool_account);
		let collateral_id = create_default_collateral_asset::<T>();
		T::CollateralCurrencies::touch(collateral_id.clone(), &pool_account, &pool_account)
			.expect("Touching should work");
		set_collateral_balance::<T>(collateral_id, &pool_account, 10000u128);

		let holder: T::AccountId = account("holder", 0, 0);
		T::Fungibles::touch(target_asset_id.clone(), &holder, &pool_account).expect("Touching should work");
		set_fungible_balance::<T>(target_asset_id, &holder, 10000u128);

		let origin = T::ForceOrigin::try_successful_origin().expect("creating origin should not fail");
		let pool_id_clone = pool_id.clone();
		let max_currencies = T::MaxCurrencies::get();

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, pool_id_clone, max_currencies);

		// Verify
		let pool_details = Pools::<T>::get(&pool_id).expect("Pool should exist");
		assert_eq!(pool_details.state, PoolStatus::Refunding);
	}

	#[benchmark]
	fn refund_account(c: Linear<1, { T::MaxCurrencies::get() }>) {
		let origin = T::DefaultOrigin::try_successful_origin().expect("creating origin should not fail");
		let account_origin = origin
			.clone()
			.into_signer()
			.expect("generating account_id from origin should not fail");
		make_free_for_deposit::<T>(&account_origin);

		let collateral_id = create_default_collateral_asset::<T>();
		let bonded_currencies = create_bonded_currencies_in_range::<T>(c, false);
		let target_asset_id = bonded_currencies[0].clone();

		let curve = get_lmsr_curve::<CurveParameterTypeOf<T>>();
		let pool_id = create_pool::<T>(curve, bonded_currencies, None, Some(PoolStatus::Refunding), None);

		let pool_account = pool_id.clone().into();
		T::CollateralCurrencies::touch(collateral_id.clone(), &pool_id.clone().into(), &account_origin)
			.expect("Touching should work");
		set_collateral_balance::<T>(collateral_id, &pool_account, 10000u128);

		set_fungible_balance::<T>(target_asset_id.clone(), &account_origin, 100000u128);

		let beneficiary = AccountIdLookupOf::<T>::from(account_origin.clone());
		let max_currencies = T::MaxCurrencies::get();

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, pool_id, beneficiary, 0, max_currencies);

		// Verify
		let balance = T::Fungibles::balance(target_asset_id, &account_origin);
		assert_eq!(balance, Zero::zero());
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
