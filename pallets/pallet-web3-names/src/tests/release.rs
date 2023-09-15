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
};

use frame_system::RawOrigin;
use kilt_support::mock::mock_origin;
use sp_runtime::traits::Zero;

use crate::{mock::*, Error, HoldReason, Names, Owner, Pallet};

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
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());
			assert_eq!(Balances::balance(&ACCOUNT_00), initial_balance);
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
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());
			assert_eq!(Balances::balance(&ACCOUNT_00), initial_balance);
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
