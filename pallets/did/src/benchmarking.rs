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

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use sp_io::crypto::{ed25519_generate, ed25519_verify, ed25519_sign, sr25519_generate, sr25519_verify, sr25519_sign};
use sp_core::ed25519;
use sp_core::crypto::KeyTypeId;
use sp_std::{collections::btree_set::BTreeSet, convert::TryInto};
use codec::Encode;
use frame_system::RawOrigin;

use crate::{Pallet as DelegationPallet, *};
use did_details::*;

const DEFAULT_ACCOUNT_ID: &str = "tx_submitter";
const DEFAULT_ACCOUNT_SEED: u32 = 0;
const AUTHENTICATION_KEY_ID: KeyTypeId = KeyTypeId(*b"0000");
const AUTHENTICATION_KEY_SEED: [u8; 32] = [0u8; 32];
const ATTESTATION_KEY_ID: KeyTypeId = KeyTypeId(*b"0001");
const ATTESTATION_KEY_SEED: [u8; 32] = [1u8; 32];
const DELEGATION_KEY_ID: KeyTypeId = KeyTypeId(*b"0002");
const DELEGATION_KEY_SEED: [u8; 32] = [2u8; 32];

fn get_ed25519_public_authentication_key() -> ed25519::Public {
	ed25519_generate(AUTHENTICATION_KEY_ID, None)
}

fn set_key_agreement_keys<T: Config>(creation_operation: &mut DidCreationOperation<T>, n_keys: u32) {
	let new_key_agreement_keys = (1..=n_keys)
		.map(|i| {
			// Converts the loop index to a 32-byte array;
			let mut seed_vec = i.to_be_bytes().to_vec();
			seed_vec.resize(32, 0u8);
			let seed: [u8; 32] = seed_vec.try_into().unwrap();
			DidEncryptionKey::X25519(seed)
		})
		.collect::<BTreeSet<DidEncryptionKey>>();

		creation_operation.new_key_agreement_keys = new_key_agreement_keys;
}

fn get_ed25519_public_attestation_key() -> ed25519::Public {
	ed25519_generate(ATTESTATION_KEY_ID, None)
}

fn get_ed25519_public_delegation_key() -> ed25519::Public {
	ed25519_generate(DELEGATION_KEY_ID, None)
}

// Assumes that length is greater than 8 (length of https://)
fn get_url_endpoint(length: u32) -> Url {
	let prefix = "https://";
	let remaining_length = length as usize - prefix.len();
	let mut url_string = "https://".bytes().collect::<Vec<u8>>();
	url_string.resize(remaining_length, b'0');
	Url::Http(HttpUrl::try_from((url_string.as_ref(), length)).unwrap())
}

fn generate_base_did_creation_operation<T: Config>(
	did: DidIdentifierOf<T>,
	new_auth_key: DidVerificationKey,
) -> DidCreationOperation<T> {
	DidCreationOperation {
		did,
		new_authentication_key: new_auth_key,
		new_key_agreement_keys: BTreeSet::new(),
		new_attestation_key: None,
		new_delegation_key: None,
		new_endpoint_url: None,
	}
}

fn generate_base_did_details<T: Config>(authentication_key: DidVerificationKey) -> DidDetails<T> {
	DidDetails::new(authentication_key, BlockNumberOf::<T>::default())
}

benchmarks! {
	submit_did_create_operation {
		let n in 1 .. T::MaxNewKeyAgreementKeys::get() - 1;
		let u in 1 .. T::MaxUrlLength::get() - 1;

		let submitter: AccountIdentifierOf<T> = account(DEFAULT_ACCOUNT_ID, 0, DEFAULT_ACCOUNT_SEED);

		let did_public_auth_key = get_ed25519_public_authentication_key();
		let did_public_att_key = get_ed25519_public_attestation_key();
		let did_public_del_key = get_ed25519_public_delegation_key();

		let mut did_creation_op = generate_base_did_creation_operation::<T>(DidIdentifierOf::<T>::default(), DidVerificationKey::from(did_public_auth_key));
		set_key_agreement_keys(&mut did_creation_op, n);
		did_creation_op.new_attestation_key = Some(DidVerificationKey::from(did_public_att_key));
		did_creation_op.new_delegation_key = Some(DidVerificationKey::from(did_public_del_key));
		did_creation_op.new_endpoint_url = Some(get_url_endpoint(u));

		let did_creation_signature = ed25519_sign(AUTHENTICATION_KEY_ID, &did_public_auth_key, did_creation_op.encode().as_ref()).unwrap();
	}: submit_did_create_operation(RawOrigin::Signed(submitter), did_creation_op, DidSignature::from(did_creation_signature))
}

impl_benchmark_test_suite! {
	DelegationPallet,
	crate::mock::ExtBuilder::default().build(None),
	crate::mock::Test
}
