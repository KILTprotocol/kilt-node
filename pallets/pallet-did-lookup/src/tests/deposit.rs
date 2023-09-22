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
use frame_support::{assert_noop, assert_ok, traits::fungible::InspectHold};
use kilt_support::mock::mock_origin;
use sp_runtime::{traits::Zero, TokenError};

use crate::{mock::*, Error, HoldReason};

#[test]
fn test_change_deposit_owner() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.with_connections(vec![(ACCOUNT_00, DID_00, LINKABLE_ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(DidLookup::change_deposit_owner(
				mock_origin::DoubleOrigin(ACCOUNT_01, DID_00).into(),
				ACCOUNT_00.into()
			));
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01),
				<Test as crate::Config>::Deposit::get()
			);
		})
}

#[test]
fn test_change_deposit_owner_insufficient_balance() {
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50)])
		.with_connections(vec![(ACCOUNT_00, DID_00, LINKABLE_ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				DidLookup::change_deposit_owner(
					mock_origin::DoubleOrigin(ACCOUNT_01, DID_00).into(),
					ACCOUNT_00.into()
				),
				TokenError::CannotCreateHold
			);
		})
}

#[test]
fn test_change_deposit_owner_not_found() {
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				DidLookup::change_deposit_owner(
					mock_origin::DoubleOrigin(ACCOUNT_01, DID_00).into(),
					ACCOUNT_00.into()
				),
				Error::<Test>::NotFound
			);
		})
}

#[test]
fn test_change_deposit_owner_not_authorized() {
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50)])
		.with_connections(vec![(ACCOUNT_00, DID_00, LINKABLE_ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				DidLookup::change_deposit_owner(
					mock_origin::DoubleOrigin(ACCOUNT_01, DID_01).into(),
					ACCOUNT_00.into()
				),
				Error::<Test>::NotAuthorized
			);
		})
}

#[test]
fn test_update_deposit() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.build_and_execute_with_sanity_tests(|| {
			insert_raw_connection::<Test>(
				ACCOUNT_00,
				DID_00,
				ACCOUNT_00.into(),
				<Test as crate::Config>::Deposit::get() * 2,
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as crate::Config>::Deposit::get() * 2
			);
			assert_ok!(DidLookup::update_deposit(
				RuntimeOrigin::signed(ACCOUNT_00),
				ACCOUNT_00.into()
			));
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as crate::Config>::Deposit::get()
			);
		})
}

#[test]
fn test_update_deposit_unauthorized() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.build_and_execute_with_sanity_tests(|| {
			insert_raw_connection::<Test>(
				ACCOUNT_00,
				DID_00,
				ACCOUNT_00.into(),
				<Test as crate::Config>::Deposit::get() * 2,
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as crate::Config>::Deposit::get() * 2
			);
			assert_noop!(
				DidLookup::update_deposit(RuntimeOrigin::signed(ACCOUNT_01), ACCOUNT_00.into()),
				Error::<Test>::NotAuthorized
			);
		})
}

#[test]
fn test_reclaim_deposit() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.with_connections(vec![(ACCOUNT_01, DID_01, LINKABLE_ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(DidLookup::reclaim_deposit(
				RuntimeOrigin::signed(ACCOUNT_01),
				ACCOUNT_00.into()
			));
			assert_eq!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01), 0);
		});
}

#[test]
fn test_reclaim_deposit_not_authorized() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.with_connections(vec![(ACCOUNT_01, DID_01, LINKABLE_ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				DidLookup::reclaim_deposit(RuntimeOrigin::signed(ACCOUNT_00), ACCOUNT_00.into()),
				Error::<Test>::NotAuthorized
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01),
				<Test as crate::Config>::Deposit::get()
			);
		});
}
