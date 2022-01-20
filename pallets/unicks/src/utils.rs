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

use sp_std::str;

/// Verify that a given slice contains only allowed ASCII characters.
pub(crate) fn is_byte_array_ascii_string(input: &[u8]) -> bool {
	if let Ok(encoded_unick) = str::from_utf8(input) {
		encoded_unick.chars().all(|c| {
			// TODO: Change once we reach a decision on which characters to allow
			matches!(c, 'a'..='z' | '0'..='9' | ':' | '#' | '@' | '$' | '&' | '(' | ')' | '*' | '+' | '-' | '.' | '_')
		})
	} else {
		false
	}
}
