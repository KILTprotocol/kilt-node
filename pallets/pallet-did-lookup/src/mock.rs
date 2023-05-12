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

use frame_support::{parameter_types, traits::ReservableCurrency};
use kilt_support::{
	deposit::Deposit,
	mock::{mock_origin, SubjectId},
};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature,
};

use crate::{
	self as pallet_did_lookup, linkable_account::LinkableAccountId, AccountIdOf, BalanceOf, Config, ConnectedAccounts,
	ConnectedDids, ConnectionRecord, CurrencyOf, DidIdentifierOf,
};

pub(crate) type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub(crate) type Block = frame_system::mocking::MockBlock<Test>;
pub(crate) type Hash = sp_core::H256;
pub(crate) type Balance = u128;
pub(crate) type Signature = MultiSignature;
pub(crate) type AccountPublic = <Signature as Verify>::Signer;
pub(crate) type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
pub(crate) type Index = u64;
pub(crate) type BlockNumber = u64;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
		DidLookup: pallet_did_lookup::{Pallet, Storage, Call, Event<T>},
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

parameter_types! {
	pub const DidLookupDeposit: Balance = 10;
}

impl pallet_did_lookup::Config for Test {
	type RuntimeEvent = RuntimeEvent;

	type Currency = Balances;
	type Deposit = DidLookupDeposit;

	type EnsureOrigin = mock_origin::EnsureDoubleOrigin<AccountId, SubjectId>;
	type OriginSuccess = mock_origin::DoubleOrigin<AccountId, SubjectId>;
	type DidIdentifier = SubjectId;

	type WeightInfo = ();
}

impl mock_origin::Config for Test {
	type RuntimeOrigin = RuntimeOrigin;
	type AccountId = AccountId;
	type SubjectId = SubjectId;
}

pub(crate) const ACCOUNT_00: AccountId = AccountId::new([1u8; 32]);
pub(crate) const ACCOUNT_01: AccountId = AccountId::new([2u8; 32]);
pub(crate) const DID_00: SubjectId = SubjectId(ACCOUNT_00);
pub(crate) const DID_01: SubjectId = SubjectId(ACCOUNT_01);
pub(crate) const LINKABLE_ACCOUNT_00: LinkableAccountId = LinkableAccountId::AccountId32(ACCOUNT_00);

pub(crate) fn insert_raw_connection<T: Config>(
	sender: AccountIdOf<T>,
	did_identifier: DidIdentifierOf<T>,
	account: LinkableAccountId,
	deposit: BalanceOf<T>,
) {
	let deposit = Deposit {
		owner: sender,
		amount: deposit,
	};
	let record = ConnectionRecord {
		deposit,
		did: did_identifier.clone(),
	};

	CurrencyOf::<T>::reserve(&record.deposit.owner, record.deposit.amount).expect("Account should have enough balance");

	ConnectedDids::<T>::mutate(&account, |did_entry| {
		if let Some(old_connection) = did_entry.replace(record) {
			ConnectedAccounts::<T>::remove(&old_connection.did, &account);
			kilt_support::free_deposit::<AccountIdOf<T>, CurrencyOf<T>>(&old_connection.deposit);
		}
	});
	ConnectedAccounts::<T>::insert(&did_identifier, &account, ());
}

#[derive(Clone, Default)]
pub struct ExtBuilder {
	balances: Vec<(AccountId, Balance)>,
	/// list of connection (sender, did, connected address)
	connections: Vec<(AccountId, SubjectId, LinkableAccountId)>,
}

impl ExtBuilder {
	#[must_use]
	pub fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	/// Add a connection: (sender, did, connected address)
	#[must_use]
	pub fn with_connections(mut self, connections: Vec<(AccountId, SubjectId, LinkableAccountId)>) -> Self {
		self.connections = connections;
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
			for (sender, did, account) in self.connections {
				pallet_did_lookup::Pallet::<Test>::add_association(sender, did, account)
					.expect("Should create connection");
			}
		});
		ext
	}

	pub fn build_and_execute_with_sanity_tests(self, test: impl FnOnce()) {
		self.build().execute_with(|| {
			test();
			crate::try_state::do_try_state::<Test>().expect("Sanity test for did lookup failed.");
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
