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

use frame_support::{assert_err, assert_noop, assert_ok, traits::ReservableCurrency};
use frame_system::RawOrigin;
use kilt_support::Deposit;
use sp_runtime::traits::Zero;

use crate::{
	mock::{Balances, DepositNamespace, ExtBuilder, TestRuntime, OTHER_ACCOUNT, OWNER},
	DepositEntryOf, DepositKeyOf, Error, HoldReason, Pallet,
};

#[test]
fn reclaim_deposit_successful() {
	let deposit = DepositEntryOf::<TestRuntime> {
		reason: HoldReason::Deposit.into(),
		deposit: Deposit {
			amount: 10_000,
			owner: OWNER,
		},
	};
	let namespace = DepositNamespace::ExampleNamespace;
	let key = DepositKeyOf::<TestRuntime>::default();
	ExtBuilder::default()
		.with_deposits(vec![(namespace.clone(), key.clone(), deposit)])
		.build()
		.execute_with(|| {
			assert!(Pallet::<TestRuntime>::deposits(&namespace, &key).is_some());
			assert_eq!(Balances::reserved_balance(OWNER), 10_000);

			assert_ok!(Pallet::<TestRuntime>::reclaim_deposit(
				RawOrigin::Signed(OWNER).into(),
				namespace.clone(),
				key.clone()
			));

			assert!(Pallet::<TestRuntime>::deposits(&namespace, &key).is_none());
			assert!(Balances::reserved_balance(OWNER).is_zero());
		});
}

#[test]
fn reclaim_deposit_not_found() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Pallet::<TestRuntime>::reclaim_deposit(
				RawOrigin::Signed(OWNER).into(),
				DepositNamespace::ExampleNamespace,
				DepositKeyOf::<TestRuntime>::default()
			),
			Error::<TestRuntime>::DepositNotFound
		);
	});
}

#[test]
fn reclaim_deposit_unauthorized() {
	let deposit = DepositEntryOf::<TestRuntime> {
		reason: HoldReason::Deposit.into(),
		deposit: Deposit {
			amount: 10_000,
			owner: OWNER,
		},
	};
	let namespace = DepositNamespace::ExampleNamespace;
	let key = DepositKeyOf::<TestRuntime>::default();
	ExtBuilder::default()
		.with_deposits(vec![(namespace.clone(), key.clone(), deposit)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Pallet::<TestRuntime>::reclaim_deposit(
					RawOrigin::Signed(OTHER_ACCOUNT).into(),
					namespace.clone(),
					key.clone()
				),
				Error::<TestRuntime>::Unauthorized
			);
		});
}

#[test]
fn reclaim_deposit_failed_to_release() {
	let deposit = DepositEntryOf::<TestRuntime> {
		reason: HoldReason::Deposit.into(),
		deposit: Deposit {
			amount: 10_000,
			owner: OWNER,
		},
	};
	let namespace = DepositNamespace::ExampleNamespace;
	let key = DepositKeyOf::<TestRuntime>::default();
	ExtBuilder::default()
		.with_deposits(vec![(namespace.clone(), key.clone(), deposit)])
		.build()
		.execute_with(|| {
			// Slash reserved balance for deposit account.
			assert!(Balances::slash_reserved(&OWNER, 10_000).1.is_zero());
			assert_err!(
				Pallet::<TestRuntime>::reclaim_deposit(RawOrigin::Signed(OWNER).into(), namespace.clone(), key.clone()),
				Error::<TestRuntime>::FailedToRelease
			);
		});
}

#[test]
fn add_deposit_new() {
	ExtBuilder::default()
		//	Deposit amount + existential deposit
		.with_balances(vec![(OWNER, 500 + 10_000)])
		.build()
		.execute_with(|| {
			let deposit = DepositEntryOf::<TestRuntime> {
				reason: HoldReason::Deposit.into(),
				deposit: Deposit {
					amount: 10_000,
					owner: OWNER,
				},
			};
			let namespace = DepositNamespace::ExampleNamespace;
			let key = DepositKeyOf::<TestRuntime>::default();

			assert!(Pallet::<TestRuntime>::deposits(&namespace, &key).is_none());
			assert!(Balances::reserved_balance(OWNER).is_zero());

			assert_ok!(Pallet::<TestRuntime>::add_deposit(
				namespace.clone(),
				key.clone(),
				deposit.clone()
			));

			assert_eq!(Pallet::<TestRuntime>::deposits(&namespace, &key), Some(deposit));
			assert_eq!(Balances::reserved_balance(OWNER), 10_000);
		});
}

#[test]
fn add_deposit_existing() {
	let deposit = DepositEntryOf::<TestRuntime> {
		reason: HoldReason::Deposit.into(),
		deposit: Deposit {
			amount: 10_000,
			owner: OWNER,
		},
	};
	let namespace = DepositNamespace::ExampleNamespace;
	let key = DepositKeyOf::<TestRuntime>::default();
	ExtBuilder::default()
		.with_deposits(vec![(namespace.clone(), key.clone(), deposit.clone())])
		.build()
		.execute_with(|| {
			assert_noop!(
				Pallet::<TestRuntime>::add_deposit(namespace.clone(), key.clone(), deposit),
				Error::<TestRuntime>::DepositExisting
			);
		});
}

#[test]
fn add_deposit_failed_to_hold() {
	ExtBuilder::default().build().execute_with(|| {
		let deposit = DepositEntryOf::<TestRuntime> {
			reason: HoldReason::Deposit.into(),
			deposit: Deposit {
				amount: 10_000,
				owner: OWNER,
			},
		};

		assert_noop!(
			Pallet::<TestRuntime>::add_deposit(
				DepositNamespace::ExampleNamespace,
				DepositKeyOf::<TestRuntime>::default(),
				deposit
			),
			Error::<TestRuntime>::FailedToHold
		);
	});
}
