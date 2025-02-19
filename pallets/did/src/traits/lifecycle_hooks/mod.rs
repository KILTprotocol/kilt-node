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

pub mod deletion;
pub use deletion::DidDeletionHook;

// We need this mock since the trait requires the implementation of the did's
// `Config` trait.
#[cfg(test)]
mod mock;

use crate::Config;

/// A collection of hooks invoked during DID operations.
pub trait DidLifecycleHooks<T>
where
	T: Config,
{
	/// Hook called when a DID deletion is requested by an authorized entity.
	type DeletionHook: DidDeletionHook<T>;
}

impl<T> DidLifecycleHooks<T> for ()
where
	T: Config,
{
	type DeletionHook = ();
}
