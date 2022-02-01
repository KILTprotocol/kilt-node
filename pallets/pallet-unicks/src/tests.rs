// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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
use runtime_common::Balance;
use sp_runtime::{traits::Zero, DispatchError};

use crate::{mock::*, Banned, Error, Owner, Pallet, UnickOwnershipOf, Unicks};

// Unick claiming

#[test]
fn claiming_successful() {
	let unick_00 = get_unick(UNICK_00_INPUT);
	let initial_balance: Balance = 100;
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, initial_balance)])
		.build()
		.execute_with(|| {
			assert!(Unicks::<Test>::get(&DID_00).is_none());
			assert!(Owner::<Test>::get(&unick_00).is_none());
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());

			assert_ok!(Pallet::<Test>::claim(
				mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(),
				unick_00.clone().0,
			));
			let unick = Unicks::<Test>::get(&DID_00).expect("Unick should be stored.");
			let owner_details = Owner::<Test>::get(&unick_00).expect("Owner should be stored.");

			// Test that the unick matches
			assert_eq!(unick, unick_00);
			// Test that the ownership details match
			assert_eq!(
				owner_details,
				UnickOwnershipOf::<Test> {
					owner: DID_00,
					claimed_at: 0,
					deposit: Deposit {
						owner: ACCOUNT_00,
						amount: UnickDeposit::get(),
					},
				}
			);
			// Test that the deposit was reserved correctly.
			assert_eq!(Balances::reserved_balance(ACCOUNT_00), UnickDeposit::get(),);
			assert_eq!(
				Balances::free_balance(ACCOUNT_00),
				initial_balance - UnickDeposit::get(),
			);

			// Test that the same unick cannot be claimed again.
			assert_noop!(
				Pallet::<Test>::claim(mock_origin::DoubleOrigin(ACCOUNT_01, DID_01).into(), unick_00.clone().0,),
				Error::<Test>::UnickAlreadyClaimed
			);

			// Test that the same owner cannot claim a new unick.
			let unick_01 = get_unick(UNICK_01_INPUT);
			assert_noop!(
				Pallet::<Test>::claim(mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(), unick_01.0,),
				Error::<Test>::OwnerAlreadyExists
			);
		})
}

#[test]
fn claiming_invalid() {
	let too_short_unicks = vec![
		// Empty unick
		BoundedVec::try_from(b"".to_vec()).unwrap(),
		// Single-char unick
		BoundedVec::try_from(b"1".to_vec()).unwrap(),
		// Two-letter unick
		BoundedVec::try_from(b"10".to_vec()).unwrap(),
	];

	let invalid_unicks = vec![
		// Not allowed ASCII character unick (invalid symbol)
		BoundedVec::try_from(b"10:1".to_vec()).unwrap(),
		// Not allowed ASCII character unick (uppercase letter)
		BoundedVec::try_from(b"abcdE".to_vec()).unwrap(),
		// Not allowed ASCII character unick (whitespace)
		BoundedVec::try_from(b"    ".to_vec()).unwrap(),
		// Non-ASCII character unick
		BoundedVec::try_from(String::from("notascii😁").as_bytes().to_owned()).unwrap(),
	];
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, 100)])
		.build()
		.execute_with(|| {
			for too_short_input in too_short_unicks.iter() {
				assert_noop!(
					Pallet::<Test>::claim(
						mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(),
						too_short_input.clone(),
					),
					Error::<Test>::UnickTooShort,
				);
			}
			for input in invalid_unicks.iter() {
				assert_noop!(
					Pallet::<Test>::claim(mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(), input.clone(),),
					Error::<Test>::InvalidUnickCharacter,
				);
			}
		})
}

#[test]
fn claiming_banned() {
	let unick_00 = get_unick(UNICK_00_INPUT);
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, 100)])
		.with_banned_unicks(vec![unick_00.clone()])
		.build()
		.execute_with(|| {
			assert_noop!(
				Pallet::<Test>::claim(mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(), unick_00.0),
				Error::<Test>::UnickBanned
			);
		})
}

#[test]
fn claiming_not_enough_funds() {
	let unick_00 = get_unick(UNICK_00_INPUT);
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, UnickDeposit::get() - 1)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Pallet::<Test>::claim(mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(), unick_00.0),
				Error::<Test>::InsufficientFunds
			);
		})
}

// Unick releasing

#[test]
fn releasing_by_owner_successful() {
	let unick_00 = get_unick(UNICK_00_INPUT);
	let initial_balance: Balance = 100;
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, initial_balance)])
		.with_unicks(vec![(DID_00, unick_00.clone(), ACCOUNT_00)])
		.build()
		.execute_with(|| {
			assert_ok!(Pallet::<Test>::release_by_owner(
				// Submitter != deposit payer, owner == unick owner
				mock_origin::DoubleOrigin(ACCOUNT_01, DID_00).into(),
				unick_00.clone().0,
			));
			assert!(Unicks::<Test>::get(&DID_00).is_none());
			assert!(Owner::<Test>::get(&unick_00).is_none());

			// Test that the deposit was returned to the payer correctly.
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());
			assert_eq!(Balances::free_balance(ACCOUNT_00), initial_balance);
		})
}

#[test]
fn releasing_by_payer_successful() {
	let unick_00 = get_unick(UNICK_00_INPUT);
	let initial_balance: Balance = 100;
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, initial_balance)])
		.with_unicks(vec![(DID_00, unick_00.clone(), ACCOUNT_00)])
		.build()
		.execute_with(|| {
			assert_ok!(Pallet::<Test>::release_by_payer(
				// Submitter == deposit payer
				RawOrigin::Signed(ACCOUNT_00).into(),
				unick_00.clone().0,
			));
			assert!(Unicks::<Test>::get(&DID_00).is_none());
			assert!(Owner::<Test>::get(&unick_00).is_none());

			// Test that the deposit was returned to the payer correctly.
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());
			assert_eq!(Balances::free_balance(ACCOUNT_00), initial_balance);
		})
}

