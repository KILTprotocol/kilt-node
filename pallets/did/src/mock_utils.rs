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

use crate::*;
use did_details::*;
use frame_support::{storage::bounded_btree_set::BoundedBTreeSet};
use sp_std::{
	collections::btree_set::BTreeSet,
	convert::{TryFrom, TryInto},
};

pub fn get_key_agreement_keys<T: Config>(n_keys: u32) -> DidNewKeyAgreementKeySet<T> {
	BoundedBTreeSet::try_from(
		(1..=n_keys)
			.map(|i| {
				// Converts the loop index to a 32-byte array;
				let mut seed_vec = i.to_be_bytes().to_vec();
				seed_vec.resize(32, 0u8);
				let seed: [u8; 32] = seed_vec
					.try_into()
					.expect("Failed to create encryption key from raw seed.");
				DidEncryptionKey::X25519(seed)
			})
			.collect::<BTreeSet<DidEncryptionKey>>(),
	)
	.expect("Failed to convert key_agreement_keys to BoundedBTreeSet")
}

pub fn generate_base_did_creation_details<T: Config>(did: DidIdentifierOf<T>) -> DidCreationDetails<T> {
	DidCreationDetails {
		did,
		new_key_agreement_keys: BoundedBTreeSet::new(),
		new_attestation_key: None,
		new_delegation_key: None,
	}
}

pub fn generate_base_did_details<T: Config>(authentication_key: DidVerificationKey) -> DidDetails<T> {
	DidDetails::new(authentication_key, BlockNumberOf::<T>::default())
		.expect("Failed to generate new DidDetails from auth_key due to BoundedBTreeSet bound")
}
