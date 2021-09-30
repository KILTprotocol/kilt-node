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

use codec::{Decode, Encode};
use kilt_primitives::Hash;

use crate::{deprecated::Url, *};
use sp_std::vec::Vec;

#[derive(Clone, Decode, Encode, PartialEq, Eq)]
pub(crate) enum ContentType {
	ApplicationJson,
	ApplicationJsonLd,
}

#[derive(Clone, Decode, Encode, PartialEq)]
pub(crate) struct ServiceEndpoints {
	pub content_hash: Hash,
	pub urls: Vec<Url>,
	pub content_type: ContentType,
}

#[derive(Clone, Decode, Encode, PartialEq)]
pub struct DidDetails<T: Config> {
	pub(crate) authentication_key: KeyIdOf<T>,
	pub(crate) key_agreement_keys: DidKeyAgreementKeySet<T>,
	pub(crate) delegation_key: Option<KeyIdOf<T>>,
	pub(crate) attestation_key: Option<KeyIdOf<T>>,
	pub(crate) public_keys: DidPublicKeyMap<T>,
	pub(crate) service_endpoints: Option<ServiceEndpoints>,
	pub(crate) last_tx_counter: u64,
}

pub(crate) mod storage {
	use frame_support::{decl_module, decl_storage};
	use sp_std::prelude::*;

	use super::*;

	decl_module! {
		pub struct OldPallet<T: Config> for enum Call where origin: <T as pallet::Config>::Origin {}
	}

	decl_storage! {
		trait Store for OldPallet<T: Config> as Did {
			pub(crate) Did get(fn did): map hasher(blake2_128_concat) DidIdentifierOf<T> => Option<super::DidDetails<T>>;
		}
	}
}
