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

use frame_benchmarking::{account, benchmarks};
use sp_core::ed25519;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::convert::TryInto;
use sp_core::Pair;

use crate::*;
use did_details::*;

const ACCOUNT_SEED: u32 = 0;
const AUTH_KEY_SEED: [u8; 32] = [0u8; 32];

fn set_key_agreement_keys<T: Config>(did_details: &mut DidDetails<T>, n_keys: u32) {
	let new_key_agreement_keys = (1..=n_keys).map(|i| {
		// Converts the loop index to a 32-byte array;
		let mut seed_vec = i.to_be_bytes().to_vec();
		seed_vec.resize(32, 0u8);
		let seed: [u8; 32] = seed_vec.try_into().unwrap();
		DidEncryptionKey::X25519(seed)
	}).collect::<BTreeSet<DidEncryptionKey>>();

	did_details.add_key_agreement_keys(new_key_agreement_keys, BlockNumberOf::<T>::default());
}

fn get_ed25519_authentication_key() -> ed25519::Pair {
	ed25519::Pair::from_seed(&AUTH_KEY_SEED)
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

        let submitter: AccountIdentifierOf<T> = account("tx_submitter", 0, ACCOUNT_SEED);

		let did_auth_key = get_ed25519_authentication_key();
		let mut base_did_details = generate_base_did_details::<T>(DidVerificationKey::from(did_auth_key.public()));

		set_key_agreement_keys(&mut base_did_details, n);
    }: {}
}
