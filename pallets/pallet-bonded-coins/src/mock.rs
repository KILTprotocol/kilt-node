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
use substrate_fixed::types::I100F28;

use crate::{
	curves_parameters::PolynomialFunctionParameters,
	types::{Curve, Locks, PoolStatus},
	Config, DepositCurrencyBalanceOf, PoolDetailsOf,
};

pub type Hash = sp_core::H256;
pub type Balance = u128;
pub type AssetId = u32;
pub type Signature = MultiSignature;
pub type AccountPublic = <Signature as Verify>::Signer;
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
pub type Float = I100F28;

// accounts
pub(crate) const ACCOUNT_00: AccountId = AccountId::new([0u8; 32]);
pub(crate) const ACCOUNT_01: AccountId = AccountId::new([1u8; 32]);

// assets
pub(crate) const DEFAULT_BONDED_CURRENCY_ID: AssetId = 0;
pub(crate) const DEFAULT_COLLATERAL_CURRENCY_ID: AssetId = AssetId::MAX;
pub(crate) const DEFAULT_COLLATERAL_DENOMINATION: u8 = 10;
pub(crate) const DEFAULT_BONDED_DENOMINATION: u8 = 10;
pub(crate) const DEFAULT_COLLATERAL_UNIT: Balance = 10u128.pow(10);
pub(crate) const DEFAULT_BONDED_UNIT: Balance = 10u128.pow(10);
pub const UNIT_NATIVE: Balance = 10u128.pow(15);

// helper functions

pub(crate) fn get_linear_bonding_curve() -> Curve<Float> {
	let m = Float::from_num(0);
	let n = Float::from_num(2);
	let o = Float::from_num(3);
	Curve::PolynomialFunction(PolynomialFunctionParameters { m, n, o })
}

pub(crate) fn calculate_pool_id(currencies: Vec<AssetId>) -> AccountId {
	AccountId::from(currencies.blake2_256())
}

pub(crate) fn get_currency_unit(denomination: u8) -> Balance {
	10u128.pow(denomination as u32)
}

#[cfg(test)]
pub mod runtime {

	use super::*;

	pub type Block = frame_system::mocking::MockBlock<Test>;

