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


use did::{did_details::DidVerificationKey, DidIdentifierOf};
use frame_benchmarking::v2::*;
use kilt_support::traits::GenerateBenchmarkOrigin;
use pallet_dip_provider::traits::IdentityProvider;
use sp_core::{ed25519::Public};
use sp_io::crypto::ed25519_generate;
use sp_runtime::{traits::IdentifyAccount, KeyTypeId, MultiSigner};
use frame_support::{traits::fungible::Mutate, BoundedVec};
use pallet_balances::Pallet as BalancePallet;
use sp_runtime::SaturatedConversion;
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_provider::traits::IdentityCommitmentGenerator;

use crate::{dip::{did::{Web3OwnershipOf ,DidIdentityProvider, DidWeb3NameProvider, DidLinkedAccountsProvider, LinkedDidInfoOf}, merkle::DidMerkleRootGenerator }, constants::KILT};

const AUTHENTICATION_KEY_ID: KeyTypeId = KeyTypeId(*b"0000");


pub trait Config: did::Config + frame_system::Config + pallet_balances::Config + pallet_web3_names::Config + pallet_did_lookup::Config {}
pub struct Pallet<T: Config>(did::Pallet<T>);


fn insert_w3n<T>(owner: <T as pallet_web3_names::Config>::Web3NameOwner , claimer: pallet_web3_names::AccountIdOf<T> ) 
	where 
		T: pallet_web3_names::Config + pallet_balances::Config,
		T::OwnerOrigin: GenerateBenchmarkOrigin<<T as frame_system::Config>::RuntimeOrigin, T::AccountId, T::Web3NameOwner> 
	{
	let web3_name_input: BoundedVec<u8, T::MaxNameLength> = BoundedVec::try_from(generate_web3_name_input(5.saturated_into())).expect("BoundedVec creation should not fail.");
	let origin_create = T::OwnerOrigin::generate_origin(claimer.clone(), owner.clone());
	let amount = KILT * 10;
	<BalancePallet<T> as Mutate<T::AccountId>>::set_balance(&claimer, amount.saturated_into());
	pallet_web3_names::Pallet::<T>::claim(origin_create, web3_name_input).expect("Claiming w3n should not fail.");	
}

fn insert_linked_acc<T>(did: <T as pallet_did_lookup::Config>::DidIdentifier,  caller: T::AccountId ) 
	where 
		T: pallet_did_lookup::Config + pallet_balances::Config,
		<T as frame_system::Config>::AccountId: AsRef<[u8; 32]> + From<[u8; 32]> + Into<LinkableAccountId>,
{
	
	let linkable_id: LinkableAccountId = caller.clone().into();
	
	let amount = KILT * 10;
	<BalancePallet<T> as Mutate<T::AccountId>>::set_balance(&caller, amount.saturated_into());
	pallet_did_lookup::Pallet::<T>::add_association(caller.clone(), did.clone(), linkable_id.clone()).expect("Inserting association should not fail.");
}


fn get_ed25519_public_authentication_key() -> Public {
	ed25519_generate(AUTHENTICATION_KEY_ID, None)
}
 
fn generate_web3_name_input(length: usize) -> Vec<u8> {
	vec![b'1'; length]
}

#[benchmarks(where 
		<T as did::Config>::DidIdentifier: From<sp_runtime::AccountId32> 
			+ Into<<T as pallet_web3_names::Config>::Web3NameOwner> 
			+ Into<<T as pallet_did_lookup::Config>::DidIdentifier>,
		<T as frame_system::Config>::AccountId: AsRef<[u8; 32]> + From<[u8; 32]> + Into<LinkableAccountId>,
		<T as frame_system::Config>::AccountId: From<sp_runtime::AccountId32>,
		<T as frame_system::Config>::Hash: From<[u8; 32]>, 
		T::OwnerOrigin: GenerateBenchmarkOrigin<<T as frame_system::Config>::RuntimeOrigin, T::AccountId, T::Web3NameOwner>,
		sp_runtime::AccountId32: From<<T as did::Config>::DidIdentifier>
	)]
