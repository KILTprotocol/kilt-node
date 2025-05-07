// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

#![warn(missing_docs)]

use substrate_frame_rpc_system::AccountNonceApi;

use std::{error::Error, sync::Arc};

use dip_provider_runtime_template::{AccountId, Balance, NodeBlock as Block, Nonce};
use jsonrpsee::RpcModule;
use pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi;
use sc_client_api::AuxStore;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

pub type RpcExtension = RpcModule<()>;

pub struct FullDeps<C, P> {
	pub client: Arc<C>,
	pub pool: Arc<P>,
}

pub fn create_full<C, P>(deps: FullDeps<C, P>) -> Result<RpcExtension, Box<dyn Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>
		+ HeaderBackend<Block>
		+ AuxStore
		+ HeaderMetadata<Block, Error = BlockChainError>
		+ Send
		+ Sync
		+ 'static,
	C::Api:
		TransactionPaymentRuntimeApi<Block, Balance> + AccountNonceApi<Block, AccountId, Nonce> + BlockBuilder<Block>,
	P: TransactionPool + Sync + Send + 'static,
{
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use substrate_frame_rpc_system::{System, SystemApiServer};

	let mut module = RpcExtension::new(());
	let FullDeps { client, pool } = deps;

	module.merge(System::new(Arc::clone(&client), pool).into_rpc())?;
	module.merge(TransactionPayment::new(client).into_rpc())?;
	Ok(module)
}
