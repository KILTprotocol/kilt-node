use frame_support::{
	parameter_types,
	traits::{ConstU128, ConstU32},
	weights::constants::RocksDbWeight,
	Hashable,
};
use frame_system::{EnsureRoot, EnsureSigned};
use sp_runtime::{
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BoundedVec, BuildStorage, MultiSignature,
};
use substrate_fixed::{
	traits::{FixedSigned, FixedUnsigned},
	types::{I75F53, U75F53},
};

use crate::{
	self as pallet_bonded_coins,
	curves::{
		polynomial::{PolynomialParameters, PolynomialParametersInput},
		Curve, CurveInput,
	},
	types::{Locks, PoolStatus},
	DepositCurrencyBalanceOf, PoolDetailsOf,
};

pub type Float = I75F53;
pub(crate) type FloatInput = U75F53;
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
pub(crate) const DEFAULT_BONDED_DENOMINATION: u8 = 10;
pub(crate) const ONE_HUNDRED_KILT: u128 = 100_000_000_000_000_000;

// helper functions
pub fn assert_relative_eq(target: Float, expected: Float, epsilon: Float) {
	assert!(
		(target - expected).abs() <= epsilon,
		"Expected {:?} but got {:?}",
		expected,
		target
	);
}

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

pub(crate) fn calculate_pool_id(currencies: &[AssetId]) -> AccountId {
	AccountId::from(currencies.to_vec().blake2_256())
}

#[cfg(test)]
pub mod runtime {

	use super::*;

	pub type Block = frame_system::mocking::MockBlock<Test>;

	pub fn generate_pool_details(
		currencies: Vec<AssetId>,
		curve: Curve<Float>,
		transferable: bool,
		state: Option<PoolStatus<Locks>>,
		manager: Option<AccountId>,
		collateral_id: Option<AssetId>,
		owner: Option<AccountId>,
	) -> PoolDetailsOf<Test> {
		let bonded_currencies = BoundedVec::truncate_from(currencies);
		let state = state.unwrap_or(PoolStatus::Active);
		let owner = owner.unwrap_or(ACCOUNT_99);
		let collateral_id = collateral_id.unwrap_or(DEFAULT_COLLATERAL_CURRENCY_ID);
		PoolDetailsOf::<Test> {
			curve,
			manager,
			transferable,
			bonded_currencies,
			state,
			collateral_id,
			denomination: DEFAULT_BONDED_DENOMINATION,
			owner,
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
		pub const MaxCurrencies: u32 = 50;
		pub const CollateralAssetId: u32 = u32::MAX;
	}

	impl pallet_bonded_coins::Config for Test {
		type AssetId = AssetId;
		type BaseDeposit = ExistentialDeposit;
		type CollateralCurrencies = Assets;
		type CurveParameterInput = FloatInput;
		type CurveParameterType = Float;
		type DefaultOrigin = EnsureSigned<AccountId>;
		type DepositCurrency = Balances;
		type DepositPerCurrency = CurrencyDeposit;
		type ForceOrigin = EnsureRoot<AccountId>;
		type Fungibles = Assets;
		type MaxCurrencies = MaxCurrencies;
		type MaxStringLength = StringLimit;
		type PoolCreateOrigin = EnsureSigned<AccountId>;
		type PoolId = AccountId;
		type RuntimeEvent = RuntimeEvent;
		type RuntimeHoldReason = RuntimeHoldReason;
	}

	#[derive(Clone, Default)]
	pub(crate) struct ExtBuilder {
		native_assets: Vec<(AccountId, DepositCurrencyBalanceOf<Test>)>,
		bonded_balance: Vec<(AssetId, AccountId, Balance)>,
		//  pool_id, PoolDetails
		pools: Vec<(AccountId, PoolDetailsOf<Test>)>,
		collaterals: Vec<AssetId>,
	}

	impl ExtBuilder {
		pub(crate) fn with_native_balances(
			mut self,
			native_assets: Vec<(AccountId, DepositCurrencyBalanceOf<Test>)>,
		) -> Self {
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

			let collateral_assets = self.collaterals.into_iter().map(|id| (id, ACCOUNT_99, false, 1));

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

			// NextAssetId is set to the maximum value of all collateral/bonded currency ids, plus one.
			// If no currencies are created, it's set to 0.
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
					.collect(),
			}
			.assimilate_storage(&mut storage)
			.expect("assimilate should not fail");

			let mut ext = sp_io::TestExternalities::new(storage);

			ext.execute_with(|| {
				System::set_block_number(System::block_number() + 1);

				self.pools.into_iter().for_each(|(pool_id, pool)| {
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
