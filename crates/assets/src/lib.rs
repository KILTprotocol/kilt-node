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

//! Library to parse the raw byte vectors into supported Asset DIDs, according
//! to the spec.
//!
//! The library is suitable for no_std environment, such as WASM-based
//! blockchain runtimes.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod asset;
pub mod chain;
pub mod v1;

mod errors;

// Re-export relevant types
pub use asset::*;
pub use chain::*;
pub use errors::*;
pub use v1::*;
