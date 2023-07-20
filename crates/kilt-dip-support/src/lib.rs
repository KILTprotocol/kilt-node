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

// TODO: Crate documentation

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::ensure;
use parity_scale_codec::{Decode, Encode, HasCompact};
use scale_info::TypeInfo;
use sp_core::{ConstU32, ConstU64, Get, Hasher, RuntimeDebug, U256};
use sp_runtime::{
	traits::{CheckedSub, Hash},
	BoundedVec, SaturatedConversion,
};
use sp_std::{marker::PhantomData, vec::Vec};
use sp_trie::{verify_trie_proof, LayoutV1};

use ::did::{
	did_details::{DidPublicKey, DidPublicKeyDetails, DidVerificationKey},
	DidVerificationKeyRelationship,
};
use pallet_dip_consumer::traits::IdentityProofVerifier;

use crate::merkle::{MerkleProof, ProofLeaf, RevealedDidKey, RevealedWeb3Name, VerificationResult};

pub mod did;
pub mod merkle;
// pub mod state_proofs;
pub mod traits;

#[derive(Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo, Clone)]
pub struct ParaRootProof<RelayBlockHeight> {
	relay_block_height: RelayBlockHeight,
	proof: Vec<Vec<u8>>,
}

// pub struct StateProofDipVerifier<
// 	RelayInfoProvider,
// 	ParaInfoProvider,
// 	ProviderKeyId,
// 	ProviderParaIdProvider,
// 	ProviderBlockNumber,
// 	ProviderAccountIdentityDetails,
// 	ProviderWeb3Name,
// 	ProviderHasher,
// 	ProviderLinkedAccountId,
// 	ConsumerAccountId,
// 	ConsumerBlockNumber,
// 	ConsumerBlockNumberProvider,
// 	ConsumerGenesisHashProvider,
// 	CallVerifier,
// 	const MAX_REVEALED_KEYS_COUNT: u32,
// 	const MAX_REVEALED_ACCOUNTS_COUNT: u32,
// 	const SIGNATURE_VALIDITY: u64,
// 	SignedExtra = (),
// 	SignedExtraProvider = (),
// >(
// 	#[allow(clippy::type_complexity)]
// 	PhantomData<(
// 		RelayInfoProvider,
// 		ParaInfoProvider,
// 		ProviderKeyId,
// 		ProviderParaIdProvider,
// 		ProviderBlockNumber,
// 		ProviderAccountIdentityDetails,
// 		ProviderWeb3Name,
// 		ProviderHasher,
// 		ProviderLinkedAccountId,
// 		ConsumerAccountId,
// 		ConsumerBlockNumber,
// 		ConsumerBlockNumberProvider,
// 		ConsumerGenesisHashProvider,
// 		CallVerifier,
// 		ConstU32<MAX_REVEALED_KEYS_COUNT>,
// 		ConstU32<MAX_REVEALED_ACCOUNTS_COUNT>,
// 		ConstU64<SIGNATURE_VALIDITY>,
// 		SignedExtra,
// 		SignedExtraProvider,
// 	)>,
// );

