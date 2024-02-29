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

use did::DidIdentifierOf;
use frame_support::{assert_noop, assert_ok};
use hex_literal::hex;
use pallet_dip_consumer::traits::IdentityProofVerifier;
use peregrine_runtime::Runtime as PeregrineRuntime;
use sp_core::crypto::Ss58Codec;
use sp_runtime::AccountId32;

use crate::{
	parachain::v0::mock::{
		correct_cross_chain_proof, ExtBuilder, RuntimeCall, TestRuntime, Verifier, MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT,
		MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE,
	},
	DipParachainStateProofVerifierError,
};

#[test]
fn verify_proof_for_call_against_details_successful() {
	let subject =
		DidIdentifierOf::<PeregrineRuntime>::from_ss58check("4p9S4FrPp4HATybUu6FoBaveQynGWzp8oTpJ5KYyfmYZ9RH4")
			.unwrap();
	let submitter = AccountId32::from_ss58check("4qbGXy3VNCxRywCooPHBCiqqC8eBCi8R61FhKMhQgfe6Pi7M").unwrap();
	let mut identity_details = Option::<u32>::None;
	let proof = correct_cross_chain_proof();

	ExtBuilder::default()
		.with_genesis_hash(hex!("fe0821e1c03846bdff40df39019205b2dce56dd0ccbff6f042d68832a56d358f").into())
		.with_relay_roots(vec![(
			21,
			hex!("23ed6624753dfc87f0721c867abfa77361636314a60d24e8e85b44072b89c3f6").into(),
		)])
		.build()
		.execute_with(|| {
			assert_ok!(
				<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
					&RuntimeCall::System(frame_system::Call::remark {
						remark: b"Hello, world!".to_vec(),
					}),
					&subject,
					&submitter,
					&mut identity_details,
					proof,
				)
			);
			// If details are none, they are inizialited with their default value.
			assert_eq!(identity_details, Some(u32::default()));
		})
}

#[test]
fn verify_proof_for_call_against_details_relay_proof_too_many_leaves() {
	let subject =
		DidIdentifierOf::<PeregrineRuntime>::from_ss58check("4p9S4FrPp4HATybUu6FoBaveQynGWzp8oTpJ5KYyfmYZ9RH4")
			.unwrap();
	let submitter = AccountId32::from_ss58check("4qbGXy3VNCxRywCooPHBCiqqC8eBCi8R61FhKMhQgfe6Pi7M").unwrap();
	let mut identity_details = Option::<u32>::None;
	let proof = {
		let mut proof = correct_cross_chain_proof();
		let leaves_count = proof.provider_head_proof.proof.len();
		// Extend the relaychain proof to include MAX + 1 leaves, causing the proof
		// verification to fail
		proof.provider_head_proof.proof.extend(
			vec![
				vec![0u8; MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE as usize];
				MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT as usize - leaves_count + 1
			]
			.into_iter(),
		);
		proof
	};

	ExtBuilder::default()
		.with_genesis_hash(hex!("fe0821e1c03846bdff40df39019205b2dce56dd0ccbff6f042d68832a56d358f").into())
		.with_relay_roots(vec![(
			21,
			hex!("23ed6624753dfc87f0721c867abfa77361636314a60d24e8e85b44072b89c3f6").into(),
		)])
		.build()
		.execute_with(|| {
			assert_noop!(
				<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
					&RuntimeCall::System(frame_system::Call::remark {
						remark: b"Hello, world!".to_vec(),
					}),
					&subject,
					&submitter,
					&mut identity_details,
					proof,
				),
				DipParachainStateProofVerifierError::ProofComponentTooLarge(0)
			);
		})
}

#[test]
fn verify_proof_for_call_against_details_relay_proof_leaf_too_large() {
	let subject =
		DidIdentifierOf::<PeregrineRuntime>::from_ss58check("4p9S4FrPp4HATybUu6FoBaveQynGWzp8oTpJ5KYyfmYZ9RH4")
			.unwrap();
	let submitter = AccountId32::from_ss58check("4qbGXy3VNCxRywCooPHBCiqqC8eBCi8R61FhKMhQgfe6Pi7M").unwrap();
	let mut identity_details = Option::<u32>::None;
	let proof = {
		let mut proof = correct_cross_chain_proof();
		let last_leave = proof.provider_head_proof.proof.last_mut().unwrap();
		let last_leave_size = last_leave.len();
		// Extend the last leaf of the relaychain proof to include MAX + 1 bytes,
		// causing the proof verification to fail
		last_leave.extend(vec![0u8; MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE as usize - last_leave_size + 1].into_iter());
		proof
	};

	ExtBuilder::default()
		.with_genesis_hash(hex!("fe0821e1c03846bdff40df39019205b2dce56dd0ccbff6f042d68832a56d358f").into())
		.with_relay_roots(vec![(
			21,
			hex!("23ed6624753dfc87f0721c867abfa77361636314a60d24e8e85b44072b89c3f6").into(),
		)])
		.build()
		.execute_with(|| {
			assert_noop!(
				<Verifier as IdentityProofVerifier<TestRuntime>>::verify_proof_for_call_against_details(
					&RuntimeCall::System(frame_system::Call::remark {
						remark: b"Hello, world!".to_vec(),
					}),
					&subject,
					&submitter,
					&mut identity_details,
					proof,
				),
				DipParachainStateProofVerifierError::ProofComponentTooLarge(1)
			);
		})
}
