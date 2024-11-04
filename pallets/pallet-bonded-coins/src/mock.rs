use frame_support::{
	parameter_types,
	traits::{ConstU128, ConstU32},
	weights::constants::RocksDbWeight,
	Hashable,
};
use frame_system::{EnsureRoot, EnsureSigned};
use sp_runtime::{
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BoundedVec, BuildStorage, DispatchError, MultiSignature,
};
use substrate_fixed::types::{I75F53, U75F53};

use crate::{
	curves::{polynomial::PolynomialParameters, Curve},
	traits::{FreezeAccounts, ResetTeam},
	types::{Locks, PoolStatus},
	Config, DepositCurrencyBalanceOf, PoolDetailsOf,
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
const ACCOUNT_99: AccountId = AccountId::new([99u8; 32]);
// assets
pub(crate) const DEFAULT_BONDED_CURRENCY_ID: AssetId = 0;
pub(crate) const DEFAULT_COLLATERAL_CURRENCY_ID: AssetId = AssetId::MAX;
pub(crate) const DEFAULT_COLLATERAL_DENOMINATION: u8 = 10;
pub(crate) const DEFAULT_BONDED_DENOMINATION: u8 = 10;
pub(crate) const DEFAULT_COLLATERAL_UNIT: Balance = 10u128.pow(10);
pub(crate) const DEFAULT_BONDED_UNIT: Balance = 10u128.pow(10);
pub const UNIT_NATIVE: Balance = 10u128.pow(15);

// helper functions
pub fn assert_relative_eq(target: Float, expected: Float, epsilon: Float) {
	assert!(
		(target - expected).abs() <= epsilon,
		"Expected {:?} but got {:?}",
		expected,
		target
	);
}

pub(crate) fn get_linear_bonding_curve() -> Curve<Float> {
	let m = Float::from_num(0);
	let n = Float::from_num(2);
	let o = Float::from_num(3);
	Curve::Polynomial(PolynomialParameters { m, n, o })
}

pub(crate) fn calculate_pool_id(currencies: Vec<AssetId>) -> AccountId {
	AccountId::from(currencies.blake2_256())
}

#[cfg(test)]
pub mod runtime {

	use super::*;

	pub type Block = frame_system::mocking::MockBlock<Test>;

	pub fn generate_pool_details(
		currencies: Vec<AssetId>,
		curve: Curve<Float>,
		transferable: Option<bool>,
		state: Option<PoolStatus<Locks>>,
		manager: Option<AccountId>,
		collateral_id: Option<AssetId>,
		owner: Option<AccountId>,
	) -> PoolDetailsOf<Test> {
		let bonded_currencies = BoundedVec::truncate_from(currencies);
		let transferable = transferable.unwrap_or(false);
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
			denomination: 10,
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

	// trait implementations
	impl ResetTeam<AccountId> for Assets {
		fn reset_team(
			_id: Self::AssetId,
			_owner: AccountId,
			_admin: AccountId,
			_issuer: AccountId,
			_freezer: AccountId,
		) -> frame_support::dispatch::DispatchResult {
			Ok(())
		}
	}

	impl FreezeAccounts<AccountId, AssetId> for Assets {
		type Error = DispatchError;
		fn freeze(_asset_id: &AssetId, _who: &AccountId) -> Result<(), Self::Error> {
			Ok(())
		}

		fn thaw(_asset_id: &AssetId, _who: &AccountId) -> Result<(), Self::Error> {
			Ok(())
		}
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
	}
	parameter_types! {
		pub const CurrencyDeposit: Balance = 500;
		pub const MaxCurrencies: u32 = 50;
		pub const CollateralAssetId: u32 = u32::MAX;
	}

	impl Config for Test {
		type AssetId = AssetId;
		type BaseDeposit = ExistentialDeposit;
		type CollateralCurrency = Assets;
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
		// denomination, pool_id, PoolDetails
		pools: Vec<(u8, AccountId, PoolDetailsOf<Test>)>,
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

		pub(crate) fn with_pools(mut self, pools: Vec<(u8, AccountId, PoolDetailsOf<Test>)>) -> Self {
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

			let collateral_assets = self.collaterals.iter().map(|id| (*id, ACCOUNT_99, false, 0));

			pallet_assets::GenesisConfig::<Test> {
				assets: self
					.pools
					.iter()
					.map(|(_, owner, pool)| {
						pool.bonded_currencies
							.iter()
							.map(|id| (*id, owner.to_owned(), false, 1u128))
							.collect::<Vec<(AssetId, AccountId, bool, Balance)>>()
					})
					.flatten()
					.chain(collateral_assets)
					.collect(),

				accounts: self.bonded_balance,
				metadata: self
					.pools
					.iter()
					.map(|(denomination, _, pool_details)| {
						pool_details
							.bonded_currencies
							.iter()
							.map(|id| (*id, vec![], vec![], *denomination))
							.collect::<Vec<(u32, Vec<u8>, Vec<u8>, u8)>>()
					})
					.flatten()
					.collect(),
			}
			.assimilate_storage(&mut storage)
			.expect("assimilate should not fail");

			let mut ext = sp_io::TestExternalities::new(storage);

			ext.execute_with(|| {
				System::set_block_number(System::block_number() + 1);

				self.pools.iter().for_each(|(_, pool_id, pool)| {
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
