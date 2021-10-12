// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::BadOrigin;

use crate::{mock::*, Error};

// set_admin_account

#[test]
fn test_set_admin_account() {
	let admin = ACCOUNT_00;
	let new_admin = ACCOUNT_01;

	ExtBuilder::default()
		.with_admin_account(admin.clone())
		.build()
		.execute_with(|| {
			assert_eq!(Crowdloan::admin_account(), admin);

			// Change admin
			assert_ok!(Crowdloan::set_admin_account(
				Origin::signed(admin.clone()),
				new_admin.clone()
			));

			// Test new admin is the one set
			assert_eq!(Crowdloan::admin_account(), new_admin);
		});
}

#[test]
fn test_set_admin_account_with_sudo() {
	let admin = ACCOUNT_00;
	let new_admin = ACCOUNT_01;

	ExtBuilder::default()
		.with_admin_account(admin.clone())
		.build()
		.execute_with(|| {
			assert_eq!(Crowdloan::admin_account(), admin);

			// Change admin with sudo account
			assert_ok!(Crowdloan::set_admin_account(Origin::root(), new_admin.clone()));

			// Test new admin is the one set
			assert_eq!(Crowdloan::admin_account(), new_admin);
		});
}

#[test]
fn test_no_custom_admin_set() {
	let admin = ACCOUNT_00;
	let new_admin = ACCOUNT_01;

	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Crowdloan::admin_account(), admin);

		// Change admin
		assert_ok!(Crowdloan::set_admin_account(
			Origin::signed(admin.clone()),
			new_admin.clone()
		));

		// Test new admin is the one set
		assert_eq!(Crowdloan::admin_account(), new_admin);
	});
}

#[test]
fn test_set_admin_account_bad_origin_error() {
	let admin = ACCOUNT_00;
	let other_admin = ACCOUNT_01;

	ExtBuilder::default()
		.with_admin_account(admin.clone())
		.build()
		.execute_with(|| {
			assert_noop!(
				Crowdloan::set_admin_account(Origin::signed(other_admin), admin),
				BadOrigin
			);
		});
}

// set_new_contribution

#[test]
fn test_set_new_contribution() {
	let admin = ACCOUNT_00;
	let contributor = ACCOUNT_01;
	let contribution = BALANCE_01;

	ExtBuilder::default()
		.with_admin_account(admin.clone())
		.build()
		.execute_with(|| {
			assert!(Crowdloan::contributions(&contributor).is_none());
			assert_ok!(Crowdloan::set_new_contribution(
				Origin::signed(admin.clone()),
				contributor.clone(),
				contribution
			));
			assert_eq!(Crowdloan::contributions(&contributor), Some(contribution));
		});
}

#[test]
fn test_override_contribution() {
	let admin = ACCOUNT_00;
	let contributor = ACCOUNT_01;
	let contribution = BALANCE_01;
	let new_contribution = BALANCE_02;

	ExtBuilder::default()
		.with_admin_account(admin.clone())
		.with_contributions(vec![(contributor.clone(), contribution)])
		.build()
		.execute_with(|| {
			assert_eq!(Crowdloan::contributions(&contributor), Some(contribution));
			assert_ok!(Crowdloan::set_new_contribution(
				Origin::signed(admin.clone()),
				contributor.clone(),
				new_contribution
			));
			assert_eq!(Crowdloan::contributions(&contributor), Some(new_contribution));
		});
}

#[test]
fn test_set_new_contribution_bad_origin_error() {
	let admin = ACCOUNT_00;
	let other_admin = ACCOUNT_01;
	let contributor = ACCOUNT_01;
	let contribution = BALANCE_01;

	ExtBuilder::default()
		.with_admin_account(admin)
		.build()
		.execute_with(|| {
			assert_noop!(
				Crowdloan::set_new_contribution(Origin::signed(other_admin.clone()), contributor, contribution),
				BadOrigin
			);
		});
}

// remove_contribution

#[test]
fn test_remove_contribution() {
	let admin = ACCOUNT_00;
	let contributor = ACCOUNT_01;
	let contribution = BALANCE_01;

	ExtBuilder::default()
		.with_admin_account(admin.clone())
		.with_contributions(vec![(contributor.clone(), contribution)])
		.build()
		.execute_with(|| {
			assert!(Crowdloan::contributions(&contributor).is_some());
			assert_ok!(Crowdloan::remove_contribution(
				Origin::signed(admin.clone()),
				contributor.clone()
			));
			assert!(Crowdloan::contributions(&contributor).is_none());
		});
}

#[test]
fn test_remove_contribution_bad_origin_error() {
	let admin = ACCOUNT_00;
	let other_admin = ACCOUNT_01;
	let contributor = ACCOUNT_01;
	let contribution = BALANCE_01;

	ExtBuilder::default()
		.with_admin_account(admin)
		.with_contributions(vec![(contributor.clone(), contribution)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Crowdloan::remove_contribution(Origin::signed(other_admin.clone()), contributor),
				BadOrigin
			);
		});
}

#[test]
fn test_remove_contribution_absent_error() {
	let admin = ACCOUNT_00;
	let contributor = ACCOUNT_01;

	ExtBuilder::default()
		.with_admin_account(admin.clone())
		.build()
		.execute_with(|| {
			assert_noop!(
				Crowdloan::remove_contribution(Origin::signed(admin), contributor),
				Error::<Test>::ContributorNotPresent
			);
		});
}
