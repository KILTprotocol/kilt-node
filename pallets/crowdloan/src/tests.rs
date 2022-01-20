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

use codec::Encode;
use frame_support::{
	assert_noop, assert_ok,
	pallet_prelude::{InvalidTransaction, TransactionLongevity, ValidTransaction},
};
use sp_runtime::{
	traits::{BadOrigin, Zero},
	Permill,
};

use crate::{mock::*, AccountIdOf, GratitudeConfig, ReserveAccounts};

// #############################################################################
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
	let registrar = AccountIdOf::<Test>::default();
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

// #############################################################################
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

// #############################################################################
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

// #############################################################################
// Send Gratitude

#[test]
fn test_send_gratitude_success() {
	let registrar = ACCOUNT_00;
	let contributor = ACCOUNT_01;
	let free_reserve = ACCOUNT_02;
	let vested_reserve = ACCOUNT_03;

	ExtBuilder::default()
		.with_reserve(ReserveAccounts {
			vested: vested_reserve.clone(),
			free: free_reserve.clone(),
		})
		.with_registrar_account(registrar)
		.with_balances(vec![
			(free_reserve.clone(), BALANCE_01),
			(vested_reserve.clone(), BALANCE_01),
		])
		.with_contributions(vec![(contributor.clone(), BALANCE_02)])
		.build()
		.execute_with(|| {
			assert_ok!(Crowdloan::receive_gratitude(Origin::none(), contributor.clone()));
			assert_eq!(
				pallet_balances::Pallet::<Test>::free_balance(contributor.clone()),
				BALANCE_02
			);
			assert!(pallet_balances::Pallet::<Test>::free_balance(free_reserve.clone()).is_zero());
			assert!(pallet_balances::Pallet::<Test>::free_balance(vested_reserve.clone()).is_zero());
			assert!(crate::Contributions::<Test>::get(&contributor).is_none());

			assert_noop!(
				Crowdloan::receive_gratitude(Origin::none(), contributor.clone()),
				crate::Error::<Test>::ContributorNotPresent
			);
		});
}

#[test]
fn test_send_gratitude_empty_free_reserve() {
	let registrar = ACCOUNT_00;
	let contributor = ACCOUNT_01;
	let free_reserve = ACCOUNT_02;
	let vested_reserve = ACCOUNT_03;

	ExtBuilder::default()
		.with_reserve(ReserveAccounts {
			vested: vested_reserve.clone(),
			free: free_reserve,
		})
		.with_registrar_account(registrar)
		.with_balances(vec![(vested_reserve, BALANCE_02)])
		.with_contributions(vec![(contributor.clone(), BALANCE_02)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Crowdloan::receive_gratitude(Origin::none(), contributor),
				crate::Error::<Test>::InsufficientBalance
			);
		});
}

#[test]
fn test_send_gratitude_empty_vest_reserve() {
	let registrar = ACCOUNT_00;
	let contributor = ACCOUNT_01;
	let free_reserve = ACCOUNT_02;
	let vested_reserve = ACCOUNT_03;

	ExtBuilder::default()
		.with_reserve(ReserveAccounts {
			vested: vested_reserve,
			free: free_reserve.clone(),
		})
		.with_registrar_account(registrar)
		.with_balances(vec![(free_reserve, BALANCE_02)])
		.with_contributions(vec![(contributor.clone(), BALANCE_02)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Crowdloan::receive_gratitude(Origin::none(), contributor),
				crate::Error::<Test>::InsufficientBalance
			);
		});
}

#[test]
fn test_send_gratitude_same_account_success() {
	let registrar = ACCOUNT_00;
	let contributor = ACCOUNT_01;
	let free_reserve = ACCOUNT_02;
	let vested_reserve = ACCOUNT_02;

	ExtBuilder::default()
		.with_reserve(ReserveAccounts {
			vested: vested_reserve,
			free: free_reserve.clone(),
		})
		.with_registrar_account(registrar)
		.with_balances(vec![(free_reserve.clone(), BALANCE_02)])
		.with_contributions(vec![(contributor.clone(), BALANCE_02)])
		.build()
		.execute_with(|| {
			assert_ok!(Crowdloan::receive_gratitude(Origin::none(), contributor.clone()));
			assert_eq!(
				pallet_balances::Pallet::<Test>::free_balance(contributor.clone()),
				BALANCE_02
			);
			assert!(pallet_balances::Pallet::<Test>::free_balance(free_reserve.clone()).is_zero());
			assert!(crate::Contributions::<Test>::get(&contributor).is_none());

			assert_noop!(
				Crowdloan::receive_gratitude(Origin::none(), contributor.clone()),
				crate::Error::<Test>::ContributorNotPresent
			);
		});
}

#[test]
fn test_send_gratitude_same_account_out_of_funds() {
	let registrar = ACCOUNT_00;
	let contributor = ACCOUNT_01;
	let free_reserve = ACCOUNT_02;
	let vested_reserve = ACCOUNT_02;

	ExtBuilder::default()
		.with_reserve(ReserveAccounts {
			vested: vested_reserve,
			free: free_reserve.clone(),
		})
		.with_registrar_account(registrar)
		.with_balances(vec![(free_reserve, BALANCE_01)])
		.with_contributions(vec![(contributor.clone(), BALANCE_02)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Crowdloan::receive_gratitude(Origin::none(), contributor),
				crate::Error::<Test>::InsufficientBalance
			);
		});
}

