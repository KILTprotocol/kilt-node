// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

// The KILT Blockchain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The KILT Blockchain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// If you feel like getting in touch with us, you can do so at info@botlabs.org

use frame_support::traits::ReservableCurrency;

use kilt_support::deposit::Deposit;

use crate::{AccountIdOf, BalanceOf, Config, CurrencyOf, Names, Owner, Web3NameOf, Web3NameOwnerOf, Web3OwnershipOf};

pub(crate) type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

pub(crate) fn insert_raw_w3n<T: Config>(
	payer: AccountIdOf<T>,
	owner: Web3NameOwnerOf<T>,
	name: Web3NameOf<T>,
	block_number: BlockNumberOf<T>,
	deposit: BalanceOf<T>,
) {
	CurrencyOf::<T>::reserve(&payer, deposit).expect("Payer should have enough funds for deposit");

	Names::<T>::insert(&owner, name.clone());
	Owner::<T>::insert(
		&name,
		Web3OwnershipOf::<T> {
			owner,
			claimed_at: block_number,
			deposit: Deposit {
				owner: payer,
				amount: deposit,
			},
		},
	);
}

#[cfg(test)]
pub use crate::mock::runtime::*;

// Mocks that are only used internally
#[cfg(test)]
pub(crate) mod runtime {
	use frame_support::parameter_types;
	use frame_system::EnsureRoot;
	use kilt_support::mock::{mock_origin, SubjectId};
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
		MultiSignature,
	};

	use crate::{self as pallet_web3_names, web3_name::AsciiWeb3Name};

	type Index = u64;
	type BlockNumber = u64;
	pub(crate) type Balance = u128;

	type Hash = sp_core::H256;
	type Signature = MultiSignature;
	type AccountPublic = <Signature as Verify>::Signer;
	type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
	type Block = frame_system::mocking::MockBlock<Test>;

	frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
			Web3Names: pallet_web3_names::{Pallet, Storage, Call, Event<T>},
			MockOrigin: mock_origin::{Pallet, Origin<T>},
		}
	);

	parameter_types! {
		pub const SS58Prefix: u8 = 38;
		pub const BlockHashCount: BlockNumber = 2400;
	}

	impl frame_system::Config for Test {
		type BaseCallFilter = frame_support::traits::Everything;
		type BlockWeights = ();
		type BlockLength = ();
		type DbWeight = ();
		type RuntimeOrigin = RuntimeOrigin;
		type RuntimeCall = RuntimeCall;
		type Index = Index;
		type BlockNumber = BlockNumber;
		type Hash = Hash;
		type Hashing = BlakeTwo256;
		type AccountId = AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type RuntimeEvent = RuntimeEvent;
		type BlockHashCount = BlockHashCount;
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = pallet_balances::AccountData<Balance>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = SS58Prefix;
		type OnSetCode = ();
		type MaxConsumers = frame_support::traits::ConstU32<16>;
	}

	parameter_types! {
		pub const ExistentialDeposit: Balance = 10;
		pub const MaxLocks: u32 = 50;
		pub const MaxReserves: u32 = 50;
	}

	impl pallet_balances::Config for Test {
		type Balance = Balance;
		type DustRemoval = ();
		type RuntimeEvent = RuntimeEvent;
		type ExistentialDeposit = ExistentialDeposit;
		type AccountStore = System;
		type MaxLocks = MaxLocks;
		type MaxReserves = MaxReserves;
		type ReserveIdentifier = [u8; 8];
		type WeightInfo = ();
	}

	pub(crate) type TestWeb3Name = AsciiWeb3Name<Test>;
	pub(crate) type TestWeb3NameOwner = SubjectId;
	pub(crate) type TestWeb3NamePayer = AccountId;
	pub(crate) type TestOwnerOrigin = mock_origin::EnsureDoubleOrigin<TestWeb3NamePayer, TestWeb3NameOwner>;
	pub(crate) type TestOriginSuccess = mock_origin::DoubleOrigin<TestWeb3NamePayer, TestWeb3NameOwner>;
	pub(crate) type TestBanOrigin = EnsureRoot<AccountId>;

	parameter_types! {
		pub const MaxNameLength: u32 = 32;
		pub const MinNameLength: u32 = 3;
		// Easier to setup insufficient funds for deposit but still above existential deposit
		pub const Web3NameDeposit: Balance = 2 * ExistentialDeposit::get();
	}

	impl pallet_web3_names::Config for Test {
		type BanOrigin = TestBanOrigin;
		type OwnerOrigin = TestOwnerOrigin;
		type OriginSuccess = TestOriginSuccess;
		type Currency = Balances;
		type Deposit = Web3NameDeposit;
		type RuntimeEvent = RuntimeEvent;
		type MaxNameLength = MaxNameLength;
		type MinNameLength = MinNameLength;
		type Web3Name = TestWeb3Name;
		type Web3NameOwner = TestWeb3NameOwner;
		type WeightInfo = ();
	}

	impl mock_origin::Config for Test {
		type RuntimeOrigin = RuntimeOrigin;
		type AccountId = AccountId;
		type SubjectId = SubjectId;
	}

	pub(crate) const ACCOUNT_00: TestWeb3NamePayer = AccountId::new([1u8; 32]);
	pub(crate) const ACCOUNT_01: TestWeb3NamePayer = AccountId::new([2u8; 32]);
	pub(crate) const DID_00: TestWeb3NameOwner = SubjectId(ACCOUNT_00);
	pub(crate) const DID_01: TestWeb3NameOwner = SubjectId(ACCOUNT_01);
	pub(crate) const WEB3_NAME_00_INPUT: &[u8; 12] = b"web3_name_00";
	pub(crate) const WEB3_NAME_01_INPUT: &[u8; 12] = b"web3_name_01";

	pub(crate) fn get_web3_name(web3_name_input: &[u8]) -> TestWeb3Name {
		AsciiWeb3Name::try_from(web3_name_input.to_vec()).expect("Invalid web3 name input.")
	}

	#[derive(Clone, Default)]
	pub struct ExtBuilder {
		balances: Vec<(TestWeb3NamePayer, Balance)>,
		claimed_web3_names: Vec<(TestWeb3NameOwner, TestWeb3Name, TestWeb3NamePayer)>,
		banned_web3_names: Vec<TestWeb3Name>,
	}

	impl ExtBuilder {
		#[must_use]
		pub fn with_balances(mut self, balances: Vec<(TestWeb3NamePayer, Balance)>) -> Self {
			self.balances = balances;
			self
		}

		#[must_use]
		pub fn with_web3_names(
			mut self,
			web3_names: Vec<(TestWeb3NameOwner, TestWeb3Name, TestWeb3NamePayer)>,
		) -> Self {
			self.claimed_web3_names = web3_names;
			self
		}

		#[must_use]
		pub fn with_banned_web3_names(mut self, web3_names: Vec<TestWeb3Name>) -> Self {
			self.banned_web3_names = web3_names;
			self
		}

		pub fn build(self) -> sp_io::TestExternalities {
			let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
			pallet_balances::GenesisConfig::<Test> {
				balances: self.balances.clone(),
			}
			.assimilate_storage(&mut storage)
			.expect("assimilate should not fail");
			let mut ext = sp_io::TestExternalities::new(storage);

			ext.execute_with(|| {
				for (owner, web3_name, payer) in self.claimed_web3_names {
					pallet_web3_names::Pallet::<Test>::register_name(web3_name, owner, payer);
				}

				for web3_name in self.banned_web3_names {
					assert!(pallet_web3_names::Owner::<Test>::get(&web3_name).is_none());
					pallet_web3_names::Pallet::<Test>::ban_name(&web3_name);
				}
			});
			ext
		}

		pub fn build_and_execute_with_sanity_tests(self, test: impl FnOnce()) {
			self.build().execute_with(|| {
				test();
				crate::try_state::do_try_state::<Test>().expect("Sanity test for w3n failed.");
			})
		}

		#[cfg(feature = "runtime-benchmarks")]
		pub fn build_with_keystore(self) -> sp_io::TestExternalities {
			let mut ext = self.build();

			let keystore = sp_keystore::testing::KeyStore::new();
			ext.register_extension(sp_keystore::KeystoreExt(std::sync::Arc::new(keystore)));

			ext
		}
	}
}
