// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

use frame_support::{assert_noop, assert_ok};
use pallet_dip_consumer::traits::IdentityProofVerifier;
use sp_runtime::AccountId32;

use crate::{
	parachain::v0::mock::{
		call, cross_chain_proof_with_authentication_key_and_web3_name, subject, submitter, wrong_call, wrong_submitter,
		ExtBuilder, TestRuntime, Verifier, GENESIS_HASH, IDENTITY_DETAILS, MAX_DID_MERKLE_LEAVES_REVEALED,
		MAX_DID_MERKLE_PROOF_LEAVE_COUNT, MAX_DID_MERKLE_PROOF_LEAVE_SIZE, MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT,
		MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE, MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT, MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE,
		RELAY_BLOCK, RELAY_STATE_ROOT, WRONG_GENESIS_HASH, WRONG_IDENTITY_DETAILS, WRONG_SIGNATURE_VALID_UNTIL,
	},
	state_proofs::MerkleProofError,
	DipParachainStateProofVerifierError, Error, RevealedAccountId,
};

#[test]
fn verify_proof_for_call_against_details_successful() {
	let details = &mut IDENTITY_DETAILS.clone();
	let proof = cross_chain_proof_with_authentication_key_and_web3_name();

	ExtBuilder::default()
		.with_genesis_hash(GENESIS_HASH)
		.with_relay_roots(vec![(RELAY_BLOCK, RELAY_STATE_ROOT)])
		.build()
		.execute_with(|| {
			assert_ok!(
				<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
					&call(),
					&subject(),
					&submitter(),
					details,
					proof,
				)
			);
			// If details are none, they are inizialited with their default value.
			assert_eq!(*details, Some(u32::default()));
		})
}

#[test]
fn verify_proof_for_call_against_details_relay_proof_too_many_leaves() {
	let details = &mut IDENTITY_DETAILS.clone();
	let proof = {
		let mut proof = cross_chain_proof_with_authentication_key_and_web3_name();
		let leaves_count = proof.provider_head_proof.proof.len();
		// Extend the relaychain proof to include MAX + 1 leaves, causing the proof
		// verification to fail
		proof.provider_head_proof.proof.extend(vec![
			vec![0u8; MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE as usize];
			MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT as usize - leaves_count + 1
		]);
		proof
	};

	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
				&call(),
				&subject(),
				&submitter(),
				details,
				proof,
			),
			DipParachainStateProofVerifierError::ProofComponentTooLarge(0)
		);
	})
}

#[test]
fn verify_proof_for_call_against_details_relay_proof_leaf_too_large() {
	let details = &mut IDENTITY_DETAILS.clone();
	let proof = {
		let mut proof = cross_chain_proof_with_authentication_key_and_web3_name();
		let last_leave = proof.provider_head_proof.proof.last_mut().unwrap();
		let last_leave_size = last_leave.len();
		// Extend the last leaf of the relaychain proof to include MAX + 1 bytes,
		// causing the proof verification to fail
		last_leave.extend(vec![
			0u8;
			MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE as usize - last_leave_size + 1
		]);
		proof
	};

	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
				&call(),
				&subject(),
				&submitter(),
				details,
				proof,
			),
			DipParachainStateProofVerifierError::ProofComponentTooLarge(1)
		);
	})
}

#[test]
fn verify_proof_for_call_against_details_relay_root_not_found() {
	let details = &mut IDENTITY_DETAILS.clone();
	let proof = cross_chain_proof_with_authentication_key_and_web3_name();

	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
				&call(),
				&subject(),
				&submitter(),
				details,
				proof,
			),
			DipParachainStateProofVerifierError::ProofVerification(Error::RelayStateRootNotFound)
		);
	})
}

