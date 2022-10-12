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

//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use jsonrpsee::RpcModule;

pub use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

use public_credentials::CredentialEntry;
use runtime_common::{
	assets::AssetDid, authorization::AuthorizationId, AccountId, Balance, Block, BlockNumber, DidIdentifier, Hash,
	Index,
};

/// Full client dependencies.
pub struct FullDeps<C, P> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P>(deps: FullDeps<C, P>) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C::Api: frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: BlockBuilder<Block>,
	C::Api: public_credentials_rpc::PublicCredentialsRuntimeApi<
		Block,
		AssetDid,
		Hash,
		CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<Hash>>,
	>,
	P: TransactionPool + 'static,
{
	use frame_rpc_system::{System, SystemApiServer};
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use public_credentials_rpc::{PublicCredentialsApiServer, PublicCredentialsQuery};

	let mut module = RpcModule::new(());
	let FullDeps {
		client,
		pool,
		deny_unsafe,
	} = deps;

	module.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client.clone()).into_rpc())?;

	// Extend this RPC with a custom API by using the following syntax.
	// `YourRpcStruct` should have a reference to a client, which is needed
	// to call into the runtime.
	//
	// `module.merge(YourRpcStruct::new(ReferenceToClient).into_rpc())?;`
	module.merge(
		PublicCredentialsQuery::<
			C,
			Block,
			// Input subject ID
			String,
			// Runtime subject ID
			AssetDid,
			// Input/output credential ID
			Hash,
			// Runtime credential ID
			Hash,
			// Input/output credential entry
			node_common::OuterCredentialEntry<
				Hash,
				DidIdentifier,
				BlockNumber,
				AccountId,
				Balance,
				AuthorizationId<Hash>,
			>,
			// Runtime credential entry
			CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<Hash>>,
			// Credential filter
			node_common::PublicCredentialFilter<Hash, DidIdentifier>,
		>::new(client)
		.into_rpc(),
	)?;

	Ok(module)
}