// impl<
// 		Call,
// 		Subject,
// 		RelayInfoProvider,
// 		ParaInfoProvider,
// 		ProviderKeyId,
// 		ProviderParaIdProvider,
// 		ProviderBlockNumber,
// 		ProviderAccountIdentityDetails,
// 		ProviderWeb3Name,
// 		ProviderHasher,
// 		ProviderLinkedAccountId,
// 		ConsumerAccountId,
// 		ConsumerBlockNumber,
// 		ConsumerBlockNumberProvider,
// 		ConsumerGenesisHashProvider,
// 		CallVerifier,
// 		const MAX_REVEALED_KEYS_COUNT: u32,
// 		const MAX_REVEALED_ACCOUNTS_COUNT: u32,
// 		const SIGNATURE_VALIDITY: u64,
// 		SignedExtra,
// 		SignedExtraProvider,
// 	> IdentityProofVerifier<Call, Subject>
// 	for StateProofDipVerifier<
// 		RelayInfoProvider,
// 		ParaInfoProvider,
// 		ProviderKeyId,
// 		ProviderParaIdProvider,
// 		ProviderBlockNumber,
// 		ProviderAccountIdentityDetails,
// 		ProviderWeb3Name,
// 		ProviderHasher,
// 		ProviderLinkedAccountId,
// 		ConsumerAccountId,
// 		ConsumerBlockNumber,
// 		ConsumerBlockNumberProvider,
// 		ConsumerGenesisHashProvider,
// 		CallVerifier,
// 		MAX_REVEALED_KEYS_COUNT,
// 		MAX_REVEALED_ACCOUNTS_COUNT,
// 		SIGNATURE_VALIDITY,
// 		SignedExtra,
// 		SignedExtraProvider,
// 	> where
// 	RelayInfoProvider: RelayChainStateInfoProvider,
// 	RelayInfoProvider::Hasher: 'static,
// 	<RelayInfoProvider::Hasher as Hash>::Output: Ord,
// 	RelayInfoProvider::BlockNumber: Copy + Into<U256> + TryFrom<U256> +
// HasCompact, 	RelayInfoProvider::Key: AsRef<[u8]>,
// 	ParaInfoProvider: ParachainStateInfoProvider<Identifier = Subject, Commitment
// = ProviderHasher::Out>, 	ParaInfoProvider::Hasher: 'static,
// 	<ParaInfoProvider::Hasher as Hash>::Output: Ord
// 		+ Encode
// 		+ PartialEq<<RelayInfoProvider::Hasher as Hash>::Output>
// 		+ From<<RelayInfoProvider::Hasher as Hash>::Output>,
// 	ParaInfoProvider::Commitment: Decode,
// 	ParaInfoProvider::Key: AsRef<[u8]>,
// 	ProviderKeyId: Encode + Clone,
// 	ProviderBlockNumber: Encode + Clone,
// 	ProviderWeb3Name: Encode + Clone,
// 	ProviderLinkedAccountId: Encode + Clone,
// 	ProviderHasher: Hasher,
// 	ProviderParaIdProvider: Get<RelayInfoProvider::ParaId>,
// 	Call: Encode,
// 	ConsumerAccountId: Encode,
// 	ProviderAccountIdentityDetails: Encode + Bump + Default,
// 	ConsumerBlockNumber: CheckedSub + Into<u64> + Encode,
// 	ConsumerBlockNumberProvider: Get<ConsumerBlockNumber>,
// 	ConsumerGenesisHashProvider: Get<<ParaInfoProvider::Hasher as Hash>::Output>,
// 	SignedExtra: Encode,
// 	SignedExtraProvider: Get<SignedExtra>,
// 	CallVerifier: DidDipOriginFilter<Call, OriginInfo = (DidVerificationKey,
// DidVerificationKeyRelationship)>, {
// 	// TODO: Better error handling
// 	type Error = ();
// 	type IdentityDetails = ProviderAccountIdentityDetails;
// 	type Proof = DipSiblingParachainStateProof<
// 		RelayInfoProvider::BlockNumber,
// 		MerkleLeavesAndDidSignature<
// 			MerkleProof<
// 				Vec<Vec<u8>>,
// 				ProofLeaf<ProviderKeyId, ProviderBlockNumber, ProviderWeb3Name,
// ProviderLinkedAccountId>, 			>,
// 			ConsumerBlockNumber,
// 		>,
// 	>;
// 	type Submitter = ConsumerAccountId;
// 	type VerificationResult = VerificationResult<
// 		ProviderKeyId,
// 		ProviderBlockNumber,
// 		ProviderWeb3Name,
// 		ProviderLinkedAccountId,
// 		MAX_REVEALED_KEYS_COUNT,
// 		MAX_REVEALED_ACCOUNTS_COUNT,
// 	>;

// 	fn verify_proof_for_call_against_details(
// 		call: &Call,
// 		subject: &Subject,
// 		submitter: &Self::Submitter,
// 		identity_details: &mut Option<Self::IdentityDetails>,
// 		proof: Self::Proof,
// 	) -> Result<Self::VerificationResult, Self::Error> {
// 		let DipSiblingParachainStateProof {
// 			para_root_proof,
// 			dip_commitment_proof,
// 			dip_proof,
// 		} = proof;
// 		// 1. Verify relay chain proof.
// 		let provider_parachain_header =
// 			state_proofs::relay_chain::ParachainHeadProofVerifier::<RelayInfoProvider,
// _,>::verify_proof_for_parachain( 				&ProviderParaIdProvider::get(),
// 				&para_root_proof.relay_block_height,
// 				para_root_proof.proof,
// 			)?;
// 		// 2. Verify parachain state proof.
// 		let subject_identity_commitment =
// 			state_proofs::parachain::DipCommitmentValueProofVerifier::<ParaInfoProvider>::verify_proof_for_identifier(
// 				subject,
// 				provider_parachain_header.state_root.into(),
// 				dip_commitment_proof,
// 			)?;

