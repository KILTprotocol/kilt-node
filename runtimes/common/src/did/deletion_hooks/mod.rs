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

use did::DidIdentifierOf;
use sp_std::marker::PhantomData;

use sp_weights::Weight;

pub struct EnsureNoLinkedWeb3NameDeletionHook<
	const READ_WEIGHT_TIME: u64,
	const READ_WEIGHT_SIZE: u64,
	Web3NameDeployment,
>(PhantomData<Web3NameDeployment>);

impl<T, const READ_WEIGHT_TIME: u64, const READ_WEIGHT_SIZE: u64, Web3NameDeployment> did::traits::DidDeletionHook<T>
	for EnsureNoLinkedWeb3NameDeletionHook<READ_WEIGHT_SIZE, READ_WEIGHT_TIME, Web3NameDeployment>
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

pub struct EnsureNoLinkedAccountDeletionHook<
	const READ_WEIGHT_TIME: u64,
	const READ_WEIGHT_SIZE: u64,
	AccountLinkingDeployment,
>(PhantomData<AccountLinkingDeployment>);

impl<T, const READ_WEIGHT_TIME: u64, const READ_WEIGHT_SIZE: u64, AccountLinkingDeployment>
	did::traits::DidDeletionHook<T>
	for EnsureNoLinkedAccountDeletionHook<READ_WEIGHT_SIZE, READ_WEIGHT_TIME, AccountLinkingDeployment>
where
	T: did::Config + pallet_did_lookup::Config<AccountLinkingDeployment, DidIdentifier = DidIdentifierOf<T>>,
	AccountLinkingDeployment: 'static,
{
	const MAX_WEIGHT: Weight = Weight::from_parts(READ_WEIGHT_TIME, READ_WEIGHT_SIZE);

	fn can_delete(did: &did::DidIdentifierOf<T>) -> Result<(), Weight> {
		// We check whether the prefix iterator for the given DID has at least one
		// element (`next == Some`).
		let is_any_account_linked =
			pallet_did_lookup::ConnectedAccounts::<T, AccountLinkingDeployment>::iter_key_prefix(did)
				.next()
				.is_some();
		if !is_any_account_linked {
			Ok(())
		} else {
			Err(<Self as did::traits::DidDeletionHook<T>>::MAX_WEIGHT)
		}
	}
}
