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

use core::borrow::Borrow;

use sp_runtime::DispatchError;
use sp_std::fmt::Debug;
use xcm::v3::MultiAsset;

pub trait IdentityProofGenerator<Identifier, Identity, Output> {
	fn generate_proof(identifier: &Identifier, identity: &Identity) -> Result<Output, DispatchError>;
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum IdentityProofAction<Identifier, Proof> {
	Updated(Identifier, Proof),
	Deleted(Identifier),
}

pub trait IdentityProofDispatcher<Identifier, AccountId, IdentityRoot, Location> {
	fn dispatch(
		action: IdentityProofAction<Identifier, IdentityRoot>,
		dispatcher: AccountId,
		asset: MultiAsset,
		location: Location,
	) -> Result<(), DispatchError>;
}

pub trait IdentityProvider<Identifier, Identity> {
	fn retrieve<I>(identifier: &Identifier) -> Result<Option<I>, DispatchError>
	where
		I: Borrow<Identity>;
}
