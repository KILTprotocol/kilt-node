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
use frame_support::ensure;
use frame_system::pallet_prelude::BlockNumberFor;
use kilt_dip_primitives::merkle::RevealedWeb3Name;
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_provider::traits::IdentityProvider;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::ConstU32;
use sp_runtime::{BoundedVec, SaturatedConversion};
use sp_std::vec::Vec;

#[cfg(feature = "runtime-benchmarks")]
use kilt_support::{benchmark::IdentityContext, traits::GetWorstCase};

#[derive(Encode, Decode, TypeInfo, Debug)]
pub enum LinkedDidInfoProviderError {
	DidNotFound,
	DidDeleted,
	TooManyLinkedAccounts,
	Internal,
}

impl From<LinkedDidInfoProviderError> for u16 {
	fn from(value: LinkedDidInfoProviderError) -> Self {
		match value {
			LinkedDidInfoProviderError::DidNotFound => 0,
			LinkedDidInfoProviderError::DidDeleted => 1,
			LinkedDidInfoProviderError::TooManyLinkedAccounts => 2,
			LinkedDidInfoProviderError::Internal => u16::MAX,
		}
	}
}

pub type Web3OwnershipOf<Runtime> =
	RevealedWeb3Name<<Runtime as pallet_web3_names::Config>::Web3Name, BlockNumberFor<Runtime>>;

/// Identity information related to a KILT DID relevant for cross-chain
/// transactions via the DIP protocol.
pub struct LinkedDidInfoOf<Runtime, const MAX_LINKED_ACCOUNTS: u32>
where
	Runtime: did::Config + pallet_web3_names::Config,
{
	/// The DID Document of the subject.
	pub did_details: DidDetails<Runtime>,
	/// The optional web3name details linked to the subject.
	pub web3_name_details: Option<Web3OwnershipOf<Runtime>>,
	/// The list of accounts the subject has previously linked via the linking
	/// pallet.
	pub linked_accounts: BoundedVec<LinkableAccountId, ConstU32<MAX_LINKED_ACCOUNTS>>,
}

/// Type implementing the [`IdentityProvider`] trait which is responsible for
/// collecting the DID information relevant for DIP cross-chain transactions by
/// interacting with the different pallets involved.
pub struct LinkedDidInfoProvider<const MAX_LINKED_ACCOUNTS: u32>;

impl<Runtime, const MAX_LINKED_ACCOUNTS: u32> IdentityProvider<Runtime> for LinkedDidInfoProvider<MAX_LINKED_ACCOUNTS>
where
	Runtime: did::Config<DidIdentifier = <Runtime as pallet_dip_provider::Config>::Identifier>
		+ pallet_web3_names::Config<Web3NameOwner = <Runtime as pallet_dip_provider::Config>::Identifier>
		+ pallet_did_lookup::Config<DidIdentifier = <Runtime as pallet_dip_provider::Config>::Identifier>
		+ pallet_dip_provider::Config,
{
	type Error = LinkedDidInfoProviderError;
	type Success = LinkedDidInfoOf<Runtime, MAX_LINKED_ACCOUNTS>;

	fn retrieve(identifier: &Runtime::Identifier) -> Result<Self::Success, Self::Error> {
		ensure!(
			did::Pallet::<Runtime>::get_deleted_did(identifier).is_none(),
			LinkedDidInfoProviderError::DidDeleted,
		);
		let did_details = did::Pallet::<Runtime>::get_did(identifier).ok_or(LinkedDidInfoProviderError::DidNotFound)?;

		let web3_name_details = retrieve_w3n::<Runtime>(identifier)?;

		let linked_accounts = retrieve_linked_accounts::<Runtime, MAX_LINKED_ACCOUNTS>(identifier)?;

		Ok(LinkedDidInfoOf {
			did_details,
			web3_name_details,
			linked_accounts,
		})
	}
}

fn retrieve_w3n<Runtime>(
	identifier: &Runtime::Identifier,
) -> Result<Option<Web3OwnershipOf<Runtime>>, LinkedDidInfoProviderError>
where
	Runtime: did::Config<DidIdentifier = <Runtime as pallet_dip_provider::Config>::Identifier>
		+ pallet_web3_names::Config<Web3NameOwner = <Runtime as pallet_dip_provider::Config>::Identifier>
		+ pallet_dip_provider::Config,
{
	let Some(web3_name) = pallet_web3_names::Pallet::<Runtime>::names(identifier) else {
		return Ok(None);
	};

	let ownership = pallet_web3_names::Pallet::<Runtime>::owner(&web3_name).ok_or_else(|| {
		log::error!(
			"Inconsistent reverse map pallet_web3_names::owner(web3_name). Cannot find owner for web3name {:#?}",
			web3_name
		);
		LinkedDidInfoProviderError::Internal
	})?;

	Ok(Some(Web3OwnershipOf::<Runtime> {
		web3_name,
		claimed_at: ownership.claimed_at,
	}))
}

