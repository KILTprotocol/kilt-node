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

use frame_support::assert_ok;
use sp_core::{ecdsa, ed25519, sr25519};
use sp_runtime::traits::Hash;

use crate::{
	did_details::DidVerificationKey, mock::*, mock_utils::*, tests::dispatch_as::blueprint_failed_dispatch,
	AccountIdOf, DidIdentifierOf, Error,
};

use super::blueprint_successful_dispatch;

fn blueprint_successful_dispatch_with_key(
	did_identifier: DidIdentifierOf<Test>,
	caller: AccountIdOf<Test>,
	verification_key: DidVerificationKey<AccountIdOf<Test>>,
	deposit_owner: AccountIdOf<Test>,
) {
	let did_details = generate_base_did_details(verification_key, Some(deposit_owner));
	let ctype_hash = <Test as frame_system::Config>::Hashing::hash(&get_authentication_key_test_input()[..]);

	blueprint_successful_dispatch(
		did_identifier,
		caller,
		did_details,
		get_authentication_key_call(),
		|| {
			assert!(
				ctype::Ctypes::<Test>::get(ctype_hash).is_none(),
				"Ctype should not exists before the call"
			);
		},
		|| {
			assert!(
				ctype::Ctypes::<Test>::get(ctype_hash).is_some(),
				"CType must exist after the call"
			);
		},
	);
}

#[test]
fn successful_key_dispatch_ed25519() {
	let deposit_owner = ACCOUNT_01;
	let caller = ACCOUNT_00;
	let did_identifier = ACCOUNT_02;
	let verification_key = DidVerificationKey::Ed25519(ed25519::Public(*ACCOUNT_00.as_ref()));
	blueprint_successful_dispatch_with_key(did_identifier, caller, verification_key, deposit_owner);
}

#[test]
fn successful_key_dispatch_sr25519() {
	let deposit_owner = ACCOUNT_01;
	let caller = ACCOUNT_00;
	let did_identifier = ACCOUNT_02;
	let verification_key = DidVerificationKey::Sr25519(sr25519::Public(*ACCOUNT_00.as_ref()));
	blueprint_successful_dispatch_with_key(did_identifier, caller, verification_key, deposit_owner)
}

#[test]
fn successful_key_dispatch_ecdsa() {
	// these values where generated with `subkey generate -n kilt --scheme ecdsa`
	let verification_key = DidVerificationKey::Ecdsa(ecdsa::Public(hex_literal::hex!(
		"02484c08122e16f2cbce7697b5a9393280ca67dd8b91a907c1bc4b93451ebf4093"
	)));
	let caller: AccountIdOf<Test> =
		hex_literal::hex!("375df6416958de6cb384516d3dead111c3a932c9e658ec1afd776e71bd2303b3").into();

	let deposit_owner = ACCOUNT_01;
	let did_identifier = ACCOUNT_02;
	blueprint_successful_dispatch_with_key(did_identifier, caller, verification_key, deposit_owner)
}

#[test]
fn successful_key_dispatch_account() {
	let deposit_owner = ACCOUNT_01;
	let caller = ACCOUNT_00;
	let did_identifier = ACCOUNT_02;
	let verification_key = DidVerificationKey::Account(ACCOUNT_00);
	blueprint_successful_dispatch_with_key(did_identifier, caller, verification_key, deposit_owner)
}

fn blueprint_failed_dispatch_with_key(
	caller: AccountIdOf<Test>,
	authentication_key: DidVerificationKey<AccountIdOf<Test>>,
	attestation_key: DidVerificationKey<AccountIdOf<Test>>,
	delegation_key: DidVerificationKey<AccountIdOf<Test>>,
) {
	let deposit_owner = ACCOUNT_02;
	let did_identifier = ACCOUNT_02;

	let mut did_details = generate_base_did_details(authentication_key, Some(deposit_owner));
	assert_ok!(did_details.update_attestation_key(attestation_key, 0));
	assert_ok!(did_details.update_delegation_key(delegation_key, 0));

	blueprint_failed_dispatch(
		did_identifier,
		caller,
		Some(did_details),
		get_authentication_key_call(),
		|| {},
		Error::<Test>::InvalidSignature,
	);
}

#[test]
fn failed_no_match_ed25519() {
	let authentication_key = DidVerificationKey::Ed25519(ed25519::Public(*ACCOUNT_01.as_ref()));
	let attestation_key = DidVerificationKey::Ed25519(ed25519::Public(*ACCOUNT_00.as_ref()));
	let delegation_key = DidVerificationKey::Ed25519(ed25519::Public(*ACCOUNT_00.as_ref()));
	blueprint_failed_dispatch_with_key(ACCOUNT_00, authentication_key, attestation_key, delegation_key);
}

#[test]
fn failed_no_match_sr25519() {
	let authentication_key = DidVerificationKey::Sr25519(sr25519::Public(*ACCOUNT_01.as_ref()));
	let attestation_key = DidVerificationKey::Ed25519(ed25519::Public(*ACCOUNT_00.as_ref()));
	let delegation_key = DidVerificationKey::Ed25519(ed25519::Public(*ACCOUNT_00.as_ref()));
	blueprint_failed_dispatch_with_key(ACCOUNT_00, authentication_key, attestation_key, delegation_key);
}

#[test]
fn failed_no_match_ecdsa() {
	let authentication_key = DidVerificationKey::Ecdsa(ecdsa::Public(hex_literal::hex!(
		"02484c08122e16f2cbce7697b5a9393280ca67dd8b91a907c1bc4b93451ebf4093"
	)));
	let attestation_key = DidVerificationKey::Ed25519(ed25519::Public(*ACCOUNT_00.as_ref()));
	let delegation_key = DidVerificationKey::Ed25519(ed25519::Public(*ACCOUNT_00.as_ref()));
	blueprint_failed_dispatch_with_key(ACCOUNT_00, authentication_key, attestation_key, delegation_key);
}

#[test]
fn failed_no_match_account() {
	let authentication_key = DidVerificationKey::Account(ACCOUNT_01);
	let attestation_key = DidVerificationKey::Ed25519(ed25519::Public(*ACCOUNT_00.as_ref()));
	let delegation_key = DidVerificationKey::Ed25519(ed25519::Public(*ACCOUNT_00.as_ref()));
	blueprint_failed_dispatch_with_key(ACCOUNT_00, authentication_key, attestation_key, delegation_key);
}
