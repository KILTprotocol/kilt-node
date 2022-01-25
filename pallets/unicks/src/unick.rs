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

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{fmt::Debug, marker::PhantomData, str, vec::Vec};

use codec::{Decode, Encode};
use frame_support::{ensure, sp_runtime::SaturatedConversion, traits::Get, BoundedVec};
use scale_info::TypeInfo;

use crate::{Config, Error};

/// A KILT Unick.
///
/// It is bounded in size (inclusive range [MinLength, MaxLength]) and can only
/// contain a subset of ASCII characters.
#[derive(Encode, Decode, TypeInfo)]
#[scale_info(skip_type_params(T, MaxLength, MinLength))]
pub struct AsciiUnick<T, MinLength, MaxLength>(pub(crate) BoundedVec<u8, MaxLength>, PhantomData<(T, MinLength)>);

impl<T: Config> TryFrom<Vec<u8>> for AsciiUnick<T, T::MinUnickLength, T::MaxUnickLength> {
	type Error = Error<T>;

	/// Fallible initialization from a provided byte vector if it is below the
	/// minimum or exceeds the maximum allowed length or contains invalid ASCII
	/// characters.
	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		ensure!(
			value.len() >= T::MinUnickLength::get().saturated_into(),
			Self::Error::UnickTooShort
		);
		let bounded_vec: BoundedVec<u8, T::MaxUnickLength> =
			BoundedVec::try_from(value).map_err(|_| Self::Error::UnickTooLong)?;
		ensure!(
			is_byte_array_ascii_string(&bounded_vec),
			Self::Error::InvalidUnickCharacter
		);
		Ok(Self(bounded_vec, PhantomData))
	}
}

/// Verify that a given slice contains only allowed ASCII characters.
fn is_byte_array_ascii_string(input: &[u8]) -> bool {
	if let Ok(encoded_unick) = str::from_utf8(input) {
		encoded_unick.chars().all(|c| {
			// TODO: Change once we reach a decision on which characters to allow
			// Decision reached: minimum 3 characters, max 20, and the following characters
			// allowed.
			matches!(c, 'a'..='z' | '0'..='9' | '-' | '_')
		})
	} else {
		false
	}
}

impl<T: Config> Debug for AsciiUnick<T, T::MinUnickLength, T::MaxUnickLength> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		f.debug_tuple("AsciiUnick").field(&self.0).finish()
	}
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for AsciiUnick<T, T::MinUnickLength, T::MaxUnickLength> {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for AsciiUnick<T, T::MinUnickLength, T::MaxUnickLength> {
	fn clone(&self) -> Self {
		Self(self.0.clone(), self.1)
	}
}

/// KILT unick ownership details.
#[derive(Clone, Encode, Decode, Debug, PartialEq, TypeInfo)]
pub struct UnickOwnership<Owner, Deposit, BlockNumber> {
	/// The owner of the unick.
	pub owner: Owner,
	/// The block number at which the unick was claimed.
	pub claimed_at: BlockNumber,
	/// The deposit associated with the unick.
	pub deposit: Deposit,
}

#[cfg(test)]
mod tests {
	use sp_runtime::SaturatedConversion;

	use crate::{mock::Test, unick::AsciiUnick, Config};

	const MIN_LENGTH: u32 = <Test as Config>::MinUnickLength::get();
	const MAX_LENGTH: u32 = <Test as Config>::MaxUnickLength::get();

	#[test]
	fn valid_unick_inputs() {
		let valid_inputs = vec![
			// Minimum length allowed
			vec![b'a'; MIN_LENGTH.saturated_into()],
			// Maximum length allowed
			vec![b'a'; MAX_LENGTH.saturated_into()],
			// All ASCII characters allowed
			b"qwertyuiopasdfghjklzxcvbnm".to_vec(),
			b"0123456789".to_vec(),
			b"---".to_vec(),
			b"___".to_vec(),
		];

		let invalid_inputs = vec![
			// Empty string
			b"".to_vec(),
			// One less than minimum length allowed
			vec![b'a'; MIN_LENGTH.saturated_into::<usize>() - 1usize],
			// One more than maximum length allowed
			vec![b'a'; MAX_LENGTH.saturated_into::<usize>() + 1usize],
			// Invalid ASCII symbol
			b"almostavalidunick!".to_vec(),
			// Non-ASCII character
			String::from("almostavalidunick😂").as_bytes().to_owned(),
		];

		for valid in valid_inputs {
			assert!(
				AsciiUnick::<Test, <Test as Config>::MinUnickLength, <Test as Config>::MaxUnickLength>::try_from(valid)
					.is_ok()
			);
		}

		for invalid in invalid_inputs {
			assert!(
				AsciiUnick::<Test, <Test as Config>::MinUnickLength, <Test as Config>::MaxUnickLength>::try_from(
					invalid
				)
				.is_err(),
			);
		}
	}
}
