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

pub use identity_generation::*;
pub mod identity_generation {
	use sp_std::marker::PhantomData;

	use crate::IdentityCommitmentVersion;

	pub trait IdentityCommitmentGenerator<Identifier, Identity> {
		type Error;
		type Output;

		fn generate_commitment(
			identifier: &Identifier,
			identity: &Identity,
			version: IdentityCommitmentVersion,
		) -> Result<Self::Output, Self::Error>;
	}

	// Implement the `IdentityCommitmentGenerator` by returning the `Default` value
	// for the `Output` type.
	pub struct DefaultIdentityCommitmentGenerator<Output>(PhantomData<Output>);

	impl<Identifier, Identity, Output> IdentityCommitmentGenerator<Identifier, Identity>
		for DefaultIdentityCommitmentGenerator<Output>
	where
		Output: Default,
	{
		type Error = ();
		type Output = Output;

		fn generate_commitment(
			_identifier: &Identifier,
			_identity: &Identity,
			_version: IdentityCommitmentVersion,
		) -> Result<Self::Output, Self::Error> {
			Ok(Output::default())
		}
	}
}

pub use identity_provision::*;
pub mod identity_provision {
	use sp_std::marker::PhantomData;

	pub trait IdentityProvider<Identifier> {
		type Error;
		type Success;

		fn retrieve(identifier: &Identifier) -> Result<Option<Self::Success>, Self::Error>;
	}

	// Return the `Default` value if `Identity` adn `Details` both implement it.
	pub struct DefaultIdentityProvider<Identity>(PhantomData<Identity>);

	impl<Identifier, Identity> IdentityProvider<Identifier> for DefaultIdentityProvider<Identity>
	where
		Identity: Default,
	{
		type Error = ();
		type Success = Identity;

		fn retrieve(_identifier: &Identifier) -> Result<Option<Self::Success>, Self::Error> {
			Ok(Some(Identity::default()))
		}
	}

	// Always return `None`. Might be useful for tests.
	pub struct NoneIdentityProvider;

	impl<Identifier> IdentityProvider<Identifier> for NoneIdentityProvider {
		type Error = ();
		type Success = ();

		fn retrieve(_identifier: &Identifier) -> Result<Option<Self::Success>, Self::Error> {
			Ok(None)
		}
	}
}

pub trait SubmitterInfo {
	type Submitter;

	fn submitter(&self) -> Self::Submitter;
}

impl SubmitterInfo for frame_support::sp_runtime::AccountId32 {
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
