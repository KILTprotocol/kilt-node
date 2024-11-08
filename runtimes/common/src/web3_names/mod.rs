// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

use frame_support::ensure;
use pallet_web3_names::{Config, Error};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::{ConstU32, RuntimeDebug};
use sp_runtime::{BoundedVec, SaturatedConversion};
use sp_std::{ops::Deref, vec::Vec};

#[cfg(test)]
mod tests;

#[derive(Debug, Eq, PartialEq)]
pub enum Web3NameValidationError {
	TooShort,
	TooLong,
	InvalidCharacter,
}

impl<T, I> From<Web3NameValidationError> for Error<T, I>
where
	T: Config<I>,
	I: 'static,
{
	fn from(value: Web3NameValidationError) -> Self {
		match value {
			Web3NameValidationError::TooLong => Self::TooLong,
			Web3NameValidationError::TooShort => Self::TooShort,
			Web3NameValidationError::InvalidCharacter => Self::InvalidCharacter,
		}
	}
}

/// A KILT web3 name.
///
/// It is bounded in size (inclusive range [MinLength, MaxLength]) and can only
/// contain a subset of ASCII characters.
#[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq, PartialOrd, Ord, Clone, Default)]
pub struct Web3Name<const MIN_LENGTH: u32, const MAX_LENGTH: u32>(BoundedVec<u8, ConstU32<MAX_LENGTH>>);

impl<const MIN_LENGTH: u32, const MAX_LENGTH: u32> TryFrom<Vec<u8>> for Web3Name<MIN_LENGTH, MAX_LENGTH> {
	type Error = Web3NameValidationError;

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		ensure!(value.len() >= MIN_LENGTH.saturated_into(), Self::Error::TooShort);
		let bounded_vec: BoundedVec<u8, ConstU32<MAX_LENGTH>> =
			BoundedVec::try_from(value).map_err(|_| Self::Error::TooLong)?;
		ensure!(is_valid_web3_name(&bounded_vec), Self::Error::InvalidCharacter);
		Ok(Self(bounded_vec))
	}
}

impl<const MIN_LENGTH: u32, const MAX_LENGTH: u32> Deref for Web3Name<MIN_LENGTH, MAX_LENGTH> {
	type Target = BoundedVec<u8, ConstU32<MAX_LENGTH>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<const MIN_LENGTH: u32, const MAX_LENGTH: u32> From<Web3Name<MIN_LENGTH, MAX_LENGTH>> for Vec<u8> {
	fn from(name: Web3Name<MIN_LENGTH, MAX_LENGTH>) -> Self {
		name.0.into_inner()
	}
}

impl<const MIN_LENGTH: u32, const MAX_LENGTH: u32> AsRef<[u8]> for Web3Name<MIN_LENGTH, MAX_LENGTH> {
	fn as_ref(&self) -> &[u8] {
		self.0.as_ref()
	}
}

/// Verify that a given slice can be used as a web3 name.
fn is_valid_web3_name(input: &[u8]) -> bool {
	input
		.iter()
		.all(|c| matches!(c, b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_'))
}
