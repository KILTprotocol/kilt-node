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

use kilt_support::{mock::mock_origin, Deposit};
use sp_runtime::{traits::Zero, TokenError};

use crate::{mock::*, Config, Error, HoldReason, Owner, Pallet};

#[test]
fn test_change_deposit_owner() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	let initial_balance: Balance = <Test as Config>::Deposit::get() * 100;
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, initial_balance), (ACCOUNT_01, initial_balance)])
		.with_web3_names(vec![(DID_00, web3_name_00.clone(), ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(Pallet::<Test>::change_deposit_owner(
				mock_origin::DoubleOrigin(ACCOUNT_01, DID_00).into(),
			));
			assert_eq!(
				Owner::<Test>::get(&web3_name_00)
					.expect("w3n should be retained")
					.deposit,
				Deposit {
					owner: ACCOUNT_01,
					amount: <Test as Config>::Deposit::get()
				}
			);
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01),
				<Test as Config>::Deposit::get()
			);
		})
}

#[test]
fn test_change_deposit_owner_insufficient_balance() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	let initial_balance: Balance = <Test as Config>::Deposit::get() * 100;
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, initial_balance)])
		.with_web3_names(vec![(DID_00, web3_name_00, ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Pallet::<Test>::change_deposit_owner(mock_origin::DoubleOrigin(ACCOUNT_01, DID_00).into()),
				TokenError::CannotCreateHold
			);
		})
}

#[test]
fn test_change_deposit_owner_not_found() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	let initial_balance: Balance = <Test as Config>::Deposit::get() * 100;
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, initial_balance)])
		.with_web3_names(vec![(DID_00, web3_name_00, ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Pallet::<Test>::change_deposit_owner(mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into()),
				Error::<Test>::NotFound
			);
		})
}

#[test]
fn test_update_deposit() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	let initial_balance: Balance = <Test as Config>::Deposit::get() * 100;
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, initial_balance)])
		.build_and_execute_with_sanity_tests(|| {
			insert_raw_w3n::<Test>(
				ACCOUNT_00,
				DID_00,
				web3_name_00.clone(),
				12,
				<Test as Config>::Deposit::get() * 2,
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get() * 2
			);
			assert_ok!(Pallet::<Test>::update_deposit(
				RuntimeOrigin::signed(ACCOUNT_00),
				WEB3_NAME_00_INPUT.to_vec().try_into().unwrap()
			));
			assert_eq!(
				Owner::<Test>::get(&web3_name_00)
					.expect("w3n should be retained")
					.deposit,
				Deposit {
					owner: ACCOUNT_00,
					amount: <Test as Config>::Deposit::get()
				}
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get()
			);
		})
}

#[test]
fn test_update_deposit_unauthorized() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	let initial_balance: Balance = <Test as Config>::Deposit::get() * 100;
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, initial_balance)])
		.build_and_execute_with_sanity_tests(|| {
			insert_raw_w3n::<Test>(
				ACCOUNT_00,
				DID_00,
				web3_name_00.clone(),
				12,
				<Test as Config>::Deposit::get() * 2,
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get() * 2
			);
			assert_noop!(
				Pallet::<Test>::update_deposit(
					RuntimeOrigin::signed(ACCOUNT_01),
					WEB3_NAME_00_INPUT.to_vec().try_into().unwrap()
				),
				Error::<Test>::NotAuthorized
			);
		})
}
