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

/// Filter that can be used after the credentials for a given subject have been
/// retrieved from the blockchain state.
pub trait PublicCredentialsFilter<Credential> {
	fn should_include(&self, credential: &Credential) -> bool;
}

#[rpc(client, server)]
pub trait PublicCredentialsApi<BlockHash, OuterSubjectId, OuterCredentialId, OuterCredentialEntry, CredentialFilter> {
	/// Return a credential that matches the provided credential ID, if found.
	#[method(name = "credentials_getCredential")]
	fn get_credential(
		&self,
		credential_id: OuterCredentialId,
		at: Option<BlockHash>,
	) -> RpcResult<Option<OuterCredentialEntry>>;

	/// Return all the credentials issued to the provided subject, optionally
	/// filtering with the provided logic. The result is a vector of (credential
	/// identifier, credential entry).
	#[method(name = "credentials_getCredentials")]
	fn get_credentials(
		&self,
		subject: OuterSubjectId,
		filter: Option<CredentialFilter>,
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
	CredentialFilter,
> {
	client: Arc<Client>,
	#[allow(clippy::type_complexity)]
	_marker: std::marker::PhantomData<(
		Block,
		OuterSubjectId,
		SubjectId,
		OuterCredentialId,
		CredentialId,
		OuterCredentialEntry,
		CredentialEntry,
		CredentialFilter,
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
		CredentialFilter,
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
		CredentialFilter,
	>
{
	pub fn new(client: Arc<Client>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

#[repr(i32)]
pub enum Error {
	Runtime = 1,
	Conversion,
	Internal = i32::MAX,
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
		CredentialFilter,
	>
	PublicCredentialsApiServer<
		<Block as BlockT>::Hash,
		OuterSubjectId,
		OuterCredentialId,
		OuterCredentialEntry,
		CredentialFilter,
	>
	for PublicCredentialsQuery<
		Client,
		Block,
		OuterSubjectId,
		SubjectId,
		OuterCredentialId,
		CredentialId,
		OuterCredentialEntry,
		CredentialEntry,
		CredentialFilter,
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
	CredentialFilter: Send + Sync + 'static + PublicCredentialsFilter<CredentialEntry>,
{
	fn get_credential(
		&self,
		credential_id: OuterCredentialId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Option<OuterCredentialEntry>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		let into_credential_id: CredentialId = credential_id.try_into().map_err(|_| {
			CallError::Custom(ErrorObject::owned(
				Error::Conversion as i32,
				"Unable to convert input to a valid credential ID.",
				Option::<String>::None,
			))
		})?;

		let credential = api.get_credential(&at, into_credential_id).map_err(|_| {
			CallError::Custom(ErrorObject::owned(
				Error::Runtime as i32,
				"Unable to get credential.",
				Option::<String>::None,
			))
		})?;
		Ok(credential.map(OuterCredentialEntry::from))
	}

	fn get_credentials(
		&self,
		subject: OuterSubjectId,
		filter: Option<CredentialFilter>,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Vec<(OuterCredentialId, OuterCredentialEntry)>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		let into_subject: SubjectId = subject.try_into().map_err(|_| {
			CallError::Custom(ErrorObject::owned(
				Error::Conversion as i32,
				"Unable to convert input to a valid subject ID",
				Option::<String>::None,
			))
		})?;

		let credentials = api.get_credentials(&at, into_subject).map_err(|_| {
			CallError::Custom(ErrorObject::owned(
				Error::Runtime as i32,
				"Unable to get credentials",
				Option::<String>::None,
			))
		})?;

		let filtered_credentials = if let Some(filter) = filter {
			credentials
				.into_iter()
				.filter(|(_, credential_entry)| filter.should_include(credential_entry))
				.collect()
		} else {
			credentials
		};

		Ok(filtered_credentials
			.into_iter()
			.map(|(id, entry)| (id.into(), entry.into()))
			.collect())
	}
}
