// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>
use frame_support::traits::fungible::MutateHold;
use frame_system::pallet_prelude::BlockNumberFor;
use kilt_support::Deposit;

use crate::{
	AccountIdOf, BalanceOf, Config, CurrencyOf, HoldReason, Names, Owner, Web3NameOf, Web3NameOwnerOf, Web3OwnershipOf,
};

pub(crate) fn insert_raw_w3n<T: Config<I>, I: 'static>(
	payer: AccountIdOf<T>,
	owner: Web3NameOwnerOf<T, I>,
	name: Web3NameOf<T, I>,
	block_number: BlockNumberFor<T>,
	deposit: BalanceOf<T, I>,
) {
	CurrencyOf::<T, I>::hold(&HoldReason::Deposit.into(), &payer, deposit)
		.expect("Payer should have enough funds for deposit");

	Names::<T, I>::insert(&owner, name.clone());
	Owner::<T, I>::insert(
		&name,
		Web3OwnershipOf::<T, I> {
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
	use frame_support::{ensure, parameter_types};
	use frame_system::EnsureRoot;
	use kilt_support::mock::{mock_origin, SubjectId};
	use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
	use scale_info::TypeInfo;
	use sp_core::RuntimeDebug;
	use sp_runtime::{
		traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
		BoundedVec, BuildStorage, MultiSignature, SaturatedConversion,
	};

	use crate::{self as pallet_web3_names, Config, Error};

	type BlockNumber = u64;
	pub(crate) type Balance = u128;

	type Hash = sp_core::H256;
	type Signature = MultiSignature;
	type AccountPublic = <Signature as Verify>::Signer;
	type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

	type Block = frame_system::mocking::MockBlock<Test>;

	frame_support::construct_runtime!(
		pub enum Test
		{
			System: frame_system,
			Balances: pallet_balances,
			Web3Names: pallet_web3_names,
			MockOrigin: mock_origin,
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
		type Block = Block;
		type Nonce = u64;
		type RuntimeOrigin = RuntimeOrigin;
		type RuntimeCall = RuntimeCall;
		type Hash = Hash;
		type Hashing = BlakeTwo256;
		type AccountId = AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
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
		type RuntimeTask = RuntimeTask;
		type MultiBlockMigrator = ();
		type SingleBlockMigrations = ();
		type PostInherents = ();
		type PostTransactions = ();
		type PreInherents = ();
	}

	parameter_types! {
		pub const ExistentialDeposit: Balance = 10;
		pub const MaxLocks: u32 = 50;
		pub const MaxReserves: u32 = 50;
		pub const MaxFreezes: u32 = 50;
	}

	impl pallet_balances::Config for Test {
		type RuntimeFreezeReason = RuntimeFreezeReason;
		type FreezeIdentifier = RuntimeFreezeReason;
		type RuntimeHoldReason = RuntimeHoldReason;
		type MaxFreezes = MaxFreezes;
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

	#[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq, PartialOrd, Ord, Clone)]
	pub struct TestWeb3Name(pub(crate) BoundedVec<u8, <Test as Config>::MaxNameLength>);

	impl TryFrom<Vec<u8>> for TestWeb3Name {
		type Error = Error<Test>;

		fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
			ensure!(
				value.len() >= <Test as Config>::MinNameLength::get().saturated_into(),
				Self::Error::TooShort
			);
			let bounded_vec: BoundedVec<u8, <Test as Config>::MaxNameLength> =
				BoundedVec::try_from(value).map_err(|_| Self::Error::TooLong)?;
			ensure!(is_valid_web3_name(&bounded_vec), Self::Error::InvalidCharacter);
			Ok(Self(bounded_vec))
		}
	}

	fn is_valid_web3_name(input: &[u8]) -> bool {
		input
			.iter()
			.all(|c| matches!(c, b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_'))
	}

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
		type ClaimOrigin = TestOwnerOrigin;
		type OwnerOrigin = TestOwnerOrigin;
		type OriginSuccess = TestOriginSuccess;
		type Currency = Balances;
		type RuntimeHoldReason = RuntimeHoldReason;
		type Deposit = Web3NameDeposit;
		type RuntimeEvent = RuntimeEvent;
		type MaxNameLength = MaxNameLength;
		type MinNameLength = MinNameLength;
		type Web3Name = TestWeb3Name;
		type Web3NameOwner = TestWeb3NameOwner;
		type WeightInfo = ();
		type BalanceMigrationManager = ();

		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper = ();
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
		TestWeb3Name::try_from(web3_name_input.to_vec()).expect("Invalid web3 name input.")
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
			let mut storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
			pallet_balances::GenesisConfig::<Test> {
				balances: self.balances.clone(),
			}
			.assimilate_storage(&mut storage)
			.expect("assimilate should not fail");
			let mut ext = sp_io::TestExternalities::new(storage);

			ext.execute_with(|| {
				// ensure that we are not at the genesis block. Events are not registered for
				// the genesis block.
				System::set_block_number(System::block_number() + 1);

				for (owner, web3_name, payer) in self.claimed_web3_names {
					pallet_web3_names::Pallet::<Test>::register_name(web3_name, owner, payer)
						.expect("Could not register name");
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
				crate::try_state::do_try_state::<Test, _>().expect("Sanity test for w3n failed.");
			})
		}

		#[cfg(feature = "runtime-benchmarks")]
		pub fn build_with_keystore(self) -> sp_io::TestExternalities {
			let mut ext = self.build();

			let keystore = sp_keystore::testing::MemoryKeystore::new();
			ext.register_extension(sp_keystore::KeystoreExt(std::sync::Arc::new(keystore)));

			ext
		}
	}
}
