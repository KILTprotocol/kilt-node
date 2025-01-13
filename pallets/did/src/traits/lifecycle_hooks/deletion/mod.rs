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

#[cfg(test)]
mod tests;

use sp_std::marker::PhantomData;
use sp_weights::Weight;

use crate::{Config, DidIdentifierOf};

/// Runtime logic evaluated by the DID pallet upon deleting an existing DID.
pub trait DidDeletionHook<T>
where
	T: Config,
{
	/// The statically computed maximum weight the implementation will consume
	/// to verify if a DID can be deleted.
	const MAX_WEIGHT: Weight;

	/// Return whether the DID can be deleted (`Ok(())`), or not. In case of
	/// error, the consumed weight (less than or equal to `MAX_WEIGHT`) is
	/// returned.
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

/// Implementation of [`DidDeletionHook`] that iterates over both
/// components, bailing out early if the first one fails. The `MAX_WEIGHT` is
/// the sum of both components.
pub struct RequireBoth<A, B>(PhantomData<(A, B)>);

impl<T, A, B> DidDeletionHook<T> for RequireBoth<A, B>
where
	T: Config,
	A: DidDeletionHook<T>,
	B: DidDeletionHook<T>,
{
	const MAX_WEIGHT: Weight = A::MAX_WEIGHT.saturating_add(B::MAX_WEIGHT);

	/// In case of failure, the returned weight is either the weight consumed by
	/// the first component, or the sum of the first component's maximum weight
	/// and the weight consumed by the second component.
	fn can_delete(did: &DidIdentifierOf<T>) -> Result<(), Weight> {
		// Bail out early with A's weight if A fails.
		A::can_delete(did)?;
		// Bail out early with A's max weight + B's if B fails.
		B::can_delete(did).map_err(|consumed_weight| A::MAX_WEIGHT.saturating_add(consumed_weight))?;
		Ok(())
	}
}