#[test]
fn verify_proof_for_call_against_details_relay_proof_invalid() {
	let details = &mut IDENTITY_DETAILS.clone();
	let proof = {
		let mut proof = cross_chain_proof_with_authentication_key_and_web3_name();
		// Reset the provider head proof to an empty proof.
		proof.provider_head_proof.proof = Default::default();
		proof
	};

	ExtBuilder::default()
		.with_relay_roots(vec![(RELAY_BLOCK, RELAY_STATE_ROOT)])
		.build()
		.execute_with(|| {
			assert_noop!(
				<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
					&call(),
					&subject(),
					&submitter(),
					details,
					proof,
				),
				DipParachainStateProofVerifierError::ProofVerification(Error::ParaHeadMerkleProof(
					MerkleProofError::InvalidProof
				))
			);
		})
}

#[test]
fn verify_proof_for_call_against_details_parachain_proof_too_many_leaves() {
	let details = &mut IDENTITY_DETAILS.clone();
	let proof = {
		let mut proof = cross_chain_proof_with_authentication_key_and_web3_name();
		let leaves_count = proof.dip_commitment_proof.0.len();
		// Extend the DIP commitment proof to include MAX + 1 leaves, causing the proof
		// verification to fail
		proof.dip_commitment_proof.0.extend(vec![
			vec![0u8; MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE as usize];
			MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT as usize - leaves_count + 1
		]);
		proof
	};

	ExtBuilder::default()
		.with_relay_roots(vec![(RELAY_BLOCK, RELAY_STATE_ROOT)])
		.build()
		.execute_with(|| {
			assert_noop!(
				<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
					&call(),
					&subject(),
					&submitter(),
					details,
					proof,
				),
				DipParachainStateProofVerifierError::ProofComponentTooLarge(2)
			);
		})
}

#[test]
fn verify_proof_for_call_against_details_parachain_proof_leaf_too_large() {
	let details = &mut IDENTITY_DETAILS.clone();
	let proof = {
		let mut proof = cross_chain_proof_with_authentication_key_and_web3_name();
		let last_leave = proof.dip_commitment_proof.0.last_mut().unwrap();
		let last_leave_size = last_leave.len();
		// Extend the last leaf of the parachain proof to include MAX + 1 bytes,
		// causing the proof verification to fail
		last_leave.extend(vec![
			0u8;
			MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE as usize - last_leave_size + 1
		]);
		proof
	};

	ExtBuilder::default()
		.with_relay_roots(vec![(RELAY_BLOCK, RELAY_STATE_ROOT)])
		.build()
		.execute_with(|| {
			assert_noop!(
				<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
					&call(),
					&subject(),
					&submitter(),
					details,
					proof,
				),
				DipParachainStateProofVerifierError::ProofComponentTooLarge(3)
			);
		})
}

#[test]
fn verify_proof_for_call_against_details_parachain_proof_invalid() {
	let details = &mut IDENTITY_DETAILS.clone();
	let proof = {
		let mut proof = cross_chain_proof_with_authentication_key_and_web3_name();
		// Reset the DIP commitment proof to an empty proof.
		proof.dip_commitment_proof.0 = Default::default();
		proof
	};

	ExtBuilder::default()
		.with_relay_roots(vec![(RELAY_BLOCK, RELAY_STATE_ROOT)])
		.build()
		.execute_with(|| {
			assert_noop!(
				<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
					&call(),
					&subject(),
					&submitter(),
					details,
					proof,
				),
				DipParachainStateProofVerifierError::ProofVerification(Error::DipCommitmentMerkleProof(
					MerkleProofError::InvalidProof
				))
			);
		})
}

#[test]
fn verify_proof_for_call_against_details_dip_proof_too_many_leaves() {
	let details = &mut IDENTITY_DETAILS.clone();
	let proof = {
		let mut proof = cross_chain_proof_with_authentication_key_and_web3_name();
		let leaves_count = proof.dip_proof.blinded.len();
		// Extend the DIP proof to include MAX + 1 leaves, causing the proof
		// verification to fail
		proof.dip_proof.blinded.extend(vec![
			vec![0u8; MAX_DID_MERKLE_PROOF_LEAVE_SIZE as usize];
			MAX_DID_MERKLE_PROOF_LEAVE_COUNT as usize - leaves_count + 1
		]);
		proof
	};

	ExtBuilder::default()
		.with_relay_roots(vec![(RELAY_BLOCK, RELAY_STATE_ROOT)])
		.build()
		.execute_with(|| {
			assert_noop!(
				<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
					&call(),
					&subject(),
					&submitter(),
					details,
					proof,
				),
				DipParachainStateProofVerifierError::ProofComponentTooLarge(4)
			);
		})
}

