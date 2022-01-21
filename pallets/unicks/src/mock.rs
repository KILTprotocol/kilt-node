// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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

use frame_support::parameter_types;
use frame_system::EnsureRoot;
use kilt_support::mock::{mock_origin, SubjectId};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

use crate as pallet_unicks;
use runtime_common::{AccountId, Balance, BlockHashCount, BlockNumber, Hash, Index};

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
		Unicks: pallet_unicks::{Pallet, Storage, Call, Event<T>},
		MockOrigin: mock_origin::{Pallet, Origin<T>},
	}
);

parameter_types! {
	pub const SS58Prefix: u8 = 38;
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = Index;
	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: Balance = 10;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

pub(crate) type TestUnick = pallet_unicks::types::AsciiUnick<Test, MaxUnickLength>;
pub(crate) type TestUnickOwner = SubjectId;
pub(crate) type TestUnickPayer = AccountId;
pub(crate) type TestRegularOrigin = mock_origin::EnsureDoubleOrigin<TestUnickPayer, TestUnickOwner>;
pub(crate) type TestOriginSuccess = mock_origin::DoubleOrigin<TestUnickPayer, TestUnickOwner>;
pub(crate) type TestBlacklistOrigin = EnsureRoot<AccountId>;

parameter_types! {
	pub const MaxUnickLength: u32 = 32;
	pub const MinUnickLength: u32 = 3;
	// Easier to setup insufficient funds for deposit but still above existential deposit
	pub const UnickDeposit: Balance = 2 * ExistentialDeposit::get();
}

impl pallet_unicks::Config for Test {
	type BlacklistOrigin = TestBlacklistOrigin;
	type Currency = Balances;
	type Deposit = UnickDeposit;
	type Event = Event;
	type MaxUnickLength = MaxUnickLength;
	type MinUnickLength = MinUnickLength;
	type OriginSuccess = TestOriginSuccess;
	type RegularOrigin = TestRegularOrigin;
	type Unick = TestUnick;
	type UnickOwner = TestUnickOwner;
}

impl mock_origin::Config for Test {
	type Origin = Origin;
	type AccountId = AccountId;
	type SubjectId = SubjectId;
}

pub(crate) const ACCOUNT_00: TestUnickPayer = AccountId::new([1u8; 32]);
pub(crate) const ACCOUNT_01: TestUnickPayer = AccountId::new([2u8; 32]);
pub(crate) const DID_00: TestUnickOwner = SubjectId(ACCOUNT_00);
pub(crate) const DID_01: TestUnickOwner = SubjectId(ACCOUNT_01);

pub(crate) fn unick_00() -> TestUnick {
	pallet_unicks::types::AsciiUnick::try_from(b"unick_00".to_vec()).unwrap()
}
pub(crate) fn unick_01() -> TestUnick {
	pallet_unicks::types::AsciiUnick::try_from(b"unick_01".to_vec()).unwrap()
}

#[derive(Clone, Default)]
pub struct ExtBuilder {
	balances: Vec<(TestUnickPayer, Balance)>,
	claimed_unicks: Vec<(TestUnickOwner, TestUnick, TestUnickPayer)>,
	blacklisted_unicks: Vec<TestUnick>,
}

impl ExtBuilder {
	pub fn with_balances(mut self, balances: Vec<(TestUnickPayer, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn with_unicks(mut self, unicks: Vec<(TestUnickOwner, TestUnick, TestUnickPayer)>) -> Self {
		self.claimed_unicks = unicks;
		self
	}

	pub fn with_blacklisted_unicks(mut self, unicks: Vec<TestUnick>) -> Self {
		self.blacklisted_unicks = unicks;
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
			for (owner, unick, payer) in self.claimed_unicks {
				pallet_unicks::Pallet::<Test>::register_unick_unsafe(unick, owner, payer);
			}

			for unick in self.blacklisted_unicks {
				assert!(pallet_unicks::Owner::<Test>::get(&unick).is_none());
				pallet_unicks::Pallet::<Test>::blacklist_unick_unsafe(&unick);
			}
		});
		ext
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub fn build_with_keystore(self) -> sp_io::TestExternalities {
		let mut ext = self.build();

		let keystore = sp_keystore::testing::KeyStore::new();
		ext.register_extension(sp_keystore::KeystoreExt(std::sync::Arc::new(keystore)));

		ext
	}
}
