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

use std::{fmt::Display, str::FromStr, sync::Arc};

use codec::{Codec, MaxEncodedLen};
use did_rpc_runtime_api::{DidLinkedInfo, ServiceEndpoint};
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

pub use did_rpc_runtime_api::DidApi as DidRuntimeApi;

pub type RpcDidLinkedInfo<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber> = DidLinkedInfo<
	DidIdentifier,
	AccountId,
	LinkableAccountId,
	String,
	String,
	String,
	String,
	Balance,
	Key,
	BlockNumber,
>;

fn raw_did_endpoint_to_rpc(
	raw: ServiceEndpoint<Vec<u8>, Vec<u8>, Vec<u8>>,
) -> Option<ServiceEndpoint<String, String, String>> {
	Some(ServiceEndpoint {
		id: String::from_utf8(raw.id).ok()?,
		service_types: raw
			.service_types
			.into_iter()
			.filter_map(|st| String::from_utf8(st).ok())
			.collect(),
		urls: raw
			.urls
			.into_iter()
			.filter_map(|url| String::from_utf8(url).ok())
			.collect(),
	})
}

pub type DidRpcResponse<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber> =
	Option<RpcDidLinkedInfo<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>>;

#[rpc(client, server)]
pub trait DidApi<BlockHash, DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>
where
	BlockNumber: MaxEncodedLen,
	Key: Ord,
	Balance: FromStr + Display,
{
	/// Given a web3name this returns:
	/// * the DID
	/// * public keys stored for the did
	/// * the web3name (optional)
	/// * associated accounts
	/// * service endpoints
	#[method(name = "did_queryByWeb3Name")]
	fn query_did_by_w3n(
		&self,
		web3name: String,
		at: Option<BlockHash>,
	) -> RpcResult<DidRpcResponse<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>>;

	/// Given an account this returns:
	/// * the DID
	/// * public keys stored for the did
	/// * the web3name (optional)
	/// * associated accounts
	/// * service endpoints
	#[method(name = "did_queryByAccount")]
	fn query_did_by_account_id(
		&self,
		account: LinkableAccountId,
		at: Option<BlockHash>,
	) -> RpcResult<DidRpcResponse<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>>;

	/// Given a did this returns:
	/// * the DID
	/// * public keys stored for the did
	/// * the web3name (optional)
	/// * associated accounts
	/// * service endpoints
	#[method(name = "did_query")]
	fn query_did(
		&self,
		account: DidIdentifier,
		at: Option<BlockHash>,
	) -> RpcResult<DidRpcResponse<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>>;
}

/// A struct that implements the [`DidRuntimeApi`].
pub struct DidQuery<Client, Block> {
	client: Arc<Client>,
	_marker: std::marker::PhantomData<Block>,
}

impl<C, B> DidQuery<C, B> {
	/// Create new `DidQuery` with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

/// Error type of this RPC api.
pub enum Error {
	/// The transaction was not decodable.
	DecodeError,
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i32 {
	fn from(e: Error) -> i32 {
		match e {
			Error::RuntimeError => 1,
			Error::DecodeError => 2,
		}
	}
}

#[async_trait]
impl<Client, Block, DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>
	DidApiServer<<Block as BlockT>::Hash, DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>
	for DidQuery<Client, Block>
where
	AccountId: Codec + Send + Sync + 'static,
	LinkableAccountId: Codec + Send + Sync + 'static,
	DidIdentifier: Codec + Send + Sync + 'static,
	Key: Codec + Ord,
	Balance: Codec + FromStr + Display,
	BlockNumber: Codec + MaxEncodedLen,
	Block: BlockT,
	Client: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	Client::Api: DidRuntimeApi<Block, DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>,
{
	fn query_did_by_w3n(
		&self,
		web3name: String,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Option<RpcDidLinkedInfo<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not provided, assume the best block.
			self.client.info().best_hash));

		match api.query_did_by_w3n(&at, web3name.into()) {
			Err(e) => Err(CallError::Custom(ErrorObject::owned(
				Error::RuntimeError.into(),
				"Unable to query DID by web3name.",
				Some(format!("{:?}", e)),
			))
			.into()),
			Ok(doc) => Ok(doc.map(|doc| RpcDidLinkedInfo {
				// convert the w3n from a byte array to a string. if it's invalid utf-8 which should never happen, we
				// ignore the w3n and pretend it doesn't exist.
				w3n: doc.w3n.and_then(|w3n| String::from_utf8(w3n).ok()),
				accounts: doc.accounts,
				identifier: doc.identifier,
				service_endpoints: doc
					.service_endpoints
					.into_iter()
					.filter_map(raw_did_endpoint_to_rpc)
					.collect(),
				details: doc.details,
			})),
		}
	}

	fn query_did_by_account_id(
		&self,
		account: LinkableAccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Option<RpcDidLinkedInfo<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		match api.query_did_by_account_id(&at, account) {
			Err(e) => Err(CallError::Custom(ErrorObject::owned(
				Error::RuntimeError.into(),
				"Unable to query account by DID.",
				Some(format!("{:?}", e)),
			))
			.into()),
			Ok(doc) => Ok(doc.map(|doc| RpcDidLinkedInfo {
				// convert the w3n from a byte array to a string. if it's invalid utf-8 which should never happen, we
				// ignore the w3n and pretend it doesn't exist.
				w3n: doc.w3n.and_then(|w3n| String::from_utf8(w3n).ok()),
				accounts: doc.accounts,
				identifier: doc.identifier,
				service_endpoints: doc
					.service_endpoints
					.into_iter()
					.filter_map(raw_did_endpoint_to_rpc)
					.collect(),
				details: doc.details,
			})),
		}
	}

	fn query_did(
		&self,
		did: DidIdentifier,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Option<RpcDidLinkedInfo<DidIdentifier, AccountId, LinkableAccountId, Balance, Key, BlockNumber>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		match api.query_did(&at, did) {
			Err(e) => Err(CallError::Custom(ErrorObject::owned(
				Error::RuntimeError.into(),
				"Unable to query DID details.",
				Some(format!("{:?}", e)),
			))
			.into()),
			Ok(doc) => Ok(doc.map(|doc| RpcDidLinkedInfo {
				// convert the w3n from a byte array to a string. if it's invalid utf-8 which should never happen, we
				// ignore the w3n and pretend it doesn't exist.
				w3n: doc.w3n.and_then(|w3n| String::from_utf8(w3n).ok()),
				accounts: doc.accounts,
				identifier: doc.identifier,
				service_endpoints: doc
					.service_endpoints
					.into_iter()
					.filter_map(raw_did_endpoint_to_rpc)
					.collect(),
				details: doc.details,
			})),
		}
	}
}
