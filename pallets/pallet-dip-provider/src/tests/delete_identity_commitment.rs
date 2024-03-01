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
use kilt_support::mock::mock_origin::DoubleOrigin;

use crate::mock::*;

#[test]
fn delete_identity_commitment_multiple_versions() {
	ExtBuilder::default()
		.with_commitments(vec![(DID, 0, u32::MAX), (DID, 1, u32::MAX - 1)])
		.build()
		.execute_with(|| {
			assert_ok!(DipProvider::delete_identity_commitment(
				DoubleOrigin(ACCOUNT_ID, DID).into(),
				DID,
				Some(0),
			));
			assert_eq!(
				crate::pallet::IdentityCommitments::<TestRuntime>::iter_key_prefix(&DID).count(),
				1
			);
			assert_ok!(DipProvider::delete_identity_commitment(
				DoubleOrigin(ACCOUNT_ID, DID).into(),
				DID,
				Some(1),
			));
			assert_eq!(
				crate::pallet::IdentityCommitments::<TestRuntime>::iter_key_prefix(&DID).count(),
				0
			);
		});
}

#[test]
fn delete_identity_commitment_not_found() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			DipProvider::delete_identity_commitment(DoubleOrigin(ACCOUNT_ID, DID).into(), DID, Some(0),),
			crate::Error::<TestRuntime>::CommitmentNotFound
		);
	});
}
