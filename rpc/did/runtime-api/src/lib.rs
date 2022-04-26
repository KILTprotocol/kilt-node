// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

#[derive(Encode, Decode, TypeInfo, PartialEq)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct ServiceEndpoint<Id, Type, Url> {
	pub id: Id,
	pub service_types: Vec<Type>,
	pub urls: Vec<Url>,
}

impl<T: did::Config> From<did::service_endpoints::DidEndpoint<T>> for ServiceEndpoint<Vec<u8>, Vec<u8>, Vec<u8>> {
	fn from(runtime_endpoint: did::service_endpoints::DidEndpoint<T>) -> Self {
		ServiceEndpoint {
			id: runtime_endpoint.id.into_inner(),
			service_types: runtime_endpoint
				.service_types
				.into_inner()
				.into_iter()
				.map(|v| v.into_inner())
				.collect(),
			urls: runtime_endpoint
				.urls
				.into_inner()
				.into_iter()
				.map(|v| v.into_inner())
				.collect(),
		}
	}
}

#[derive(Encode, Decode, TypeInfo, PartialEq)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct DidDocument<DidIdentifier, AccountId, Web3Name, Id, Type, Url> {
	pub identifier: DidIdentifier,
	pub accounts: Vec<AccountId>,
	pub w3n: Option<Web3Name>,
	pub service_endpoints: Vec<ServiceEndpoint<Id, Type, Url>>,
}

/// The DidDocument with a Web3Name represented as a byte array.
///
/// This will be returned by the runtime and processed by the client side RPC
/// implementation.
pub type RawDidDocument<DidIdentifier, AccountId> =
	DidDocument<DidIdentifier, AccountId, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>>;

sp_api::decl_runtime_apis! {
	/// The API to query account nonce (aka transaction index).
	pub trait DidApi<DidIdentifier, AccountId> where
		DidIdentifier: Codec,
		AccountId: Codec,
	{
		fn query_did_by_w3n(name: Vec<u8>) -> Option<RawDidDocument<DidIdentifier, AccountId>>;
		fn query_did_by_account_id(account: AccountId) -> Option<RawDidDocument<DidIdentifier, AccountId>>;
		fn query_did(did: DidIdentifier) -> Option<RawDidDocument<DidIdentifier, AccountId>>;
	}
}
