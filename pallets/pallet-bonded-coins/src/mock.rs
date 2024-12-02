use frame_support::Hashable;
use parity_scale_codec::Codec;
use substrate_fixed::traits::{FixedSigned, FixedUnsigned};

use crate::curves::{
	polynomial::{PolynomialParameters, PolynomialParametersInput},
	Curve, CurveInput,
};

pub(crate) fn get_linear_bonding_curve<Float: FixedSigned>() -> Curve<Float> {
	let m = Float::from_num(0);
	let n = Float::from_num(2);
	let o = Float::from_num(3);
	Curve::Polynomial(PolynomialParameters { m, n, o })
}

pub(crate) fn get_linear_bonding_curve_input<Float: FixedUnsigned>() -> CurveInput<Float> {
	let m = Float::from_num(0);
	let n = Float::from_num(2);
	let o = Float::from_num(3);
	CurveInput::Polynomial(PolynomialParametersInput { m, n, o })
}

pub(crate) fn calculate_pool_id<AssetId, AccountId>(currencies: &[AssetId]) -> AccountId
where
	AssetId: Clone + Hashable + Codec,
	AccountId: From<[u8; 32]>,
{
	AccountId::from(currencies.to_vec().blake2_256())
}

#[cfg(test)]
pub mod runtime {

	use super::*;

	use frame_support::{
		parameter_types,
		traits::{fungible::hold::Mutate, ConstU128, ConstU32},
		weights::constants::RocksDbWeight,
	};
	use frame_system::{EnsureRoot, EnsureSigned};
	use sp_core::U256;
	use sp_runtime::{
		traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
		BoundedVec, BuildStorage, MultiSignature, Permill,
	};
	use substrate_fixed::types::{I75F53, U75F53};

	use crate::{
		self as pallet_bonded_coins,
		types::{Locks, PoolStatus},
		DepositBalanceOf, HoldReason, PoolDetailsOf,
	};

	pub type Hash = sp_core::H256;
	pub type Balance = u128;
	pub type AssetId = u32;
	pub type Signature = MultiSignature;
	pub type AccountPublic = <Signature as Verify>::Signer;
	pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

	// accounts
	pub(crate) const ACCOUNT_00: AccountId = AccountId::new([0u8; 32]);
	pub(crate) const ACCOUNT_01: AccountId = AccountId::new([1u8; 32]);
	pub(crate) const ACCOUNT_99: AccountId = AccountId::new([99u8; 32]);
	// assets
	pub(crate) const DEFAULT_BONDED_CURRENCY_ID: AssetId = 1;
	pub(crate) const DEFAULT_COLLATERAL_CURRENCY_ID: AssetId = 0;
	pub(crate) const DEFAULT_COLLATERAL_DENOMINATION: u8 = 10;
	pub(crate) const ONE_HUNDRED_KILT: u128 = 100_000_000_000_000_000;
	pub(crate) const DEFAULT_BONDED_DENOMINATION: u8 = 10;
	// Testing
	pub(crate) const MAX_ERROR: Permill = Permill::from_perthousand(1);
	pub(crate) type Float = I75F53;
	pub(crate) type FloatInput = U75F53;

	pub type Block = frame_system::mocking::MockBlock<Test>;

	#[allow(clippy::too_many_arguments)]
	pub fn generate_pool_details(
		currencies: Vec<AssetId>,
		curve: Curve<Float>,
		transferable: bool,
		state: Option<PoolStatus<Locks>>,
		manager: Option<AccountId>,
		collateral_id: Option<AssetId>,
		owner: Option<AccountId>,
		min_operation_balance: Option<u128>,
	) -> PoolDetailsOf<Test> {
		let bonded_currencies = BoundedVec::truncate_from(currencies.clone());
		let state = state.unwrap_or(PoolStatus::Active);
		let owner = owner.unwrap_or(ACCOUNT_99);
		let collateral_id = collateral_id.unwrap_or(DEFAULT_COLLATERAL_CURRENCY_ID);
		let min_operation_balance = min_operation_balance.unwrap_or(1);
		PoolDetailsOf::<Test> {
			curve,
			manager,
			transferable,
			bonded_currencies,
			state,
			collateral_id,
			denomination: DEFAULT_BONDED_DENOMINATION,
			owner,
			min_operation_balance,
			deposit: BondingPallet::calculate_pool_deposit(currencies.len()),
		}
	}

	pub(crate) fn events() -> Vec<crate::Event<Test>> {
		System::events()
			.into_iter()
			.map(|r| r.event)
			.filter_map(|e| {
				if let RuntimeEvent::BondingPallet(e) = e {
					Some(e)
				} else {
					None
				}
			})
			.collect::<Vec<_>>()
	}

