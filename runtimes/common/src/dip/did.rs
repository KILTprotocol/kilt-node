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
use kilt_dip_support::merkle::RevealedWeb3Name;
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_provider::traits::IdentityProvider;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

#[cfg(feature = "runtime-benchmarks")]
use kilt_support::traits::GetWorstCase;

#[derive(Encode, Decode, TypeInfo, Debug)]
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
}

#[cfg(feature = "runtime-benchmarks")]
impl<Runtime> GetWorstCase for LinkedDidInfoOf<Runtime>
where
	Runtime: did::Config<DidIdentifier = <Runtime as pallet_dip_provider::Config>::Identifier>
		+ pallet_web3_names::Config<Web3NameOwner = <Runtime as pallet_dip_provider::Config>::Identifier>
		+ pallet_did_lookup::Config<DidIdentifier = <Runtime as pallet_dip_provider::Config>::Identifier>
		+ pallet_dip_provider::Config
		+ pallet_balances::Config,
	<Runtime as frame_system::Config>::AccountId: Into<LinkableAccountId> + From<sp_core::sr25519::Public>,
	<Runtime as frame_system::Config>::AccountId: AsRef<[u8; 32]> + From<[u8; 32]>,
{
	/// The worst case for the [LinkedDidInfor] is a DID with all keys set, a web3name and linked accounts in the palled_did_lookup pallet.
	fn worst_case() -> Self {
		use did::{
			did_details::DidVerificationKey,
			mock_utils::{generate_base_did_creation_details, get_key_agreement_keys},
		};
		use frame_benchmarking::{account, vec, Zero};
		use sp_io::crypto::{ed25519_generate, sr25519_generate};
		use sp_runtime::{traits::Get, BoundedVec, KeyTypeId};

		// Did Details.
		let did: <Runtime as did::Config>::DidIdentifier = account("did", 0, 0);
		let submitter: <Runtime as frame_system::Config>::AccountId = account("submitter", 1, 1);

		let max_new_keys = <Runtime as did::Config>::MaxNewKeyAgreementKeys::get();

		let new_key_agreement_keys = get_key_agreement_keys::<Runtime>(max_new_keys);

		let mut did_creation_details = generate_base_did_creation_details(did.clone(), submitter.clone());

		// TODO check if I have to set different seed. Prob not but lets be sure.
		let attestation_key = ed25519_generate(KeyTypeId(*b"0001"), None);
		let delegation_key = ed25519_generate(KeyTypeId(*b"0002"), None);
		let auth_key = ed25519_generate(KeyTypeId(*b"0003"), None);
		did_creation_details.new_attestation_key = Some(DidVerificationKey::from(attestation_key));
		did_creation_details.new_delegation_key = Some(DidVerificationKey::from(delegation_key));
		did_creation_details.new_key_agreement_keys = new_key_agreement_keys;

		let did_details = did::did_details::DidDetails::new_with_creation_details(
			did_creation_details,
			DidVerificationKey::from(auth_key),
		)
		.expect("Creation of DID details should not fail.");

		// add to storage.
		did::Pallet::<Runtime>::try_insert_did(did.clone(), did_details.clone(), submitter.clone())
			.expect("Inserting Did should not fail.");

		let max_name_length = <Runtime as pallet_web3_names::Config>::MaxNameLength::get()
			.try_into()
			.expect("max name length should not fail.");

		let web3_name_input: BoundedVec<u8, <Runtime as pallet_web3_names::Config>::MaxNameLength> =
			BoundedVec::try_from(vec![b'1'; max_name_length]).expect("BoundedVec creation should not fail.");

		let web3_name = pallet_web3_names::Web3NameOf::<Runtime>::try_from(web3_name_input.to_vec())
			.expect("Creation of w3n from w3n input should not fail.");

		pallet_web3_names::Pallet::<Runtime>::register_name(web3_name.clone(), did.clone(), submitter.clone())
			.expect("Inserting w3n into storage should not fail.");

		let web3_name_details = Some(RevealedWeb3Name {
			web3_name,
			claimed_at: BlockNumberFor::<Runtime>::zero(),
		});

		let mut linked_accounts = vec![];

		(0..pallet_did_lookup::MAX_LINKED_ACCOUNT).for_each(|index| {
			let connected_acc = sr25519_generate(KeyTypeId(*b"aura"), Some(index.to_be_bytes().to_vec()));
			let connected_acc_id: <Runtime as frame_system::Config>::AccountId = connected_acc.into();
			let linkable_id: LinkableAccountId = connected_acc_id.clone().into();
			pallet_did_lookup::Pallet::<Runtime>::add_association(submitter.clone(), did.clone(), linkable_id.clone())
				.expect("association should not fail.");

			linked_accounts.push(linkable_id);
		});

		LinkedDidInfoOf {
			did_details,
			linked_accounts,
			web3_name_details,
		}
	}
}