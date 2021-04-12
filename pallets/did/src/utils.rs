// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

/// Verifies that an input string contains only URL-allowed ASCII characters.
/// For more info about what those characters are, please visit the official RFC 3986.
pub fn is_valid_ascii_url(input: &str) -> bool {
	for c in input.chars() {
		if !match c {
			':' | '/' | '?' | '#' | '[' | ']' | '@' | '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';'
			| '=' | '-' | '.' | '_' | '~' => true,
			'0'..='9' => true,
			'a'..='z' => true,
			'A'..='Z' => true,
			_ => false,
		} { return false };
	}
	true
}

/// Verifies that an input string contains only Base-32 ASCII characters.
/// For more info about what those characters are, please visit the official RFC 4648.
pub fn is_base_32(input: &str) -> bool {
	for c in input.chars() {
		if !match c {
			'A'..='Z' => true,
			'2'..='7' => true,
			// Padding character. At the moment, no check is performed the verify that padding characters are only at the end of the char sequence.
			'=' => true,
			_ => false,
		} { return false };
	}
	true
}

/// Verifies that an input string contains only Base-58 ASCII characters.
/// For more info about what those characters are, please visit the official IETF draft.
pub fn is_base_58(input: &str) -> bool {
	for c in input.chars() {
		if !match c {
			'A'..='H' => true,
			// Skip I
			'J'..='N' => true,
			// Skip O (capital o)
			'P'..='Z' => true,
			'a'..='k' => true,
			// Skip l (lower L)
			'm'..='z' => true,
			// Skip 0 (zero)
			'1'..='9' => true,
			_ => false,
		} { return false };
	}
	true
}
