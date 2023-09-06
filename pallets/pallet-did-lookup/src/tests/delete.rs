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
use frame_support::{assert_noop, traits::fungible::InspectHold};
use kilt_support::mock::mock_origin;

use crate::{linkable_account::LinkableAccountId, mock::*, ConnectedAccounts, ConnectedDids, Error, HoldReason};

#[test]
fn test_remove_association_sender() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.with_connections(vec![(ACCOUNT_00, DID_01, LINKABLE_ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			// remove association
			assert!(DidLookup::remove_sender_association(RuntimeOrigin::signed(ACCOUNT_00)).is_ok());
			assert_eq!(ConnectedDids::<Test>::get(LinkableAccountId::from(ACCOUNT_00)), None);
			assert!(ConnectedAccounts::<Test>::get(DID_01, LinkableAccountId::from(ACCOUNT_00)).is_none());
			assert_eq!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00), 0);
		});
}

#[test]
fn test_remove_association_sender_not_found() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				DidLookup::remove_sender_association(RuntimeOrigin::signed(ACCOUNT_00)),
				Error::<Test>::NotFound
			);
		});
}

#[test]
fn test_remove_association_account() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.with_connections(vec![(ACCOUNT_01, DID_01, LINKABLE_ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert!(DidLookup::remove_account_association(
				mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into(),
				LinkableAccountId::from(ACCOUNT_00.clone())
			)
			.is_ok());
			assert_eq!(ConnectedDids::<Test>::get(LinkableAccountId::from(ACCOUNT_00)), None);
			assert!(ConnectedAccounts::<Test>::get(DID_01, LinkableAccountId::from(ACCOUNT_00)).is_none());
			assert_eq!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01), 0);
		});
}

#[test]
fn test_remove_association_account_not_found() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(ConnectedDids::<Test>::get(LinkableAccountId::from(ACCOUNT_00)), None);

			assert_noop!(
				DidLookup::remove_account_association(
					mock_origin::DoubleOrigin(ACCOUNT_01, DID_01).into(),
					LinkableAccountId::from(ACCOUNT_00)
				),
				Error::<Test>::NotFound
			);
		});
}

#[test]
fn test_remove_association_account_not_authorized() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.with_connections(vec![(ACCOUNT_01, DID_01, LINKABLE_ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				DidLookup::remove_account_association(
					mock_origin::DoubleOrigin(ACCOUNT_01, DID_00).into(),
					ACCOUNT_00.into()
				),
				Error::<Test>::NotAuthorized
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01),
				<Test as crate::Config>::Deposit::get()
			);
		});
}
