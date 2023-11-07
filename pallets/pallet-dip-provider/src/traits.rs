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

use did::DidRawOrigin;
use frame_support::sp_runtime::AccountId32;

use crate::{Config, IdentityCommitmentOf, IdentityCommitmentVersion};

pub use identity_provision::*;
pub mod identity_provision {
	use super::*;

	use sp_std::marker::PhantomData;

	pub trait IdentityProvider<Runtime>
	where
		Runtime: Config,
	{
		type Error: Into<u16>;
		type Identity;

		fn retrieve(identifier: &Runtime::Identifier) -> Result<Option<Self::Identity>, Self::Error>;
	}

	// Return the `Default` value if `Identity` adn `Details` both implement it.
	pub struct DefaultIdentityProvider<Identity>(PhantomData<Identity>);

	impl<Runtime, Identity> IdentityProvider<Runtime> for DefaultIdentityProvider<Identity>
	where
		Runtime: Config,
		Identity: Default,
	{
		type Error = u16;
		type Identity = Identity;

		fn retrieve(_identifier: &Runtime::Identifier) -> Result<Option<Self::Identity>, Self::Error> {
			Ok(Some(Identity::default()))
		}
	}

	// Always return `None`. Might be useful for tests.
	pub struct NoneIdentityProvider;

	impl<Runtime> IdentityProvider<Runtime> for NoneIdentityProvider
	where
		Runtime: Config,
	{
		type Error = u16;
		type Identity = ();

		fn retrieve(_identifier: &Runtime::Identifier) -> Result<Option<Self::Identity>, Self::Error> {
			Ok(None)
		}
	}
}

pub use identity_generation::*;
pub mod identity_generation {
	use super::*;

	use parity_scale_codec::{FullCodec, MaxEncodedLen};
	use scale_info::TypeInfo;
	use sp_std::{fmt::Debug, marker::PhantomData};

	use crate::IdentityOf;

	pub trait IdentityCommitmentGenerator<Runtime>
	where
		Runtime: Config,
	{
		type Error: Into<u16>;
		type IdentityCommitment: Clone + Eq + Debug + TypeInfo + FullCodec + MaxEncodedLen;

		fn generate_commitment(
			identifier: &Runtime::Identifier,
			identity: &IdentityOf<Runtime>,
			version: IdentityCommitmentVersion,
		) -> Result<Self::IdentityCommitment, Self::Error>;
	}

	// Implement the `IdentityCommitmentGenerator` by returning the `Default` value
	// for the `Output` type.
	pub struct DefaultIdentityCommitmentGenerator<Output>(PhantomData<Output>);

	impl<Runtime, Output> IdentityCommitmentGenerator<Runtime> for DefaultIdentityCommitmentGenerator<Output>
	where
		Runtime: Config,
		Runtime::IdentityProvider: IdentityProvider<Runtime>,
		Output: Default + Clone + Eq + Debug + TypeInfo + FullCodec + MaxEncodedLen,
	{
		type Error = u16;
		type IdentityCommitment = Output;

		fn generate_commitment(
			_identifier: &Runtime::Identifier,
			_identity: &IdentityOf<Runtime>,
			_version: IdentityCommitmentVersion,
		) -> Result<Self::IdentityCommitment, Self::Error> {
			Ok(Output::default())
		}
	}
}

pub trait SubmitterInfo<Runtime>
where
	Runtime: Config,
{
	fn submitter(&self) -> Runtime::AccountId;
}

impl<Runtime> SubmitterInfo<Runtime> for AccountId32
where
	Runtime: Config,
	Runtime::AccountId: From<AccountId32>,
{
	fn submitter(&self) -> Runtime::AccountId {
		self.clone().into()
	}
}

impl<Runtime> SubmitterInfo<Runtime> for DidRawOrigin<Runtime::Identifier, Runtime::AccountId>
where
	Runtime: Config,
{
	fn submitter(&self) -> Runtime::AccountId {
		self.submitter.clone()
	}
}

pub trait ProviderHooks<Runtime>
where
	Runtime: Config,
{
	type Error: Into<u16>;

	fn on_identity_committed(
		identifier: &Runtime::Identifier,
		submitter: &Runtime::AccountId,
		commitment: &IdentityCommitmentOf<Runtime>,
		version: IdentityCommitmentVersion,
	) -> Result<(), Self::Error>;

	fn on_commitment_removed(
		identifier: &Runtime::Identifier,
		submitter: &Runtime::AccountId,
		commitment: &IdentityCommitmentOf<Runtime>,
		version: IdentityCommitmentVersion,
	) -> Result<(), Self::Error>;
}

pub struct NoopHooks;

impl<Runtime> ProviderHooks<Runtime> for NoopHooks
where
	Runtime: Config,
{
	type Error = u16;

	fn on_commitment_removed(
		_identifier: &Runtime::Identifier,
		_submitter: &Runtime::AccountId,
		_commitment: &IdentityCommitmentOf<Runtime>,
		_version: IdentityCommitmentVersion,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn on_identity_committed(
		_identifier: &Runtime::Identifier,
		_submitter: &Runtime::AccountId,
		_commitment: &IdentityCommitmentOf<Runtime>,
		_version: IdentityCommitmentVersion,
	) -> Result<(), Self::Error> {
		Ok(())
	}
}
