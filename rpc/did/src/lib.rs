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
use did_rpc_runtime_api::{DidDocument, ServiceEndpoint};
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

pub use did_rpc_runtime_api::DidApi as DidRuntimeApi;

pub type RpcDidDocument<DidIdentifier, AccountId, Balance, Key, BlockNumber> =
	DidDocument<DidIdentifier, AccountId, String, String, String, String, Balance, Key, BlockNumber>;

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

pub type DidRpcResponse<DidIdentifier, AccountId, Balance, Key, BlockNumber> =
	Option<RpcDidDocument<DidIdentifier, AccountId, Balance, Key, BlockNumber>>;

#[rpc]
pub trait DidApi<BlockHash, DidIdentifier, AccountId, Balance, Key, BlockNumber>
where
	BlockNumber: MaxEncodedLen,
	Key: Ord,
	Balance: FromStr + Display,
{
	#[rpc(name = "did_queryByWeb3Name")]
	fn query_did_by_w3n(
		&self,
		web3name: String,
		at: Option<BlockHash>,
	) -> Result<DidRpcResponse<DidIdentifier, AccountId, Balance, Key, BlockNumber>>;

	#[rpc(name = "did_queryByAccount")]
	fn query_did_by_account_id(
		&self,
		account: AccountId,
		at: Option<BlockHash>,
	) -> Result<DidRpcResponse<DidIdentifier, AccountId, Balance, Key, BlockNumber>>;

	#[rpc(name = "did_query")]
	fn query_did(
		&self,
		account: DidIdentifier,
		at: Option<BlockHash>,
	) -> Result<DidRpcResponse<DidIdentifier, AccountId, Balance, Key, BlockNumber>>;
}

/// A struct that implements the [`TransactionPaymentApi`].
pub struct DidQuery<C, P> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<P>,
}

impl<C, P> DidQuery<C, P> {
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

impl From<Error> for i64 {
	fn from(e: Error) -> i64 {
		match e {
			Error::RuntimeError => 1,
			Error::DecodeError => 2,
		}
	}
}

impl<C, Block, DidIdentifier, AccountId, Balance, Key, BlockNumber>
	DidApi<<Block as BlockT>::Hash, DidIdentifier, AccountId, Balance, Key, BlockNumber> for DidQuery<C, Block>
where
	AccountId: Codec + std::marker::Send,
	DidIdentifier: Codec + std::marker::Send,
	Key: Codec + Ord,
	Balance: Codec + FromStr + Display,
	BlockNumber: Codec + MaxEncodedLen,
	Block: BlockT,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: DidRuntimeApi<Block, DidIdentifier, AccountId, Balance, Key, BlockNumber>,
{
	fn query_did_by_w3n(
		&self,
		web3name: String,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Option<RpcDidDocument<DidIdentifier, AccountId, Balance, Key, BlockNumber>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		match api.query_did_by_w3n(&at, web3name.into()) {
			Err(e) => Err(RpcError {
				code: ErrorCode::ServerError(Error::RuntimeError.into()),
				message: "Unable to query dispatch info.".into(),
				data: Some(e.to_string().into()),
			}),
			Ok(doc) => Ok(doc.map(|doc| RpcDidDocument {
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
		account: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Option<RpcDidDocument<DidIdentifier, AccountId, Balance, Key, BlockNumber>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		match api.query_did_by_account_id(&at, account) {
			Err(e) => Err(RpcError {
				code: ErrorCode::ServerError(Error::RuntimeError.into()),
				message: "Unable to query fee details.".into(),
				data: Some(e.to_string().into()),
			}),
			Ok(doc) => Ok(doc.map(|doc| RpcDidDocument {
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
	) -> Result<Option<RpcDidDocument<DidIdentifier, AccountId, Balance, Key, BlockNumber>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		match api.query_did(&at, did) {
			Err(e) => Err(RpcError {
				code: ErrorCode::ServerError(Error::RuntimeError.into()),
				message: "Unable to query fee details.".into(),
				data: Some(e.to_string().into()),
			}),
			Ok(doc) => Ok(doc.map(|doc| RpcDidDocument {
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
