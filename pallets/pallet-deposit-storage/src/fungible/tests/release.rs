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

use frame_support::{
	assert_err,
	traits::{fungible::MutateHold, tokens::Precision},
};
use kilt_support::Deposit;
use sp_runtime::{AccountId32, TokenError};

use crate::{
	deposit::DepositEntry,
	fungible::{
		tests::mock::{DepositNamespace, ExtBuilder, TestRuntime, TestRuntimeHoldReason, OWNER},
		PalletDepositStorageReason,
	},
	Pallet, SystemDeposits,
};

#[test]
fn release() {
	ExtBuilder::default()
		.with_balances(vec![(OWNER, 100_000)])
		.build_and_execute_with_sanity_tests(|| {
			let reason = PalletDepositStorageReason::default();

			<Pallet<TestRuntime> as MutateHold<AccountId32>>::hold(&reason, &OWNER, 10)
				.expect("Failed to hold amount for user.");

			<Pallet<TestRuntime> as MutateHold<AccountId32>>::release(&reason, &OWNER, 5, Precision::Exact)
				.expect("Failed to release partial amount for user.");
			let deposit_entry = SystemDeposits::<TestRuntime>::get(&reason.namespace, &reason.key)
				.expect("Deposit entry should not be None.");
			assert_eq!(
				deposit_entry,
				DepositEntry {
					deposit: Deposit {
						amount: 5,
						owner: OWNER
					},
					reason: TestRuntimeHoldReason::Deposit,
				}
			);

			// Remove the outstanding holds.
			<Pallet<TestRuntime> as MutateHold<AccountId32>>::release(&reason, &OWNER, 5, Precision::Exact)
				.expect("Failed to release remaining amount for user.");
			assert!(SystemDeposits::<TestRuntime>::get(&reason.namespace, &reason.key).is_none());
		});
}

#[test]
fn release_different_reason() {
	ExtBuilder::default()
		.with_balances(vec![(OWNER, 100_000)])
		.build_and_execute_with_sanity_tests(|| {
			let reason = PalletDepositStorageReason {
				namespace: DepositNamespace::ExampleNamespace,
				key: [0].to_vec().try_into().unwrap(),
			};
			let other_reason = PalletDepositStorageReason {
				namespace: DepositNamespace::ExampleNamespace,
				key: [1].to_vec().try_into().unwrap(),
			};

			<Pallet<TestRuntime> as MutateHold<AccountId32>>::hold(&reason, &OWNER, 10)
				.expect("Failed to hold amount for user.");
			assert_err!(
				<Pallet<TestRuntime> as MutateHold<AccountId32>>::release(&other_reason, &OWNER, 10, Precision::Exact),
				TokenError::FundsUnavailable
			);
		});
}

#[test]
fn release_all() {
	ExtBuilder::default()
		.with_balances(vec![(OWNER, 100_000)])
		.build_and_execute_with_sanity_tests(|| {
			let reason = PalletDepositStorageReason::default();

			<Pallet<TestRuntime> as MutateHold<AccountId32>>::hold(&reason, &OWNER, 10)
				.expect("Failed to hold amount for user.");

			<Pallet<TestRuntime> as MutateHold<AccountId32>>::release_all(&reason, &OWNER, Precision::Exact)
				.expect("Failed to release all amount for user.");
			assert!(SystemDeposits::<TestRuntime>::get(&reason.namespace, &reason.key).is_none());
		});
}
