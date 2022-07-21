// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

use kilt_asset_dids::AssetDid as AssetIdentifier;

#[cfg(feature = "runtime-benchmarks")]
pub use benchmarks::*;

/// Thin wrapper around the `AssetDid` type, that implements the required
/// TryFrom<Vec<u8>> trait.
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

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks {
	use super::*;

	use codec::alloc::string::ToString;
	use sp_std::vec::Vec;

	use kilt_asset_dids::{asset, chain};

	impl From<AssetDid> for Vec<u8> {
		fn from(value: AssetDid) -> Self {
			// UTF-8 encode the asset DID (generates the string with the "did:asset:"
			// prefix)
			value.to_string().as_bytes().to_vec()
		}
	}

	impl kilt_support::traits::GetWorstCase for AssetDid {
		fn worst_case() -> Self {
			// Returns the worst case for an AssetDID, which is represented by the longest
			// identifier according to the spec.
			Self::try_from(
				[
					b"did:asset:",
					// Chain part
					&[b'0'; chain::MAXIMUM_NAMESPACE_LENGTH][..],
					b":",
					&[b'1'; chain::MAXIMUM_REFERENCE_LENGTH][..],
					// "." separator
					b".",
					// Asset part
					&[b'2'; asset::MAXIMUM_NAMESPACE_LENGTH][..],
					b":",
					&[b'3'; asset::MAXIMUM_REFERENCE_LENGTH][..],
					b":",
					&[b'4'; asset::MAXIMUM_IDENTIFIER_LENGTH][..],
				]
				.concat(),
			)
			.expect("Worst case creation should not fail.")
		}
	}
}
