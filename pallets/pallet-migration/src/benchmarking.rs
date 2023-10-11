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
	traits::{
		fungible::{Inspect, Mutate},
		Get, ReservableCurrency,
	},
	BoundedVec,
};
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use kilt_support::{
	signature::VerifySignature,
	traits::{GenerateBenchmarkOrigin, GetWorstCase},
	Deposit,
};
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_web3_names::{Web3NameOf, Web3NameOwnerOf};
use runtime_common::constants::KILT;
use sp_core::{sr25519, H256};
use sp_runtime::{
	traits::{Hash, IdentifyAccount},
	AccountId32, MultiSigner, SaturatedConversion,
};
use sp_std::{boxed::Box, num::NonZeroU32, vec, vec::Vec};

use crate::*;

const SEED: u32 = 0;
const MICROKILT: u128 = 10u128.pow(9);
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

benchmarks! {
	where_clause {
		where
		T::AccountId: Into<LinkableAccountId>,
		T::Hash: From<H256>,
		T::OwnerOrigin: GenerateBenchmarkOrigin<<T as frame_system::Config>::RuntimeOrigin, T::AccountId, T::Web3NameOwner>,
		T: frame_system::Config,
		T: pallet_balances::Config,
		<T as attestation::Config>::Currency: ReservableCurrency<AccountIdOf<T>, Balance = <<T as attestation::Config>::Currency as Inspect<AccountIdOf<T>>>::Balance>,
		<T as delegation::Config>::Currency: ReservableCurrency<AccountIdOf<T>, Balance = <<T as delegation::Config>::Currency as Inspect<AccountIdOf<T>>>::Balance>,
		<T as did::Config>::Currency: ReservableCurrency<AccountIdOf<T>, Balance = <<T as did::Config>::Currency as Inspect<AccountIdOf<T>>>::Balance>,
		<T as pallet_did_lookup::Config>::Currency: ReservableCurrency<AccountIdOf<T>,Balance = <<T as pallet_did_lookup::Config>::Currency as Inspect<AccountIdOf<T>>>::Balance>,
		<T as pallet_web3_names::Config>::Currency: ReservableCurrency<AccountIdOf<T>, Balance = <<T as pallet_web3_names::Config>::Currency as Inspect<AccountIdOf<T>>>::Balance>,
		<T as public_credentials::Config>::Currency: ReservableCurrency<AccountIdOf<T>, Balance = <<T as public_credentials::Config>::Currency as Inspect<AccountIdOf<T>>>::Balance>,
		<T as did::Config>::DidIdentifier: From<AccountId32>,
		<T as frame_system::Config>::AccountId: From<AccountId32>,
		<T as public_credentials::Config>::EnsureOrigin: GenerateBenchmarkOrigin<<T as frame_system::Config>::RuntimeOrigin, T::AccountId, <T as public_credentials::Config>::AttesterId>,
		BlockNumberFor<T>: From<u64>,
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
		<T as pallet_balances::Config>::RuntimeHoldReason: From<delegation::HoldReason> + From<pallet_did_lookup::HoldReason> + From<pallet_web3_names::HoldReason> + From<public_credentials::HoldReason>,
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

		let entries_to_migrate = EntriesToMigrate {
			attestation : BoundedVec::try_from(vec![claim_hash]).expect("Vector initialization should not fail."),
			..Default::default()
		};

		let origin= RawOrigin::Signed(sender);
	}: update_balance(origin, entries_to_migrate)
	verify {}

	delegation_migration_weight {
		let sender: T::AccountId = account("sender", 0, SEED);
		let (_, _, leaf_acc, leaf_id)  = setup_delegations::<T>(1,NonZeroU32::new(1).expect("NoneZero init should not fail"), Permissions::DELEGATE)?;

		let entries_to_migrate = EntriesToMigrate {
			delegation: BoundedVec::try_from(vec![leaf_id]).expect("Vector initialization should not fail."),
			..Default::default()
		};

		kilt_support::migration::translate_holds_to_reserve::<T>(delegation::HoldReason::Deposit.into());

		let origin= RawOrigin::Signed(sender);
	}: update_balance(origin, entries_to_migrate)
	verify {}

	did_migration_weight {
		let sender : AccountIdOf<T> = account("sender", 0, SEED);
		//create only one did. The migration for did, delegation, attestation, w3n, public credentials, and did lookup is the same.
		let did_public_auth_key = get_ed25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(did_public_auth_key).into_account().into();
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(did_public_auth_key), Some(sender.clone()));
		did_details.deposit.amount = MICROKILT.saturated_into();

		did::Did::<T>::insert(did_subject.clone(), did_details.clone());
		pallet_balances::Pallet::<T>::set_balance(&sender, KILT.saturated_into());

		pallet_balances::Pallet::<T>::reserve(&sender, did_details.deposit.amount.saturated_into::<u128>().saturated_into())
			.expect("User should have enough balance");

		let entries_to_migrate = EntriesToMigrate {
			did: BoundedVec::try_from(vec![did_subject]).expect("Vector initialization should not fail."),
			..Default::default()
		};


		let origin= RawOrigin::Signed(sender);
	}: update_balance(origin, entries_to_migrate)
	verify {}

	did_lookup_migration_weight {
		let sender: T::AccountId = account("sender", 0, SEED);
		let linkable_id: LinkableAccountId = sender.clone().into();
		let did: <T as pallet_did_lookup::Config>::DidIdentifier = account("did", 0, SEED);

		pallet_balances::Pallet::<T>::set_balance(&sender, KILT.saturated_into());
		pallet_did_lookup::Pallet::<T>::add_association(sender.clone(), did, linkable_id.clone()).expect("should create association");
		kilt_support::migration::translate_holds_to_reserve::<T>(pallet_did_lookup::HoldReason::Deposit.into());

		let entries_to_migrate = EntriesToMigrate{
			lookup: BoundedVec::try_from(vec![linkable_id]).expect("Vector initialization should not fail."),
			..Default::default()
		};


		let origin= RawOrigin::Signed(sender);
	}: update_balance(origin, entries_to_migrate)
	verify {}

	w3n_migration_weight {
		let sender: AccountIdOf<T> = account("caller", 0, SEED);
		let owner: Web3NameOwnerOf<T> = account("owner", 0, SEED);
		let web3_name_input: BoundedVec<u8, <T as pallet_web3_names::Config>::MaxNameLength> = BoundedVec::try_from(vec![104, 101, 105, 105,111]).expect("Vector initialization should not fail.");
		let origin = <T as pallet_web3_names::Config>::OwnerOrigin::generate_origin(sender.clone(), owner);

		pallet_balances::Pallet::<T>::set_balance(&sender, KILT.saturated_into());
		pallet_web3_names::Pallet::<T>::claim(origin, web3_name_input.clone()).expect("Should register the claimed web3 name.");
		kilt_support::migration::translate_holds_to_reserve::<T>(pallet_web3_names::HoldReason::Deposit.into());
		let web3_name = Web3NameOf::<T>::try_from(web3_name_input.to_vec()).unwrap();

		let entries_to_migrate = EntriesToMigrate {
			w3n: BoundedVec::try_from(vec![web3_name]).expect("Vector initialization should not fail."),
			..Default::default()
		};

		let origin= RawOrigin::Signed(sender);
	}: update_balance(origin, entries_to_migrate)
	verify {}

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

		let entries_to_migrate = EntriesToMigrate{
			public_credentials: BoundedVec::try_from(vec![(subject_id, credential_id)]).expect("Vector initialization should not fail."),
			..Default::default()
		};

		let origin_migration_pallet = RawOrigin::Signed(sender);
	}: update_balance(origin_migration_pallet, entries_to_migrate)
	verify {}

}

impl_benchmark_test_suite! {
	Pallet,
	crate::mock::runtime::ExtBuilder::default().build_with_keystore(),
	crate::mock::runtime::Test
}
