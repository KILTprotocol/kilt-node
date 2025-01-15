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

mod mock;

use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;

use crate::did::deletion_hooks::tests::mock::{Did, ExtBuilder, TestRuntime, DID};

#[test]
fn test_delete_with_no_dangling_resources() {
	ExtBuilder::default()
		.with_dids(vec![(DID, None, false)])
		.build()
		.execute_with(|| {
			assert_ok!(Did::delete(RawOrigin::Signed(DID).into(), 0));
		});
}

#[test]
fn test_delete_with_dangling_web3_name() {
	ExtBuilder::default()
		.with_dids(vec![(DID, Some(b"t".to_vec().try_into().unwrap()), false)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Did::delete(RawOrigin::Signed(DID).into(), 0),
				did::Error::<TestRuntime>::CannotDelete
			);
		});
}

#[test]
fn test_delete_with_dangling_linked_account() {
	ExtBuilder::default()
		.with_dids(vec![(DID, None, true)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Did::delete(RawOrigin::Signed(DID).into(), 0),
				did::Error::<TestRuntime>::CannotDelete
			);
		});
}

// If someone tries to re-delete a delete DID with dangling resources, they get
// a `NotFound` error. We are testing that we always check for DID existence
// before we check for linked resources.
#[test]
fn test_delete_with_no_did_and_dangling_web3_name() {
	ExtBuilder::default()
		.with_dangling_dids(vec![(DID, Some(b"t".to_vec().try_into().unwrap()), false)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Did::delete(RawOrigin::Signed(DID).into(), 0),
				did::Error::<TestRuntime>::NotFound
			);
		});
}

// If someone tries to re-delete a delete DID with dangling resources, they get
// a `NotFound` error. We are testing that we always check for DID existence
// before we check for linked resources.
#[test]
fn test_delete_with_no_did_and_dangling_linked_account() {
	ExtBuilder::default()
		.with_dangling_dids(vec![(DID, None, true)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Did::delete(RawOrigin::Signed(DID).into(), 0),
				did::Error::<TestRuntime>::NotFound
			);
		});
}
