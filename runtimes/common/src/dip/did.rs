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
use frame_support::weights::Weight;
use frame_system::pallet_prelude::BlockNumberFor;
use kilt_dip_support::merkle::RevealedWeb3Name;
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_provider::traits::IdentityProvider;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

use crate::dip::weights::{SubstrateWeight, WeightInfo};

#[derive(Encode, Decode, TypeInfo)]
pub enum LinkedDidInfoProviderError {
	DidNotFound,
	DidDeleted,
	Internal,
}

impl From<LinkedDidInfoProviderError> for u16 {
	fn from(value: LinkedDidInfoProviderError) -> Self {
		match value {
			LinkedDidInfoProviderError::DidNotFound => 0,
			LinkedDidInfoProviderError::DidDeleted => 1,
			LinkedDidInfoProviderError::Internal => u16::MAX,
		}
	}
}

pub type Web3OwnershipOf<Runtime> =
	RevealedWeb3Name<<Runtime as pallet_web3_names::Config>::Web3Name, BlockNumberFor<Runtime>>;

pub struct LinkedDidInfoOf<Runtime>
where
	Runtime: did::Config + pallet_web3_names::Config,
{
	pub did_details: DidDetails<Runtime>,
	pub web3_name_details: Option<Web3OwnershipOf<Runtime>>,
	pub linked_accounts: Vec<LinkableAccountId>,
}

pub struct LinkedDidInfoProvider;

impl<Runtime> IdentityProvider<Runtime> for LinkedDidInfoProvider
where
	Runtime: did::Config<DidIdentifier = <Runtime as pallet_dip_provider::Config>::Identifier>
		+ pallet_web3_names::Config<Web3NameOwner = <Runtime as pallet_dip_provider::Config>::Identifier>
		+ pallet_did_lookup::Config<DidIdentifier = <Runtime as pallet_dip_provider::Config>::Identifier>
		+ pallet_dip_provider::Config,
{
	type Error = LinkedDidInfoProviderError;
	type Success = LinkedDidInfoOf<Runtime>;

	fn retrieve(identifier: &Runtime::Identifier) -> Result<Self::Success, Self::Error> {
		let did_details = match (
			did::Pallet::<Runtime>::get_did(identifier),
			did::Pallet::<Runtime>::get_deleted_did(identifier),
		) {
			(Some(details), _) => Ok(details),
			(_, Some(_)) => Err(LinkedDidInfoProviderError::DidDeleted),
			_ => Err(LinkedDidInfoProviderError::DidNotFound),
		}?;
		let web3_name_details = if let Some(web3_name) = pallet_web3_names::Pallet::<Runtime>::names(identifier) {
			let Some(ownership) = pallet_web3_names::Pallet::<Runtime>::owner(&web3_name) else {
				log::error!(
					"Inconsistent reverse map pallet_web3_names::owner(web3_name). Cannot find owner for web3name {:#?}",
					web3_name
				);
				return Err(LinkedDidInfoProviderError::Internal);
			};
			Ok(Some(Web3OwnershipOf::<Runtime> {
				web3_name,
				claimed_at: ownership.claimed_at,
			}))
		} else {
			Ok(None)
		}?;
		let linked_accounts = pallet_did_lookup::ConnectedAccounts::<Runtime>::iter_key_prefix(identifier).collect();
		Ok(LinkedDidInfoOf {
			did_details,
			web3_name_details,
			linked_accounts,
		})
	}

	fn get_retrieve_weight() -> Weight {
		SubstrateWeight::<Runtime>::retrieve_linked_accounts()
	}
}
