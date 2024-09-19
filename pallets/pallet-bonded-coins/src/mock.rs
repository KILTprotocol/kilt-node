#[cfg(test)]
pub mod runtime {
	use crate::{Config, DepositCurrencyBalanceOf};
	use frame_support::{
		parameter_types,
		traits::{ConstU128, ConstU32},
		weights::constants::RocksDbWeight,
	};
	use frame_system::{EnsureRoot, EnsureSigned};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
		BuildStorage, MultiSignature,
	};

	pub type Block = frame_system::mocking::MockBlock<Test>;
	pub type Hash = sp_core::H256;
	pub type Balance = u128;
	pub type Signature = MultiSignature;
	pub type AccountPublic = <Signature as Verify>::Signer;
	pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

	pub(crate) const ACCOUNT_00: AccountId = AccountId::new([1u8; 32]);
	pub(super) const DEFAULT_COLLATERAL_CURRENCY: (u32, AccountId, Balance, [u8; 4], u8) =
		(0, ACCOUNT_00, 1_000_000_000_000, [85, 83, 68, 84], 10);

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
		type RuntimeEvent = ();
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
		type RuntimeEvent = ();
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
		type RuntimeEvent = ();
		type Balance = Balance;
		type AssetId = u32;
		type AssetIdParameter = u32;
		type Currency = Balances;
		type CreateOrigin = EnsureSigned<AccountId>;
		type ForceOrigin = EnsureRoot<AccountId>;
		type AssetDeposit = ConstU128<1>;
		type AssetAccountDeposit = ConstU128<10>;
		type MetadataDepositBase = ConstU128<1>;
		type MetadataDepositPerByte = ConstU128<1>;
		type ApprovalDeposit = ConstU128<1>;
		type StringLimit = StringLimit;
		type Freezer = ();
		type WeightInfo = ();
		type CallbackHandle = ();
		type Extra = ();
		type RemoveItemsLimit = ConstU32<5>;
		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper = ();
	}

	parameter_types! {
		pub const CurrencyDeposit: Balance = 500;
		pub const MaxCurrencies: u32 = 50;
		pub const CollateralAssetId: u32 = 0;
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
		type RuntimeEvent = ();
		type RuntimeHoldReason = RuntimeHoldReason;
	}

	#[derive(Clone, Default)]
	pub(crate) struct ExtBuilder {
		balances: Vec<(AccountId, DepositCurrencyBalanceOf<Test>)>,
		// id, owner, balance amount, Name, Decimals
		bonded_currency: Vec<(u32, AccountId, Balance, [u8; 4], u8)>,
	}

	impl ExtBuilder {
		pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, DepositCurrencyBalanceOf<Test>)>) -> Self {
			self.balances = balances;
			self
		}

		pub(crate) fn with_currencies(mut self, bonded_currency: Vec<(u32, AccountId, Balance, [u8; 4], u8)>) -> Self {
			self.bonded_currency = bonded_currency;
			self
		}

		pub(crate) fn build(self) -> sp_io::TestExternalities {
			let mut storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
			pallet_balances::GenesisConfig::<Test> {
				balances: self.balances.clone(),
			}
			.assimilate_storage(&mut storage)
			.expect("assimilate should not fail");

			pallet_assets::GenesisConfig::<Test> {
				
				assets: self
					.bonded_currency
					.clone()
					.into_iter()
					// id, admin, is_sufficient, min_balance 
					.map(|(id, acc, _, _, _)| (id, acc, false, 1))
					.collect(),
				
				metadata: self
					.bonded_currency
					.clone()
					.into_iter()
					// id, name, symbol, decimals
					.map(|(id, _, _, name, denomination)| (id, name.clone().into(), name.into(), denomination))
					.collect(),
				
				accounts: self
					.bonded_currency
					.into_iter()
					// id, owner, balance
					.map(|(id, acc, balance, _, _)| (id, acc, balance))
					.collect(),
			}
			.assimilate_storage(&mut storage)
			.expect("assimilate should not fail");

			let mut ext = sp_io::TestExternalities::new(storage);

			ext.execute_with(|| {});

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
