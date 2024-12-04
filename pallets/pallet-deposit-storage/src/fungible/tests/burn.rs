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

use frame_support::traits::{
	fungible::MutateHold,
	tokens::{Fortitude, Precision},
};
use kilt_support::Deposit;
// use kilt_support::Deposit;
use sp_runtime::AccountId32;

use crate::{
	// deposit::DepositEntry,
	deposit::DepositEntry,
	fungible::{
		tests::mock::{ExtBuilder, TestRuntime, OWNER},
		PalletDepositStorageReason,
	},
	Deposits,
	Pallet,
};

#[test]
fn burn_held() {
	ExtBuilder::default()
		.with_balances(vec![(OWNER, 100_000)])
		.build()
		.execute_with(|| {
			let reason = PalletDepositStorageReason::default();

			<Pallet<TestRuntime> as MutateHold<AccountId32>>::hold(&reason, &OWNER, 10)
				.expect("Failed to hold amount for user.");

			<Pallet<TestRuntime> as MutateHold<AccountId32>>::burn_held(
				&reason,
				&OWNER,
				9,
				Precision::Exact,
				Fortitude::Polite,
			)
			.expect("Failed to burn partial amount for user.");
			let deposit_entry = Deposits::<TestRuntime>::get(&reason.namespace, &reason.key)
				.expect("Deposit entry should not be None.");
			assert_eq!(
				deposit_entry,
				DepositEntry {
					deposit: Deposit {
						amount: 1,
						owner: OWNER
					},
					reason: reason.clone().into(),
					reclaim_locked: true,
				}
			);

			// Burn the outstanding holds.
			<Pallet<TestRuntime> as MutateHold<AccountId32>>::burn_held(
				&reason,
				&OWNER,
				1,
				Precision::Exact,
				Fortitude::Polite,
			)
			.expect("Failed to burn remaining amount for user.");
			assert!(Deposits::<TestRuntime>::get(&reason.namespace, &reason.key).is_none());
		});
}

#[test]
fn burn_all_held() {
	ExtBuilder::default()
		.with_balances(vec![(OWNER, 100_000)])
		.build()
		.execute_with(|| {
			let reason = PalletDepositStorageReason::default();

			<Pallet<TestRuntime> as MutateHold<AccountId32>>::hold(&reason, &OWNER, 10)
				.expect("Failed to hold amount for user.");

			<Pallet<TestRuntime> as MutateHold<AccountId32>>::burn_all_held(
				&reason,
				&OWNER,
				Precision::Exact,
				Fortitude::Polite,
			)
			.expect("Failed to burn all amount for user.");
			assert!(Deposits::<TestRuntime>::get(&reason.namespace, &reason.key).is_none());
		});
}
