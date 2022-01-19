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

use frame_support::ensure;
use sp_runtime::DispatchError;
use sp_std::str;

use crate::{Config, Error, UnickOf};

pub(crate) fn check_unick_validity<T: Config>(unick: &UnickOf<T>) -> Result<(), DispatchError> {
	let byte_ref: &[u8] = unick.as_ref();
	let encoded_unick = str::from_utf8(byte_ref).map_err(|_| DispatchError::from(Error::<T>::InvalidUnickFormat))?;

	let is_unick_valid = encoded_unick.chars().all(|c| {
		// TODO: Change once we reach a decision on which characters to allow
		matches!(c, ':' | '#' | '@' | '$' | '&' | '(' | ')' | '*' | '+' | '-' | '.' | '_' | '0'..='9' | 'a'..='z' | 'A'..='Z')
	});

	ensure!(is_unick_valid, DispatchError::from(Error::<T>::InvalidUnickFormat));

	Ok(())
}
