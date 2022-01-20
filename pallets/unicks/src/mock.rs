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

parameter_types! {
	pub const MaxUnickLength: u32 = 64;
	pub const UnickDeposit: u32 = 10;
}

impl pallet_unicks::Config for Test {
	type BlacklistOrigin = EnsureRoot<AccountId>;
	type Currency = Balances;
	type Deposit = UnickDeposit;
	type Event = Event;
	type MaxUnickLength = MaxUnickLength;
	type OriginSuccess = mock_origin::DoubleOrigin<AccountId, SubjectId>;
	type RegularOrigin = mock_origin::EnsureDoubleOrigin<AccountId, SubjectId>;
	type Unick = pallet_unicks::types::AsciiUnick<Test, MaxUnickLength>;
	type UnickOwner = SubjectId;
}

impl mock_origin::Config for Test {
	type Origin = Origin;
	type AccountId = AccountId;
	type SubjectId = SubjectId;
}
