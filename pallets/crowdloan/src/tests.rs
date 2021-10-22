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

// set_registrar_account

#[test]
fn test_set_registrar_account() {
	let registrar = ACCOUNT_00;
	let new_registrar = ACCOUNT_01;

	ExtBuilder::default()
		.with_registrar_account(registrar.clone())
		.build()
		.execute_with(|| {
			assert_eq!(Crowdloan::registrar_account(), registrar);

			// Change registrar
			assert_ok!(Crowdloan::set_registrar_account(
				Origin::signed(registrar.clone()),
				new_registrar.clone()
			));

			// Test new registrar is the one set
			assert_eq!(Crowdloan::registrar_account(), new_registrar);

			// Test that the expected event is properly generated
			let mut crowdloan_events = get_generated_events();
			assert_eq!(crowdloan_events.len(), 1);
			assert_eq!(
				crowdloan_events.pop().unwrap().event,
				Event::Crowdloan(crate::Event::NewRegistrarAccountSet(registrar, new_registrar,),)
			)
		});
}

#[test]
fn test_set_registrar_account_with_allowed_registrar_origin() {
	let registrar = ACCOUNT_00;
	let new_registrar = ACCOUNT_01;

	ExtBuilder::default()
		.with_registrar_account(registrar.clone())
		.build()
		.execute_with(|| {
			assert_eq!(Crowdloan::registrar_account(), registrar);

			// Change registrar with sudo account
			assert_ok!(Crowdloan::set_registrar_account(Origin::root(), new_registrar.clone()));

			// Test new registrar is the one set
			assert_eq!(Crowdloan::registrar_account(), new_registrar);
		});
}

#[test]
fn test_no_custom_registrar_set() {
	let registrar = ACCOUNT_00;
	let new_registrar = ACCOUNT_01;

	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Crowdloan::registrar_account(), registrar);

		// Change registrar
		assert_ok!(Crowdloan::set_registrar_account(
			Origin::signed(registrar.clone()),
			new_registrar.clone()
		));

		// Test new registrar is the one set
		assert_eq!(Crowdloan::registrar_account(), new_registrar);
	});
}

#[test]
fn test_set_registrar_account_bad_origin_error() {
	let registrar = ACCOUNT_00;
	let other_registrar = ACCOUNT_01;

	ExtBuilder::default()
		.with_registrar_account(registrar.clone())
		.build()
		.execute_with(|| {
			assert_noop!(
				Crowdloan::set_registrar_account(Origin::signed(other_registrar), registrar),
				BadOrigin
			);
		});
}

// set_contribution

#[test]
fn test_set_contribution() {
	let registrar = ACCOUNT_00;
	let contributor = ACCOUNT_01;
	let contribution = BALANCE_01;

	ExtBuilder::default()
		.with_registrar_account(registrar.clone())
		.build()
		.execute_with(|| {
			assert!(Crowdloan::contributions(&contributor).is_none());
			assert_ok!(Crowdloan::set_contribution(
				Origin::signed(registrar.clone()),
				contributor.clone(),
				contribution
			));
			assert_eq!(Crowdloan::contributions(&contributor), Some(contribution));

			// Test that the expected event is properly generated
			let mut crowdloan_events = get_generated_events();
			assert_eq!(crowdloan_events.len(), 1);
			assert_eq!(
				crowdloan_events.pop().unwrap().event,
				Event::Crowdloan(crate::Event::ContributionSet(contributor, None, contribution,),)
			)
		});
}

#[test]
fn test_override_contribution() {
	let registrar = ACCOUNT_00;
	let contributor = ACCOUNT_01;
	let contribution = BALANCE_01;
	let new_contribution = BALANCE_02;

	ExtBuilder::default()
		.with_registrar_account(registrar.clone())
		.with_contributions(vec![(contributor.clone(), contribution)])
		.build()
		.execute_with(|| {
			assert_eq!(Crowdloan::contributions(&contributor), Some(contribution));
			assert_ok!(Crowdloan::set_contribution(
				Origin::signed(registrar.clone()),
				contributor.clone(),
				new_contribution
			));
			assert_eq!(Crowdloan::contributions(&contributor), Some(new_contribution));
		});
}

#[test]
fn test_set_contribution_bad_origin_error() {
	let registrar = ACCOUNT_00;
	let other_registrar = ACCOUNT_01;
	let contributor = ACCOUNT_01;
	let contribution = BALANCE_01;

	ExtBuilder::default()
		.with_registrar_account(registrar)
		.build()
		.execute_with(|| {
			assert_noop!(
				Crowdloan::set_contribution(Origin::signed(other_registrar.clone()), contributor, contribution),
				BadOrigin
			);
		});
}

// remove_contribution

#[test]
fn test_remove_contribution() {
	let registrar = ACCOUNT_00;
	let contributor = ACCOUNT_01;
	let contribution = BALANCE_01;

	ExtBuilder::default()
		.with_registrar_account(registrar.clone())
		.with_contributions(vec![(contributor.clone(), contribution)])
		.build()
		.execute_with(|| {
			assert!(Crowdloan::contributions(&contributor).is_some());
			assert_ok!(Crowdloan::remove_contribution(
				Origin::signed(registrar.clone()),
				contributor.clone()
			));
			assert!(Crowdloan::contributions(&contributor).is_none());

			// Test that the expected event is properly generated
			let mut crowdloan_events = get_generated_events();
			assert_eq!(crowdloan_events.len(), 1);
			assert_eq!(
				crowdloan_events.pop().unwrap().event,
				Event::Crowdloan(crate::Event::ContributionRemoved(contributor),)
			)
		});
}

#[test]
fn test_remove_contribution_bad_origin_error() {
	let registrar = ACCOUNT_00;
	let other_registrar = ACCOUNT_01;
	let contributor = ACCOUNT_01;
	let contribution = BALANCE_01;

	ExtBuilder::default()
		.with_registrar_account(registrar)
		.with_contributions(vec![(contributor.clone(), contribution)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Crowdloan::remove_contribution(Origin::signed(other_registrar.clone()), contributor),
				BadOrigin
			);
		});
}

#[test]
fn test_remove_contribution_absent_error() {
	let registrar = ACCOUNT_00;
	let contributor = ACCOUNT_01;

	ExtBuilder::default()
		.with_registrar_account(registrar.clone())
		.build()
		.execute_with(|| {
			assert_noop!(
				Crowdloan::remove_contribution(Origin::signed(registrar), contributor),
				crate::Error::<Test>::ContributorNotPresent
			);
		});
}
