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

use did::did_details::DidDetails;
use kilt_dip_support::{
	merkle::RevealedWeb3Name,
	utils::{CombineIdentityFrom, CombinedIdentityResult},
};
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_provider::traits::IdentityProvider;
use sp_std::{marker::PhantomData, vec::Vec};

pub struct DidIdentityProvider<T>(PhantomData<T>);

impl<T> IdentityProvider<T::DidIdentifier> for DidIdentityProvider<T>
where
	T: did::Config,
{
	// TODO: Proper error handling
	type Error = ();
	type Success = DidDetails<T>;

	fn retrieve(identifier: &T::DidIdentifier) -> Result<Option<Self::Success>, Self::Error> {
		match (
			did::Pallet::<T>::get_did(identifier),
			did::Pallet::<T>::get_deleted_did(identifier),
		) {
			(Some(details), _) => Ok(Some(details)),
			(_, Some(_)) => Ok(None),
			_ => Err(()),
		}
	}
}

pub type Web3OwnershipOf<T> =
	RevealedWeb3Name<<T as pallet_web3_names::Config>::Web3Name, <T as frame_system::Config>::BlockNumber>;

pub struct DidWeb3NameProvider<T>(PhantomData<T>);

impl<T> IdentityProvider<T::Web3NameOwner> for DidWeb3NameProvider<T>
where
	T: pallet_web3_names::Config,
{
	// TODO: Proper error handling
	type Error = ();
	type Success = Web3OwnershipOf<T>;

	fn retrieve(identifier: &T::Web3NameOwner) -> Result<Option<Self::Success>, Self::Error> {
		let Some(web3_name) = pallet_web3_names::Pallet::<T>::names(identifier) else { return Ok(None) };
		let Some(details) = pallet_web3_names::Pallet::<T>::owner(&web3_name) else { return Err(()) };
		Ok(Some(Web3OwnershipOf::<T> {
			web3_name,
			claimed_at: details.claimed_at,
		}))
	}
}

pub struct DidLinkedAccountsProvider<T>(PhantomData<T>);

impl<T> IdentityProvider<T::DidIdentifier> for DidLinkedAccountsProvider<T>
where
	T: pallet_did_lookup::Config,
{
	// TODO: Proper error handling
	type Error = ();
	type Success = Vec<LinkableAccountId>;

	fn retrieve(identifier: &T::DidIdentifier) -> Result<Option<Self::Success>, Self::Error> {
		Ok(Some(
			pallet_did_lookup::ConnectedAccounts::<T>::iter_key_prefix(identifier).collect(),
		))
	}
}

pub type LinkedDidInfoProviderOf<T> =
	CombineIdentityFrom<DidIdentityProvider<T>, DidWeb3NameProvider<T>, DidLinkedAccountsProvider<T>>;
pub type LinkedDidInfoOf<T> =
	CombinedIdentityResult<Option<DidDetails<T>>, Option<Web3OwnershipOf<T>>, Option<Vec<LinkableAccountId>>>;
