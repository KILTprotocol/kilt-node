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
#![cfg_attr(not(feature = "std"), no_std)]

mod deposit;
pub use deposit::Deposit;
pub mod migration;
pub mod signature;
pub mod traits;
pub mod xcm;

#[cfg(any(feature = "runtime-benchmarks", feature = "mock"))]
pub mod mock;

#[cfg(any(feature = "try-runtime", test))]
pub mod test_utils;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmark;
