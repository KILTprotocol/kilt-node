use did::{
	did_details::{DidDetails, DidPublicKeyDetails, DidVerificationKey},
	DidVerificationKeyRelationship, KeyIdOf,
};
use frame_support::{assert_err, assert_ok};
use kilt_dip_primitives::{
	DidKeyRelationship, DipDidProofWithVerifiedSubjectCommitment, RevealedDidKey, RevealedDidMerkleProofLeaf,
	RevealedWeb3Name, TimeBoundDidSignature,
};
use pallet_web3_names::Web3NameOf;
use parity_scale_codec::Encode;
use sp_core::{ed25519, sr25519, Pair};
use sp_runtime::AccountId32;

use crate::{
	constants::{
		did::{MAX_KEY_AGREEMENT_KEYS, MAX_PUBLIC_KEYS_PER_DID},
		dip_provider::MAX_LINKED_ACCOUNTS,
	},
	dip::{
		merkle::{v0::generate_proof, CompleteMerkleProof, DidMerkleProofError},
		mock::{create_linked_info, TestRuntime},
	},
	AccountId, BlockNumber, Hasher,
};

const MAX_LEAVES_REVEALED: u32 = MAX_LINKED_ACCOUNTS + MAX_PUBLIC_KEYS_PER_DID + 1;

// Verify if a given DID key revealed in a DIP proof matches the key from the
// provided DID Document. The comparison checks for the actual key information
// (public key and creation block number) and for its relationship to the DID
// Document.
fn do_stored_key_and_revealed_key_match(
	did_details: &DidDetails<TestRuntime>,
	stored_key: &DidPublicKeyDetails<BlockNumber, AccountId>,
	revealed_key: &RevealedDidKey<KeyIdOf<TestRuntime>, BlockNumber, AccountId>,
) -> bool {
	let RevealedDidKey {
		id: revealed_key_id,
		relationship: revealed_key_relationship,
		details: revealed_key_details,
	} = revealed_key;
	let is_same_key_material = revealed_key_details == stored_key;
	let is_of_right_relationship = match revealed_key_relationship {
		DidKeyRelationship::Encryption => did_details.key_agreement_keys.contains(revealed_key_id),
		DidKeyRelationship::Verification(DidVerificationKeyRelationship::Authentication) => {
			did_details.authentication_key == *revealed_key_id
		}
		DidKeyRelationship::Verification(DidVerificationKeyRelationship::AssertionMethod) => {
			did_details.attestation_key == Some(*revealed_key_id)
		}
		DidKeyRelationship::Verification(DidVerificationKeyRelationship::CapabilityDelegation) => {
			did_details.delegation_key == Some(*revealed_key_id)
		}
		DidKeyRelationship::Verification(DidVerificationKeyRelationship::CapabilityInvocation) => {
			panic!("DID document should not have any key for capability delegation.")
		}
	};
	is_same_key_material && is_of_right_relationship
}

