// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

use crate::{Config, DidIdentifierOf};

/// Runtime-injected logic to support the DID pallet in making sure no dangling
/// references is left before deleting a given DID.
pub trait DeletionHelper<T>
where
	T: Config,
{
	/// Return the count of resources linked to a given DID.
	fn linked_resources_count(did: &DidIdentifierOf<T>) -> u32;
}

impl<T> DeletionHelper<T> for ()
where
	T: Config,
{
	fn linked_resources_count(_did: &DidIdentifierOf<T>) -> u32 {
		0
	}
}
