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

use frame_support::{assert_noop, assert_ok, sp_runtime::traits::Hash};

use crate::{self as ctype, mock::*};

// submit_ctype_creation_operation

#[test]
fn check_successful_ctype_creation() {
	let creator = ALICE;
	let ctype = [9u8; 256].to_vec();
	let ctype_hash = <Test as frame_system::Config>::Hashing::hash(&ctype[..]);

	ExtBuilder::default().build(None).execute_with(|| {
		assert_ok!(Ctype::add(get_origin(creator.clone()), ctype));
		let stored_ctype_creator = Ctype::ctypes(&ctype_hash).expect("CType hash should be present on chain.");

		// Verify the CType has the right owner
		assert_eq!(stored_ctype_creator, creator);
	});
}

#[test]
fn check_duplicate_ctype_creation() {
	let creator = ALICE;
	let ctype = [9u8; 256].to_vec();
	let ctype_hash = <Test as frame_system::Config>::Hashing::hash(&ctype[..]);

	ExtBuilder::default()
		.with_ctypes(vec![(ctype_hash, creator.clone())])
		.build(None)
		.execute_with(|| {
			assert_noop!(
				Ctype::add(get_origin(creator.clone()), ctype),
				ctype::Error::<Test>::CTypeAlreadyExists
			);
		});
}
