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

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use codec::{Codec, Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

mod did_details;
mod service_endpoint;

pub use did_details::*;
pub use service_endpoint::*;

#[derive(Encode, Decode, TypeInfo, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct DidLinkedInfo<
	DidIdentifier,
	AccountId,
	LinkableAccountId,
	Web3Name,
	Id,
	Type,
	Url,
	Balance,
	Key: Ord,
	BlockNumber: MaxEncodedLen,
> {
	pub identifier: DidIdentifier,
	pub accounts: Vec<LinkableAccountId>,
	pub w3n: Option<Web3Name>,
	pub service_endpoints: Vec<ServiceEndpoint<Id, Type, Url>>,
	#[cfg_attr(
		feature = "std",
		serde(bound(
			serialize = "DidDetails<Key, BlockNumber, AccountId, Balance>: Serialize",
			deserialize = "DidDetails<Key, BlockNumber, AccountId, Balance>: Deserialize<'de>"
		))
	)]
	pub details: DidDetails<Key, BlockNumber, AccountId, Balance>,
}

/// The DidLinkedInfo with a Web3Name represented as a byte array.
///
/// This will be returned by the runtime and processed by the client side RPC
/// implementation.
pub type RawDidLinkedInfo<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber> = DidLinkedInfo<
	DidIdentifier,
	AccountId,
	LinkableAccountId,
	Vec<u8>,
	Vec<u8>,
	Vec<u8>,
	Vec<u8>,
	Balance,
	Key,
	BlockNumber,
>;

sp_api::decl_runtime_apis! {
	/// The API to query account nonce (aka transaction index).
	pub trait DidApi<DidIdentifier, AccountId, LinkableAccountId, Balance, Key: Ord, BlockNumber> where
		DidIdentifier: Codec,
		AccountId: Codec,
		LinkableAccountId: Codec,
		BlockNumber: Codec + MaxEncodedLen,
		Key: Codec,
		Balance: Codec,
	{
		/// Given a web3name this returns:
		/// * the DID
		/// * public keys stored for the did
		/// * the web3name (optional)
		/// * associated accounts
		/// * service endpoints
		fn query_did_by_w3n(name: Vec<u8>) -> Option<RawDidLinkedInfo<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>>;
		/// Given an account address this returns:
		/// * the DID
		/// * public keys stored for the did
		/// * the web3name (optional)
		/// * associated accounts
		/// * service endpoints
		fn query_did_by_account_id(account: LinkableAccountId) -> Option<RawDidLinkedInfo<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>>;
		/// Given a did this returns:
		/// * the DID
		/// * public keys stored for the did
		/// * the web3name (optional)
		/// * associated accounts
		/// * service endpoints
		fn query_did(did: DidIdentifier) -> Option<RawDidLinkedInfo<DidIdentifier, AccountId, LinkableAccountId,Balance, Key, BlockNumber>>;
	}
}
