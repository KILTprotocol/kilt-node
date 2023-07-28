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

use attestation::{AttestationDetails, AttestationDetailsOf};
use ctype::CtypeEntryOf;
use delegation::{benchmarking::setup_delegations, delegation_hierarchy::Permissions};
use did::{
	benchmarking::get_ed25519_public_authentication_key, did_details::DidVerificationKey,
	mock_utils::generate_base_did_details, DidIdentifierOf,
};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::{
	traits::{fungible::Mutate, Get, LockableCurrency, ReservableCurrency, WithdrawReasons},
	BoundedVec,
};
use frame_system::RawOrigin;
use kilt_support::{
	signature::VerifySignature,
	traits::{GenerateBenchmarkOrigin, GetWorstCase},
	Deposit,
};
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_web3_names::{Web3NameOf, Web3NameOwnerOf};
use parachain_staking::migrations::STAKING_ID;
use runtime_common::constants::KILT;
use sp_core::{sr25519, H256};
use sp_runtime::{
	traits::{Hash, IdentifyAccount},
	AccountId32, MultiSigner, SaturatedConversion,
};
use sp_std::{boxed::Box, num::NonZeroU32, vec, vec::Vec};

use crate::{mock::get_default_entries_to_migrate, *};

const SEED: u32 = 0;
const MICROKILT: u128 = 10u128.pow(9);
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

