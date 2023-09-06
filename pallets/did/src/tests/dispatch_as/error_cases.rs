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

use sp_core::ed25519;

use crate::{
	did_details::DidVerificationKey, mock::*, mock_utils::generate_base_did_details,
	tests::dispatch_as::blueprint_failed_dispatch, Error, Pallet,
};

#[test]
fn no_did() {
	let did_identifier = ACCOUNT_02;

	blueprint_failed_dispatch(
		did_identifier,
		ACCOUNT_00,
		None,
		get_attestation_key_call(),
		|| {},
		Error::<Test>::NotFound,
	);
}

#[test]
fn deleted_did() {
	let did_identifier = ACCOUNT_02;
	let authentication_key = DidVerificationKey::Ed25519(ed25519::Public(*ACCOUNT_00.as_ref()));
	let did_details = generate_base_did_details(authentication_key, Some(ACCOUNT_01.clone()));

	blueprint_failed_dispatch(
		did_identifier.clone(),
		ACCOUNT_00,
		Some(did_details),
		get_attestation_key_call(),
		|| {
			Pallet::<Test>::delete_did(did_identifier, 0).expect("DID should be removable");
		},
		Error::<Test>::NotFound,
	);
}
