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

use did::DidIdentifierOf;
use sp_std::marker::PhantomData;

use sp_weights::Weight;

// If you feel like getting in touch with us, you can do so at info@botlabs.org
pub struct LinkedWeb3NameDeletionHook<const READ_WEIGHT_TIME: u64, const READ_WEIGHT_SIZE: u64, Web3NameDeployment>(
	PhantomData<Web3NameDeployment>,
);

impl<T, const READ_WEIGHT_TIME: u64, const READ_WEIGHT_SIZE: u64, Web3NameDeployment> did::traits::DidDeletionHook<T>
	for LinkedWeb3NameDeletionHook<READ_WEIGHT_SIZE, READ_WEIGHT_TIME, Web3NameDeployment>
where
	T: did::Config + pallet_web3_names::Config<Web3NameDeployment, Web3NameOwner = DidIdentifierOf<T>>,
	Web3NameDeployment: 'static,
{
	const MAX_WEIGHT: Weight = Weight::from_parts(READ_WEIGHT_TIME, READ_WEIGHT_SIZE);

	fn can_delete(did: &did::DidIdentifierOf<T>) -> Result<(), Weight> {
		if pallet_web3_names::Names::<T, Web3NameDeployment>::contains_key(did) {
			Ok(())
		} else {
			Err(<Self as did::traits::DidDeletionHook<T>>::MAX_WEIGHT)
		}
	}
}

// TODO: Add the other ones, and then implement the trait for a tuple of
// elements, summing up their `MAX_WEIGHT` as the overall `MAX_WEIGHT`.
