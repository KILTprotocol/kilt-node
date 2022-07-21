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
pub trait PublicCredentialsApi<BlockHash, OuterSubjectId, OuterCredentialId, OuterCredentialEntry> {
	/// Return a credential that matches the provided root hash and issued to
	/// the provided subject, if found.
	#[method(name = "get_credential")]
	fn get_credential(
		&self,
		subject: OuterSubjectId,
		credential_id: OuterCredentialId,
		at: Option<BlockHash>,
	) -> RpcResult<Option<OuterCredentialEntry>>;

	/// Return all the credentials issued to the provided subject.
	/// The result is a vector of (credential root hash, credential entry).
	#[method(name = "get_credentials")]
	fn get_credentials(
		&self,
		subject: OuterSubjectId,
		at: Option<BlockHash>,
	) -> RpcResult<Vec<(OuterCredentialId, OuterCredentialEntry)>>;
}

pub struct PublicCredentialsQuery<
	Client,
	Block,
	OuterSubjectId,
	SubjectId,
	OuterCredentialId,
	CredentialId,
	OuterCredentialEntry,
	CredentialEntry,
> {
	client: Arc<Client>,
	_marker: std::marker::PhantomData<(
		Block,
		OuterSubjectId,
		SubjectId,
		OuterCredentialId,
		CredentialId,
		OuterCredentialEntry,
		CredentialEntry,
	)>,
}

impl<
		Client,
		Block,
		OuterSubjectId,
		SubjectId,
		OuterCredentialId,
		CredentialId,
		OuterCredentialEntry,
		CredentialEntry,
	>
	PublicCredentialsQuery<
		Client,
		Block,
		OuterSubjectId,
		SubjectId,
		OuterCredentialId,
		CredentialId,
		OuterCredentialEntry,
		CredentialEntry,
	>
{
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
impl<
		Client,
		Block,
		OuterSubjectId,
		SubjectId,
		OuterCredentialId,
		CredentialId,
		OuterCredentialEntry,
		CredentialEntry,
	> PublicCredentialsApiServer<<Block as BlockT>::Hash, OuterSubjectId, OuterCredentialId, OuterCredentialEntry>
	for PublicCredentialsQuery<
		Client,
		Block,
		OuterSubjectId,
		SubjectId,
		OuterCredentialId,
		CredentialId,
		OuterCredentialEntry,
		CredentialEntry,
	> where
	Block: BlockT,
	Client: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	Client::Api: PublicCredentialsRuntimeApi<Block, SubjectId, CredentialId, CredentialEntry>,
	OuterSubjectId: Send + Sync + 'static,
	SubjectId: Codec + Send + Sync + 'static + TryFrom<OuterSubjectId>,
	OuterCredentialId: Send + Sync + 'static,
	CredentialId: Codec + Send + Sync + 'static + TryFrom<OuterCredentialId> + Into<OuterCredentialId>,
	CredentialEntry: Codec + Send + Sync + 'static,
	OuterCredentialEntry: Send + Sync + 'static + From<CredentialEntry>,
{
	fn get_credential(
		&self,
		subject: OuterSubjectId,
		credential_id: OuterCredentialId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Option<OuterCredentialEntry>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		let into_subject: SubjectId = subject.try_into().map_err(|_| {
			CallError::Custom(ErrorObject::owned(
				Error::Conversion.into(),
				"Unable to convert input to a valid subject ID.",
				Option::<String>::None,
			))
		})?;

		let into_credential_id: CredentialId = credential_id.try_into().map_err(|_| {
			CallError::Custom(ErrorObject::owned(
				Error::Conversion.into(),
				"Unable to convert input to a valid credential ID.",
				Option::<String>::None,
			))
		})?;

		let credential = api.get_credential(&at, into_subject, into_credential_id).map_err(|_| {
			CallError::Custom(ErrorObject::owned(
				Error::Runtime.into(),
				"Unable to get credential.",
				Option::<String>::None,
			))
		})?;
		Ok(credential.map(OuterCredentialEntry::from))
	}

	fn get_credentials(
		&self,
		subject: OuterSubjectId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Vec<(OuterCredentialId, OuterCredentialEntry)>> {
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

		Ok(credentials
			.into_iter()
			.map(|(id, entry)| (id.into(), entry.into()))
			.collect())
	}
}
