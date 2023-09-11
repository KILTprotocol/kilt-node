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

use frame_support::{
	assert_noop, assert_ok,
	traits::fungible::{Inspect, InspectHold},
	BoundedVec,
};

use kilt_support::{mock::mock_origin, Deposit};
use sp_runtime::traits::Zero;

use crate::{mock::*, Error, HoldReason, Names, Owner, Pallet, Web3OwnershipOf};

#[test]
fn claiming_successful() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	let initial_balance: Balance = 100;
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, initial_balance)])
		.build_and_execute_with_sanity_tests(|| {
			assert!(Names::<Test>::get(&DID_00).is_none());
			assert!(Owner::<Test>::get(&web3_name_00).is_none());
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());

			assert_ok!(Pallet::<Test>::claim(
				mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(),
				web3_name_00.clone().0,
			));
			let web3_name = Names::<Test>::get(&DID_00).expect("Web3 name should be stored.");
			let owner_details = Owner::<Test>::get(&web3_name_00).expect("Owner should be stored.");

			// Test that the name matches
			assert_eq!(web3_name, web3_name_00);
			// Test that the ownership details match
			assert_eq!(
				owner_details,
				Web3OwnershipOf::<Test> {
					owner: DID_00,
					claimed_at: 0,
					deposit: Deposit {
						owner: ACCOUNT_00,
						amount: Web3NameDeposit::get(),
					},
				}
			);
			// Test that the deposit was reserved correctly.
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				Web3NameDeposit::get()
			);
			assert_eq!(Balances::balance(&ACCOUNT_00), initial_balance - Web3NameDeposit::get(),);

			// Test that the same name cannot be claimed again.
			assert_noop!(
				Pallet::<Test>::claim(
					mock_origin::DoubleOrigin(ACCOUNT_01, DID_01).into(),
					web3_name_00.clone().0,
				),
				Error::<Test>::AlreadyExists
			);

			// Test that the same owner cannot claim a new name.
			let web3_name_01 = get_web3_name(WEB3_NAME_01_INPUT);
			assert_noop!(
				Pallet::<Test>::claim(mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(), web3_name_01.0,),
				Error::<Test>::OwnerAlreadyExists
			);
		})
}

#[test]
fn claiming_invalid() {
	let too_short_web3_names = vec![
		// Empty name
		BoundedVec::try_from(b"".to_vec()).unwrap(),
		// Single-char name
		BoundedVec::try_from(b"1".to_vec()).unwrap(),
		// Two-letter name
		BoundedVec::try_from(b"10".to_vec()).unwrap(),
	];

	let invalid_web3_names = vec![
		// Not allowed ASCII character name (invalid symbol)
		BoundedVec::try_from(b"10:1".to_vec()).unwrap(),
		// Not allowed ASCII character name (uppercase letter)
		BoundedVec::try_from(b"abcdE".to_vec()).unwrap(),
		// Not allowed ASCII character name (whitespace)
		BoundedVec::try_from(b"    ".to_vec()).unwrap(),
		// Non-ASCII character name
		BoundedVec::try_from(String::from("notascii😁").as_bytes().to_owned()).unwrap(),
	];
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, 100)])
		.build_and_execute_with_sanity_tests(|| {
			for too_short_input in too_short_web3_names.iter() {
				assert_noop!(
					Pallet::<Test>::claim(
						mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(),
						too_short_input.clone(),
					),
					Error::<Test>::TooShort,
				);
			}
			for input in invalid_web3_names.iter() {
				assert_noop!(
					Pallet::<Test>::claim(mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(), input.clone()),
					Error::<Test>::InvalidCharacter,
				);
			}
		})
}

#[test]
fn claiming_banned() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, 100)])
		.with_banned_web3_names(vec![web3_name_00.clone()])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Pallet::<Test>::claim(mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(), web3_name_00.0),
				Error::<Test>::Banned
			);
		})
}

#[test]
fn claiming_not_enough_funds() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, Web3NameDeposit::get() - 1)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Pallet::<Test>::claim(mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(), web3_name_00.0),
				Error::<Test>::InsufficientFunds
			);
		})
}