	// helper functions
	pub fn assert_relative_eq<T: FixedSigned>(target: T, expected: T, epsilon: T) {
		assert!(
			(target - expected).abs() <= epsilon,
			"Expected {:?} +/- {:?} but got {:?}",
			expected,
			epsilon,
			target
		);
	}

	pub(crate) fn mocks_curve_get_collateral_at_supply(supply: u128) -> u128 {
		let supply_u256 = U256::from(supply);
		let sup_squared = supply_u256 * supply_u256;
		// curve is f(x) = 4x + 3, resulting in f'(x) = 2x^2 + 3x.
		// The actual implementation operates on denomination-scaled amounts; we can
		// replicate this behaviour based on smallest units by denomination-scaling 'n',
		// the factor of x
		let result =
			U256::from(2) * sup_squared / U256::exp10(DEFAULT_BONDED_DENOMINATION.into()) + U256::from(3) * supply_u256;
		result
			.try_into()
			.expect("expected collateral too large for balance type")
	}

	frame_support::construct_runtime!(
		pub enum Test
		{
			System: frame_system,
			Balances: pallet_balances,
			Assets: pallet_assets,
			BondingPallet: crate,
		}
	);

	parameter_types! {
		pub const SS58Prefix: u8 = 38;
		pub const BlockHashCount: u64 = 250;
	}

	impl frame_system::Config for Test {
		type AccountData = pallet_balances::AccountData<Balance>;
		type AccountId = AccountId;
		type BaseCallFilter = frame_support::traits::Everything;
		type Block = Block;
		type BlockHashCount = BlockHashCount;
		type BlockLength = ();
		type BlockWeights = ();
		type DbWeight = RocksDbWeight;
		type Hash = Hash;
		type Hashing = BlakeTwo256;
		type Lookup = IdentityLookup<Self::AccountId>;
		type MaxConsumers = ConstU32<16>;
		type Nonce = u64;
		type OnKilledAccount = ();
		type OnNewAccount = ();
		type OnSetCode = ();
		type PalletInfo = PalletInfo;
		type RuntimeCall = RuntimeCall;
		type RuntimeEvent = RuntimeEvent;
		type RuntimeOrigin = RuntimeOrigin;
		type RuntimeTask = ();
		type SS58Prefix = SS58Prefix;
		type SystemWeightInfo = ();
		type Version = ();
	}

	parameter_types! {
		pub const ExistentialDeposit: Balance = 500;
		pub const MaxLocks: u32 = 50;
		pub const MaxReserves: u32 = 50;
	}

	impl pallet_balances::Config for Test {
		type AccountStore = System;
		type Balance = Balance;
		type DustRemoval = ();
		type ExistentialDeposit = ExistentialDeposit;
		type FreezeIdentifier = ();
		type MaxFreezes = ();
		type MaxLocks = MaxLocks;
		type MaxReserves = MaxReserves;
		type ReserveIdentifier = [u8; 8];
		type RuntimeEvent = RuntimeEvent;
		type RuntimeFreezeReason = ();
		type RuntimeHoldReason = RuntimeHoldReason;
		type WeightInfo = ();
	}

	parameter_types! {
		pub const StringLimit: u32 = 50;

	}
	impl pallet_assets::Config for Test {
		type ApprovalDeposit = ConstU128<0>;
		type AssetAccountDeposit = ConstU128<0>;
		type AssetDeposit = ConstU128<0>;
		type AssetId = AssetId;
		type AssetIdParameter = AssetId;
		type Balance = Balance;
		type CallbackHandle = ();
		type CreateOrigin = EnsureSigned<AccountId>;
		type Currency = Balances;
		type Extra = ();
		type ForceOrigin = EnsureRoot<AccountId>;
		type Freezer = ();
		type MetadataDepositBase = ConstU128<0>;
		type MetadataDepositPerByte = ConstU128<0>;
		type RemoveItemsLimit = ConstU32<5>;
		type RuntimeEvent = RuntimeEvent;
		type StringLimit = StringLimit;
		type WeightInfo = ();

		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper = ();
	}
	parameter_types! {
		pub const CurrencyDeposit: Balance = 500;
		pub const MaxCurrenciesPerPool: u32 = 50;
		pub const CollateralAssetId: u32 = u32::MAX;
		pub const MaxDenomination: u8 = 15;
	}

