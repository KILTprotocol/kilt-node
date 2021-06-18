// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

#![allow(clippy::from_over_into)]

use crate as kilt_launch;
use frame_support::{assert_noop, assert_ok, parameter_types, traits::GenesisBuild};
use frame_system as system;
use kilt_primitives::{constants::MIN_VESTED_TRANSFER_AMOUNT, AccountId, Balance, BlockNumber, Hash, Index};
use pallet_balances::{BalanceLock, Locks, Reasons};
use pallet_vesting::VestingInfo;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, ConvertInto, IdentityLookup, Zero},
	AccountId32,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const PSEUDO_1: AccountId = AccountId32::new([0u8; 32]);
pub const PSEUDO_2: AccountId = AccountId32::new([1u8; 32]);
pub const PSEUDO_3: AccountId = AccountId32::new([2u8; 32]);
pub const PSEUDO_4: AccountId = AccountId32::new([3u8; 32]);
pub const USER: AccountId = AccountId32::new([10u8; 32]);
pub const TRANSFER_ACCOUNT: AccountId = AccountId32::new([100u8; 32]);

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>},
		KiltLaunch: kilt_launch::{Pallet, Call, Config<T>, Storage, Event<T>},
		Vesting: pallet_vesting::{Pallet, Call, Config<T>, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 38;
}

impl system::Config for Test {
	type BaseCallFilter = ();
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
	pub const ExistentialDeposit: Balance = 500;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const MaxClaims: u32 = 4;
	pub const UsableBalance: Balance = 1;
}

impl kilt_launch::Config for Test {
	type Event = Event;
	type MaxClaims = MaxClaims;
	type UsableBalance = UsableBalance;
	type WeightInfo = ();
}

parameter_types! {
	pub const MinVestedTransfer: Balance = MIN_VESTED_TRANSFER_AMOUNT;
}

impl pallet_vesting::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	// disable vested transfers by setting min amount to max balance
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = ();
}

pub struct ExtBuilder {
	balance_locks: Vec<(AccountId, BlockNumber, Balance)>,
	vesting: Vec<(AccountId, BlockNumber, Balance)>,
	#[allow(dead_code)]
	transfer_account: AccountId,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balance_locks: vec![],
			vesting: vec![],
			transfer_account: TRANSFER_ACCOUNT,
		}
	}
}

/// Calls `migrate_genesis_account` and checks whether balance, vesting and
/// balance locks have been migrated properly to the destination address.
pub fn ensure_single_migration_works(
	source: &AccountId,
	dest: &AccountId,
	vesting_info: Option<VestingInfo<Balance, BlockNumber>>,
	locked_info: Option<(kilt_launch::LockedBalance<Test>, Balance)>,
) {
	assert_noop!(
		KiltLaunch::migrate_genesis_account(Origin::signed(PSEUDO_1), source.to_owned(), dest.to_owned()),
		kilt_launch::Error::<Test>::Unauthorized
	);
	assert_ok!(KiltLaunch::migrate_genesis_account(
		Origin::signed(TRANSFER_ACCOUNT),
		source.to_owned(),
		dest.to_owned()
	));
	let now: BlockNumber = System::block_number();

	// Check for desired death of allocation account
	assert_eq!(Balances::free_balance(source), 0);
	assert_eq!(Vesting::vesting(source), None);
	assert_eq!(kilt_launch::BalanceLocks::<Test>::get(source), None);
	assert!(!frame_system::Account::<Test>::contains_key(source));

	// Check storage migration to dest
	let mut locked_balance: Balance = Balance::zero();
	let mut num_of_locks = 0;
	if let Some(vesting) = vesting_info {
		assert_eq!(Vesting::vesting(dest), Some(vesting));
		locked_balance = vesting.locked;
		num_of_locks += 1;
	}
	if let Some((lock, _)) = locked_info.clone() {
		// only if the lock is not expired, it should show up here
		if lock.block > now {
			assert_eq!(kilt_launch::BalanceLocks::<Test>::get(dest), Some(lock.clone()));
			assert_eq!(
				kilt_launch::UnlockingAt::<Test>::get(lock.block),
				Some(vec![dest.to_owned()])
			);
			locked_balance = locked_balance.max(lock.amount);
			num_of_locks += 1;
		}
	}

	// Check correct setting of locks for dest
	let balance_locks = Locks::<Test>::get(dest);
	let mut fee_balance: Balance = Balance::zero();
	let mut maybe_balance: Balance = Balance::zero();
	let mut usable_balance: Balance = Balance::zero();
	assert_eq!(balance_locks.len(), num_of_locks);
	for BalanceLock { id, amount, reasons } in balance_locks {
		match id {
			crate::VESTING_ID => {
				let VestingInfo { locked, per_block, .. } = vesting_info.expect("No vesting schedule found");
				fee_balance = locked;
				usable_balance = per_block;
				assert_eq!(reasons, Reasons::Misc);
			}
			crate::KILT_LAUNCH_ID => {
				let (lock, add) = locked_info.clone().expect("No vesting schedule found");
				assert_eq!(amount, lock.amount);
				assert_eq!(reasons, Reasons::All);
				maybe_balance = add + <Test as crate::Config>::UsableBalance::get();
			}
			_ => panic!("Unexpected balance lock id {:?}", id),
		};
	}

	if num_of_locks > 0 {
		assert_noop!(
			KiltLaunch::migrate_genesis_account(Origin::signed(TRANSFER_ACCOUNT), dest.to_owned(), TRANSFER_ACCOUNT),
			kilt_launch::Error::<Test>::UnexpectedLocks
		);
	}

	// TODO: Add positive check for staking once it has been added

	// Check correct migration of balance
	// In our tests, vesting and locking is not resolved before the 10th block. At
	// most times, now should be the first block.
	if now < 10 {
		// locked balance should be free
		// custom locks: + UsableBalance
		assert_eq!(Balances::free_balance(dest), locked_balance + maybe_balance);
		// balance which is usable for fees
		// vesting: locked_balance
		// custom lock: UsableBalance
		assert_eq!(Balances::usable_balance_for_fees(dest), fee_balance + maybe_balance);
		// balance which is usable for anything but fees and other
		// vesting: per_block * now
		// locks custom locked: UsableBalance
		assert_eq!(Balances::usable_balance(dest), usable_balance + maybe_balance);
		// there should be nothing reserved
		assert_eq!(Balances::reserved_balance(dest), 0);

		// Should not be able to transfer more than which is unlocked in first block
		assert_noop!(
			Balances::transfer(
				Origin::signed(dest.to_owned()),
				TRANSFER_ACCOUNT,
				usable_balance + maybe_balance + 1
			),
			pallet_balances::Error::<Test, ()>::LiquidityRestrictions
		);
	}
}

