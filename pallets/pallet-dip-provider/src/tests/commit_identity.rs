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

use frame_support::assert_ok;
use kilt_support::mock::mock_origin::DoubleOrigin;

use crate::mock::*;

#[test]
fn commit_identity_multiple_commitments_for_same_subject() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(DipProvider::commit_identity(
			DoubleOrigin(ACCOUNT_ID, DID).into(),
			DID,
			Some(0),
		));
		let expected_identity_commitment = get_expected_commitment_for(&DID, 0);
		assert_eq!(
			DipProvider::identity_commitments(&DID, 0),
			Some(expected_identity_commitment)
		);

		// A second commitment for a different version must be possbile.
		assert_ok!(DipProvider::commit_identity(
			DoubleOrigin(ACCOUNT_ID, DID).into(),
			DID,
			Some(1),
		));
		let expected_identity_commitment = get_expected_commitment_for(&DID, 1);
		assert_eq!(
			crate::pallet::IdentityCommitments::<TestRuntime>::iter_key_prefix(&DID).count(),
			2
		);
		// Right now the commitment is the same as before, but it could be different in
		// the future. This test should catch that.
		assert_eq!(
			DipProvider::identity_commitments(&DID, 1),
			Some(expected_identity_commitment)
		);
	});
}

#[test]
fn commit_identity_override_same_version_commitment() {
	ExtBuilder::default()
		.with_commitments(vec![(DID, 0, u32::MAX)])
		.build()
		.execute_with(|| {
			let expected_identity_commitment = get_expected_commitment_for(&DID, 0);
			assert_ne!(
				DipProvider::identity_commitments(&DID, 0),
				Some(expected_identity_commitment)
			);
			assert_ok!(DipProvider::commit_identity(
				DoubleOrigin(ACCOUNT_ID, DID).into(),
				DID,
				Some(0),
			));
			assert_eq!(
				DipProvider::identity_commitments(&DID, 0),
				Some(expected_identity_commitment)
			);
		});
}
