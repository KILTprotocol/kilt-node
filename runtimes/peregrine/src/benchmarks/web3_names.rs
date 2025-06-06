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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

pub(crate) use web3_names_deployment::Web3NamesBenchmarkHelper;
mod web3_names_deployment {
	use sp_std::{vec, vec::Vec};

	use crate::Runtime;

	pub struct Web3NamesBenchmarkHelper;

	impl pallet_web3_names::BenchmarkHelper for Web3NamesBenchmarkHelper {
		fn generate_name_input_with_length(length: usize) -> Vec<u8> {
			let input = vec![b'a'; length];

			debug_assert!(<Runtime as pallet_web3_names::Config<()>>::Web3Name::try_from(input.clone()).is_ok());
			input
		}
	}
}
