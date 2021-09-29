// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

use codec::Encode;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, Zero};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use kilt_primitives::AccountId;
use sp_core::{crypto::KeyTypeId, ecdsa, ed25519, sr25519};
use sp_io::crypto::{ecdsa_generate, ecdsa_sign, ed25519_generate, ed25519_sign, sr25519_generate, sr25519_sign};
use sp_runtime::{traits::IdentifyAccount, MultiSigner};

use crate::{
	did_details::*,
	mock_utils::{
		generate_base_did_creation_details, generate_base_did_details, get_key_agreement_keys
	},
	*,
};

const DEFAULT_ACCOUNT_ID: &str = "tx_submitter";
const DEFAULT_ACCOUNT_SEED: u32 = 0;
const AUTHENTICATION_KEY_ID: KeyTypeId = KeyTypeId(*b"0000");
const ATTESTATION_KEY_ID: KeyTypeId = KeyTypeId(*b"0001");
const DELEGATION_KEY_ID: KeyTypeId = KeyTypeId(*b"0002");
const UNUSED_KEY_ID: KeyTypeId = KeyTypeId(*b"1111");

fn get_ed25519_public_authentication_key() -> ed25519::Public {
	ed25519_generate(AUTHENTICATION_KEY_ID, None)
}

fn get_sr25519_public_authentication_key() -> sr25519::Public {
	sr25519_generate(AUTHENTICATION_KEY_ID, None)
}

fn get_ecdsa_public_authentication_key() -> ecdsa::Public {
	ecdsa_generate(AUTHENTICATION_KEY_ID, None)
}

fn get_ed25519_public_attestation_key() -> ed25519::Public {
	ed25519_generate(ATTESTATION_KEY_ID, None)
}

fn get_sr25519_public_attestation_key() -> sr25519::Public {
	sr25519_generate(ATTESTATION_KEY_ID, None)
}

fn get_ecdsa_public_attestation_key() -> ecdsa::Public {
	ecdsa_generate(ATTESTATION_KEY_ID, None)
}

fn get_ed25519_public_delegation_key() -> ed25519::Public {
	ed25519_generate(DELEGATION_KEY_ID, None)
}

fn get_sr25519_public_delegation_key() -> sr25519::Public {
	sr25519_generate(DELEGATION_KEY_ID, None)
}

fn get_ecdsa_public_delegation_key() -> ecdsa::Public {
	ecdsa_generate(DELEGATION_KEY_ID, None)
}

// Must always be dispatched with the DID authentication key
fn generate_base_did_call_operation<T: Config>(
	did: DidIdentifierOf<T>,
	submitter: AccountIdentifierOf<T>,
) -> DidAuthorizedCallOperation<T> {
	let test_call = <T as Config>::Call::get_call_for_did_call_benchmark();

	DidAuthorizedCallOperation {
		did,
		call: test_call,
		tx_counter: 1u64,
		block_number: T::BlockNumber::default(),
		submitter,
	}
}

