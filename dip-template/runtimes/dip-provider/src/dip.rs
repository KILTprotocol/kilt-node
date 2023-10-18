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

use did::{DidRawOrigin, EnsureDidOrigin, KeyIdOf};
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_provider::traits::IdentityProvider;
use parity_scale_codec::{Decode, Encode};
use runtime_common::dip::{
	did::LinkedDidInfoProviderOf,
	merkle::{DidMerkleProofError, DidMerkleRootGenerator},
};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

use crate::{AccountId, DidIdentifier, Hash, Runtime, RuntimeEvent};

#[derive(Encode, Decode, TypeInfo)]
pub struct RuntimeApiDipProofRequest {
	pub(crate) identifier: DidIdentifier,
	pub(crate) keys: Vec<KeyIdOf<Runtime>>,
	pub(crate) accounts: Vec<LinkableAccountId>,
	pub(crate) should_include_web3_name: bool,
}

#[derive(Encode, Decode, TypeInfo)]
pub enum RuntimeApiDipProofError {
	IdentityProviderError(<LinkedDidInfoProviderOf<Runtime> as IdentityProvider<DidIdentifier>>::Error),
	IdentityNotFound,
	MerkleProofError(DidMerkleProofError),
}

impl pallet_dip_provider::Config for Runtime {
	type CommitOriginCheck = EnsureDidOrigin<DidIdentifier, AccountId>;
	type CommitOrigin = DidRawOrigin<DidIdentifier, AccountId>;
	type Identifier = DidIdentifier;
	type IdentityCommitment = Hash;
	type IdentityCommitmentGenerator = DidMerkleRootGenerator<Runtime>;
	type IdentityProvider = LinkedDidInfoProviderOf<Runtime>;
	type RuntimeEvent = RuntimeEvent;
}
