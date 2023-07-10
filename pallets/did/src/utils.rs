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

use parity_scale_codec::Encode;
use sp_runtime::traits::Hash;
use sp_std::vec::Vec;

use crate::{did_details::DidPublicKey, Config, KeyIdOf};

pub fn calculate_key_id<T: Config>(key: &DidPublicKey) -> KeyIdOf<T> {
	let hashed_values: Vec<u8> = key.encode();
	T::Hashing::hash(&hashed_values)
}

/// Verifies that an input string contains only traditional (non-extended) ASCII
/// characters.
pub(crate) fn is_valid_ascii_string(input: &str) -> bool {
	input.chars().all(|c| c.is_ascii())
}

/// Verifies that an input string contains only a subset of characters allowed
/// for a URI fragment according to W3C RFC3986. The subset is composed of all
/// the elements that can form a "fragment" component, minus the 'pct-encoded'
/// sequences that make the 'pchar' component.
pub(crate) fn is_valid_uri_fragment(input: &str) -> bool {
	input.chars().all(|c| {
		matches!(
			c,
			// ALPHA
			'a'..='z' |
			'A'..='Z' |

			// DIGIT
			'0'..='9'|

			'-' |
			'.' |
			'_' |
			'~' |

			// sub-delims
			'!' |
			'$' |
			'&' |
			'\'' |
			'(' |
			')' |
			'*' |
			'+' |
			',' |
			';' |
			'=' |

			':' |
			'@'
		)
	})
}

#[test]
fn check_is_valid_ascii_string() {
	let test_cases = [
		("kilt.io", true),
		("super.long.domain.com:12345/path/to/directory#fragment?arg=value", true),
		("super.long.domain.com:12345/path/to/directory/file.txt", true),
		("domain.with.only.valid.characters.:/?#[]@!$&'()*+,;=-._~", true),
		("invalid.châracter.domain.org", false),
		("âinvalid.character.domain.org", false),
		("invalid.character.domain.orgâ", false),
		("", true),
		("例子.領域.cn", false),
		("kilt.io/%3Ctag%3E/encoded_upper_case_ascii.com", true),
		("kilt.io/%3ctag%3e/encoded_lower_case_ascii.com", true),
	];

	test_cases.iter().for_each(|(input, expected_result)| {
		assert_eq!(
			is_valid_ascii_string(input),
			*expected_result,
			"Test case for \"{}\" returned wrong result.",
			input
		);
	});
}

#[test]
fn check_is_valid_uri_fragment_string() {
	let test_cases = [
		("kilt.io", true),
		(
			"super.long.domain.com:12345/path/to/directory#fragment?arg=value",
			false,
		),
		("super.long.domain.com:12345/path/to/directory/file.txt", false),
		("domain.with.only.valid.characters.:/?#[]@!$&'()*+,;=-._~", false),
		("invalid.châracter.domain.org", false),
		("âinvalid.character.domain.org", false),
		("invalid.character.domain.orgâ", false),
		("", true),
		("例子.領域.cn", false),
		("kilt.io/%3Ctag%3E/encoded_upper_case_ascii.com", false),
		("kilt.io/%3ctag%3e/encoded_lower_case_ascii.com", false),
	];

	test_cases.iter().for_each(|(input, expected_result)| {
		assert_eq!(
			is_valid_uri_fragment(input),
			*expected_result,
			"Test case for \"{}\" returned wrong result.",
			input
		);
	});
}
