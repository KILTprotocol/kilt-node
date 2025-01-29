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

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Codec, Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

mod did_details;
mod service_endpoint;

pub use did_details::*;
pub use service_endpoint::*;

#[derive(Encode, Decode, TypeInfo, Eq, PartialEq)]
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
	BlockNumber,
> {
	pub identifier: DidIdentifier,
	pub accounts: Vec<LinkableAccountId>,
	pub w3n: Option<Web3Name>,
	pub service_endpoints: Vec<ServiceEndpoint<Id, Type, Url>>,
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
	#[api_version(4)]
	pub trait Did<DidIdentifier, AccountId, LinkableAccountId, Balance, Key: Ord, BlockNumber: MaxEncodedLen, LinkedResource, RuntimeCall> where
		DidIdentifier: Codec,
		AccountId: Codec,
		LinkableAccountId: Codec,
		BlockNumber: Codec,
		Key: Codec,
		Balance: Codec,
		LinkedResource: Codec,
		RuntimeCall: Codec,
	{
		/// Given a web3name this returns:
		/// * the DID
		/// * public keys stored for the did
		/// * the web3name (optional)
		/// * associated accounts
		/// * service endpoints
		#[changed_in(2)]
		fn query_by_web3_name(name: Vec<u8>) -> Option<RawDidLinkedInfo<DidIdentifier, AccountId, AccountId, Balance, Key, BlockNumber>>;
		fn query_by_web3_name(name: Vec<u8>) -> Option<RawDidLinkedInfo<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>>;
		/// Allows for batching multiple `query_by_web3_name` requests into one. For each requested name, the corresponding vector entry contains either `Some` or `None` depending on the result of each query.
		#[allow(clippy::type_complexity)]
		fn batch_query_by_web3_name(names: Vec<Vec<u8>>) -> Vec<Option<RawDidLinkedInfo<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>>>;
		/// Given an account address this returns:
		/// * the DID
		/// * public keys stored for the did
		/// * the web3name (optional)
		/// * associated accounts
		/// * service endpoints
		#[changed_in(2)]
		fn query_by_account(account: AccountId) -> Option<RawDidLinkedInfo<DidIdentifier, AccountId, AccountId, Balance, Key, BlockNumber>>;
		fn query_by_account(account: LinkableAccountId) -> Option<RawDidLinkedInfo<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>>;
		/// Allows for batching multiple `query_by_account` requests into one. For each requested name, the corresponding vector entry contains either `Some` or `None` depending on the result of each query.
		#[allow(clippy::type_complexity)]
		fn batch_query_by_account(accounts: Vec<LinkableAccountId>) -> Vec<Option<RawDidLinkedInfo<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>>>;
		/// Given a did this returns:
		/// * the DID
		/// * public keys stored for the did
		/// * the web3name (optional)
		/// * associated accounts
		/// * service endpoints
		#[changed_in(2)]
		fn query(did: DidIdentifier) -> Option<RawDidLinkedInfo<DidIdentifier, AccountId, AccountId, Balance, Key, BlockNumber>>;
		fn query(did: DidIdentifier) -> Option<RawDidLinkedInfo<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>>;
		/// Allows for batching multiple `query` requests into one. For each requested name, the corresponding vector entry contains either `Some` or `None` depending on the result of each query.
		#[allow(clippy::type_complexity)]
		fn batch_query(dids: Vec<DidIdentifier>) -> Vec<Option<RawDidLinkedInfo<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>>>;

		/// Returns the list of linked resources for a given DID that must be deleted before the DID itself can be deleted.
		fn linked_resources(did: DidIdentifier) -> Vec<LinkedResource>;
		/// Returns the list of calls that must be executed to delete the linked resources of a given DID, before deleting the DID itself.
		fn linked_resources_deletion_calls(did: DidIdentifier) -> Vec<RuntimeCall>;
	}
}
