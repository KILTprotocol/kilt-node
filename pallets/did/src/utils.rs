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
/// For more info about what those characters are, please visit the official RFC
/// 3986.
pub fn is_valid_ascii_url(input: &str) -> bool {
	for c in input.chars() {
		// Matches [0-9], [a-z], [A-Z], plus the symbols as in the RFC.
		if !matches!(c, ':' | '/' | '?' | '#' | '[' | ']' | '@' | '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';'
		| '=' | '-' | '.' | '_' | '~' | '0'..='9' | 'a'..='z' | 'A'..='Z')
		{
			return false;
		}
	}
	true
}

/// Verifies that an input string contains only Base-32 ASCII characters.
/// For more info about what those characters are, please visit the official RFC
/// 4648.
pub fn is_base_32(input: &str) -> bool {
	for c in input.chars() {
		// Matches [A-Z], and [2-7].
		// At the moment, no check is performed the verify that padding characters are
		// only at the end of the char sequence.
		if !matches!(c, 'A'..='Z' | '2'..='7' | '=') {
			return false;
		}
	}
	true
}

/// Verifies that an input string contains only Base-58 ASCII characters.
/// For more info about what those characters are, please visit the official
/// IETF draft.
pub fn is_base_58(input: &str) -> bool {
	for c in input.chars() {
		// Matches [A-H], [J-N], [P-Z], [a-k], [m-z], and [1-9].
		if !matches!(c, 'A'..='H' | 'J'..='N' | 'P'..='Z' | 'a'..='k' | 'm'..='z' | '1'..='9') {
			return false;
		};
	}
	true
}
