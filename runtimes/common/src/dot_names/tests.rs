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

use crate::dot_names::DotNameValidationError;

// Min is 2 characters + ".dot"
const MIN_LENGTH: u32 = 6;
// Max is 6 characters + ".dot"
const MAX_LENGTH: u32 = 10;
type DotName = crate::DotName<MIN_LENGTH, MAX_LENGTH>;

#[test]
fn valid_names() {
	let valid_names = [
		"a0.dot",
		"0a.dot",
		"az.dot",
		"za.dot",
		"a9.dot",
		"0z.dot",
		"123456.dot",
		"abcdef.dot",
	]
	.map(|w| w.bytes().collect::<Vec<_>>());

	for input in valid_names {
		let Ok(_) = DotName::try_from(input.clone()) else {
			panic!("Failed to create dotname from input string: {:?}", input);
		};
	}
}

#[test]
fn invalid_names() {
	let invalid_names = [
		// Too short names
		("", DotNameValidationError::TooShort),
		(".dot", DotNameValidationError::TooShort),
		("a.dot", DotNameValidationError::TooShort),
		("1.dot", DotNameValidationError::TooShort),
		// Too long names
		("1234567.dot", DotNameValidationError::TooLong),
		("abcdefg.dot", DotNameValidationError::TooLong),
		// Wrong suffixes
		("ntn.dott", DotNameValidationError::InvalidCharacter),
		("ntn..dot", DotNameValidationError::InvalidCharacter),
		("ntn.do", DotNameValidationError::InvalidCharacter),
		("ntn.ot", DotNameValidationError::InvalidCharacter),
		("ntndot", DotNameValidationError::InvalidCharacter),
		// Upper case
		("oN.dot", DotNameValidationError::InvalidCharacter),
		("No.dot", DotNameValidationError::InvalidCharacter),
		// Not ASCII alphanumerical
		("#0.dot", DotNameValidationError::InvalidCharacter),
		("0#.dot", DotNameValidationError::InvalidCharacter),
		// Emoji
		("0😃.dot", DotNameValidationError::InvalidCharacter),
		("😃0.dot", DotNameValidationError::InvalidCharacter),
	]
	.map(|(input, error)| (input.bytes().collect::<Vec<_>>(), error));

	for (input, expected_error) in invalid_names {
		match DotName::try_from(input.clone()) {
			Err(e) if e == expected_error => {}
			other => {
				panic!(
					"Parsing input {:?}. Expected error {:?}. Obtained {:?}",
					String::from_utf8(input).unwrap(),
					expected_error,
					other
				);
			}
		}
	}
}
