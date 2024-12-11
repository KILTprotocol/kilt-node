// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

use frame_support::{assert_err, assert_ok, traits::fungible::MutateHold};
use kilt_support::Deposit;
use sp_runtime::AccountId32;

use crate::{
	deposit::DepositEntry,
	fungible::{
		tests::mock::{Balances, ExtBuilder, TestRuntime, TestRuntimeHoldReason, OTHER_ACCOUNT, OWNER},
		PalletDepositStorageReason,
	},
	Error, Pallet, SystemDeposits,
};

#[test]
fn hold() {
	ExtBuilder::default()
		.with_balances(vec![(OWNER, 100_000)])
		.build_and_execute_with_sanity_tests(|| {
			let reason = PalletDepositStorageReason::default();

			<Pallet<TestRuntime> as MutateHold<AccountId32>>::hold(&reason, &OWNER, 10)
				.expect("Failed to hold amount for user.");
			let deposit_entry = SystemDeposits::<TestRuntime>::get(&reason.namespace, &reason.key)
				.expect("Deposit entry should not be None.");
			assert_eq!(
				deposit_entry,
				DepositEntry {
					deposit: Deposit {
						amount: 10,
						owner: OWNER
					},
					reason: TestRuntimeHoldReason::Deposit,
				}
			);

			<Pallet<TestRuntime> as MutateHold<AccountId32>>::hold(&reason, &OWNER, 5)
				.expect("Failed to hold amount for user.");
			let deposit_entry = SystemDeposits::<TestRuntime>::get(&reason.namespace, &reason.key)
				.expect("Deposit entry should not be None.");
			assert_eq!(
				deposit_entry,
				DepositEntry {
					deposit: Deposit {
						amount: 15,
						owner: OWNER
					},
					reason: TestRuntimeHoldReason::Deposit,
				}
			);
		});
}

#[test]
fn zero_hold() {
	ExtBuilder::default()
		.with_balances(vec![(OWNER, 100_000)])
		.build_and_execute_with_sanity_tests(|| {
			let reason = PalletDepositStorageReason::default();
			<Pallet<TestRuntime> as MutateHold<AccountId32>>::hold(&reason, &OWNER, 0)
				.expect("Failed to hold amount for user.");
			// A hold of zero for a new deposit should not create any new storage entry.
			assert!(SystemDeposits::<TestRuntime>::get(&reason.namespace, &reason.key).is_none());
		});
}

#[test]
fn hold_same_reason_different_user() {
	ExtBuilder::default()
		.with_balances(vec![(OWNER, 100_000), (OTHER_ACCOUNT, 100_000)])
		.build_and_execute_with_sanity_tests(|| {
			let reason = PalletDepositStorageReason::default();
			assert_ok!(<Pallet<TestRuntime> as MutateHold<AccountId32>>::hold(
				&reason, &OWNER, 10
			));
			assert_err!(
				<Pallet<TestRuntime> as MutateHold<AccountId32>>::hold(&reason, &OTHER_ACCOUNT, 10),
				Error::<TestRuntime>::DepositExisting
			);
		});
}

#[test]
fn too_many_holds() {
	ExtBuilder::default()
		.with_balances(vec![(OWNER, 100_000)])
		.build_and_execute_with_sanity_tests(|| {
			// Occupy the only hold available with a different reason.
			<Balances as MutateHold<AccountId32>>::hold(&TestRuntimeHoldReason::Else, &OWNER, 1)
				.expect("Failed to hold amount for user.");
			// Try to hold a second time, hitting the mock limit of 1.
			assert_err!(
				<Pallet<TestRuntime> as MutateHold<AccountId32>>::hold(
					&PalletDepositStorageReason::default(),
					&OWNER,
					10
				),
				pallet_balances::Error::<TestRuntime>::TooManyHolds
			);
		});
}