	impl pallet_bonded_coins::Config for Test {
		type AssetId = AssetId;
		type BaseDeposit = ExistentialDeposit;
		type Collaterals = Assets;
		type CurveParameterInput = FloatInput;
		type CurveParameterType = Float;
		type DefaultOrigin = EnsureSigned<AccountId>;
		type DepositCurrency = Balances;
		type DepositPerCurrency = CurrencyDeposit;
		type ForceOrigin = EnsureRoot<AccountId>;
		type Fungibles = Assets;
		type MaxCurrenciesPerPool = MaxCurrenciesPerPool;
		type MaxDenomination = MaxDenomination;
		type MaxStringInputLength = StringLimit;
		type PoolCreateOrigin = EnsureSigned<AccountId>;
		type PoolId = AccountId;
		type RuntimeEvent = RuntimeEvent;
		type RuntimeHoldReason = RuntimeHoldReason;
		type WeightInfo = ();

		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper = ();
	}

	#[derive(Clone, Default)]
	pub(crate) struct ExtBuilder {
		native_assets: Vec<(AccountId, DepositBalanceOf<Test>)>,
		bonded_balance: Vec<(AssetId, AccountId, Balance)>,
		//  pool_id, PoolDetails
		pools: Vec<(AccountId, PoolDetailsOf<Test>)>,
		collaterals: Vec<AssetId>,
	}

	impl ExtBuilder {
		pub(crate) fn with_native_balances(mut self, native_assets: Vec<(AccountId, DepositBalanceOf<Test>)>) -> Self {
			self.native_assets = native_assets;
			self
		}

		pub(crate) fn with_collaterals(mut self, collaterals: Vec<AssetId>) -> Self {
			self.collaterals = collaterals;
			self
		}

		pub(crate) fn with_pools(mut self, pools: Vec<(AccountId, PoolDetailsOf<Test>)>) -> Self {
			self.pools = pools;
			self
		}

		pub(crate) fn with_bonded_balance(mut self, bonded_balance: Vec<(AssetId, AccountId, Balance)>) -> Self {
			self.bonded_balance = bonded_balance;
			self
		}

		pub(crate) fn build(self) -> sp_io::TestExternalities {
			let mut storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
			pallet_balances::GenesisConfig::<Test> {
				balances: self.native_assets.clone(),
			}
			.assimilate_storage(&mut storage)
			.expect("assimilate should not fail");

			let collateral_assets = self.collaterals.iter().map(|id| (*id, ACCOUNT_99, true, 1));

			let all_assets: Vec<_> = self
				.pools
				.iter()
				.flat_map(|(owner, pool)| {
					pool.bonded_currencies
						.iter()
						.map(|id| (*id, owner.to_owned(), false, 1u128))
						.collect::<Vec<(AssetId, AccountId, bool, Balance)>>()
				})
				.chain(collateral_assets)
				.collect();

			// NextAssetId is set to the maximum value of all collateral/bonded currency
			// ids, plus one. If no currencies are created, it's set to 0.
			let next_asset_id = all_assets.iter().map(|(id, ..)| id).max().map_or(0, |id| id + 1);

			pallet_assets::GenesisConfig::<Test> {
				assets: all_assets,
				accounts: self.bonded_balance,
				metadata: self
					.pools
					.iter()
					.flat_map(|(_, pool_details)| {
						pool_details
							.bonded_currencies
							.iter()
							.map(|id| (*id, vec![], vec![], pool_details.denomination))
							.collect::<Vec<(u32, Vec<u8>, Vec<u8>, u8)>>()
					})
					.chain(
						self.collaterals
							.into_iter()
							.map(|id| (id, vec![], vec![], DEFAULT_COLLATERAL_DENOMINATION)),
					)
					.collect(),
			}
			.assimilate_storage(&mut storage)
			.expect("assimilate should not fail");

			let mut ext = sp_io::TestExternalities::new(storage);

			ext.execute_with(|| {
				System::set_block_number(System::block_number() + 1);

				self.pools.into_iter().for_each(|(pool_id, pool)| {
					// try to continue if we can't create the hold - might not be needed for all
					// tests
					let _ = <Test as crate::Config>::DepositCurrency::hold(
						&HoldReason::Deposit.into(),
						&pool.owner,
						BondingPallet::calculate_pool_deposit(pool.bonded_currencies.len()),
					);
					crate::Pools::<Test>::insert(pool_id, pool);
				});

				crate::NextAssetId::<Test>::set(next_asset_id);
			});

			ext
		}

		#[cfg(feature = "runtime-benchmarks")]
		pub(crate) fn build_with_keystore(self) -> sp_io::TestExternalities {
			use sp_keystore::{testing::MemoryKeystore, KeystoreExt};
			use sp_std::sync::Arc;

			let mut ext = self.build();

			let keystore = MemoryKeystore::new();
			ext.register_extension(KeystoreExt(Arc::new(keystore)));

			ext
		}
	}
}