benchmarks! {
	where_clause {
		where
		T::AccountId: Into<LinkableAccountId> + Into<[u8; 32]>,
		T::Hash: From<H256>,
		T::OwnerOrigin: GenerateBenchmarkOrigin<<T as frame_system::Config>::RuntimeOrigin, T::AccountId, T::Web3NameOwner>,
		T: frame_system::Config,
		<T as attestation::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as delegation::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as did::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as pallet_did_lookup::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as pallet_web3_names::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as parachain_staking::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as parachain_staking::Config>::Currency: LockableCurrency<AccountIdOf<T>>,
		<T as public_credentials::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as did::Config>::DidIdentifier: From<AccountId32>,
		<T as frame_system::Config>::AccountId: From<AccountId32>,
		<T as public_credentials::Config>::EnsureOrigin: GenerateBenchmarkOrigin<<T as frame_system::Config>::RuntimeOrigin, T::AccountId, <T as public_credentials::Config>::AttesterId>,
		T::BlockNumber: From<u64>,
		<T as public_credentials:: Config>::SubjectId: GetWorstCase + sp_std::fmt::Debug + Into<Vec<u8>> ,
		T: ctype::Config<CtypeCreatorId = <T as attestation::Config>::AttesterId>,
		<T as delegation::Config>::DelegationNodeId: From<T::Hash>,
		<T as delegation::Config>::DelegationEntityId: From<sr25519::Public>,
		<<T as delegation::Config>::DelegationSignatureVerification as VerifySignature>::Signature: From<(
			T::DelegationEntityId,
			<<T as delegation::Config>::DelegationSignatureVerification as VerifySignature>::Payload,
		)>,
		<T as delegation::Config>::Currency: Mutate<T::AccountId>,
		<T as ctype::Config>::CtypeCreatorId: From<T::DelegationEntityId>,
		<T as delegation::Config>::EnsureOrigin: GenerateBenchmarkOrigin<<T as frame_system::Config>::RuntimeOrigin, T::AccountId, <T as delegation::Config>::DelegationEntityId>,
		<T as pallet_balances::Config>::HoldIdentifier: From<delegation::HoldReason> + From<pallet_did_lookup::HoldReason> + From<pallet_web3_names::HoldReason> + From<public_credentials::HoldReason>,
	}

	attestation_migration_weight {
		let sender: T::AccountId = account("sender", 0, SEED);
		let attester: <T as attestation::Config>::AttesterId = account("attester", 0, SEED);
		let claim_hash: T::Hash = T::Hashing::hash(b"claim");
		let ctype_hash: T::Hash = T::Hash::default();

		ctype::Ctypes::<T>::insert(ctype_hash, CtypeEntryOf::<T> {
			creator: attester.clone(),
			created_at: 0u64.into()
		});

		let details : AttestationDetailsOf<T> = AttestationDetails {
			attester,
			authorization_id: None,
			ctype_hash,
			revoked: false,
			deposit: Deposit {
				owner: sender.clone(),
				amount: MICROKILT.saturated_into(),
			},
		};

		pallet_balances::Pallet::<T>::set_balance(&sender, KILT.saturated_into());
		attestation::Attestations::<T>::insert(claim_hash, details.clone());
		pallet_balances::Pallet::<T>::reserve(&sender, details.deposit.amount.saturated_into::<u128>().saturated_into())
		.expect("User should have enough balance");

		let mut entries_to_migrate: EntriesToMigrate<T> = get_default_entries_to_migrate();
		entries_to_migrate.attestation = BoundedVec::try_from(vec![claim_hash]).expect("Vector initialization should not fail.");

		let origin= RawOrigin::Signed(sender);
	}: update_balance(origin, entries_to_migrate)
	verify {
		let migrated_key = Pallet::<T>::calculate_full_key(claim_hash.as_ref(), ATTESTATION_PALLET, ATTESTATION_STORAGE_NAME);
		assert!(MigratedKeys::<T>::contains_key(migrated_key));
	}

	delegation_migration_weight {
		let sender: T::AccountId = account("sender", 0, SEED);
		let (_, _, leaf_acc, leaf_id)  = setup_delegations::<T>(1,NonZeroU32::new(1).expect("NoneZero init should not fail"), Permissions::DELEGATE)?;

		let mut entries_to_migrate: EntriesToMigrate<T> = get_default_entries_to_migrate();
		entries_to_migrate.delegation = BoundedVec::try_from(vec![leaf_id]).expect("Vector initialization should not fail.");

		kilt_support::migration::translate_holds_to_reserve::<T>(delegation::HoldReason::Deposit.into());

		let origin= RawOrigin::Signed(sender);
	}: update_balance(origin, entries_to_migrate)
	verify {
		let migrated_key = Pallet::<T>::calculate_full_key(leaf_id.as_ref(), DELEGATION_PALLET, DELEGATION_STORAGE_NAME);
		assert!(MigratedKeys::<T>::contains_key(migrated_key));
	}

	did_migration_weight {
		let sender : AccountIdOf<T> = account("sender", 0, SEED);
		//create only one did. The migration for did, delegation, attestation, w3n, public credentials, and did lookup is the same.
		let did_public_auth_key = get_ed25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(did_public_auth_key).into_account().into();
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(did_public_auth_key), Some(sender.clone()));
		did_details.deposit.amount = <T as did::Config>::BaseDeposit::get();

		did::Did::<T>::insert(did_subject.clone(), did_details.clone());
		let initial_balance =  did_details.deposit.amount.saturated_into::<u128>().saturating_mul(2);
		pallet_balances::Pallet::<T>::set_balance(&sender, KILT.saturated_into());

		pallet_balances::Pallet::<T>::reserve(&sender, did_details.deposit.amount.saturated_into::<u128>().saturated_into())
			.expect("User should have enough balance");

		let mut entries_to_migrate: EntriesToMigrate<T> = get_default_entries_to_migrate();
		entries_to_migrate.did = BoundedVec::try_from(vec![did_subject.clone()]).expect("Vector initialization should not fail.");

		let origin= RawOrigin::Signed(sender);
	}: update_balance(origin, entries_to_migrate)
	verify {
		let migrated_key = Pallet::<T>::calculate_full_key(did_subject.as_ref(), b"did", b"Did" );
		assert!(MigratedKeys::<T>::contains_key(migrated_key));
	}

	did_lookup_migration_weight {
		let sender: T::AccountId = account("sender", 0, SEED);
		let linkable_id: LinkableAccountId = sender.clone().into();
		let did: <T as pallet_did_lookup::Config>::DidIdentifier = account("did", 0, SEED);

		pallet_balances::Pallet::<T>::set_balance(&sender, KILT.saturated_into());
		pallet_did_lookup::Pallet::<T>::add_association(sender.clone(), did, linkable_id.clone()).expect("should create association");
		kilt_support::migration::translate_holds_to_reserve::<T>(pallet_did_lookup::HoldReason::Deposit.into());

		let mut entries_to_migrate: EntriesToMigrate<T> = get_default_entries_to_migrate();
		entries_to_migrate.lookup = BoundedVec::try_from(vec![linkable_id.clone()]).expect("Vector initialization should not fail.");

		let origin= RawOrigin::Signed(sender);
	}: update_balance(origin, entries_to_migrate)
	verify {
		let migrated_key = Pallet::<T>::calculate_full_key(linkable_id.as_ref(), DID_LOOKUP_PALLET, DID_LOOKUP_STORAGE_NAME);
		assert!(MigratedKeys::<T>::contains_key(migrated_key));
	}

	w3n_migration_weight {
		let sender: AccountIdOf<T> = account("caller", 0, SEED);
		let owner: Web3NameOwnerOf<T> = account("owner", 0, SEED);
		let web3_name_input: BoundedVec<u8, <T as pallet_web3_names::Config>::MaxNameLength> = BoundedVec::try_from(vec![104, 101, 105, 105,111]).expect("Vector initialization should not fail.");
		let origin = <T as pallet_web3_names::Config>::OwnerOrigin::generate_origin(sender.clone(), owner);

		pallet_balances::Pallet::<T>::set_balance(&sender, KILT.saturated_into());
		pallet_web3_names::Pallet::<T>::claim(origin, web3_name_input.clone()).expect("Should register the claimed web3 name.");
		kilt_support::migration::translate_holds_to_reserve::<T>(pallet_web3_names::HoldReason::Deposit.into());
		let web3_name = Web3NameOf::<T>::try_from(web3_name_input.to_vec()).unwrap();

		let mut entries_to_migrate: EntriesToMigrate<T> = get_default_entries_to_migrate();
		entries_to_migrate.w3n = BoundedVec::try_from(vec![web3_name.clone()]).expect("Vector initialization should not fail.");

		let origin= RawOrigin::Signed(sender);
	}: update_balance(origin, entries_to_migrate)
	verify {
		let migrated_key = Pallet::<T>::calculate_full_key(web3_name.as_ref(), W3N_PALLET, W3N_STORAGE_NAME);
		assert!(MigratedKeys::<T>::contains_key(migrated_key));
	}



	staking_migration_weight {
		let caller : AccountIdOf<T> = account("caller", 0, SEED);
		let initial_lock = KILT.saturating_div(2);

		pallet_balances::Pallet::<T>::set_balance(&caller, KILT.saturated_into());
		pallet_balances::Pallet::<T>::set_lock(STAKING_ID, &caller, initial_lock.saturated_into(), WithdrawReasons::all());

		let mut entires_to_migrate: EntriesToMigrate<T> = get_default_entries_to_migrate();
		entires_to_migrate.staking =  BoundedVec::try_from(vec![caller.clone()]).expect("Vector initialization should not fail.");

		let origin= RawOrigin::Signed(caller.clone());
	}: update_balance(origin, entires_to_migrate)
	verify {
		let account_bytes: [u8; 32] = caller.into();
		let migrated_key = Pallet::<T>::calculate_full_key(&account_bytes, BALANCES_PALLET, BALANCES_STORAGE_NAME);
		assert!(MigratedKeys::<T>::contains_key(migrated_key));
	}

	public_credentials_migration_weight {
		let sender: AccountIdOf<T> = account("sender", 0, SEED);
		let attester: <T as public_credentials::Config>::AttesterId = account("attester", 0, SEED);
		let ctype_hash: T::Hash = T::Hash::default();
		let subject_id = <T as public_credentials::Config>::SubjectId::worst_case();
		let contents = BoundedVec::try_from(vec![0; <T as public_credentials::Config>::MaxEncodedClaimsLength::get() as usize]).expect("Contents should not fail.");
		let origin = <T as public_credentials::Config>::EnsureOrigin::generate_origin(sender.clone(), attester.clone());

		let creation_op = Box::new(public_credentials::mock::generate_base_public_credential_creation_op::<T>(
			subject_id.clone().into().try_into().expect("Input conversion should not fail."),
			ctype_hash,
			contents,
		));
		let credential_id = public_credentials::mock::generate_credential_id::<T>(&creation_op, &attester);

		ctype::Ctypes::<T>::insert(ctype_hash, CtypeEntryOf::<T> {
			creator: account("caller", 0, SEED),
			created_at: 0u64.into()
		});

		pallet_balances::Pallet::<T>::set_balance(&sender, KILT.saturated_into());
		public_credentials::Pallet::<T>::add(origin, creation_op).expect("Pallet::add should not fail");
		kilt_support::migration::translate_holds_to_reserve::<T>(public_credentials::HoldReason::Deposit.into());

		let mut entries_to_migrate: EntriesToMigrate<T> = get_default_entries_to_migrate();
		entries_to_migrate.public_credentials = BoundedVec::try_from(vec![(subject_id.clone(), credential_id.clone())]).expect("Vector initialization should not fail.");

		let origin_migration_pallet = RawOrigin::Signed(sender);
	}: update_balance(origin_migration_pallet, entries_to_migrate)
	verify {
		let key = Pallet::<T>::calculate_public_credentials_key(&subject_id, &credential_id);
		let migrated_key = Pallet::<T>::calculate_full_key(&key, PUBLIC_CREDENTIALS_PALLET, PUBLIC_CREDENTIALS_STORAGE_NAME);
		assert!(MigratedKeys::<T>::contains_key(migrated_key));
	}

}

impl_benchmark_test_suite! {
	Pallet,
	crate::mock::runtime::ExtBuilder::default().build_with_keystore(),
	crate::mock::runtime::Test
}
