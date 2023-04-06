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

// TODO: Crate documentation

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::RuntimeDebug;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

// Export v1 behind a namespace and also as the latest
pub mod v1;
pub mod latest {
	pub use crate::v1::*;
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[non_exhaustive]
pub enum VersionedIdentityProofAction<Identifier, Proof, Details = ()> {
	#[codec(index = 1)]
	V1(v1::IdentityProofAction<Identifier, Proof, Details>),
}

impl<Identifier, Proof, Details> From<v1::IdentityProofAction<Identifier, Proof, Details>>
	for VersionedIdentityProofAction<Identifier, Proof, Details>
{
	fn from(value: v1::IdentityProofAction<Identifier, Proof, Details>) -> Self {
		Self::V1(value)
	}
}

#[derive(Encode, Decode, RuntimeDebug, Clone, Eq, PartialEq, TypeInfo)]
#[non_exhaustive]
pub enum VersionedIdentityProof<BlindedValue, Leaf> {
	#[codec(index = 1)]
	V1(v1::Proof<BlindedValue, Leaf>),
}

impl<BlindedValue, Leaf> From<v1::Proof<BlindedValue, Leaf>> for VersionedIdentityProof<BlindedValue, Leaf> {
	fn from(value: v1::Proof<BlindedValue, Leaf>) -> Self {
		Self::V1(value)
	}
}

impl<BlindedValue, Leaf> TryFrom<VersionedIdentityProof<BlindedValue, Leaf>> for v1::Proof<BlindedValue, Leaf> {
	// Proper error handling
	type Error = ();

	fn try_from(value: VersionedIdentityProof<BlindedValue, Leaf>) -> Result<Self, Self::Error> {
		#[allow(irrefutable_let_patterns)]
		if let VersionedIdentityProof::V1(v1::Proof { blinded, revealed }) = value {
			Ok(Self { blinded, revealed })
		} else {
			Err(())
		}
	}
}
