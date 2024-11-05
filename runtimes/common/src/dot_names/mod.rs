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
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::{ConstU32, RuntimeDebug};
use sp_runtime::{BoundedVec, SaturatedConversion};

use pallet_web3_names::{Config, Error};

#[cfg(test)]
mod tests;

pub enum DotNameValidationError {
	TooShort,
	TooLong,
	InvalidCharacter,
}

impl<T, I> From<DotNameValidationError> for Error<T, I>
where
	T: Config<I>,
	I: 'static,
{
	fn from(value: DotNameValidationError) -> Self {
		match value {
			DotNameValidationError::TooLong => Self::TooLong,
			DotNameValidationError::TooShort => Self::TooShort,
			DotNameValidationError::InvalidCharacter => Self::InvalidCharacter,
		}
	}
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct DotName<const MIN_LENGTH: u32, const MAX_LENGTH: u32>(BoundedVec<u8, ConstU32<MAX_LENGTH>>);

impl<const MIN_LENGTH: u32, const MAX_LENGTH: u32> TryFrom<Vec<u8>> for DotName<MIN_LENGTH, MAX_LENGTH> {
	type Error = DotNameValidationError;

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		ensure!(value.len() >= MIN_LENGTH.saturated_into(), Self::Error::TooShort);
		let bounded_vec: BoundedVec<u8, ConstU32<MAX_LENGTH>> =
			BoundedVec::try_from(value).map_err(|_| Self::Error::TooLong)?;
		ensure!(is_valid_dot_name(&bounded_vec), Self::Error::InvalidCharacter);
		Ok(Self(bounded_vec))
	}
}
fn is_valid_dot_name(input: &[u8]) -> bool {
	let Some(dot_name_without_suffix) = input.strip_suffix(b".dot") else {
		return false;
	};
	// Char validation logic taken from https://github.com/paritytech/polkadot-sdk/blob/657b5503a04e97737696fa7344641019350fb521/substrate/frame/identity/src/lib.rs#L1435
	dot_name_without_suffix
		.iter()
		.all(|c| c.is_ascii_digit() || c.is_ascii_lowercase())
}