// 		// 3. Verify DIP identity proof (taken from existing implementation).
// 		let proof_leaves = dip_proof
// 			.merkle_leaves
// 			.revealed
// 			.iter()
// 			.map(|leaf| (leaf.encoded_key(), Some(leaf.encoded_value())))
// 			.collect::<Vec<(Vec<u8>, Option<Vec<u8>>)>>();
// 		verify_trie_proof::<LayoutV1<ProviderHasher>, _, _, _>(
// 			&subject_identity_commitment,
// 			&dip_proof.merkle_leaves.blinded,
// 			&proof_leaves,
// 		)
// 		.map_err(|_| ())?;
// 		#[allow(clippy::type_complexity)]
// 		let (did_keys, web3_name, linked_accounts): (
// 			BoundedVec<RevealedDidKey<ProviderKeyId, ProviderBlockNumber>,
// ConstU32<MAX_REVEALED_KEYS_COUNT>>, 			Option<RevealedWeb3Name<ProviderWeb3Name,
// ProviderBlockNumber>>, 			BoundedVec<ProviderLinkedAccountId,
// ConstU32<MAX_REVEALED_ACCOUNTS_COUNT>>, 		) = dip_proof.merkle_leaves.revealed.
// iter().try_fold( 			(
// 				BoundedVec::with_bounded_capacity(MAX_REVEALED_KEYS_COUNT.saturated_into()),
// 				None,
// 				BoundedVec::with_bounded_capacity(MAX_REVEALED_ACCOUNTS_COUNT.
// saturated_into()), 			),
// 			|(mut keys, web3_name, mut linked_accounts), leaf| match leaf {
// 				ProofLeaf::DidKey(key_id, key_value) => {
// 					keys.try_push(RevealedDidKey {
// 						// TODO: Avoid cloning if possible
// 						id: key_id.0.clone(),
// 						relationship: key_id.1,
// 						details: key_value.0.clone(),
// 					})
// 					.map_err(|_| ())?;
// 					Ok::<_, ()>((keys, web3_name, linked_accounts))
// 				}
// 				// TODO: Avoid cloning if possible
// 				ProofLeaf::Web3Name(revealed_web3_name, details) => Ok((
// 					keys,
// 					Some(RevealedWeb3Name {
// 						web3_name: revealed_web3_name.0.clone(),
// 						claimed_at: details.0.clone(),
// 					}),
// 					linked_accounts,
// 				)),
// 				ProofLeaf::LinkedAccount(account_id, _) => {
// 					linked_accounts.try_push(account_id.0.clone()).map_err(|_| ())?;
// 					Ok::<_, ()>((keys, web3_name, linked_accounts))
// 				}
// 			},
// 		)?;
// 		let verification_result = VerificationResult {
// 			did_keys,
// 			web3_name,
// 			linked_accounts,
// 		};

// 		// 4. Verify DID signature (taken from existing implementation).
// 		let block_number = ConsumerBlockNumberProvider::get();
// 		let is_signature_fresh =
// 			if let Some(blocks_ago_from_now) =
// block_number.checked_sub(&dip_proof.did_signature.block_number) {
// 				blocks_ago_from_now.into() <= SIGNATURE_VALIDITY
// 			} else {
// 				false
// 			};
// 		ensure!(is_signature_fresh, ());
// 		let encoded_payload = (
// 			call,
// 			&identity_details,
// 			submitter,
// 			&dip_proof.did_signature.block_number,
// 			ConsumerGenesisHashProvider::get(),
// 			SignedExtraProvider::get(),
// 		)
// 			.encode();
// 		let mut proof_verification_keys =
// verification_result.as_ref().iter().filter_map(|RevealedDidKey {
// 			relationship, details: DidPublicKeyDetails { key, .. }, .. } | {
// 				let DidPublicKey::PublicVerificationKey(key) = key else { return None };
// 					Some((key,
// DidVerificationKeyRelationship::try_from(*relationship).expect("Should never
// fail to build a VerificationRelationship from the given DidKeyRelationship
// because we have already made sure the conditions hold."))) 		});
// 		let valid_signing_key = proof_verification_keys.find(|(verification_key, _)|
// { 			verification_key
// 				.verify_signature(&encoded_payload, &dip_proof.did_signature.signature)
// 				.is_ok()
// 		});
// 		let Some((key, relationship)) = valid_signing_key else { return Err(()) };
// 		if let Some(details) = identity_details {
// 			details.bump();
// 		} else {
// 			*identity_details = Some(Self::IdentityDetails::default());
// 		}
// 		// 4.1 Verify call required relationship
// 		CallVerifier::check_call_origin_info(call, &(key.clone(),
// relationship)).map_err(|_| ())?; 		Ok(verification_result)
// 	}
// }
