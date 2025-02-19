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

use frame_support::{assert_err, assert_ok};
use sp_runtime::SaturatedConversion;

use crate::web3_names::Web3NameValidationError;

const MIN_LENGTH: u32 = 1;
const MAX_LENGTH: u32 = 32;
type Web3Name = crate::Web3Name<MIN_LENGTH, MAX_LENGTH>;

#[test]
fn valid_web3_name_inputs() {
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
		(b"".to_vec(), Web3NameValidationError::TooShort),
		// One less than minimum length allowed
		(
			vec![b'a'; MIN_LENGTH.saturated_into::<usize>() - 1usize],
			Web3NameValidationError::TooShort,
		),
		// One more than maximum length allowed
		(
			vec![b'a'; MAX_LENGTH.saturated_into::<usize>() + 1usize],
			Web3NameValidationError::TooLong,
		),
		// Invalid ASCII symbol
		(
			b"almostavalidweb3_name!".to_vec(),
			Web3NameValidationError::InvalidCharacter,
		),
		// Non-ASCII character
		(
			String::from("almostavalidweb3_name😂").as_bytes().to_owned(),
			Web3NameValidationError::InvalidCharacter,
		),
	];

	for valid in valid_inputs {
		assert_ok!(Web3Name::try_from(valid));
	}

	for (input, expected_error) in invalid_inputs {
		assert_err!(Web3Name::try_from(input), expected_error);
	}
}
