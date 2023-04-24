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
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::integer_arithmetic)]
#![warn(clippy::integer_division)]
#![warn(clippy::as_conversions)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::arithmetic_side_effects)]
#![deny(clippy::index_refutable_slice)]
#![deny(clippy::indexing_slicing)]
#![warn(clippy::float_arithmetic)]
#![warn(clippy::cast_possible_wrap)]

pub mod deposit;
pub use deposit::{free_deposit, reserve_deposit};

#[cfg(any(feature = "runtime-benchmarks", feature = "mock"))]
pub mod mock;
pub mod signature;
pub mod traits;