#[test]
fn generate_proof_for_complete_linked_info() {
	let auth_key = ed25519::Pair::from_seed(&[10u8; 32]);
	let did_auth_key = DidVerificationKey::Ed25519(auth_key.public());
	let linked_info = create_linked_info(did_auth_key, Some(b"ntn_x2"), MAX_LINKED_ACCOUNTS);
	let signature = auth_key.sign(&().encode());

	// 1. Generate a proof over all the linked info.
	let CompleteMerkleProof { proof, root } = generate_proof(
		&linked_info,
		linked_info.did_details.public_keys.keys(),
		true,
		linked_info.linked_accounts.iter(),
	)
	.unwrap();
	let cross_chain_proof = DipDidProofWithVerifiedSubjectCommitment::new(
		root,
		proof,
		TimeBoundDidSignature::new(signature.clone().into(), 100),
	);

	let dip_origin_info = cross_chain_proof
		.verify_dip_proof::<Hasher, MAX_LEAVES_REVEALED>()
		.and_then(|r| r.verify_signature_time(&50))
		.and_then(|r| r.retrieve_signing_leaves_for_payload(&().encode()))
		.unwrap();
	// All key agreement keys, plus authentication, attestation, and delegation key,
	// plus all linked accounts, plus web3name.
	let expected_leaves_revealed = (MAX_KEY_AGREEMENT_KEYS + 3 + MAX_LINKED_ACCOUNTS + 1) as usize;
	assert_eq!(dip_origin_info.iter_leaves().count(), expected_leaves_revealed);

	let did_keys = dip_origin_info
		.iter_leaves()
		.cloned()
		.filter_map(|leaf| {
			if let RevealedDidMerkleProofLeaf::DidKey(key) = leaf {
				Some(key)
			} else {
				None
			}
		})
		.collect::<Vec<_>>();
	// Make sure the revealed keys all belong to the DID Document...
	assert!(did_keys.iter().all(|revealed_did_key| {
		let stored_key = linked_info.did_details.public_keys.get(&revealed_did_key.id).unwrap();
		do_stored_key_and_revealed_key_match(&linked_info.did_details, stored_key, revealed_did_key)
	}));
	// ...and that no key from the DID document is left out.
	assert!(linked_info
		.did_details
		.public_keys
		.iter()
		.all(|(stored_key_id, stored_key_details)| {
			let matching_revealed_key = did_keys.iter().find(|did_key| did_key.id == *stored_key_id).unwrap();
			do_stored_key_and_revealed_key_match(&linked_info.did_details, stored_key_details, matching_revealed_key)
		}));

	let web3names = dip_origin_info
		.iter_leaves()
		.cloned()
		.filter_map(|leaf| {
			if let RevealedDidMerkleProofLeaf::Web3Name(name) = leaf {
				Some(name)
			} else {
				None
			}
		})
		.collect::<Vec<_>>();
	// Make sure the only web3name is revealed and it is the correct one.
	assert_eq!(web3names.len(), 1);
	assert_eq!(
		web3names.first(),
		Some(&RevealedWeb3Name {
			web3_name: b"ntn_x2".to_vec().try_into().unwrap(),
			claimed_at: BlockNumber::default()
		})
	);

	let linked_accounts = dip_origin_info
		.iter_leaves()
		.cloned()
		.filter_map(|leaf| {
			if let RevealedDidMerkleProofLeaf::LinkedAccount(acc) = leaf {
				Some(acc)
			} else {
				None
			}
		})
		.collect::<Vec<_>>();
	// Make sure the revealed accounts all belong to the DID Document...
	assert!(linked_accounts
		.iter()
		.all(|revealed_account| { linked_info.linked_accounts.contains(&revealed_account.0) }));
	// ...and that no account from the ones linked to the DID document is left out.
	assert!(linked_info
		.linked_accounts
		.iter()
		.all(|linked_account| { linked_accounts.iter().any(|l| l.0 == *linked_account) }));

	// 2. Generate a proof without any parts revealed.
	let CompleteMerkleProof { proof, root } = generate_proof(&linked_info, [].iter(), false, [].iter()).unwrap();
	let cross_chain_proof = DipDidProofWithVerifiedSubjectCommitment::new(
		root,
		proof,
		TimeBoundDidSignature::new(signature.clone().into(), 100),
	);
	// Should verify the merkle proof successfully.
	assert_ok!(cross_chain_proof.verify_dip_proof::<Hasher, MAX_LEAVES_REVEALED>());

	// 3. Generate a proof with only the authentication key revealed.
	let CompleteMerkleProof { proof, root } = generate_proof(
		&linked_info,
		[linked_info.did_details.authentication_key].iter(),
		false,
		[].iter(),
	)
	.unwrap();
	let cross_chain_proof = DipDidProofWithVerifiedSubjectCommitment::new(
		root,
		proof,
		TimeBoundDidSignature::new(signature.clone().into(), 100),
	);

	let dip_origin_info = cross_chain_proof
		.verify_dip_proof::<Hasher, MAX_LEAVES_REVEALED>()
		.and_then(|r| r.verify_signature_time(&50))
		.and_then(|r| r.retrieve_signing_leaves_for_payload(&().encode()))
		.unwrap();
	// Only the authentication key.
	let expected_leaves_revealed = 1;
	assert_eq!(dip_origin_info.iter_leaves().count(), expected_leaves_revealed);

	let did_key = &dip_origin_info
		.iter_leaves()
		.cloned()
		.filter_map(|leaf| {
			if let RevealedDidMerkleProofLeaf::DidKey(key) = leaf {
				Some(key)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()[0];
	assert_eq!(did_key.id, linked_info.did_details.authentication_key);
	assert!(do_stored_key_and_revealed_key_match(
		&linked_info.did_details,
		linked_info
			.did_details
			.public_keys
			.get(&linked_info.did_details.authentication_key)
			.unwrap(),
		did_key
	));

	// 4. Generate a proof with only the web3name revealed.
	let CompleteMerkleProof { proof, root } = generate_proof(&linked_info, [].iter(), true, [].iter()).unwrap();
	let cross_chain_proof = DipDidProofWithVerifiedSubjectCommitment::new(
		root,
		proof,
		TimeBoundDidSignature::new(signature.clone().into(), 100),
	);
	// Should verify the merkle proof successfully.
	assert_ok!(cross_chain_proof.verify_dip_proof::<Hasher, MAX_LEAVES_REVEALED>());

	// 5. Generate a proof with only one linked account revealed.
	let CompleteMerkleProof { proof, root } = generate_proof(
		&linked_info,
		[].iter(),
		true,
		[linked_info.linked_accounts[0].clone()].iter(),
	)
	.unwrap();
	let cross_chain_proof = DipDidProofWithVerifiedSubjectCommitment::new(
		root,
		proof,
		TimeBoundDidSignature::new(signature.clone().into(), 100),
	);
	// Should verify the merkle proof successfully.
	assert_ok!(cross_chain_proof.verify_dip_proof::<Hasher, MAX_LEAVES_REVEALED>());

	// 6. Generate a proof with only the authentication key and the web3name
	//    revealed.
	let CompleteMerkleProof { proof, root } = generate_proof(
		&linked_info,
		[linked_info.did_details.authentication_key].iter(),
		true,
		[].iter(),
	)
	.unwrap();
	let cross_chain_proof = DipDidProofWithVerifiedSubjectCommitment::new(
		root,
		proof,
		TimeBoundDidSignature::new(signature.clone().into(), 100),
	);
	let dip_origin_info = cross_chain_proof
		.verify_dip_proof::<Hasher, MAX_LEAVES_REVEALED>()
		.and_then(|r| r.verify_signature_time(&50))
		.and_then(|r| r.retrieve_signing_leaves_for_payload(&().encode()))
		.unwrap();
	// The authentication key and the web3name.
	let expected_leaves_revealed = 2;
	assert_eq!(dip_origin_info.iter_leaves().count(), expected_leaves_revealed);

	let did_key = &dip_origin_info
		.iter_leaves()
		.cloned()
		.filter_map(|leaf| {
			if let RevealedDidMerkleProofLeaf::DidKey(key) = leaf {
				Some(key)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()[0];
	assert_eq!(did_key.id, linked_info.did_details.authentication_key);
	assert!(do_stored_key_and_revealed_key_match(
		&linked_info.did_details,
		linked_info
			.did_details
			.public_keys
			.get(&linked_info.did_details.authentication_key)
			.unwrap(),
		did_key
	));
	let web3_name = &dip_origin_info
		.iter_leaves()
		.cloned()
		.filter_map(|leaf| {
			if let RevealedDidMerkleProofLeaf::Web3Name(web3_name) = leaf {
				Some(web3_name)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()[0];
	assert_eq!(linked_info.web3_name_details.as_ref(), Some(web3_name));

	// 7. Generate a proof with only the authentication key and one linked account
	//    revealed.
	let CompleteMerkleProof { proof, root } = generate_proof(
		&linked_info,
		[linked_info.did_details.authentication_key].iter(),
		false,
		[linked_info.linked_accounts[0].clone()].iter(),
	)
	.unwrap();
	let cross_chain_proof =
		DipDidProofWithVerifiedSubjectCommitment::new(root, proof, TimeBoundDidSignature::new(signature.into(), 100));
	let dip_origin_info = cross_chain_proof
		.verify_dip_proof::<Hasher, MAX_LEAVES_REVEALED>()
		.and_then(|r| r.verify_signature_time(&50))
		.and_then(|r| r.retrieve_signing_leaves_for_payload(&().encode()))
		.unwrap();
	// The authentication key and the web3name.
	let expected_leaves_revealed = 2;
	assert_eq!(dip_origin_info.iter_leaves().count(), expected_leaves_revealed);

	let did_key = &dip_origin_info
		.iter_leaves()
		.cloned()
		.filter_map(|leaf| {
			if let RevealedDidMerkleProofLeaf::DidKey(key) = leaf {
				Some(key)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()[0];
	assert_eq!(did_key.id, linked_info.did_details.authentication_key);
	assert!(do_stored_key_and_revealed_key_match(
		&linked_info.did_details,
		linked_info
			.did_details
			.public_keys
			.get(&linked_info.did_details.authentication_key)
			.unwrap(),
		did_key
	));
	let linked_account = &dip_origin_info
		.iter_leaves()
		.cloned()
		.filter_map(|leaf| {
			if let RevealedDidMerkleProofLeaf::LinkedAccount(linked_account) = leaf {
				Some(linked_account)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()[0];
	assert!(linked_info.linked_accounts.contains(&linked_account.0));

	// 8. Fails to generate the proof for a key that does not exist.
	assert_err!(
		generate_proof(
			&linked_info,
			[KeyIdOf::<TestRuntime>::default()].iter(),
			false,
			[].iter(),
		),
		DidMerkleProofError::KeyNotFound
	);

	// 9. Fails to generate the proof for an account that does not exist.
	assert_err!(
		generate_proof(
			&linked_info,
			[].iter(),
			false,
			[AccountId32::new([u8::MAX; 32]).into()].iter(),
		),
		DidMerkleProofError::LinkedAccountNotFound
	);
}

#[test]
fn generate_proof_with_only_auth_key() {
	let auth_key = sr25519::Pair::from_seed(&[10u8; 32]);
	let did_auth_key = DidVerificationKey::Sr25519(auth_key.public());
	let linked_info = create_linked_info(did_auth_key, Option::<Web3NameOf<TestRuntime>>::None, 0);

	// 1. Fails to generate the proof for a key that does not exist.
	assert_err!(
		generate_proof(
			&linked_info,
			[KeyIdOf::<TestRuntime>::default()].iter(),
			false,
			[].iter(),
		),
		DidMerkleProofError::KeyNotFound
	);

	// 2. Fails to generate the proof for the web3name.
	assert_err!(
		generate_proof(&linked_info, [].iter(), true, [].iter(),),
		DidMerkleProofError::Web3NameNotFound
	);

	// 3. Fails to generate the proof for an account that does not exist.
	assert_err!(
		generate_proof(
			&linked_info,
			[].iter(),
			false,
			[AccountId32::new([u8::MAX; 32]).into()].iter(),
		),
		DidMerkleProofError::LinkedAccountNotFound
	);
}

#[test]
fn generate_proof_with_two_keys_with_same_id() {
	let auth_key = ed25519::Pair::from_seed(&[10u8; 32]);
	let did_auth_key = DidVerificationKey::Ed25519(auth_key.public());
	let linked_info = {
		let mut info = create_linked_info(did_auth_key.clone(), Option::<Web3NameOf<TestRuntime>>::None, 0);
		info.did_details
			.update_attestation_key(did_auth_key, BlockNumber::default())
			.unwrap();
		// Remove all key agreement keys
		let key_agreement_key_ids = info
			.did_details
			.key_agreement_keys
			.clone()
			.into_iter()
			.collect::<Vec<_>>();
		key_agreement_key_ids.into_iter().for_each(|k: sp_core::H256| {
			info.did_details.remove_key_agreement_key(k).unwrap();
		});
		// Remove delegation key, if present
		let _ = info.did_details.remove_delegation_key();
		info
	};
	let signature = auth_key.sign(&().encode());

	let CompleteMerkleProof { proof, root } = generate_proof(
		&linked_info,
		linked_info.did_details.public_keys.keys(),
		false,
		linked_info.linked_accounts.iter(),
	)
	.unwrap();
	let cross_chain_proof =
		DipDidProofWithVerifiedSubjectCommitment::new(root, proof, TimeBoundDidSignature::new(signature.into(), 100));

	let dip_origin_info = cross_chain_proof
		.verify_dip_proof::<Hasher, MAX_LEAVES_REVEALED>()
		.and_then(|r| r.verify_signature_time(&50))
		.and_then(|r| r.retrieve_signing_leaves_for_payload(&().encode()))
		.unwrap();
	// Authentication key and attestation key have the same key ID, but they are
	// different keys, so there should be 2 leaves.
	let expected_leaves_revealed = 2;
	assert_eq!(dip_origin_info.iter_leaves().count(), expected_leaves_revealed);

	let did_keys = {
		let mut did_keys = dip_origin_info
			.iter_leaves()
			.cloned()
			.filter_map(|leaf| {
				if let RevealedDidMerkleProofLeaf::DidKey(key) = leaf {
					Some(key)
				} else {
					None
				}
			})
			.collect::<Vec<_>>();
		did_keys.sort();
		did_keys
	};
	assert_eq!(
		did_keys,
		vec![
			RevealedDidKey {
				id: linked_info.did_details.authentication_key,
				relationship: DidVerificationKeyRelationship::Authentication.into(),
				details: linked_info
					.did_details
					.public_keys
					.get(&linked_info.did_details.authentication_key)
					.unwrap()
					.clone()
			},
			RevealedDidKey {
				id: linked_info.did_details.attestation_key.unwrap(),
				relationship: DidVerificationKeyRelationship::AssertionMethod.into(),
				details: linked_info
					.did_details
					.public_keys
					.get(&linked_info.did_details.attestation_key.unwrap())
					.unwrap()
					.clone()
			}
		]
	);
}
