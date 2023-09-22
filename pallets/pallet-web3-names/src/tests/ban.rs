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
use sp_runtime::{traits::Zero, DispatchError};

use crate::{mock::*, Banned, Error, HoldReason, Names, Owner, Pallet};

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

			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());
			assert_eq!(Balances::balance(&ACCOUNT_00), initial_balance);

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
