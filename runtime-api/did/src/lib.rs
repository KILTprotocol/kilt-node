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
	BlockNumber: MaxEncodedLen,
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
	#[api_version(2)]
	pub trait Did<DidIdentifier, AccountId, LinkableAccountId, Balance, Key: Ord, BlockNumber> where
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
		#[changed_in(2)]
		fn query_by_web3_name(name: Vec<u8>) -> Option<RawDidLinkedInfo<DidIdentifier, AccountId, AccountId, Balance, Key, BlockNumber>>;
		fn query_by_web3_name(name: Vec<u8>) -> Option<RawDidLinkedInfo<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>>;
		/// Given an account address this returns:
		/// * the DID
		/// * public keys stored for the did
		/// * the web3name (optional)
		/// * associated accounts
		/// * service endpoints
		#[changed_in(2)]
		fn query_by_account(account: AccountId) -> Option<RawDidLinkedInfo<DidIdentifier, AccountId, AccountId, Balance, Key, BlockNumber>>;
		fn query_by_account(account: LinkableAccountId) -> Option<RawDidLinkedInfo<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>>;
		/// Given a did this returns:
		/// * the DID
		/// * public keys stored for the did
		/// * the web3name (optional)
		/// * associated accounts
		/// * service endpoints
		#[changed_in(2)]
		fn query(did: DidIdentifier) -> Option<RawDidLinkedInfo<DidIdentifier, AccountId, AccountId, Balance, Key, BlockNumber>>;
		fn query(did: DidIdentifier) -> Option<RawDidLinkedInfo<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>>;
	}
}
