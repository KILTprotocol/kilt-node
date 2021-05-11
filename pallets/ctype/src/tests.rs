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

use crate::{self as ctype, mock::*};

// submit_ctype_creation_operation

#[test]
fn check_successful_ctype_creation() {
	let creator = ALICE;

	let operation = generate_base_ctype_creation_details();

	let builder = ExtBuilder::default();

	let mut ext = builder.build(None);

	// Write CTYPE on chain
	ext.execute_with(|| {
		assert_ok!(Ctype::add(get_origin(creator.clone()), operation.hash));
	});

	// Verify the CTYPE has the right owner
	let stored_ctype_creator =
		ext.execute_with(|| Ctype::ctypes(&operation.hash).expect("CTYPE hash should be present on chain."));
	assert_eq!(stored_ctype_creator, creator);
}

#[test]
fn check_duplicate_ctype_creation() {
	let creator = ALICE;

	let operation = generate_base_ctype_creation_details();

	let builder = ExtBuilder::default().with_ctypes(vec![(operation.hash, creator.clone())]);

	let mut ext = builder.build(None);

	ext.execute_with(|| {
		assert_noop!(
			Ctype::add(get_origin(creator.clone()), operation.hash),
			ctype::Error::<Test>::CTypeAlreadyExists
		);
	});
}
