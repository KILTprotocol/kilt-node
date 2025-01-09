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

use sp_weights::Weight;

use crate::{Config, DidIdentifierOf};

pub trait DidLifecycleHooks<T>
where
	T: Config,
{
	type DeletionHook: DidDeletionHook<T>;
}

impl<T> DidLifecycleHooks<T> for ()
where
	T: Config,
{
	type DeletionHook = ();
}

pub trait DidDeletionHook<T>
where
	T: Config,
{
	const MAX_WEIGHT: Weight;

	// Return `Ok(())` consuming `MAX_WEIGHT` if the DID can be deleted, or
	// `Err(Weight)` with the consumed weight if not.
	fn can_delete(did: &DidIdentifierOf<T>) -> Result<(), Weight>;
}

impl<T> DidDeletionHook<T> for ()
where
	T: Config,
{
	const MAX_WEIGHT: Weight = Weight::from_parts(0, 0);

	fn can_delete(_did: &DidIdentifierOf<T>) -> Result<(), Weight> {
		Ok(())
	}
}