// Checks whether the usable balance meets the expectations and if exists, if it
// can be transferred which we expect once locks are removed
pub fn assert_balance(who: AccountId, free: Balance, usable_for_fees: Balance, usable: Balance, do_transfer: bool) {
	// Check balance after unlocking
	assert_eq!(Balances::free_balance(&who), free);
	// locked balance should be usable for fees
	assert_eq!(Balances::usable_balance_for_fees(&who), usable_for_fees);
	// locked balance should not be usable for anything but fees and other locks
	assert_eq!(Balances::usable_balance(&who), usable);
	// there should be nothing reserved
	assert_eq!(Balances::reserved_balance(&who), 0);

	if do_transfer && usable > ExistentialDeposit::get() {
		// Should be able to transfer all tokens but ExistentialDeposit
		assert_ok!(Balances::transfer(
			Origin::signed(who),
			TRANSFER_ACCOUNT,
			usable - ExistentialDeposit::get()
		));
	}
}

impl ExtBuilder {
	pub fn vest(mut self, vesting: Vec<(AccountId, BlockNumber, Balance)>) -> Self {
		self.vesting = vesting;
		self
	}

	pub fn pseudos_vest_all(self) -> Self {
		self.vest(vec![
			(PSEUDO_1, 10, 10_000),
			(PSEUDO_2, 20, 10_000),
			(PSEUDO_3, 30, 300_000),
		])
	}

	pub fn lock_balance(mut self, balance_locks: Vec<(AccountId, BlockNumber, Balance)>) -> Self {
		self.balance_locks = balance_locks;
		self
	}

	pub fn pseudos_lock_something(self) -> Self {
		self.lock_balance(vec![(PSEUDO_1, 100, 1111), (PSEUDO_2, 1337, 2222)])
	}

	pub fn pseudos_lock_all(self) -> Self {
		self.lock_balance(vec![
			(PSEUDO_1, 100, 10_000),
			(PSEUDO_2, 1337, 10_000),
			(PSEUDO_3, 100, 300_000),
		])
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		pallet_balances::GenesisConfig::<Test> {
			balances: vec![
				(PSEUDO_1, 10_000),
				(PSEUDO_2, 10_000),
				(PSEUDO_3, 300_000),
				(PSEUDO_4, 10_000),
				(TRANSFER_ACCOUNT, 10_000),
			],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		kilt_launch::GenesisConfig::<Test> {
			balance_locks: self.balance_locks,
			vesting: self.vesting,
			transfer_account: TRANSFER_ACCOUNT,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	pub fn build_panic(
		self,
		balances: Vec<(AccountId, Balance)>,
		balance_locks: Vec<(AccountId, BlockNumber, Balance)>,
		vesting: Vec<(AccountId, BlockNumber, Balance)>,
	) {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		pallet_balances::GenesisConfig::<Test> { balances }
			.assimilate_storage(&mut t)
			.unwrap();

		kilt_launch::GenesisConfig::<Test> {
			balance_locks,
			vesting,
			transfer_account: TRANSFER_ACCOUNT,
		}
		.assimilate_storage(&mut t)
		.unwrap()
	}
}