#[test]
fn releasing_not_found() {
	let unick_00 = get_unick(UNICK_00_INPUT);
	ExtBuilder::default().build().execute_with(|| {
		// Fail to claim by owner
		assert_noop!(
			Pallet::<Test>::release_by_owner(mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(), unick_00.clone().0),
			Error::<Test>::UnickNotFound
		);
		// Fail to claim by payer
		assert_noop!(
			Pallet::<Test>::release_by_payer(RawOrigin::Signed(ACCOUNT_00).into(), unick_00.clone().0),
			Error::<Test>::UnickNotFound
		);
	})
}

#[test]
fn releasing_not_authorized() {
	let unick_00 = get_unick(UNICK_00_INPUT);
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, 100)])
		.with_unicks(vec![(DID_00, unick_00.clone(), ACCOUNT_00)])
		.build()
		.execute_with(|| {
			// Fail to claim by different owner
			assert_noop!(
				Pallet::<Test>::release_by_owner(
					mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into(),
					unick_00.clone().0
				),
				Error::<Test>::NotAuthorized
			);
			// Fail to claim by different payer
			assert_noop!(
				Pallet::<Test>::release_by_payer(RawOrigin::Signed(ACCOUNT_01).into(), unick_00.clone().0),
				Error::<Test>::NotAuthorized
			);
		})
}

#[test]
fn releasing_banned() {
	let unick_00 = get_unick(UNICK_00_INPUT);
	ExtBuilder::default()
		.with_banned_unicks(vec![(unick_00.clone())])
		.build()
		.execute_with(|| {
			assert_noop!(
				Pallet::<Test>::release_by_owner(
					mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(),
					unick_00.clone().0
				),
				// A banned unick will be removed from the map of used unicks, so it will be considered not
				// existing.
				Error::<Test>::UnickNotFound
			);
		})
}

// Unick banning

#[test]
fn banning_successful() {
	let unick_00 = get_unick(UNICK_00_INPUT);
	let unick_01 = get_unick(UNICK_01_INPUT);
	let initial_balance = 100;
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, initial_balance)])
		.with_unicks(vec![(DID_00, unick_00.clone(), ACCOUNT_00)])
		.build()
		.execute_with(|| {
			// Ban a claimed unick
			assert_ok!(Pallet::<Test>::ban(RawOrigin::Root.into(), unick_00.clone().0),);

			assert!(Unicks::<Test>::get(&DID_00).is_none());
			assert!(Owner::<Test>::get(&unick_00).is_none());
			assert!(Banned::<Test>::get(&unick_00).is_some());

			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());
			assert_eq!(Balances::free_balance(ACCOUNT_00), initial_balance);

			// Ban an unclaimed unick
			assert_ok!(Pallet::<Test>::ban(RawOrigin::Root.into(), unick_01.clone().0),);

			assert!(Owner::<Test>::get(&unick_01).is_none());
			assert!(Banned::<Test>::get(&unick_01).is_some());
		})
}

#[test]
fn banning_already_banned() {
	let unick_00 = get_unick(UNICK_00_INPUT);
	ExtBuilder::default()
		.with_banned_unicks(vec![unick_00.clone()])
		.build()
		.execute_with(|| {
			assert_noop!(
				Pallet::<Test>::ban(RawOrigin::Root.into(), unick_00.clone().0),
				Error::<Test>::UnickAlreadyBanned
			);
		})
}

#[test]
fn banning_unauthorized_origin() {
	let unick_00 = get_unick(UNICK_00_INPUT);
	ExtBuilder::default().build().execute_with(|| {
		// Signer origin
		assert_noop!(
			Pallet::<Test>::ban(RawOrigin::Signed(ACCOUNT_00).into(), unick_00.clone().0),
			DispatchError::BadOrigin
		);
		// Owner origin
		assert_noop!(
			Pallet::<Test>::ban(mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into(), unick_00.clone().0),
			DispatchError::BadOrigin
		);
	})
}

// Unick unbanning

#[test]
fn unbanning_successful() {
	let unick_00 = get_unick(UNICK_00_INPUT);
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, 100)])
		.with_banned_unicks(vec![unick_00.clone()])
		.build()
		.execute_with(|| {
			assert_ok!(Pallet::<Test>::unban(RawOrigin::Root.into(), unick_00.clone().0),);

			// Test that claiming is possible again
			assert_ok!(Pallet::<Test>::claim(
				mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(),
				unick_00.clone().0,
			));
		})
}

#[test]
fn unbanning_not_banned() {
	let unick_00 = get_unick(UNICK_00_INPUT);
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Pallet::<Test>::unban(RawOrigin::Root.into(), unick_00.clone().0),
			Error::<Test>::UnickNotBanned
		);
	})
}

#[test]
fn unbanning_unauthorized_origin() {
	let unick_00 = get_unick(UNICK_00_INPUT);
	ExtBuilder::default()
		.with_banned_unicks(vec![unick_00.clone()])
		.build()
		.execute_with(|| {
			// Signer origin
			assert_noop!(
				Pallet::<Test>::unban(RawOrigin::Signed(ACCOUNT_00).into(), unick_00.clone().0),
				DispatchError::BadOrigin
			);
			// Owner origin
			assert_noop!(
				Pallet::<Test>::ban(mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into(), unick_00.clone().0),
				DispatchError::BadOrigin
			);
		})
}
