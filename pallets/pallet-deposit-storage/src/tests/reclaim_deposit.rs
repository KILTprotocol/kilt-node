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

use frame_support::{assert_noop, assert_ok, traits::fungible::InspectHold};
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
			assert_eq!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &OWNER), 10_000);

			assert_ok!(Pallet::<TestRuntime>::reclaim_deposit(
				RawOrigin::Signed(OWNER).into(),
				namespace.clone(),
				key.clone()
			));

			assert!(Pallet::<TestRuntime>::deposits(&namespace, &key).is_none());
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &OWNER).is_zero());
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
