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

// If you feel like getting in touch with us, you can do so at <hello@kilt.org>

use crate::{Config, DidIdentifierOf};

/// Runtime logic evaluated by the DID pallet upon deleting an existing DID.
pub trait DidDeletionHook<T>
where
	T: Config,
{
	/// Return whether the DID can be deleted (`Ok(())`), or not. In case of
	/// error, the consumed weight (less than or equal to `MAX_WEIGHT`) is
	/// returned.
	fn can_delete(did: &DidIdentifierOf<T>) -> bool;
}

impl<T> DidDeletionHook<T> for ()
where
	T: Config,
{
	fn can_delete(_did: &DidIdentifierOf<T>) -> bool {
		true
	}
}