pub mod benchmarks {
	
use frame_system::pallet_prelude::BlockNumberFor;

use super::{Config, Pallet, *};

	#[benchmark]
	fn retrieve_did() {
		let owner: <T as frame_system::Config>::AccountId = account("ALICE", 0, 0);
		let amount = KILT * 10;
		<BalancePallet<T> as Mutate<T::AccountId>>::set_balance(&owner, amount.saturated_into());
		let authentication_key = get_ed25519_public_authentication_key();
		let did_public_auth_key = get_ed25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(did_public_auth_key).into_account().into();
		let entry = did::mock_utils::generate_base_did_details::<T>(DidVerificationKey::from(authentication_key), Some(owner.clone()));
		did::Pallet::<T>::try_insert_did(did_subject.clone(), entry, owner).expect("Inserting DID should not fail.");

		#[block]
		{
			DidIdentityProvider::<T>::retrieve(&did_subject).expect("Retrieve DID should not fail.");
		}
	}

	#[benchmark]
	fn retrieve_w3n() {
	let claimer: pallet_web3_names::AccountIdOf<T> = account("ALICE", 0, 0);
	let owner: <T as pallet_web3_names::Config>::Web3NameOwner = account("BOB", 0, 0);
	insert_w3n::<T>(owner.clone(), claimer);
		#[block]
		{
			DidWeb3NameProvider::<T>::retrieve(&owner).expect("Retrieve w3n should not fail.");
		}
	}

	#[benchmark]
	fn retrieve_linked_accounts() {

		let did: <T as pallet_did_lookup::Config>::DidIdentifier = account("did", 0, 0);
		let caller: T::AccountId = account("caller", 0, 0);
		insert_linked_acc::<T>(did.clone(), caller);
	
		#[block]
		{
			DidLinkedAccountsProvider::<T>::retrieve(&did).expect("Retrieve linked accounts should not fail.");	
		}
	}

	#[benchmark]
	fn create_commitment() {

		// insert DID
		let owner: <T as frame_system::Config>::AccountId = account("ALICE", 0, 0);
		let amount = KILT * 10;
		<BalancePallet<T> as Mutate<T::AccountId>>::set_balance(&owner, amount.saturated_into());
		let authentication_key = get_ed25519_public_authentication_key();
		let did_public_auth_key = get_ed25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(did_public_auth_key).into_account().into();
		let entry = did::mock_utils::generate_base_did_details::<T>(DidVerificationKey::from(authentication_key.clone()), Some(owner.clone()));
		
		// Todo. fill up with keys.
		// let key: <T as frame_system::Config>::Hash = H256::from_slice(&[1;32]) .0.into();
		// entry.delegation_key = Some(key.clone());
		// entry.attestation_key = Some(key);
		
		
		// TODO
		// entry.key_agreement_keys = BoundedBTreeSet::try_from(vec![key]).expect("Did details setup should not fail.");

		did::Pallet::<T>::try_insert_did(did_subject.clone(), entry.clone(), owner.clone()).expect("Inserting DID should not fail.");
		

		// insert w3n 

		insert_w3n::<T>(did_subject.clone().into(), owner.clone());

		// insert linked acc 
		insert_linked_acc::<T>(did_subject.clone().into(), owner.clone());


		// prepare combined identity
		let web3_name = pallet_web3_names::Pallet::<T>::names::<<T as pallet_web3_names::Config>::Web3NameOwner>(did_subject.clone().into()).expect("w3n should be in storage");
		let w3n_ownership = Web3OwnershipOf::<T> {
			web3_name,
			claimed_at: BlockNumberFor::<T>::zero()
		};

		let identity = LinkedDidInfoOf::<T> {
		 	did_details: entry,
			web3_name_details: Some(w3n_ownership),
			linked_accounts: vec![owner.into()]
		};
		
		let version = 0;
 	
		#[block]
		{
			DidMerkleRootGenerator::<T>::generate_commitment(&did_subject.into(), &identity, version).expect("Generate commitment should not fail.");
		}
	}
}
