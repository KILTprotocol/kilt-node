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

use frame_support::RuntimeDebug;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// Configuration for the runtime.
#[derive(Clone, Encode, Decode, RuntimeDebug, MaxEncodedLen, Eq, PartialEq, TypeInfo)]
pub struct Configuration {
	/// Enables the check that the blocknumber of the relay chain strictly
	/// increases.
	pub relay_block_strictly_increasing: bool,
}

impl Default for Configuration {
	fn default() -> Self {
		Self {
			relay_block_strictly_increasing: true,
		}
	}
}
