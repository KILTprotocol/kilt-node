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

use std::sync::Arc;

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

pub use did_rpc_runtime_api::DidApi as DidRuntimeApi;

#[rpc]
pub trait DidApi<BlockHash, DidDoc, AccountId> {
	#[rpc(name = "did_queryByWeb3Name")]
	fn query_did_by_w3n(&self, web3name: String, at: Option<BlockHash>) -> Result<Option<DidDoc>>;

	#[rpc(name = "did_queryByAccount")]
	fn query_did_by_account_id(&self, account: AccountId, at: Option<BlockHash>) -> Result<Option<DidDoc>>;
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

impl<C, Block, DidDoc, AccountId> DidApi<<Block as BlockT>::Hash, DidDoc, AccountId> for DidQuery<C, Block>
where
	AccountId: Codec + std::marker::Send,
	DidDoc: Codec + std::marker::Send,
	Block: BlockT,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: DidRuntimeApi<Block, DidDoc, AccountId>,
{
	fn query_did_by_w3n(&self, web3name: String, at: Option<<Block as BlockT>::Hash>) -> Result<Option<DidDoc>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.query_did_by_w3n(&at, web3name.into()).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to query dispatch info.".into(),
			data: Some(e.to_string().into()),
		})
	}

	fn query_did_by_account_id(
		&self,
		account: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Option<DidDoc>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.query_did_by_account_id(&at, account).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to query fee details.".into(),
			data: Some(e.to_string().into()),
		})
	}
}