#[test]
fn test_send_gratitude_contribution_not_found() {
	let registrar = ACCOUNT_00;
	let contributor = ACCOUNT_01;
	let free_reserve = ACCOUNT_02;
	let vested_reserve = ACCOUNT_02;

	ExtBuilder::default()
		.with_reserve(ReserveAccounts {
			vested: vested_reserve,
			free: free_reserve.clone(),
		})
		.with_registrar_account(registrar)
		.with_balances(vec![(free_reserve, BALANCE_01)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Crowdloan::receive_gratitude(Origin::none(), contributor),
				crate::Error::<Test>::ContributorNotPresent
			);
		});
}

// #############################################################################
// validate_unsigned

#[test]
fn validate_unsigned_works() {
	use sp_runtime::traits::ValidateUnsigned;
	let source = sp_runtime::transaction_validity::TransactionSource::External;
	let contributor = ACCOUNT_00;
	let free_reserve = ACCOUNT_01;
	let vested_reserve = ACCOUNT_02;
	let contributor2 = ACCOUNT_03;

	ExtBuilder::default()
		.with_contributions(vec![
			(contributor.clone(), BALANCE_02),
			(contributor2.clone(), BALANCE_02 + BALANCE_02),
		])
		.with_balances(vec![
			(free_reserve.clone(), BALANCE_01),
			(vested_reserve.clone(), BALANCE_01),
		])
		.with_reserve(ReserveAccounts {
			vested: vested_reserve,
			free: free_reserve,
		})
		.build()
		.execute_with(|| {
			assert_eq!(
				crate::Pallet::<Test>::validate_unsigned(
					source,
					&crate::Call::receive_gratitude {
						receiver: contributor.clone()
					}
				),
				Ok(ValidTransaction {
					priority: 100,
					requires: vec![],
					provides: vec![("gratitude", contributor.clone()).encode()],
					longevity: TransactionLongevity::max_value(),
					propagate: true,
				})
			);

			assert_eq!(
				crate::Pallet::<Test>::validate_unsigned(
					source,
					&crate::Call::receive_gratitude {
						receiver: ACCOUNT_02.clone()
					}
				),
				Err(InvalidTransaction::Custom(crate::ValidityError::NoContributor as u8).into())
			);

			assert_eq!(
				crate::Pallet::<Test>::validate_unsigned(
					source,
					&crate::Call::receive_gratitude { receiver: contributor2 }
				),
				Err(InvalidTransaction::Custom(crate::ValidityError::CannotSendGratitude as u8).into())
			);

			assert_eq!(
				crate::Pallet::<Test>::validate_unsigned(
					source,
					&crate::Call::remove_contribution {
						contributor: ACCOUNT_02.clone()
					}
				),
				Err(InvalidTransaction::Call.into())
			);
		})
}

// #############################################################################
// Set configuration

#[test]
fn test_set_configuration() {
	let registrar = ACCOUNT_00;
	let contributor = ACCOUNT_01;
	let free_reserve = ACCOUNT_02;
	let vested_reserve = ACCOUNT_02;
	let config = GratitudeConfig {
		vested_share: Permill::from_percent(5),
		start_block: 2,
		vesting_length: 20,
	};

	ExtBuilder::default()
		.with_reserve(ReserveAccounts {
			vested: vested_reserve,
			free: free_reserve,
		})
		.with_registrar_account(registrar.clone())
		.build()
		.execute_with(|| {
			assert!(crate::Configuration::<Test>::get() != config);
			assert_ok!(Crowdloan::set_config(Origin::signed(registrar), config.clone()));
			assert_eq!(crate::Configuration::<Test>::get(), config);
			assert_noop!(Crowdloan::set_config(Origin::signed(contributor), config), BadOrigin);
		});
}

// #############################################################################
// Set reserve

#[test]
fn test_set_reserve() {
	let registrar = ACCOUNT_00;
	let contributor = ACCOUNT_01;
	let free_reserve_old = ACCOUNT_01;
	let vested_reserve_old = ACCOUNT_02;
	let free_reserve_new = ACCOUNT_03;
	let vested_reserve_new = ACCOUNT_04;

	ExtBuilder::default()
		.with_reserve(ReserveAccounts {
			vested: vested_reserve_old.clone(),
			free: free_reserve_old.clone(),
		})
		.with_registrar_account(registrar.clone())
		.build()
		.execute_with(|| {
			assert_ok!(Crowdloan::set_reserve_accounts(
				Origin::signed(registrar.clone()),
				vested_reserve_new.clone(),
				free_reserve_new.clone()
			));
			assert_eq!(
				crate::Reserve::<Test>::get(),
				ReserveAccounts {
					vested: vested_reserve_new,
					free: free_reserve_new,
				}
			);

			assert_noop!(
				Crowdloan::set_reserve_accounts(Origin::signed(contributor), vested_reserve_old, free_reserve_old),
				BadOrigin
			);
		});
}