#[test]
fn verify_proof_for_call_against_details_dip_proof_leaf_too_large() {
	let details = &mut IDENTITY_DETAILS.clone();
	let proof = {
		let mut proof = cross_chain_proof_with_authentication_key_and_web3_name();
		let last_leave = proof.dip_proof.blinded.last_mut().unwrap();
		let last_leave_size = last_leave.len();
		// Extend the last leaf of the parachain proof to include MAX + 1 bytes,
		// causing the proof verification to fail
		last_leave.extend(vec![
			0u8;
			MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE as usize - last_leave_size + 1
		]);
		proof
	};

	ExtBuilder::default()
		.with_relay_roots(vec![(RELAY_BLOCK, RELAY_STATE_ROOT)])
		.build()
		.execute_with(|| {
			assert_noop!(
				<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
					&call(),
					&subject(),
					&submitter(),
					details,
					proof,
				),
				DipParachainStateProofVerifierError::ProofComponentTooLarge(5)
			);
		})
}

#[test]
fn verify_proof_for_call_against_details_dip_proof_too_many_revealed_keys() {
	let details = &mut IDENTITY_DETAILS.clone();
	let proof = {
		let mut proof = cross_chain_proof_with_authentication_key_and_web3_name();
		let leaves_count = proof.dip_proof.revealed.len();
		// Extend the DIP proof to include MAX + 1 revealed leaves, causing the proof
		// verification to fail
		proof.dip_proof.revealed.extend(vec![
			RevealedAccountId(AccountId32::new([100; 32]).into()).into();
			MAX_DID_MERKLE_LEAVES_REVEALED as usize - leaves_count + 1
		]);
		proof
	};

	ExtBuilder::default()
		.with_relay_roots(vec![(RELAY_BLOCK, RELAY_STATE_ROOT)])
		.build()
		.execute_with(|| {
			assert_noop!(
				<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
					&call(),
					&subject(),
					&submitter(),
					details,
					proof,
				),
				DipParachainStateProofVerifierError::ProofVerification(Error::TooManyLeavesRevealed)
			);
		})
}

#[test]
fn verify_proof_for_call_against_details_dip_proof_invalid() {
	let details = &mut IDENTITY_DETAILS.clone();
	let proof = {
		let mut proof = cross_chain_proof_with_authentication_key_and_web3_name();
		// Reset the DIP proof to an empty proof.
		proof.dip_proof.blinded = Default::default();
		proof
	};

	ExtBuilder::default()
		.with_relay_roots(vec![(RELAY_BLOCK, RELAY_STATE_ROOT)])
		.build()
		.execute_with(|| {
			assert_noop!(
				<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
					&call(),
					&subject(),
					&submitter(),
					details,
					proof,
				),
				DipParachainStateProofVerifierError::ProofVerification(Error::InvalidDidMerkleProof)
			);
		})
}

#[test]
fn verify_proof_for_call_against_details_did_signature_signature_not_fresh() {
	let details = &mut IDENTITY_DETAILS.clone();
	let proof = cross_chain_proof_with_authentication_key_and_web3_name();

	ExtBuilder::default()
		// We get past the maximum block at which the signature is to be considered valid.
		.with_block_number(proof.signature.valid_until + 1)
		.with_relay_roots(vec![(RELAY_BLOCK, RELAY_STATE_ROOT)])
		.build()
		.execute_with(|| {
			assert_noop!(
				<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
					&call(),
					&subject(),
					&submitter(),
					details,
					proof,
				),
				DipParachainStateProofVerifierError::ProofVerification(Error::InvalidSignatureTime)
			);
		})
}

