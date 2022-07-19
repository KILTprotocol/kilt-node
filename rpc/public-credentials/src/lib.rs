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

pub use public_credentials_runtime_api::PublicCredentialsApi as PublicCredentialsRuntimeApi;

use std::sync::Arc;

use codec::Codec;
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

#[rpc(client, server)]
pub trait PublicCredentialsApi<BlockHash> {
	#[method(name = "get_credential")]
	fn get_credential(
		&self,
		subject: String,
		credential_id: [u8; 32],
		at: Option<BlockHash>,
	) -> RpcResult<Option<String>>;

	#[method(name = "get_credentials")]
	fn get_credentials(&self, subject: String, at: Option<BlockHash>) -> RpcResult<Vec<String>>;
}

pub struct PublicCredentialsQuery<Client, Block, SubjectId, CredentialId, CredentialEntry> {
	client: Arc<Client>,
	_marker: std::marker::PhantomData<(Block, SubjectId, CredentialId, CredentialEntry)>,
}

impl<Client, Block, SubjectId, CredentialId, CredentialEntry> PublicCredentialsQuery<Client, Block, SubjectId, CredentialId, CredentialEntry> {
	pub fn new(client: Arc<Client>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

pub enum Error {
	Runtime,
	Conversion,
	Internal,
}

impl From<Error> for i32 {
	fn from(e: Error) -> Self {
		match e {
			Error::Runtime => 1,
			Error::Conversion => 2,
			Error::Internal => i32::MAX,
		}
	}
}

#[async_trait]
impl<Client, Block, SubjectId, CredentialId, CredentialEntry>
	PublicCredentialsApiServer<<Block as BlockT>::Hash>
	for PublicCredentialsQuery<Client, Block, SubjectId, CredentialId, CredentialEntry>
where
	Block: BlockT,
	Client: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	Client::Api: PublicCredentialsRuntimeApi<Block, SubjectId, CredentialId, CredentialEntry>,
	SubjectId: Codec + Send + Sync + 'static + TryFrom<String>,
	CredentialId: Codec + Send + Sync + 'static + From<[u8; 32]>,
	CredentialEntry: Codec + Send + Sync + 'static,
{
	fn get_credential(
		&self,
		subject: String,
		credential_id: [u8; 32],
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Option<String>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		let into_subject: SubjectId = subject.try_into().map_err(|_| {
			CallError::Custom(ErrorObject::owned(
				Error::Conversion.into(),
				"Unable to convert input to a valid subject ID.",
				Option::<String>::None,
			))
		})?;

		let into_credential_id: CredentialId = credential_id.into();

		let credential = api.get_credential(&at, into_subject, into_credential_id).map_err(|_| {
			CallError::Custom(ErrorObject::owned(
				Error::Runtime.into(),
				"Unable to get credential.",
				Option::<String>::None,
			))
		})?;
		Ok(Some(String::default()))
	}

	fn get_credentials(
		&self,
		subject: String,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Vec<String>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		let into_subject: SubjectId = subject.try_into().map_err(|_| {
			CallError::Custom(ErrorObject::owned(
				Error::Conversion.into(),
				"Unable to convert input to a valid subject ID",
				Option::<String>::None,
			))
		})?;

		let credentials = api.get_credentials(&at, into_subject).map_err(|_| {
			CallError::Custom(ErrorObject::owned(
				Error::Runtime.into(),
				"Unable to get credentials",
				Option::<String>::None,
			))
		})?;

		Ok(credentials.into_iter().map(|_| String::default()).collect())
	}
}
