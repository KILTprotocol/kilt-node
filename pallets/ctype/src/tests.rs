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

use frame_support::{assert_noop, assert_ok, sp_runtime::traits::Hash};
use frame_system::RawOrigin;
use sp_runtime::DispatchError;

use kilt_support::mock::mock_origin::DoubleOrigin;

use crate::{self as ctype, mock::runtime::*};

// submit_ctype_creation_operation

#[test]
fn check_successful_ctype_creation() {
	let creator = DID_00;
	let deposit_owner = ACCOUNT_00;
	let ctype = [9u8; 256].to_vec();
	let ctype_hash = <Test as frame_system::Config>::Hashing::hash(&ctype[..]);
	let initial_balance = <Test as ctype::Config>::Fee::get() * 2;
	ExtBuilder::default()
		.with_balances(vec![(deposit_owner.clone(), initial_balance)])
		.build()
		.execute_with(|| {
			System::set_block_number(200);
			assert_ok!(Ctype::add(
				DoubleOrigin(deposit_owner.clone(), creator.clone()).into(),
				ctype
			));
			let stored_ctype_creator = Ctype::ctypes(ctype_hash).expect("CType hash should be present on chain.");

			// Verify the CType has the right owner and block number
			assert_eq!(
				stored_ctype_creator,
				ctype::CtypeEntryOf::<Test> {
					creator,
					created_at: 200
				}
			);
			assert_eq!(
				Balances::free_balance(deposit_owner),
				initial_balance.saturating_sub(<Test as ctype::Config>::Fee::get())
			);
		});
}

#[test]
fn insufficient_funds() {
	let creator = DID_00;
	let deposit_owner = ACCOUNT_00;
	let ctype = [9u8; 256].to_vec();

	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Ctype::add(DoubleOrigin(deposit_owner, creator).into(), ctype),
			ctype::Error::<Test>::UnableToPayFees
		);
	});
}

#[test]
fn check_duplicate_ctype_creation() {
	let creator = DID_00;
	let deposit_owner = ACCOUNT_00;
	let ctype = [9u8; 256].to_vec();
	let ctype_hash = <Test as frame_system::Config>::Hashing::hash(&ctype[..]);

	ExtBuilder::default()
		.with_ctypes(vec![(ctype_hash, creator.clone())])
		.with_balances(vec![(deposit_owner.clone(), <Test as ctype::Config>::Fee::get() * 2)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Ctype::add(DoubleOrigin(deposit_owner, creator).into(), ctype),
				ctype::Error::<Test>::AlreadyExists
			);
		});
}

// set_block_number

#[test]
fn set_block_number_ok() {
	let creator = DID_00;
	let ctype = [9u8; 256].to_vec();
	let ctype_hash = <Test as frame_system::Config>::Hashing::hash(&ctype[..]);
	let new_block_number = 500u64;

	ExtBuilder::default()
		.with_ctypes(vec![(ctype_hash, creator)])
		.build()
		.execute_with(|| {
			assert_ok!(Ctype::set_block_number(
				RawOrigin::Signed(ACCOUNT_00).into(),
				ctype_hash,
				new_block_number
			));
			assert_eq!(
				ctype::Ctypes::<Test>::get(ctype_hash)
					.expect("CType with provided hash should exist.")
					.created_at,
				new_block_number
			);
		})
}

#[test]
fn set_block_number_ctype_not_found() {
	let ctype = [9u8; 256].to_vec();
	let ctype_hash = <Test as frame_system::Config>::Hashing::hash(&ctype[..]);

	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Ctype::set_block_number(RawOrigin::Signed(ACCOUNT_00).into(), ctype_hash, 100u64),
			ctype::Error::<Test>::NotFound
		);
	})
}

#[test]
fn set_block_number_bad_origin() {
	let creator = DID_00;
	let ctype = [9u8; 256].to_vec();
	let ctype_hash = <Test as frame_system::Config>::Hashing::hash(&ctype[..]);

	ExtBuilder::default()
		.with_ctypes(vec![(ctype_hash, creator)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Ctype::set_block_number(RawOrigin::Signed(ACCOUNT_01).into(), ctype_hash, 100u64),
				DispatchError::BadOrigin
			);
		})
}