	pub fn calculate_pool_details(
		currencies: Vec<AssetId>,
		manager: AccountId,
		transferable: bool,
		curve: Curve<Float>,
		state: PoolStatus<Locks>,
	) -> PoolDetailsOf<Test> {
		let bonded_currencies = BoundedVec::truncate_from(currencies);
		PoolDetailsOf::<Test> {
			curve,
			manager,
			transferable,
			bonded_currencies,
			state,
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
		type RuntimeTask = ();
		type RuntimeOrigin = RuntimeOrigin;
		type RuntimeCall = RuntimeCall;
		type Block = Block;
		type Nonce = u64;

		type Hash = Hash;
		type Hashing = BlakeTwo256;
		type AccountId = AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
		type RuntimeEvent = RuntimeEvent;
		type BlockHashCount = BlockHashCount;
		type DbWeight = RocksDbWeight;
		type Version = ();

		type PalletInfo = PalletInfo;
		type AccountData = pallet_balances::AccountData<Balance>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type BaseCallFilter = frame_support::traits::Everything;
		type SystemWeightInfo = ();
		type BlockWeights = ();
		type BlockLength = ();
		type SS58Prefix = SS58Prefix;
		type OnSetCode = ();
		type MaxConsumers = ConstU32<16>;
	}

	parameter_types! {
		pub const ExistentialDeposit: Balance = 500;
		pub const MaxLocks: u32 = 50;
		pub const MaxReserves: u32 = 50;
	}

	impl pallet_balances::Config for Test {
		type RuntimeFreezeReason = ();
		type FreezeIdentifier = ();
		type RuntimeHoldReason = RuntimeHoldReason;
		type MaxFreezes = ();
		type Balance = Balance;
		type DustRemoval = ();
		type RuntimeEvent = RuntimeEvent;
		type ExistentialDeposit = ExistentialDeposit;
		type AccountStore = System;
		type WeightInfo = ();
		type MaxLocks = MaxLocks;
		type MaxReserves = MaxReserves;
		type ReserveIdentifier = [u8; 8];
	}

	parameter_types! {
		pub const StringLimit: u32 = 50;

	}

	impl pallet_assets::Config for Test {
		type RuntimeEvent = RuntimeEvent;
		type Balance = Balance;
		type AssetId = AssetId;
		type AssetIdParameter = AssetId;
		type Currency = Balances;
		type CreateOrigin = EnsureSigned<AccountId>;
		type ForceOrigin = EnsureRoot<AccountId>;
		type AssetDeposit = ConstU128<0>;
		type AssetAccountDeposit = ConstU128<0>;
		type MetadataDepositBase = ConstU128<0>;
		type MetadataDepositPerByte = ConstU128<0>;
		type ApprovalDeposit = ConstU128<0>;
		type StringLimit = StringLimit;
		type Freezer = ();
		type WeightInfo = ();
		type CallbackHandle = ();
		type Extra = ();
		type RemoveItemsLimit = ConstU32<5>;
	}

	parameter_types! {
		pub const CurrencyDeposit: Balance = 500;
		pub const MaxCurrencies: u32 = 50;
		pub const CollateralAssetId: u32 = u32::MAX;
	}

	impl Config for Test {
		type DepositCurrency = Balances;
		type CollateralAssetId = CollateralAssetId;
		type CollateralCurrency = Assets;
		type DepositPerCurrency = CurrencyDeposit;
		type Fungibles = Assets;
		type MaxCurrencies = MaxCurrencies;
		type MaxStringLength = StringLimit;
		type PoolCreateOrigin = EnsureSigned<AccountId>;
		type PoolId = AccountId;
		type RuntimeEvent = RuntimeEvent;
		type RuntimeHoldReason = RuntimeHoldReason;
		type AssetId = AssetId;
		type BaseDeposit = ExistentialDeposit;
		type CurveParameterType = Float;
	}

	#[derive(Clone, Default)]
	pub(crate) struct ExtBuilder {
		native_assets: Vec<(AccountId, DepositCurrencyBalanceOf<Test>)>,
		currencies: Vec<Vec<AssetId>>,
		bonded_balance: Vec<(AssetId, AccountId, Balance)>,
		meta_data: Vec<(AssetId, u8)>,
		pools: Vec<(AccountId, PoolDetailsOf<Test>)>,
		collateral_asset_id: AssetId,
	}

	impl ExtBuilder {
		pub(crate) fn with_native_balances(
			mut self,
			native_assets: Vec<(AccountId, DepositCurrencyBalanceOf<Test>)>,
		) -> Self {
			self.native_assets = native_assets;
			self
		}

		pub(crate) fn with_collateral_asset_id(mut self, collateral_asset_id: AssetId) -> Self {
			self.collateral_asset_id = collateral_asset_id;
			self
		}

		pub(crate) fn with_currencies(mut self, currencies: Vec<Vec<AssetId>>) -> Self {
			self.currencies = currencies;
			self
		}

		pub(crate) fn with_metadata(mut self, meta_data: Vec<(AssetId, u8)>) -> Self {
			self.meta_data = meta_data;
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

		pub(crate) fn build(mut self) -> sp_io::TestExternalities {
			let mut storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
			pallet_balances::GenesisConfig::<Test> {
				balances: self.native_assets.clone(),
			}
			.assimilate_storage(&mut storage)
			.expect("assimilate should not fail");

			self.currencies.push(vec![self.collateral_asset_id]);

			pallet_assets::GenesisConfig::<Test> {
				assets: self
					.currencies
					.into_iter()
					.map(|x| {
						let admin = calculate_pool_id(x.clone());
						x.into_iter()
							.map(|id| (id, admin.clone(), false, 1u128))
							.collect::<Vec<(u32, AccountId, bool, u128)>>()
					})
					.flatten()
					.collect(),

				accounts: self.bonded_balance,
				metadata: self
					.meta_data
					.into_iter()
					.map(|(id, denomination)| (id, vec![], vec![], denomination))
					.collect(),
			}
			.assimilate_storage(&mut storage)
			.expect("assimilate should not fail");

			let mut ext = sp_io::TestExternalities::new(storage);

			ext.execute_with(|| {
				System::set_block_number(System::block_number() + 1);

				self.pools.iter().for_each(|(pool_id, pool)| {
					crate::Pools::<Test>::insert(pool_id.clone(), pool.clone());
				});
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
