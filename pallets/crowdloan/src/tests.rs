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

use crate::mock::*;

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
			assert_ok!(
				Crowdloan::set_admin_account(Origin::signed(admin.clone()), new_admin.clone())
			);

			// Test new admin is the one set
			assert_eq!(Crowdloan::admin_account(), new_admin);
		});
}

#[test]
fn test_set_admin_account_bad_origin_error() {}

// set_new_contribution

#[test]
fn test_set_new_contribution() {}

#[test]
fn test_override_contribution() {}

#[test]
fn test_set_new_contribution_bad_origin_error() {}

// remove_contribution

#[test]
fn test_remove_contribution() {}

#[test]
fn test_remove_contribution_bad_origin_error() {}

#[test]
fn test_remove_contribution_absent_error() {}
