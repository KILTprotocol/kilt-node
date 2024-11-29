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

/// Runtime-injected logic to support the DID pallet in making sure no dangling
/// references is left before deleting a given DID.
pub trait DeletionHelper<T>
where
	T: Config,
	<Self::DeletionIter as Iterator>::Item: SteppedDeletion,
{
	type DeletionIter: Iterator;

	fn deletion_iter(did: &DidIdentifierOf<T>) -> Self::DeletionIter;
}

impl<T> DeletionHelper<T> for ()
where
	T: Config,
{
	type DeletionIter = EmptyIterator;

	fn deletion_iter(_did: &DidIdentifierOf<T>) -> Self::DeletionIter {
		EmptyIterator
	}
}

pub struct EmptyIterator;

impl Iterator for EmptyIterator {
	type Item = ();

	fn next(&mut self) -> Option<Self::Item> {
		Some(())
	}
}

pub trait SteppedDeletion {
	type VerifiedInfo;

	fn pre_check(remaining_weight: Weight) -> Option<Self::VerifiedInfo>;

	fn execute(info: Self::VerifiedInfo);
}

impl SteppedDeletion for () {
	type VerifiedInfo = ();

	fn pre_check(_remaining_weight: Weight) -> Self::VerifiedInfo {
		()
	}

	fn execute(info: Self::VerifiedInfo) {}
}