#[test]
fn verify_proof_for_call_against_details_did_signature_different_call() {
	let details = &mut IDENTITY_DETAILS.clone();
	let proof = cross_chain_proof_with_authentication_key_and_web3_name();

	ExtBuilder::default()
		.with_relay_roots(vec![(RELAY_BLOCK, RELAY_STATE_ROOT)])
		.with_genesis_hash(GENESIS_HASH)
		.build()
		.execute_with(|| {
			assert_noop!(
				<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
					// Different encoding for the call, will result in DID signature verification failure.
					&wrong_call(),
					&subject(),
					&submitter(),
					details,
					proof,
				),
				DipParachainStateProofVerifierError::ProofVerification(Error::InvalidDidKeyRevealed)
			);
		})
}

#[test]
fn verify_proof_for_call_against_details_did_signature_different_identity_details() {
	// Wrong details, will result in DID signature verification failure.
	let details = &mut WRONG_IDENTITY_DETAILS.clone();
	let proof = cross_chain_proof_with_authentication_key_and_web3_name();

	ExtBuilder::default()
		.with_relay_roots(vec![(RELAY_BLOCK, RELAY_STATE_ROOT)])
		.with_genesis_hash(GENESIS_HASH)
		.build()
		.execute_with(|| {
			assert_noop!(
				<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
					&call(),
					&subject(),
					&submitter(),
					details,
					proof,
				),
				DipParachainStateProofVerifierError::ProofVerification(Error::InvalidDidKeyRevealed)
			);
		})
}

#[test]
fn verify_proof_for_call_against_details_did_signature_different_submitter_address() {
	let details = &mut IDENTITY_DETAILS.clone();
	let proof = cross_chain_proof_with_authentication_key_and_web3_name();

	ExtBuilder::default()
		.with_relay_roots(vec![(RELAY_BLOCK, RELAY_STATE_ROOT)])
		.with_genesis_hash(GENESIS_HASH)
		.build()
		.execute_with(|| {
			assert_noop!(
				<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
					&call(),
					&subject(),
					// Different submitter, will result in DID signature verification failure.
					&wrong_submitter(),
					details,
					proof,
				),
				DipParachainStateProofVerifierError::ProofVerification(Error::InvalidDidKeyRevealed)
			);
		})
}

#[test]
fn verify_proof_for_call_against_details_did_signature_different_signature_expiration() {
	let details = &mut IDENTITY_DETAILS.clone();
	let proof = {
		let mut proof = cross_chain_proof_with_authentication_key_and_web3_name();
		// Different signature expiration, will result in DID signature verification
		// failure.
		proof.signature.valid_until = WRONG_SIGNATURE_VALID_UNTIL;
		proof
	};

	ExtBuilder::default()
		.with_relay_roots(vec![(RELAY_BLOCK, RELAY_STATE_ROOT)])
		.with_genesis_hash(GENESIS_HASH)
		.build()
		.execute_with(|| {
			assert_noop!(
				<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
					&call(),
					&subject(),
					&submitter(),
					details,
					proof,
				),
				DipParachainStateProofVerifierError::ProofVerification(Error::InvalidDidKeyRevealed)
			);
		})
}

#[test]
fn verify_proof_for_call_against_details_did_signature_different_genesis_hash() {
	let details = &mut IDENTITY_DETAILS.clone();
	let proof = cross_chain_proof_with_authentication_key_and_web3_name();

	ExtBuilder::default()
		.with_relay_roots(vec![(RELAY_BLOCK, RELAY_STATE_ROOT)])
		// Different genesis hash, will result in DID signature verification failure.
		.with_genesis_hash(WRONG_GENESIS_HASH)
		.build()
		.execute_with(|| {
			assert_noop!(
				<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
					&call(),
					&subject(),
					&submitter(),
					details,
					proof,
				),
				DipParachainStateProofVerifierError::ProofVerification(Error::InvalidDidKeyRevealed)
			);
		})
}
