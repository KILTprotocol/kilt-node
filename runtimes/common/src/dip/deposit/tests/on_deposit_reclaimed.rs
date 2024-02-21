// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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
use kilt_support::Deposit;
use pallet_deposit_storage::{traits::DepositStorageHooks, DepositEntryOf, DepositKeyOf, HoldReason};
use parity_scale_codec::Encode;

use crate::dip::deposit::{
	mock::{DipProvider, ExtBuilder, TestRuntime, SUBJECT, SUBMITTER},
	CommitmentDepositRemovalHookError, DepositKey, DepositNamespace,
};

#[test]
fn on_deposit_reclaimed_successful() {
	ExtBuilder::default()
		.with_commitments(vec![(SUBJECT, 0, SUBMITTER)])
		.build()
		.execute_with(|| {
			let deposit_key: DepositKeyOf<TestRuntime> = DepositKey::DipProvider {
				identifier: SUBJECT,
				version: 0,
			}
			.encode()
			.try_into()
			.unwrap();
			assert_ok!(<<TestRuntime as pallet_deposit_storage::Config>::DepositHooks as DepositStorageHooks<TestRuntime>>::on_deposit_reclaimed(
				&DepositNamespace::DipProvider,
				&deposit_key,
				DepositEntryOf::<TestRuntime> {
					reason: HoldReason::Deposit.into(),
					deposit: Deposit {
						amount: 10_000,
						owner: SUBMITTER
					}
				}
			));

			assert!(DipProvider::identity_commitments(SUBJECT, 0).is_none());
		});
}

#[test]
fn on_deposit_reclaimed_key_decoding_error() {
	ExtBuilder::default()
		.build()
		.execute_with(|| {
			assert_noop!(<<TestRuntime as pallet_deposit_storage::Config>::DepositHooks as DepositStorageHooks<TestRuntime>>::on_deposit_reclaimed(
				&DepositNamespace::DipProvider,
				&DepositKeyOf::<TestRuntime>::default(),
				DepositEntryOf::<TestRuntime> {
					reason: HoldReason::Deposit.into(),
					deposit: Deposit {
						amount: 10_000,
						owner: SUBMITTER
					}
				}
			), CommitmentDepositRemovalHookError::DecodeKey);
		});
}
