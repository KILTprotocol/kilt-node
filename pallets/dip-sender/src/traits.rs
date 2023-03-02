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

use sp_runtime::DispatchError;
use xcm::{
	v3::{MultiAsset, MultiLocation},
	DoubleEncoded,
};

use dip_support::latest::IdentityProofAction;

pub trait IdentityProofGenerator<Identifier, Identity, Output> {
	fn generate_proof(identifier: &Identifier, identity: &Identity) -> Result<Output, DispatchError>;
}

pub struct DefaultIdentityProofGenerator;

impl<Identifier, Identity, Output> IdentityProofGenerator<Identifier, Identity, Output>
	for DefaultIdentityProofGenerator
where
	Output: Default,
{
	fn generate_proof(_identifier: &Identifier, _identity: &Identity) -> Result<Output, DispatchError> {
		Ok(Output::default())
	}
}

pub trait IdentityProofDispatcher<Identifier, AccountId, IdentityRoot> {
	type Error;

	fn dispatch<B: TxBuilder<Identifier, IdentityRoot>>(
		action: IdentityProofAction<Identifier, IdentityRoot>,
		dispatcher: AccountId,
		asset: MultiAsset,
		destination: MultiLocation,
	) -> Result<(), Self::Error>;
}

pub struct NullIdentityProofDispatcher;

impl<Identifier, AccountId, IdentityRoot> IdentityProofDispatcher<Identifier, AccountId, IdentityRoot>
	for NullIdentityProofDispatcher
{
	type Error = &'static str;

	fn dispatch<_B>(
		_action: IdentityProofAction<Identifier, IdentityRoot>,
		_dispatcher: AccountId,
		_asset: MultiAsset,
		_destination: MultiLocation,
	) -> Result<(), Self::Error> {
		Ok(())
	}
}

pub trait IdentityProvider<Identifier, Identity> {
	fn retrieve(identifier: &Identifier) -> Result<Option<Identity>, DispatchError>;
}

pub struct DefaultIdentityProvider;

impl<Identifier, Identity> IdentityProvider<Identifier, Identity> for DefaultIdentityProvider
where
	Identity: Default,
{
	fn retrieve(_identifier: &Identifier) -> Result<Option<Identity>, DispatchError> {
		Ok(Some(Identity::default()))
	}
}

pub struct NoneIdentityProvider;

impl<Identifier, Identity> IdentityProvider<Identifier, Identity> for NoneIdentityProvider {
	fn retrieve(_identifier: &Identifier) -> Result<Option<Identity>, DispatchError> {
		Ok(None)
	}
}

pub trait TxBuilder<Identifier, Proof> {
	type Error;

	fn build(
		dest: MultiLocation,
		action: IdentityProofAction<Identifier, Proof>,
	) -> Result<DoubleEncoded<()>, Self::Error>;
}