//TODO: We might want to extract the logic about which key is the longest
// encoded and which key takes the longest to verify and always use that.
benchmarks! {

	where_clause { where T::DidIdentifier: From<AccountId>, <T as frame_system::Config>::Origin: From<RawOrigin<T::DidIdentifier>>}

	/* create extrinsic */
	create_ed25519_keys {
		let n in 1 .. T::MaxNewKeyAgreementKeys::get();

		let submitter: AccountIdentifierOf<T> = account(DEFAULT_ACCOUNT_ID, 0, DEFAULT_ACCOUNT_SEED);

		let did_public_auth_key = get_ed25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(did_public_auth_key).into_account().into();
		let did_key_agreement_keys = get_key_agreement_keys::<T>(n);
		let did_public_att_key = get_ed25519_public_attestation_key();
		let did_public_del_key = get_ed25519_public_delegation_key();

		let mut did_creation_details = generate_base_did_creation_details::<T>(did_subject.clone());
		did_creation_details.new_key_agreement_keys = did_key_agreement_keys;
		did_creation_details.new_attestation_key = Some(DidVerificationKey::from(did_public_att_key));
		did_creation_details.new_delegation_key = Some(DidVerificationKey::from(did_public_del_key));

		let did_creation_signature = ed25519_sign(AUTHENTICATION_KEY_ID, &did_public_auth_key, did_creation_details.encode().as_ref()).expect("Failed to create DID signature from raw ed25519 signature.");
	}: create(RawOrigin::Signed(submitter), did_creation_details.clone(), DidSignature::from(did_creation_signature))
	verify {
		let stored_did = Did::<T>::get(&did_subject).expect("New DID should be stored on chain.");
		let stored_key_agreement_keys_ids = stored_did.key_agreement_keys;

		let expected_authentication_key_id = utils::calculate_key_id::<T>(&DidVerificationKey::from(did_public_auth_key).into());
		let expected_attestation_key_id = utils::calculate_key_id::<T>(&DidVerificationKey::from(did_public_att_key).into());
		let expected_delegation_key_id = utils::calculate_key_id::<T>(&DidVerificationKey::from(did_public_del_key).into());

		assert_eq!(
			stored_did.authentication_key,
			expected_authentication_key_id
		);
		for new_key in did_creation_details.new_key_agreement_keys.iter().copied() {
			assert!(
				stored_key_agreement_keys_ids.contains(&utils::calculate_key_id::<T>(&new_key.into())))
		}
		assert_eq!(
			stored_did.delegation_key,
			Some(expected_delegation_key_id)
		);
		assert_eq!(
			stored_did.attestation_key,
			Some(expected_attestation_key_id)
		);
		assert_eq!(stored_did.last_tx_counter, 0u64);
	}
	create_sr25519_keys {
		let n in 1 .. T::MaxNewKeyAgreementKeys::get();

		let submitter: AccountIdentifierOf<T> = account(DEFAULT_ACCOUNT_ID, 0, DEFAULT_ACCOUNT_SEED);

		let did_public_auth_key = get_sr25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(did_public_auth_key).into_account().into();
		let did_key_agreement_keys = get_key_agreement_keys::<T>(n);
		let did_public_att_key = get_sr25519_public_attestation_key();
		let did_public_del_key = get_sr25519_public_delegation_key();

		let mut did_creation_details = generate_base_did_creation_details::<T>(did_subject.clone());
		did_creation_details.new_key_agreement_keys = did_key_agreement_keys;
		did_creation_details.new_attestation_key = Some(DidVerificationKey::from(did_public_att_key));
		did_creation_details.new_delegation_key = Some(DidVerificationKey::from(did_public_del_key));

		let did_creation_signature = sr25519_sign(AUTHENTICATION_KEY_ID, &did_public_auth_key, did_creation_details.encode().as_ref()).expect("Failed to create DID signature from raw sr25519 signature.");
	}: create(RawOrigin::Signed(submitter), did_creation_details.clone(), DidSignature::from(did_creation_signature))
	verify {
		let stored_did = Did::<T>::get(&did_subject).expect("New DID should be stored on chain.");
		let stored_key_agreement_keys_ids = stored_did.key_agreement_keys;

		let expected_authentication_key_id = utils::calculate_key_id::<T>(&DidVerificationKey::from(did_public_auth_key).into());
		let expected_attestation_key_id = utils::calculate_key_id::<T>(&DidVerificationKey::from(did_public_att_key).into());
		let expected_delegation_key_id = utils::calculate_key_id::<T>(&DidVerificationKey::from(did_public_del_key).into());

		assert_eq!(
			stored_did.authentication_key,
			expected_authentication_key_id
		);
		for new_key in did_creation_details.new_key_agreement_keys.iter().copied() {
			assert!(
				stored_key_agreement_keys_ids.contains(&utils::calculate_key_id::<T>(&new_key.into())))
		}
		assert_eq!(
			stored_did.delegation_key,
			Some(expected_delegation_key_id)
		);
		assert_eq!(
			stored_did.attestation_key,
			Some(expected_attestation_key_id)
		);
		assert_eq!(stored_did.last_tx_counter, 0u64);
	}
	create_ecdsa_keys {
		let n in 1 .. T::MaxNewKeyAgreementKeys::get();

		let submitter: AccountIdentifierOf<T> = account(DEFAULT_ACCOUNT_ID, 0, DEFAULT_ACCOUNT_SEED);

		let did_public_auth_key = get_ecdsa_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(did_public_auth_key.clone()).into_account().into();
		let did_key_agreement_keys = get_key_agreement_keys::<T>(n);
		let did_public_att_key = get_ecdsa_public_attestation_key();
		let did_public_del_key = get_ecdsa_public_delegation_key();

		let mut did_creation_details = generate_base_did_creation_details::<T>(did_subject.clone());
		did_creation_details.new_key_agreement_keys = did_key_agreement_keys;
		did_creation_details.new_attestation_key = Some(DidVerificationKey::from(did_public_att_key.clone()));
		did_creation_details.new_delegation_key = Some(DidVerificationKey::from(did_public_del_key.clone()));

		let did_creation_signature = ecdsa_sign(AUTHENTICATION_KEY_ID, &did_public_auth_key, did_creation_details.encode().as_ref()).expect("Failed to create DID signature from raw ecdsa signature.");
	}: create(RawOrigin::Signed(submitter), did_creation_details.clone(), DidSignature::from(did_creation_signature))
	verify {
		let stored_did = Did::<T>::get(&did_subject).expect("New DID should be stored on chain.");
		let stored_key_agreement_keys_ids = stored_did.key_agreement_keys;

		let expected_authentication_key_id = utils::calculate_key_id::<T>(&DidVerificationKey::from(did_public_auth_key).into());
		let expected_attestation_key_id = utils::calculate_key_id::<T>(&DidVerificationKey::from(did_public_att_key).into());
		let expected_delegation_key_id = utils::calculate_key_id::<T>(&DidVerificationKey::from(did_public_del_key).into());

		assert_eq!(
			stored_did.authentication_key,
			expected_authentication_key_id
		);
		for new_key in did_creation_details.new_key_agreement_keys.iter().copied() {
			assert!(
				stored_key_agreement_keys_ids.contains(&utils::calculate_key_id::<T>(&new_key.into())))
		}
		assert_eq!(
			stored_did.delegation_key,
			Some(expected_delegation_key_id)
		);
		assert_eq!(
			stored_did.attestation_key,
			Some(expected_attestation_key_id)
		);
		assert_eq!(stored_did.last_tx_counter, 0u64);
	}

	delete {
		let did_public_auth_key = get_ed25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(did_public_auth_key).into_account().into();

		let did_details = generate_base_did_details::<T>(DidVerificationKey::from(did_public_auth_key));
		Did::<T>::insert(&did_subject, did_details);
	}: _(RawOrigin::Signed(did_subject.clone()))
	verify {
		assert!(
			Did::<T>::get(&did_subject).is_none()
		);
	}

	/* submit_did_call extrinsic */
	submit_did_call_ed25519_key {
		let submitter: AccountIdentifierOf<T> = account(DEFAULT_ACCOUNT_ID, 0, DEFAULT_ACCOUNT_SEED);

		let did_public_auth_key = get_ed25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(did_public_auth_key).into_account().into();

		let did_details = generate_base_did_details::<T>(DidVerificationKey::from(did_public_auth_key));
		Did::<T>::insert(&did_subject, did_details);

		let did_call_op = generate_base_did_call_operation::<T>(did_subject, submitter.clone());

		let did_call_signature = ed25519_sign(AUTHENTICATION_KEY_ID, &did_public_auth_key, did_call_op.encode().as_ref()).expect("Failed to create DID signature from raw ed25519 signature.");
	}: submit_did_call(RawOrigin::Signed(submitter), Box::new(did_call_op), DidSignature::from(did_call_signature))
	submit_did_call_sr25519_key {
		let submitter: AccountIdentifierOf<T> = account(DEFAULT_ACCOUNT_ID, 0, DEFAULT_ACCOUNT_SEED);

		let did_public_auth_key = get_sr25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(did_public_auth_key).into_account().into();

		let did_details = generate_base_did_details::<T>(DidVerificationKey::from(did_public_auth_key));
		Did::<T>::insert(&did_subject, did_details);

		let did_call_op = generate_base_did_call_operation::<T>(did_subject, submitter.clone());

		let did_call_signature = sr25519_sign(AUTHENTICATION_KEY_ID, &did_public_auth_key, did_call_op.encode().as_ref()).expect("Failed to create DID signature from raw sr25519 signature.");
	}: submit_did_call(RawOrigin::Signed(submitter), Box::new(did_call_op), DidSignature::from(did_call_signature))
	submit_did_call_ecdsa_key {
		let submitter: AccountIdentifierOf<T> = account(DEFAULT_ACCOUNT_ID, 0, DEFAULT_ACCOUNT_SEED);

		let did_public_auth_key = get_ecdsa_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(did_public_auth_key.clone()).into_account().into();

		let did_details = generate_base_did_details::<T>(DidVerificationKey::from(did_public_auth_key.clone()));
		Did::<T>::insert(&did_subject, did_details);

		let did_call_op = generate_base_did_call_operation::<T>(did_subject, submitter.clone());

		let did_call_signature = ecdsa_sign(AUTHENTICATION_KEY_ID, &did_public_auth_key, did_call_op.encode().as_ref()).expect("Failed to create DID signature from raw ecdsa signature.");
	}: submit_did_call(RawOrigin::Signed(submitter), Box::new(did_call_op), DidSignature::from(did_call_signature))

	/* set_authentication_key extrinsic */
	set_ed25519_authentication_key {
		let block_number = T::BlockNumber::zero();
		let old_did_public_auth_key = get_ed25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(old_did_public_auth_key).into_account().into();

		// fill up public keys to its max size because max public keys = # of max key agreement keys + 3
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(old_did_public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(get_ed25519_public_attestation_key()), block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(get_ed25519_public_delegation_key()), block_number));

		Did::<T>::insert(&did_subject, did_details);

		let new_did_public_auth_key = ed25519_generate(UNUSED_KEY_ID, None);
	}: set_authentication_key(RawOrigin::Signed(did_subject.clone()), DidVerificationKey::from(new_did_public_auth_key))
	verify {
		let auth_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(DidVerificationKey::from(new_did_public_auth_key)));
		assert_eq!(Did::<T>::get(&did_subject).unwrap().authentication_key, auth_key_id);
	}
	set_sr25519_authentication_key {
		let block_number = T::BlockNumber::zero();
		let old_did_public_auth_key = get_sr25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(old_did_public_auth_key).into_account().into();

		// fill up public keys to its max size because max public keys = # of max key agreement keys + 3
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(old_did_public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(get_sr25519_public_attestation_key()), block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(get_sr25519_public_delegation_key()), block_number));

		Did::<T>::insert(&did_subject, did_details);

		let new_did_public_auth_key = sr25519_generate(UNUSED_KEY_ID, None);
	}: set_authentication_key(RawOrigin::Signed(did_subject.clone()), DidVerificationKey::from(new_did_public_auth_key))
	verify {
		let auth_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(DidVerificationKey::from(new_did_public_auth_key)));
		assert_eq!(Did::<T>::get(&did_subject).unwrap().authentication_key, auth_key_id);
	}
	set_ecdsa_authentication_key {
		let block_number = T::BlockNumber::zero();
		let old_did_public_auth_key = get_ecdsa_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(old_did_public_auth_key.clone()).into_account().into();

		// fill up public keys to its max size because max public keys = # of max key agreement keys + 3
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(old_did_public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(get_ecdsa_public_attestation_key()), block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(get_ecdsa_public_delegation_key()), block_number));

		Did::<T>::insert(&did_subject, did_details);

		let new_did_public_auth_key = ecdsa_generate(UNUSED_KEY_ID, None);
	}: set_authentication_key(RawOrigin::Signed(did_subject.clone()), DidVerificationKey::from(new_did_public_auth_key.clone()))
	verify {
		let auth_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(DidVerificationKey::from(new_did_public_auth_key)));
		assert_eq!(Did::<T>::get(&did_subject).unwrap().authentication_key, auth_key_id);
	}

	/* set_delegation_key extrinsic */
	set_ed25519_delegation_key {
		let block_number = T::BlockNumber::zero();
		let public_auth_key = get_ed25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(public_auth_key).into_account().into();
		let old_delegation_key = get_ed25519_public_delegation_key();
		let new_delegation_key = ed25519_generate(UNUSED_KEY_ID, None);

		// fill up public keys to its max size because max public keys = # of max key agreement keys + 3
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(get_ed25519_public_attestation_key()), block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(old_delegation_key), block_number));

		Did::<T>::insert(&did_subject, did_details);
	}: set_delegation_key(RawOrigin::Signed(did_subject.clone()), DidVerificationKey::from(new_delegation_key))
	verify {
		let new_delegation_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(DidVerificationKey::from(new_delegation_key)));
		assert_eq!(Did::<T>::get(&did_subject).unwrap().delegation_key, Some(new_delegation_key_id));
	}
	set_sr25519_delegation_key {
		let block_number = T::BlockNumber::zero();
		let public_auth_key = get_sr25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(public_auth_key).into_account().into();
		let old_delegation_key = get_sr25519_public_delegation_key();
		let new_delegation_key = sr25519_generate(UNUSED_KEY_ID, None);

		// fill up public keys to its max size because max public keys = # of max key agreement keys + 3
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(get_sr25519_public_attestation_key()), block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(old_delegation_key), block_number));

		Did::<T>::insert(&did_subject, did_details);
	}: set_delegation_key(RawOrigin::Signed(did_subject.clone()), DidVerificationKey::from(new_delegation_key))
	verify {
		let new_delegation_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(DidVerificationKey::from(new_delegation_key)));
		assert_eq!(Did::<T>::get(&did_subject).unwrap().delegation_key, Some(new_delegation_key_id));
	}
	set_ecdsa_delegation_key {
		let block_number = T::BlockNumber::zero();
		let public_auth_key = get_ecdsa_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(public_auth_key.clone()).into_account().into();
		let old_delegation_key = get_ecdsa_public_delegation_key();
		let new_delegation_key = ecdsa_generate(UNUSED_KEY_ID, None);

		// fill up public keys to its max size because max public keys = # of max key agreement keys + 3
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(get_ecdsa_public_attestation_key()), block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(old_delegation_key), block_number));

		Did::<T>::insert(&did_subject, did_details);
	}: set_delegation_key(RawOrigin::Signed(did_subject.clone()), DidVerificationKey::from(new_delegation_key.clone()))
	verify {
		let new_delegation_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(DidVerificationKey::from(new_delegation_key)));
		assert_eq!(Did::<T>::get(&did_subject).unwrap().delegation_key, Some(new_delegation_key_id));
	}

	/* remove_delegation_key extrinsic */
	remove_ed25519_delegation_key {
		let block_number = T::BlockNumber::zero();
		let public_auth_key = get_ed25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(public_auth_key).into_account().into();
		let old_delegation_key = get_ed25519_public_delegation_key();

		// fill up public keys to its max size because max public keys = # of max key agreement keys + 3
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(get_ed25519_public_attestation_key()), block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(old_delegation_key), block_number));

		Did::<T>::insert(&did_subject, did_details);
	}: remove_delegation_key(RawOrigin::Signed(did_subject.clone()))
	verify {
		let did_details = Did::<T>::get(&did_subject).unwrap();
		let delegation_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(DidVerificationKey::from(old_delegation_key)));
		assert!(did_details.delegation_key.is_none());
		assert!(!did_details.public_keys.contains_key(&delegation_key_id));
	}
	remove_sr25519_delegation_key {
		let block_number = T::BlockNumber::zero();
		let public_auth_key = get_sr25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(public_auth_key).into_account().into();
		let old_delegation_key = get_sr25519_public_delegation_key();

		// fill up public keys to its max size because max public keys = # of max key agreement keys + 3
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(get_sr25519_public_attestation_key()), block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(old_delegation_key), block_number));

		Did::<T>::insert(&did_subject, did_details);
	}: remove_delegation_key(RawOrigin::Signed(did_subject.clone()))
	verify {
		let did_details = Did::<T>::get(&did_subject).unwrap();
		let delegation_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(DidVerificationKey::from(old_delegation_key)));
		assert!(did_details.delegation_key.is_none());
		assert!(!did_details.public_keys.contains_key(&delegation_key_id));
	}
	remove_ecdsa_delegation_key {
		let block_number = T::BlockNumber::zero();
		let public_auth_key = get_ecdsa_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(public_auth_key.clone()).into_account().into();
		let old_delegation_key = get_ecdsa_public_delegation_key();

		// fill up public keys to its max size because max public keys = # of max key agreement keys + 3
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(get_ecdsa_public_attestation_key()), block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(old_delegation_key.clone()), block_number));

		Did::<T>::insert(&did_subject, did_details);
	}: remove_delegation_key(RawOrigin::Signed(did_subject.clone()))
	verify {
		let did_details = Did::<T>::get(&did_subject).unwrap();
		let delegation_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(DidVerificationKey::from(old_delegation_key)));
		assert!(did_details.delegation_key.is_none());
		assert!(!did_details.public_keys.contains_key(&delegation_key_id));
	}

	/* set_attestation_key extrinsic */
	set_ed25519_attestation_key {
		let block_number = T::BlockNumber::zero();
		let public_auth_key = get_ed25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(public_auth_key).into_account().into();
		let old_attestation_key = get_ed25519_public_attestation_key();
		let new_attestation_key = ed25519_generate(UNUSED_KEY_ID, None);

		// fill up public keys to its max size because max public keys = # of max key agreement keys + 3
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get()), block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(get_ed25519_public_delegation_key()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(old_attestation_key), block_number));

		Did::<T>::insert(&did_subject, did_details);
	}: set_attestation_key(RawOrigin::Signed(did_subject.clone()), DidVerificationKey::from(new_attestation_key))
	verify {
		let new_attestation_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(DidVerificationKey::from(new_attestation_key)));
		assert_eq!(Did::<T>::get(&did_subject).unwrap().attestation_key, Some(new_attestation_key_id));
	}
	set_sr25519_attestation_key {
		let block_number = T::BlockNumber::zero();
		let public_auth_key = get_sr25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(public_auth_key).into_account().into();
		let old_attestation_key = get_sr25519_public_attestation_key();
		let new_attestation_key = sr25519_generate(UNUSED_KEY_ID, None);

		// fill up public keys to its max size because max public keys = # of max key agreement keys + 3
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get()), block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(get_sr25519_public_delegation_key()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(old_attestation_key), block_number));

		Did::<T>::insert(&did_subject, did_details);
	}: set_attestation_key(RawOrigin::Signed(did_subject.clone()), DidVerificationKey::from(new_attestation_key))
	verify {
		let new_attestation_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(DidVerificationKey::from(new_attestation_key)));
		assert_eq!(Did::<T>::get(&did_subject).unwrap().attestation_key, Some(new_attestation_key_id));
	}
	set_ecdsa_attestation_key {
		let block_number = T::BlockNumber::zero();
		let public_auth_key = get_ecdsa_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(public_auth_key.clone()).into_account().into();
		let old_attestation_key = get_ecdsa_public_attestation_key();
		let new_attestation_key = ecdsa_generate(UNUSED_KEY_ID, None);

		// fill up public keys to its max size because max public keys = # of max key agreement keys + 3
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get()), block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(get_ecdsa_public_delegation_key()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(old_attestation_key), block_number));

		Did::<T>::insert(&did_subject, did_details);
	}: set_attestation_key(RawOrigin::Signed(did_subject.clone()), DidVerificationKey::from(new_attestation_key.clone()))
	verify {
		let new_attestation_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(DidVerificationKey::from(new_attestation_key)));
		assert_eq!(Did::<T>::get(&did_subject).unwrap().attestation_key, Some(new_attestation_key_id));
	}

	/* remove_attestation_key extrinsic */
	remove_ed25519_attestation_key {
		let block_number = T::BlockNumber::zero();
		let public_auth_key = get_ed25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(public_auth_key).into_account().into();
		let old_attestation_key = get_ed25519_public_attestation_key();

		// fill up public keys to its max size because max public keys = # of max key agreement keys + 3
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get()), block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(get_ed25519_public_delegation_key()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(old_attestation_key), block_number));

		Did::<T>::insert(&did_subject, did_details);
	}: remove_attestation_key(RawOrigin::Signed(did_subject.clone()))
	verify {
		let did_details = Did::<T>::get(&did_subject).unwrap();
		let attestation_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(DidVerificationKey::from(old_attestation_key)));
		assert!(did_details.attestation_key.is_none());
		assert!(!did_details.public_keys.contains_key(&attestation_key_id));
	}
	remove_sr25519_attestation_key {
		let block_number = T::BlockNumber::zero();
		let public_auth_key = get_sr25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(public_auth_key).into_account().into();
		let old_attestation_key = get_sr25519_public_attestation_key();

		// fill up public keys to its max size because max public keys = # of max key agreement keys + 3
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get()), block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(get_sr25519_public_delegation_key()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(old_attestation_key), block_number));

		Did::<T>::insert(&did_subject, did_details);
	}: remove_attestation_key(RawOrigin::Signed(did_subject.clone()))
	verify {
		let did_details = Did::<T>::get(&did_subject).unwrap();
		let attestation_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(DidVerificationKey::from(old_attestation_key)));
		assert!(did_details.attestation_key.is_none());
		assert!(!did_details.public_keys.contains_key(&attestation_key_id));
	}
	remove_ecdsa_attestation_key {
		let block_number = T::BlockNumber::zero();
		let public_auth_key = get_ecdsa_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(public_auth_key.clone()).into_account().into();
		let old_attestation_key = get_ecdsa_public_attestation_key();

		// fill up public keys to its max size because max public keys = # of max key agreement keys + 3
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get()), block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(get_ecdsa_public_delegation_key()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(old_attestation_key.clone()), block_number));

		Did::<T>::insert(&did_subject, did_details);
	}: remove_attestation_key(RawOrigin::Signed(did_subject.clone()))
	verify {
		let did_details = Did::<T>::get(&did_subject).unwrap();
		let attestation_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(DidVerificationKey::from(old_attestation_key)));
		assert!(did_details.attestation_key.is_none());
		assert!(!did_details.public_keys.contains_key(&attestation_key_id));
	}

	/* add_key_agreement_keys extrinsic */
	add_ed25519_key_agreement_key {
		let block_number = T::BlockNumber::zero();
		let public_auth_key = get_ed25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(public_auth_key).into_account().into();
		let mut key_agreement_keys = get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get());

		// remove first entry
		let new_key_agreement_key = *key_agreement_keys.clone().into_inner().iter().next().unwrap();
		assert!(key_agreement_keys.remove(&new_key_agreement_key));

		// fill up public keys to its max minus one size (due to removal of new key_agreement_key)
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(get_ed25519_public_delegation_key()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(get_ed25519_public_attestation_key()), block_number));

		Did::<T>::insert(&did_subject, did_details);
	}: add_key_agreement_key(RawOrigin::Signed(did_subject.clone()), new_key_agreement_key)
	verify {
		let new_key_agreement_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(new_key_agreement_key));
		assert!(Did::<T>::get(&did_subject).unwrap().key_agreement_keys.contains(&new_key_agreement_key_id));
	}
	add_sr25519_key_agreement_key {
		let block_number = T::BlockNumber::zero();
		let public_auth_key = get_sr25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(public_auth_key).into_account().into();
		let mut key_agreement_keys = get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get());

		// remove first entry
		let new_key_agreement_key = *key_agreement_keys.clone().into_inner().iter().next().unwrap();
		assert!(key_agreement_keys.remove(&new_key_agreement_key));

		// fill up public keys to its max minus one size (due to removal of new key_agreement_key)
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(get_sr25519_public_delegation_key()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(get_sr25519_public_attestation_key()), block_number));

		Did::<T>::insert(&did_subject, did_details);
	}: add_key_agreement_key(RawOrigin::Signed(did_subject.clone()), new_key_agreement_key)
	verify {
		let new_key_agreement_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(new_key_agreement_key));
		assert!(Did::<T>::get(&did_subject).unwrap().key_agreement_keys.contains(&new_key_agreement_key_id));
	}
	add_ecdsa_key_agreement_key {
		let block_number = T::BlockNumber::zero();
		let public_auth_key = get_ecdsa_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(public_auth_key.clone()).into_account().into();
		let mut key_agreement_keys = get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get());

		// remove first entry
		let new_key_agreement_key = *key_agreement_keys.clone().into_inner().iter().next().unwrap();
		assert!(key_agreement_keys.remove(&new_key_agreement_key));

		// fill up public keys to its max minus one size (due to removal of new key_agreement_key)
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(get_ecdsa_public_delegation_key()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(get_ecdsa_public_attestation_key()), block_number));

		Did::<T>::insert(&did_subject, did_details);
	}: add_key_agreement_key(RawOrigin::Signed(did_subject.clone()), new_key_agreement_key)
	verify {
		let new_key_agreement_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(new_key_agreement_key));
		assert!(Did::<T>::get(&did_subject).unwrap().key_agreement_keys.contains(&new_key_agreement_key_id));
	}

	/* remove_key_agreement_keys extrinsic */
	remove_ed25519_key_agreement_key {
		let block_number = T::BlockNumber::zero();
		let public_auth_key = get_ed25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(public_auth_key).into_account().into();
		let key_agreement_keys = get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get());

		// get first entry
		let key_agreement_key = *key_agreement_keys.clone().into_inner().iter().next().unwrap();
		let key_agreement_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(key_agreement_key));

		// fill up public keys to its max size because max public keys = # of max key agreement keys + 3
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(get_ed25519_public_delegation_key()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(get_ed25519_public_attestation_key()), block_number));

		Did::<T>::insert(&did_subject, did_details);
	}: remove_key_agreement_key(RawOrigin::Signed(did_subject.clone()), key_agreement_key_id)
	verify {
		assert!(!Did::<T>::get(&did_subject).unwrap().key_agreement_keys.contains(&key_agreement_key_id));
	}
	remove_sr25519_key_agreement_key {
		let block_number = T::BlockNumber::zero();
		let public_auth_key = get_sr25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(public_auth_key).into_account().into();
		let key_agreement_keys = get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get());

		// get first entry
		let key_agreement_key = *key_agreement_keys.clone().into_inner().iter().next().unwrap();
		let key_agreement_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(key_agreement_key));

		// fill up public keys to its max size because max public keys = # of max key agreement keys + 3
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(get_sr25519_public_delegation_key()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(get_sr25519_public_attestation_key()), block_number));

		Did::<T>::insert(&did_subject, did_details);
	}: remove_key_agreement_key(RawOrigin::Signed(did_subject.clone()), key_agreement_key_id)
	verify {
		assert!(!Did::<T>::get(&did_subject).unwrap().key_agreement_keys.contains(&key_agreement_key_id));
	}
	remove_ecdsa_key_agreement_key {
		let block_number = T::BlockNumber::zero();
		let public_auth_key = get_ecdsa_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(public_auth_key.clone()).into_account().into();
		let key_agreement_keys = get_key_agreement_keys::<T>(T::MaxNewKeyAgreementKeys::get());

		// get first entry
		let key_agreement_key = *key_agreement_keys.clone().into_inner().iter().next().unwrap();
		let key_agreement_key_id = utils::calculate_key_id::<T>(&DidPublicKey::from(key_agreement_key));

		// fill up public keys to its max size because max public keys = # of max key agreement keys + 3
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(public_auth_key));
		assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, block_number));
		assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(get_ecdsa_public_delegation_key()), block_number));
		assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(get_ecdsa_public_attestation_key()), block_number));

		Did::<T>::insert(&did_subject, did_details);
	}: remove_key_agreement_key(RawOrigin::Signed(did_subject.clone()), key_agreement_key_id)
	verify {
		assert!(!Did::<T>::get(&did_subject).unwrap().key_agreement_keys.contains(&key_agreement_key_id));
	}
}

impl_benchmark_test_suite! {
	Pallet,
	crate::mock::ExtBuilder::default().build_with_keystore(None),
	crate::mock::Test
}
