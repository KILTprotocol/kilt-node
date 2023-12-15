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

use did::KeyIdOf;
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_provider::IdentityCommitmentVersion;
use parity_scale_codec::{Decode, Encode};
use runtime_common::{
	dip::{did::LinkedDidInfoProviderError, merkle::DidMerkleProofError},
	DidIdentifier,
};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

use crate::Runtime;

/// Parameters for a DIP proof request.
#[derive(Encode, Decode, TypeInfo)]
pub struct DipProofRequest {
	/// The subject identifier for which to generate the DIP proof.
	pub(crate) identifier: DidIdentifier,
	/// The DIP version.
	pub(crate) version: IdentityCommitmentVersion,
	/// The DID key IDs of the subject's DID Document to reveal in the DIP
	/// proof.
	pub(crate) keys: Vec<KeyIdOf<Runtime>>,
	/// The list of accounts linked to the subject's DID to reveal in the
	/// DIP proof.
	pub(crate) accounts: Vec<LinkableAccountId>,
	/// A flag indicating whether the web3name claimed by the DID subject
	/// should revealed in the DIP proof.
	pub(crate) should_include_web3_name: bool,
}

#[derive(Encode, Decode, TypeInfo)]
pub enum DipProofError {
	IdentityProvider(LinkedDidInfoProviderError),
	MerkleProof(DidMerkleProofError),
}
