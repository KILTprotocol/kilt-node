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

use frame_support::{assert_noop, assert_ok, BoundedVec};

use frame_system::RawOrigin;
use kilt_support::{deposit::Deposit, mock::mock_origin};
use sp_runtime::{traits::Zero, DispatchError};

use crate::{mock::*, Banned, Config, Error, Names, Owner, Pallet, Web3OwnershipOf};

// #############################################################################
// Name claiming

#[test]
fn claiming_successful() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	let initial_balance: Balance = 100;
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, initial_balance)])
		.build_and_execute_with_sanity_tests(|| {
			assert!(Names::<Test>::get(&DID_00).is_none());
			assert!(Owner::<Test>::get(&web3_name_00).is_none());
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());

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
			assert_eq!(Balances::reserved_balance(ACCOUNT_00), Web3NameDeposit::get());
			assert_eq!(
				Balances::free_balance(ACCOUNT_00),
				initial_balance - Web3NameDeposit::get(),
			);

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

// #############################################################################
// Name releasing

#[test]
fn releasing_by_owner_successful() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	let initial_balance: Balance = 100;
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, initial_balance)])
		.with_web3_names(vec![(DID_00, web3_name_00.clone(), ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(Pallet::<Test>::release_by_owner(
				// Submitter != deposit payer, owner == name owner
				mock_origin::DoubleOrigin(ACCOUNT_01, DID_00).into(),
			));
			assert!(Names::<Test>::get(&DID_00).is_none());
			assert!(Owner::<Test>::get(&web3_name_00).is_none());

			// Test that the deposit was returned to the payer correctly.
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());
			assert_eq!(Balances::free_balance(ACCOUNT_00), initial_balance);
		})
}

#[test]
fn releasing_by_payer_successful() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	let initial_balance: Balance = 100;
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, initial_balance)])
		.with_web3_names(vec![(DID_00, web3_name_00.clone(), ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(Pallet::<Test>::reclaim_deposit(
				// Submitter == deposit payer
				RawOrigin::Signed(ACCOUNT_00).into(),
				web3_name_00.clone().0,
			));
			assert!(Names::<Test>::get(&DID_00).is_none());
			assert!(Owner::<Test>::get(&web3_name_00).is_none());

			// Test that the deposit was returned to the payer correctly.
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());
			assert_eq!(Balances::free_balance(ACCOUNT_00), initial_balance);
		})
}

#[test]
fn releasing_not_found() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	ExtBuilder::default().build().execute_with(|| {
		// Fail to claim by owner
		assert_noop!(
			Pallet::<Test>::release_by_owner(mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into()),
			Error::<Test>::OwnerNotFound
		);
		// Fail to claim by payer
		assert_noop!(
			Pallet::<Test>::reclaim_deposit(RawOrigin::Signed(ACCOUNT_00).into(), web3_name_00.0),
			Error::<Test>::NotFound
		);
	})
}

#[test]
fn releasing_not_authorized() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, 100)])
		.with_web3_names(vec![(DID_00, web3_name_00.clone(), ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			// Fail to claim by different payer
			assert_noop!(
				Pallet::<Test>::reclaim_deposit(RawOrigin::Signed(ACCOUNT_01).into(), web3_name_00.clone().0),
				Error::<Test>::NotAuthorized
			);
		})
}

#[test]
fn releasing_banned() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	ExtBuilder::default()
		.with_banned_web3_names(vec![(web3_name_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Pallet::<Test>::release_by_owner(mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into()),
				// A banned name will be removed from the map of used names, so it will be considered not
				// existing.
				Error::<Test>::OwnerNotFound
			);
		})
}

// #############################################################################
// Name banning

#[test]
fn banning_successful() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	let web3_name_01 = get_web3_name(WEB3_NAME_01_INPUT);
	let initial_balance = 100;
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, initial_balance)])
		.with_web3_names(vec![(DID_00, web3_name_00.clone(), ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			// Ban a claimed name
			assert_ok!(Pallet::<Test>::ban(RawOrigin::Root.into(), web3_name_00.clone().0));

			assert!(Names::<Test>::get(&DID_00).is_none());
			assert!(Owner::<Test>::get(&web3_name_00).is_none());
			assert!(Banned::<Test>::get(&web3_name_00).is_some());

			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());
			assert_eq!(Balances::free_balance(ACCOUNT_00), initial_balance);

			// Ban an unclaimed name
			assert_ok!(Pallet::<Test>::ban(RawOrigin::Root.into(), web3_name_01.clone().0));

			assert!(Owner::<Test>::get(&web3_name_01).is_none());
			assert!(Banned::<Test>::get(&web3_name_01).is_some());
		})
}

#[test]
fn banning_already_banned() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	ExtBuilder::default()
		.with_banned_web3_names(vec![web3_name_00.clone()])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Pallet::<Test>::ban(RawOrigin::Root.into(), web3_name_00.clone().0),
				Error::<Test>::AlreadyBanned
			);
		})
}

#[test]
fn banning_unauthorized_origin() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	ExtBuilder::default().build().execute_with(|| {
		// Signer origin
		assert_noop!(
			Pallet::<Test>::ban(RawOrigin::Signed(ACCOUNT_00).into(), web3_name_00.clone().0),
			DispatchError::BadOrigin
		);
		// Owner origin
		assert_noop!(
			Pallet::<Test>::ban(
				mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into(),
				web3_name_00.clone().0
			),
			DispatchError::BadOrigin
		);
	})
}

// #############################################################################
// Name unbanning

#[test]
fn unbanning_successful() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, 100)])
		.with_banned_web3_names(vec![web3_name_00.clone()])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(Pallet::<Test>::unban(RawOrigin::Root.into(), web3_name_00.clone().0));

			// Test that claiming is possible again
			assert_ok!(Pallet::<Test>::claim(
				mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(),
				web3_name_00.clone().0,
			));
		})
}

#[test]
fn unbanning_not_banned() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Pallet::<Test>::unban(RawOrigin::Root.into(), web3_name_00.clone().0),
			Error::<Test>::NotBanned
		);
	})
}

#[test]
fn unbanning_unauthorized_origin() {
	let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
	ExtBuilder::default()
		.with_banned_web3_names(vec![web3_name_00.clone()])
		.build_and_execute_with_sanity_tests(|| {
			// Signer origin
			assert_noop!(
				Pallet::<Test>::unban(RawOrigin::Signed(ACCOUNT_00).into(), web3_name_00.clone().0),
				DispatchError::BadOrigin
			);
			// Owner origin
			assert_noop!(
				Pallet::<Test>::ban(
					mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into(),
					web3_name_00.clone().0
				),
				DispatchError::BadOrigin
			);
		})
}

// #############################################################################
// transfer deposit

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
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());
			assert_eq!(Balances::reserved_balance(ACCOUNT_01), <Test as Config>::Deposit::get());
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
				pallet_balances::Error::<Test>::InsufficientBalance
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
				Balances::reserved_balance(ACCOUNT_00),
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
			assert_eq!(Balances::reserved_balance(ACCOUNT_00), <Test as Config>::Deposit::get());
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
				Balances::reserved_balance(ACCOUNT_00),
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
