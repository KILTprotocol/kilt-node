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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

use kilt_asset_dids::AssetDid as AssetIdentifier;

use kilt_support::traits::ItemFilter;
use public_credentials::CredentialEntry;

use crate::{authorization::AuthorizationId, AccountId, Balance, BlockNumber, Hash};

#[cfg(feature = "runtime-benchmarks")]
#[allow(unused_imports)]
pub use benchmarks::*;

/// Thin wrapper around the `AssetDid` type, that implements the required
/// `TryFrom<Vec<u8>>` trait.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct AssetDid(AssetIdentifier);

impl core::ops::Deref for AssetDid {
	type Target = AssetIdentifier;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl TryFrom<Vec<u8>> for AssetDid {
	type Error = &'static str;

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		let asset = AssetIdentifier::from_utf8_encoded(&value[..])
			.map_err(|_| "Cannot convert provided input to a valid Asset DID.")?;
		Ok(Self(asset))
	}
}

#[cfg(feature = "std")]
impl TryFrom<String> for AssetDid {
	type Error = &'static str;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::try_from(value.into_bytes())
	}
}

/// Filter for public credentials retrieved for a provided subject as specified
/// in the runtime API interface.
#[derive(Encode, Decode, TypeInfo)]
pub enum PublicCredentialsFilter<CTypeHash, Attester> {
	/// Filter credentials that match a specified Ctype.
	CtypeHash(CTypeHash),
	/// Filter credentials that have been issued by the specified attester.
	Attester(Attester),
}

impl ItemFilter<CredentialEntry<Hash, AccountId, BlockNumber, AccountId, Balance, AuthorizationId<Hash>>>
	for PublicCredentialsFilter<Hash, AccountId>
{
	fn should_include(
		&self,
		credential: &CredentialEntry<Hash, AccountId, BlockNumber, AccountId, Balance, AuthorizationId<Hash>>,
	) -> bool {
		match self {
			Self::CtypeHash(ctype_hash) => ctype_hash == &credential.ctype_hash,
			Self::Attester(attester) => attester == &credential.attester,
		}
	}
}

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks {
	use super::*;

	use parity_scale_codec::alloc::string::ToString;
	use sp_std::vec::Vec;

	use kilt_asset_dids::{asset, chain};

	impl From<AssetDid> for Vec<u8> {
		fn from(value: AssetDid) -> Self {
			// UTF-8 encode the asset DID (generates the string with the "did:asset:"
			// prefix)
			value.to_string().as_bytes().to_vec()
		}
	}

	impl<Context> kilt_support::traits::GetWorstCase<Context> for AssetDid {
		type Output = Self;

		fn worst_case(_context: Context) -> Self::Output {
			// Returns the worst case for an AssetDID, which is represented by the longest
			// identifier according to the spec.
			Self::try_from(
				[
					b"did:asset:",
					// Chain part
					&[b'0'; chain::MAXIMUM_CHAIN_NAMESPACE_LENGTH][..],
					b":",
					&[b'1'; chain::MAXIMUM_CHAIN_REFERENCE_LENGTH][..],
					// "." separator
					b".",
					// Asset part
					&[b'2'; asset::MAXIMUM_NAMESPACE_LENGTH][..],
					b":",
					&[b'3'; asset::MAXIMUM_ASSET_REFERENCE_LENGTH][..],
					b":",
					&[b'4'; asset::MAXIMUM_ASSET_IDENTIFIER_LENGTH][..],
				]
				.concat(),
			)
			.expect("Worst case creation should not fail.")
		}
	}
}
