// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.org>

// The `RuntimeDebug` macro uses these internally.
#![allow(clippy::ref_patterns)]

use pallet_dip_provider::IdentityCommitmentVersion;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;

use crate::DidIdentifier;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum DepositNamespace {
	DipProvider,
	BondedTokens,
}

#[cfg(feature = "runtime-benchmarks")]
impl Default for DepositNamespace {
	fn default() -> Self {
		Self::DipProvider
	}
}

/// The various different keys that can be stored in the storage-tracking
/// pallet.
/// Although the namespace is used to distinguish between keys, it is useful to
/// group all keys under the same enum to calculate the maximum length that a
/// key can take.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum DepositKey {
	DipProvider {
		identifier: DidIdentifier,
		version: IdentityCommitmentVersion,
	},
}