fn retrieve_linked_accounts<Runtime, const MAX_LINKED_ACCOUNTS: u32>(
	identifier: &Runtime::Identifier,
) -> Result<BoundedVec<LinkableAccountId, ConstU32<MAX_LINKED_ACCOUNTS>>, LinkedDidInfoProviderError>
where
	Runtime: did::Config<DidIdentifier = <Runtime as pallet_dip_provider::Config>::Identifier>
		+ pallet_did_lookup::Config<DidIdentifier = <Runtime as pallet_dip_provider::Config>::Identifier>
		+ pallet_dip_provider::Config,
{
	// Check if the user has too many linked accounts. If they have more than
	// [MAX_LINKED_ACCOUNTS], we throw an error.
	let are_linked_accounts_within_limit = pallet_did_lookup::ConnectedAccounts::<Runtime>::iter_key_prefix(identifier)
		.nth(MAX_LINKED_ACCOUNTS.saturated_into())
		.is_none();

	ensure!(
		are_linked_accounts_within_limit,
		LinkedDidInfoProviderError::TooManyLinkedAccounts
	);

	pallet_did_lookup::ConnectedAccounts::<Runtime>::iter_key_prefix(identifier)
		.take(MAX_LINKED_ACCOUNTS.saturated_into())
		.collect::<Vec<_>>()
		.try_into()
		// Should never happen since we checked above.
		.map_err(|_| LinkedDidInfoProviderError::TooManyLinkedAccounts)
}

#[cfg(feature = "runtime-benchmarks")]
impl<Runtime, const MAX_LINKED_ACCOUNTS: u32> GetWorstCase<IdentityContext<Runtime::Identifier, Runtime::AccountId>>
	for LinkedDidInfoOf<Runtime, MAX_LINKED_ACCOUNTS>
where
	Runtime: did::Config<DidIdentifier = <Runtime as pallet_dip_provider::Config>::Identifier>
		+ pallet_web3_names::Config<Web3NameOwner = <Runtime as pallet_dip_provider::Config>::Identifier>
		+ pallet_did_lookup::Config<DidIdentifier = <Runtime as pallet_dip_provider::Config>::Identifier>
		+ pallet_dip_provider::Config
		+ pallet_balances::Config,
	<Runtime as frame_system::Config>::AccountId: Into<LinkableAccountId> + From<sp_core::sr25519::Public>,
	<Runtime as frame_system::Config>::AccountId: AsRef<[u8; 32]> + From<[u8; 32]>,
{
	fn worst_case(context: IdentityContext<Runtime::Identifier, Runtime::AccountId>) -> Self {
		use did::{
			did_details::DidVerificationKey,
			mock_utils::{generate_base_did_creation_details, get_key_agreement_keys},
		};
		use frame_benchmarking::{vec, Zero};
		use frame_support::traits::fungible::Mutate;
		use sp_io::crypto::{ed25519_generate, sr25519_generate};
		use sp_runtime::{traits::Get, KeyTypeId};

		use crate::constants::KILT;

		// Did Details.

		let submitter = context.submitter;
		let did = context.did;

		let amount = KILT * 100;

		// give some money
		<pallet_balances::Pallet<Runtime> as Mutate<<Runtime as frame_system::Config>::AccountId>>::set_balance(
			&submitter,
			amount.saturated_into(),
		);

		let max_new_keys = <Runtime as did::Config>::MaxNewKeyAgreementKeys::get();

		let new_key_agreement_keys = get_key_agreement_keys::<Runtime>(max_new_keys);

		let mut did_creation_details = generate_base_did_creation_details(did.clone(), submitter.clone());

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

		(0..MAX_LINKED_ACCOUNTS).for_each(|index| {
			let connected_acc = sr25519_generate(KeyTypeId(index.to_be_bytes()), None);
			let connected_acc_id: <Runtime as frame_system::Config>::AccountId = connected_acc.into();
			let linkable_id: LinkableAccountId = connected_acc_id.into();
			pallet_did_lookup::Pallet::<Runtime>::add_association(submitter.clone(), did.clone(), linkable_id.clone())
				.expect("association should not fail.");

			linked_accounts.push(linkable_id);
		});

		LinkedDidInfoOf {
			did_details,
			linked_accounts: linked_accounts
				.try_into()
				.expect("BoundedVec creation of linked accounts should not fail."),
			web3_name_details,
		}
	}
}
