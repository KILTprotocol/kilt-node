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
use frame_system::RawOrigin;

use crate::{
	mock::{ExtBuilder, System, TestRuntime, SUBJECT, SUBMITTER},
	Error, IdentityEntries, Pallet,
};

#[test]
fn dispatch_as_successful_no_details() {
	ExtBuilder::default()
		.with_balances(vec![(SUBMITTER, 10_000)])
		.build()
		.execute_with(|| {
			// Needed to test event generation. See <https://substrate.stackexchange.com/a/1105/1795> for more context.
			frame_system::Pallet::<TestRuntime>::set_block_number(1);
			assert!(IdentityEntries::<TestRuntime>::get(SUBJECT).is_none());
			assert_ok!(Pallet::<TestRuntime>::dispatch_as(
				RawOrigin::Signed(SUBMITTER).into(),
				SUBJECT,
				true,
				Box::new(pallet_did_lookup::Call::associate_sender {}.into())
			));
			System::assert_last_event(
				pallet_did_lookup::Event::<TestRuntime>::AssociationEstablished(SUBMITTER.into(), SUBJECT).into(),
			);
			assert_eq!(IdentityEntries::<TestRuntime>::get(SUBJECT), Some(0));
		});
}

#[test]
fn dispatch_as_successful_existing_details() {
	ExtBuilder::default()
		.with_balances(vec![(SUBMITTER, 10_000)])
		.with_identity_details(vec![(SUBJECT, 100)])
		.build()
		.execute_with(|| {
			// Needed to test event generation. See <https://substrate.stackexchange.com/a/1105/1795> for more context.
			frame_system::Pallet::<TestRuntime>::set_block_number(1);
			assert_ok!(Pallet::<TestRuntime>::dispatch_as(
				RawOrigin::Signed(SUBMITTER).into(),
				SUBJECT,
				true,
				Box::new(pallet_did_lookup::Call::associate_sender {}.into())
			));
			System::assert_last_event(
				pallet_did_lookup::Event::<TestRuntime>::AssociationEstablished(SUBMITTER.into(), SUBJECT).into(),
			);
			// Details have been bumped up by the proof verifier, and correctly stored in
			// the storage.
			assert_eq!(IdentityEntries::<TestRuntime>::get(SUBJECT), Some(101));
		});
}

#[test]
fn dispatch_as_filtered() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Pallet::<TestRuntime>::dispatch_as(
				RawOrigin::Signed(SUBMITTER).into(),
				SUBJECT,
				true,
				Box::new(
					frame_system::Call::remark_with_event {
						remark: b"Hello!".to_vec(),
					}
					.into(),
				),
			),
			Error::<TestRuntime>::Filtered
		);
	});
}

#[test]
fn dispatch_as_invalid_proof() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Pallet::<TestRuntime>::dispatch_as(
				RawOrigin::Signed(SUBMITTER).into(),
				SUBJECT,
				false,
				Box::new(
					frame_system::Call::remark {
						remark: b"Hello!".to_vec(),
					}
					.into(),
				),
			),
			Error::<TestRuntime>::InvalidProof(1)
		);
	});
}
