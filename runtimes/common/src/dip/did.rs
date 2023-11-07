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
use frame_system::pallet_prelude::BlockNumberFor;
use kilt_dip_support::{
	merkle::RevealedWeb3Name,
	utils::{CombineIdentityFrom, CombinedIdentityResult},
};
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_provider::traits::IdentityProvider;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::{marker::PhantomData, vec::Vec};

#[derive(Encode, Decode, TypeInfo)]
pub enum DidIdentityProviderError {
	DidNotFound,
	Internal,
}

impl From<DidIdentityProviderError> for u16 {
	fn from(value: DidIdentityProviderError) -> Self {
		match value {
			DidIdentityProviderError::DidNotFound => 0,
			DidIdentityProviderError::Internal => u16::MAX,
		}
	}
}

pub struct DidIdentityProvider<T>(PhantomData<T>);

impl<Runtime> IdentityProvider<Runtime> for DidIdentityProvider<Runtime>
where
	Runtime:
		did::Config<DidIdentifier = <Runtime as pallet_dip_provider::Config>::Identifier> + pallet_dip_provider::Config,
{
	type Error = DidIdentityProviderError;
	type Identity = DidDetails<Runtime>;

	fn retrieve(identifier: &Runtime::Identifier) -> Result<Option<Self::Identity>, Self::Error> {
		match (
			did::Pallet::<Runtime>::get_did(identifier),
			did::Pallet::<Runtime>::get_deleted_did(identifier),
		) {
			(Some(details), _) => Ok(Some(details)),
			(_, Some(_)) => Ok(None),
			_ => Err(DidIdentityProviderError::DidNotFound),
		}
	}
}

pub type Web3OwnershipOf<Runtime> =
	RevealedWeb3Name<<Runtime as pallet_web3_names::Config>::Web3Name, BlockNumberFor<Runtime>>;

pub struct DidWeb3NameProvider<T>(PhantomData<T>);

impl<Runtime> IdentityProvider<Runtime> for DidWeb3NameProvider<Runtime>
where
	Runtime: pallet_web3_names::Config<Web3NameOwner = <Runtime as pallet_dip_provider::Config>::Identifier>
		+ pallet_dip_provider::Config,
{
	type Error = DidIdentityProviderError;
	type Identity = Web3OwnershipOf<Runtime>;

	fn retrieve(identifier: &Runtime::Web3NameOwner) -> Result<Option<Self::Identity>, Self::Error> {
		let Some(web3_name) = pallet_web3_names::Pallet::<Runtime>::names(identifier) else {
			return Ok(None);
		};
		let Some(details) = pallet_web3_names::Pallet::<Runtime>::owner(&web3_name) else {
			log::error!(
				"Inconsistent reverse map pallet_web3_names::owner(web3_name). Cannot find owner for web3name {:#?}",
				web3_name
			);
			return Err(DidIdentityProviderError::Internal);
		};
		Ok(Some(Web3OwnershipOf::<Runtime> {
			web3_name,
			claimed_at: details.claimed_at,
		}))
	}
}

pub struct DidLinkedAccountsProvider<T>(PhantomData<T>);

impl<Runtime> IdentityProvider<Runtime> for DidLinkedAccountsProvider<Runtime>
where
	Runtime: pallet_did_lookup::Config<DidIdentifier = <Runtime as pallet_dip_provider::Config>::Identifier>
		+ pallet_dip_provider::Config,
{
	type Error = DidIdentityProviderError;
	type Identity = Vec<LinkableAccountId>;

	fn retrieve(identifier: &Runtime::DidIdentifier) -> Result<Option<Self::Identity>, Self::Error> {
		Ok(Some(
			pallet_did_lookup::ConnectedAccounts::<Runtime>::iter_key_prefix(identifier).collect(),
		))
	}
}

pub type LinkedDidInfoProviderOf<T> =
	CombineIdentityFrom<DidIdentityProvider<T>, DidWeb3NameProvider<T>, DidLinkedAccountsProvider<T>>;
pub type LinkedDidInfoOf<T> =
	CombinedIdentityResult<Option<DidDetails<T>>, Option<Web3OwnershipOf<T>>, Option<Vec<LinkableAccountId>>>;
