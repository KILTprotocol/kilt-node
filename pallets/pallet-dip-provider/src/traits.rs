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

	/// A trait to retrieve identity information for a given identifier. The
	/// information can come from a variety of different sources, as this pallet
	/// does not impose any restrictions on that.
	pub trait IdentityProvider<Runtime>
	where
		Runtime: Config,
	{
		type Error: Into<u16>;
		type Success;

		/// Return the identity information for the identifier, if found.
		/// Otherwise, return an error.
		fn retrieve(identifier: &Runtime::Identifier) -> Result<Self::Success, Self::Error>;
	}

	/// Return the `Default` value of the provided `Identity` type if it
	/// implements the `Default` trait.
	pub struct DefaultIdentityProvider<Identity>(PhantomData<Identity>);

	impl<Runtime, Identity> IdentityProvider<Runtime> for DefaultIdentityProvider<Identity>
	where
		Runtime: Config,
		Identity: Default,
	{
		type Error = u16;
		type Success = Identity;

		fn retrieve(_identifier: &Runtime::Identifier) -> Result<Self::Success, Self::Error> {
			Ok(Identity::default())
		}
	}
}

pub use identity_generation::*;
pub mod identity_generation {
	use super::*;

	use crate::IdentityOf;

	use parity_scale_codec::{FullCodec, MaxEncodedLen};
	use scale_info::TypeInfo;
	use sp_std::{fmt::Debug, marker::PhantomData};

	/// A trait to generate an identity commitment of a given version for some
	/// identity info retrieved by the [`IdentityProvider`].
	pub trait IdentityCommitmentGenerator<Runtime>
	where
		Runtime: Config,
		Runtime::IdentityProvider: IdentityProvider<Runtime>,
	{
		type Error: Into<u16>;
		type Output: Clone + Eq + Debug + TypeInfo + FullCodec + MaxEncodedLen;

		/// Return the identity commitment for the given version and identity
		/// information.
		fn generate_commitment(
			identifier: &Runtime::Identifier,
			identity: &IdentityOf<Runtime>,
			version: IdentityCommitmentVersion,
		) -> Result<Self::Output, Self::Error>;
	}

	/// Implement the [`IdentityCommitmentGenerator`] trait by returning the
	/// `Default` value for the `Output` type.
	pub struct DefaultIdentityCommitmentGenerator<Output>(PhantomData<Output>);

	impl<Runtime, Output> IdentityCommitmentGenerator<Runtime> for DefaultIdentityCommitmentGenerator<Output>
	where
		Runtime: Config,
		Output: Default + Clone + Eq + Debug + TypeInfo + FullCodec + MaxEncodedLen,
	{
		type Error = u16;
		type Output = Output;

		fn generate_commitment(
			_identifier: &Runtime::Identifier,
			_identity: &IdentityOf<Runtime>,
			_version: IdentityCommitmentVersion,
		) -> Result<Self::Output, Self::Error> {
			Ok(Output::default())
		}
	}
}

/// A trait for types that, among other things, contain information about the
/// submitter of a tx.
pub trait SubmitterInfo {
	type Submitter;

	fn submitter(&self) -> Self::Submitter;
}

impl SubmitterInfo for AccountId32 {
	type Submitter = Self;

	fn submitter(&self) -> Self::Submitter {
		self.clone()
	}
}

impl<DidIdentifier, AccountId> SubmitterInfo for DidRawOrigin<DidIdentifier, AccountId>
where
	AccountId: Clone,
{
	type Submitter = AccountId;

	fn submitter(&self) -> Self::Submitter {
		self.submitter.clone()
	}
}

#[cfg(any(feature = "runtime-benchmarks", test))]
impl<DidIdentifier, AccountId> SubmitterInfo for kilt_support::mock::mock_origin::DoubleOrigin<AccountId, DidIdentifier>
where
	AccountId: Clone,
{
	type Submitter = AccountId;
	fn submitter(&self) -> Self::Submitter {
		self.0.clone()
	}
}

/// Hooks for additional customizable logic to be executed when new identity
/// commitments are stored or old ones are removed.
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

/// Implement the [`ProviderHooks`] trait with noops.
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
