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

use frame_support::{pallet_prelude::ValueQuery, parameter_types, storage_alias};
use frame_system::pallet_prelude::BlockNumberFor;
use kilt_support::{
	mock::{mock_origin, SubjectId},
	traits::StorageDepositCollector,
};

use sp_core::Get;
use sp_runtime::{
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage, MultiSignature,
};

use crate::{
	self as pallet_did_lookup, linkable_account::LinkableAccountId, AccountIdOf, BalanceOf, Config, ConnectedAccounts,
	ConnectedDids, ConnectionRecord, DidIdentifierOf, LinkableAccountDepositCollector,
};

pub(crate) type Block = frame_system::mocking::MockBlock<Test>;
pub(crate) type Hash = sp_core::H256;
pub(crate) type Balance = u128;
pub(crate) type Signature = MultiSignature;
pub(crate) type AccountPublic = <Signature as Verify>::Signer;
pub(crate) type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		DidLookup: pallet_did_lookup,
		MockOrigin: mock_origin,
	}
);

parameter_types! {
	pub const SS58Prefix: u8 = 38;
	pub const BlockHashCount: BlockNumberFor<Test> = 2400;
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
	type RuntimeTask = ();
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

parameter_types! {
	pub const DidLookupDeposit: Balance = 10;
}

pub struct UniqueLinkEnabledFlag;

#[storage_alias]
type FlagStorage = StorageValue<DidLookup, bool, ValueQuery>;

impl UniqueLinkEnabledFlag {
	fn set(flag: bool) {
		FlagStorage::set(flag)
	}
}

impl Get<bool> for UniqueLinkEnabledFlag {
	fn get() -> bool {
		FlagStorage::get()
	}
}

impl pallet_did_lookup::Config for Test {
	type BalanceMigrationManager = ();
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type Currency = Balances;
	type Deposit = DidLookupDeposit;
	type EnsureOrigin = mock_origin::EnsureDoubleOrigin<AccountId, SubjectId>;
	type AssociateOrigin = mock_origin::EnsureDoubleOrigin<AccountId, SubjectId>;
	type OriginSuccess = mock_origin::DoubleOrigin<AccountId, SubjectId>;
	type DidIdentifier = SubjectId;
	type WeightInfo = ();
	type UniqueLinkingEnabled = UniqueLinkEnabledFlag;
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
pub(crate) const LINKABLE_ACCOUNT_01: LinkableAccountId = LinkableAccountId::AccountId32(ACCOUNT_01);

pub(crate) fn insert_raw_connection<T: Config>(
	sender: AccountIdOf<T>,
	did_identifier: DidIdentifierOf<T, ()>,
	account: LinkableAccountId,
	deposit: BalanceOf<T, ()>,
) {
	let deposit = LinkableAccountDepositCollector::<T>::create_deposit(sender, deposit)
		.expect("Account should have enough balance");

	let record = ConnectionRecord {
		deposit,
		did: did_identifier.clone(),
	};

	ConnectedDids::<T>::mutate(&account, |did_entry| {
		if let Some(old_connection) = did_entry.replace(record) {
			ConnectedAccounts::<T>::remove(&old_connection.did, &account);
			LinkableAccountDepositCollector::<T>::free_deposit(old_connection.deposit)
				.expect("Could not release deposit of account");
		}
	});
	ConnectedAccounts::<T>::insert(&did_identifier, &account, ());
}

#[derive(Clone, Default)]
pub struct ExtBuilder {
	balances: Vec<(AccountId, Balance)>,
	/// list of connection (sender, did, connected address)
	connections: Vec<(AccountId, SubjectId, LinkableAccountId)>,
	unique_flag: bool,
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

	pub fn with_unique_connections(mut self) -> Self {
		self.unique_flag = true;
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

			for (sender, did, account) in self.connections {
				pallet_did_lookup::Pallet::<Test>::add_association(sender, did, account)
					.expect("Should create connection");
			}

			UniqueLinkEnabledFlag::set(self.unique_flag);
		});
		ext
	}

	pub fn build_and_execute_with_sanity_tests(self, test: impl FnOnce()) {
		self.build().execute_with(|| {
			test();
			crate::try_state::do_try_state::<Test, _>().expect("Sanity test for did lookup failed.");
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub fn build_with_keystore(self) -> sp_io::TestExternalities {
		let mut ext = self.build();

		let keystore = sp_keystore::testing::MemoryKeystore::new();
		ext.register_extension(sp_keystore::KeystoreExt(sp_std::sync::Arc::new(keystore)));

		ext
	}
}
