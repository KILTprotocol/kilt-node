// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

use frame_support::{
	assert_noop, assert_ok,
	traits::{fungible::InspectHold, Get},
};
use kilt_support::Deposit;
use pallet_dip_provider::{traits::ProviderHooks, IdentityCommitmentOf, IdentityCommitmentVersion};
use parity_scale_codec::Encode;
use sp_runtime::traits::Zero;

use crate::{
	deposit::{
		mock::{DepositCollectorHook, DepositNamespaces, ExtBuilder, TestRuntime, SUBJECT, SUBMITTER},
		DepositEntry, FixedDepositCollectorViaDepositsPalletError,
	},
	DepositKeyOf, HoldReason, Pallet,
};

#[test]
fn on_identity_committed_successful() {
	ExtBuilder::default()
		.with_balances(vec![(SUBMITTER, 100_000)])
		.build_and_execute_with_sanity_tests(|| {
			let namespace = DepositNamespaces::get();
			let key: DepositKeyOf<TestRuntime> = (SUBJECT, SUBMITTER, 0 as IdentityCommitmentVersion)
				.encode()
				.try_into()
				.unwrap();

			assert!(Pallet::<TestRuntime>::deposits(&namespace, &key).is_none(),);
			assert!(
				pallet_balances::Pallet::<TestRuntime>::balance_on_hold(&HoldReason::Deposit.into(), &SUBMITTER)
					.is_zero()
			);

			assert_ok!(
				<DepositCollectorHook::<TestRuntime> as ProviderHooks<TestRuntime>>::on_identity_committed(
					&SUBJECT,
					&SUBMITTER,
					&IdentityCommitmentOf::<TestRuntime>::default(),
					0
				)
			);

			assert_eq!(
				Pallet::<TestRuntime>::deposits(namespace, key),
				Some(DepositEntry {
					reason: HoldReason::Deposit.into(),
					deposit: Deposit {
						amount: 1_000,
						owner: SUBMITTER
					}
				})
			);
			assert_eq!(
				pallet_balances::Pallet::<TestRuntime>::balance_on_hold(&HoldReason::Deposit.into(), &SUBMITTER),
				1_000
			);
		});
}

#[test]
fn on_identity_committed_existing_deposit() {
	let key: DepositKeyOf<TestRuntime> = (SUBJECT, SUBMITTER, 0 as IdentityCommitmentVersion)
		.encode()
		.try_into()
		.unwrap();
	ExtBuilder::default()
		.with_deposits(vec![(
			key,
			DepositEntry {
				reason: HoldReason::Deposit.into(),
				deposit: Deposit {
					amount: 1_000,
					owner: SUBMITTER,
				},
			},
		)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				<DepositCollectorHook::<TestRuntime> as ProviderHooks<TestRuntime>>::on_identity_committed(
					&SUBJECT,
					&SUBMITTER,
					&IdentityCommitmentOf::<TestRuntime>::default(),
					0
				),
				FixedDepositCollectorViaDepositsPalletError::DepositAlreadyTaken
			);
		});
}

#[test]
fn on_identity_committed_insufficient_balance() {
	ExtBuilder::default().build_and_execute_with_sanity_tests(|| {
		assert_noop!(
			<DepositCollectorHook::<TestRuntime> as ProviderHooks<TestRuntime>>::on_identity_committed(
				&SUBJECT,
				&SUBMITTER,
				&IdentityCommitmentOf::<TestRuntime>::default(),
				0
			),
			FixedDepositCollectorViaDepositsPalletError::FailedToHold,
		);
	});
}
