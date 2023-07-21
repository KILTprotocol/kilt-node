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
use pallet_dip_provider::traits::IdentityProvider;
use parity_scale_codec::{Decode, Encode, HasCompact};
use scale_info::TypeInfo;
use sp_core::{ConstU32, Get, RuntimeDebug, U256};
use sp_runtime::{traits::CheckedSub, BoundedVec, SaturatedConversion};
use sp_std::{marker::PhantomData, vec::Vec};
use sp_trie::{verify_trie_proof, LayoutV1};

use ::did::{
	did_details::{DidPublicKey, DidPublicKeyDetails, DidVerificationKey},
	DidVerificationKeyRelationship,
};
use pallet_dip_consumer::traits::IdentityProofVerifier;

use crate::{
	did::TimeBoundDidSignature,
	merkle::{MerkleProof, ProofLeaf, RevealedDidKey, RevealedWeb3Name, VerificationResult},
	traits::{
		Bump, DidDipOriginFilter, DidSignatureVerifierContextProvider, OutputOf, ParachainStateInfoProvider,
		RelayChainStateInfoProvider,
	},
};

pub mod did;
pub mod merkle;
pub mod state_proofs;
pub mod traits;

pub struct CombinedIdentityResult<OutputA, OutputB, OutputC> {
	pub a: OutputA,
	pub b: OutputB,
	pub c: OutputC,
}

impl<OutputA, OutputB, OutputC> From<(OutputA, OutputB, OutputC)>
	for CombinedIdentityResult<OutputA, OutputB, OutputC>
{
	fn from(value: (OutputA, OutputB, OutputC)) -> Self {
		Self {
			a: value.0,
			b: value.1,
			c: value.2,
		}
	}
}

impl<OutputA, OutputB, OutputC> CombinedIdentityResult<OutputA, OutputB, OutputC>
where
	OutputB: Default,
	OutputC: Default,
{
	pub fn from_a(a: OutputA) -> Self {
		Self {
			a,
			b: OutputB::default(),
			c: OutputC::default(),
		}
	}
}

impl<OutputA, OutputB, OutputC> CombinedIdentityResult<OutputA, OutputB, OutputC>
where
	OutputA: Default,
	OutputC: Default,
{
	pub fn from_b(b: OutputB) -> Self {
		Self {
			a: OutputA::default(),
			b,
			c: OutputC::default(),
		}
	}
}

impl<OutputA, OutputB, OutputC> CombinedIdentityResult<OutputA, OutputB, OutputC>
where
	OutputA: Default,
	OutputB: Default,
{
	pub fn from_c(c: OutputC) -> Self {
		Self {
			a: OutputA::default(),
			b: OutputB::default(),
			c,
		}
	}
}

pub struct CombineIdentityFrom<A, B, C>(PhantomData<(A, B, C)>);

impl<Identifier, A, B, C> IdentityProvider<Identifier> for CombineIdentityFrom<A, B, C>
where
	A: IdentityProvider<Identifier>,
	B: IdentityProvider<Identifier>,
	C: IdentityProvider<Identifier>,
{
	// TODO: Proper error handling
	type Error = ();
	type Success = CombinedIdentityResult<Option<A::Success>, Option<B::Success>, Option<C::Success>>;

	fn retrieve(identifier: &Identifier) -> Result<Option<Self::Success>, Self::Error> {
		match (
			A::retrieve(identifier),
			B::retrieve(identifier),
			C::retrieve(identifier),
		) {
			// If no details is returned, return None for the whole result
			(Ok(None), Ok(None), Ok(None)) => Ok(None),
			// Otherwise, return `Some` or `None` depending on each result
			(Ok(ok_a), Ok(ok_b), Ok(ok_c)) => Ok(Some(CombinedIdentityResult {
				a: ok_a,
				b: ok_b,
				c: ok_c,
			})),
			// If any of them returns an `Err`, return an `Err`
			_ => Err(()),
		}
	}
}

#[derive(Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, Clone)]
pub struct DipStateProof<RelayBlockHeight, BlindedValues, Leaf, BlockNumber> {
	para_root_proof: ParaRootProof<RelayBlockHeight>,
	dip_commitment_proof: Vec<Vec<u8>>,
	dip_proof: DipMerkleProofAndDidSignature<BlindedValues, Leaf, BlockNumber>,
}

#[derive(Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo, Clone)]
pub struct ParaRootProof<RelayBlockHeight> {
	relay_block_height: RelayBlockHeight,
	proof: Vec<Vec<u8>>,
}

#[derive(Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, Clone)]
pub struct DipMerkleProofAndDidSignature<BlindedValues, Leaf, BlockNumber> {
	merkle_proof: MerkleProof<BlindedValues, Leaf>,
	did_signature: TimeBoundDidSignature<BlockNumber>,
}

pub struct StateProofDipIdentifier<
	RelayInfoProvider,
	ProviderParaIdProvider,
	ParaInfoProvider,
	TxSubmitter,
	ProviderDipMerkleHasher,
	ProviderDidKeyId,
	ProviderBlockNumber,
	ProviderWeb3Name,
	ProviderLinkedAccountId,
	const MAX_REVEALED_KEYS_COUNT: u32,
	const MAX_REVEALED_ACCOUNTS_COUNT: u32,
	DidLocalDetails,
	LocalContextProvider,
	LocalDidCallVerifier,
>(
	#[allow(clippy::type_complexity)]
	PhantomData<(
		RelayInfoProvider,
		ProviderParaIdProvider,
		ParaInfoProvider,
		TxSubmitter,
		ProviderDipMerkleHasher,
		ProviderDidKeyId,
		ProviderBlockNumber,
		ProviderWeb3Name,
		ProviderLinkedAccountId,
		DidLocalDetails,
		LocalContextProvider,
		LocalDidCallVerifier,
	)>,
);

impl<
		Call,
		Subject,
		RelayInfoProvider,
		ProviderParaIdProvider,
		ParaInfoProvider,
		TxSubmitter,
		ProviderDipMerkleHasher,
		ProviderDidKeyId,
		ProviderBlockNumber,
		ProviderWeb3Name,
		ProviderLinkedAccountId,
		const MAX_REVEALED_KEYS_COUNT: u32,
		const MAX_REVEALED_ACCOUNTS_COUNT: u32,
		DidLocalDetails,
		LocalContextProvider,
		LocalDidCallVerifier,
	> IdentityProofVerifier<Call, Subject>
	for StateProofDipIdentifier<
		RelayInfoProvider,
		ProviderParaIdProvider,
		ParaInfoProvider,
		TxSubmitter,
		ProviderDipMerkleHasher,
		ProviderDidKeyId,
		ProviderBlockNumber,
		ProviderWeb3Name,
		ProviderLinkedAccountId,
		MAX_REVEALED_KEYS_COUNT,
		MAX_REVEALED_ACCOUNTS_COUNT,
		DidLocalDetails,
		LocalContextProvider,
		LocalDidCallVerifier,
	> where
	Call: Encode,
	TxSubmitter: Encode,

	RelayInfoProvider: RelayChainStateInfoProvider,
	RelayInfoProvider::Hasher: 'static,
	OutputOf<RelayInfoProvider::Hasher>: Ord,
	RelayInfoProvider::BlockNumber: Copy + Into<U256> + TryFrom<U256> + HasCompact,
	RelayInfoProvider::Key: AsRef<[u8]>,

	ProviderParaIdProvider: Get<RelayInfoProvider::ParaId>,

	ParaInfoProvider: ParachainStateInfoProvider<Identifier = Subject, Commitment = ProviderDipMerkleHasher::Out>,
	ParaInfoProvider::Hasher: 'static,
	OutputOf<ParaInfoProvider::Hasher>: Ord + From<OutputOf<RelayInfoProvider::Hasher>>,
	ParaInfoProvider::Commitment: Decode,
	ParaInfoProvider::Key: AsRef<[u8]>,

	LocalContextProvider: DidSignatureVerifierContextProvider,
	LocalContextProvider::BlockNumber: Encode + CheckedSub + PartialOrd<u16>,
	LocalContextProvider::Hash: Encode,
	LocalContextProvider::SignedExtra: Encode,
	DidLocalDetails: Bump + Default + Encode,
	LocalDidCallVerifier: DidDipOriginFilter<Call, OriginInfo = (DidVerificationKey, DidVerificationKeyRelationship)>,

	ProviderBlockNumber: Encode + Clone,
	ProviderDipMerkleHasher: sp_core::Hasher,
	ProviderDidKeyId: Encode + Clone + Ord + Into<ProviderDipMerkleHasher::Out>,
	ProviderLinkedAccountId: Encode + Clone,
	ProviderWeb3Name: Encode + Clone,
{
	type Error = ();
	type IdentityDetails = DidLocalDetails;
	type Proof = DipStateProof<
		RelayInfoProvider::BlockNumber,
		Vec<Vec<u8>>,
		ProofLeaf<ProviderDidKeyId, ProviderBlockNumber, ProviderWeb3Name, ProviderLinkedAccountId>,
		LocalContextProvider::BlockNumber,
	>;
	type Submitter = TxSubmitter;
	type VerificationResult = VerificationResult<
		ProviderDidKeyId,
		ProviderBlockNumber,
		ProviderWeb3Name,
		ProviderLinkedAccountId,
		MAX_REVEALED_KEYS_COUNT,
		MAX_REVEALED_ACCOUNTS_COUNT,
	>;

	fn verify_proof_for_call_against_details(
		call: &Call,
		subject: &Subject,
		submitter: &Self::Submitter,
		identity_details: &mut Option<Self::IdentityDetails>,
		proof: Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		// 1. Verify relay chain proof.
		let provider_parachain_header =
			state_proofs::relay_chain::ParachainHeadProofVerifier::<RelayInfoProvider>::verify_proof_for_parachain(
				&ProviderParaIdProvider::get(),
				&proof.para_root_proof.relay_block_height,
				proof.para_root_proof.proof,
			)?;

		// 2. Verify parachain state proof.
		let subject_identity_commitment =
			state_proofs::parachain::DipCommitmentValueProofVerifier::<ParaInfoProvider>::verify_proof_for_identifier(
				subject,
				provider_parachain_header.state_root.into(),
				proof.dip_commitment_proof,
			)?;

		// 3. Verify DIP identity proof.
		let proof_leaves = proof
			.dip_proof
			.merkle_proof
			.revealed
			.iter()
			.map(|leaf| (leaf.encoded_key(), Some(leaf.encoded_value())))
			.collect::<Vec<(Vec<u8>, Option<Vec<u8>>)>>();
		verify_trie_proof::<LayoutV1<ProviderDipMerkleHasher>, _, _, _>(
			&subject_identity_commitment,
			&proof.dip_proof.merkle_proof.blinded,
			&proof_leaves,
		)
		.map_err(|_| ())?;

		#[allow(clippy::type_complexity)]
		let (did_keys, web3_name, linked_accounts): (
			BoundedVec<RevealedDidKey<ProviderDidKeyId, ProviderBlockNumber>, ConstU32<MAX_REVEALED_KEYS_COUNT>>,
			Option<RevealedWeb3Name<ProviderWeb3Name, ProviderBlockNumber>>,
			BoundedVec<ProviderLinkedAccountId, ConstU32<MAX_REVEALED_ACCOUNTS_COUNT>>,
		) = proof.dip_proof.merkle_proof.revealed.iter().try_fold(
			(
				BoundedVec::with_bounded_capacity(MAX_REVEALED_KEYS_COUNT.saturated_into()),
				None,
				BoundedVec::with_bounded_capacity(MAX_REVEALED_ACCOUNTS_COUNT.saturated_into()),
			),
			|(mut keys, web3_name, mut linked_accounts), leaf| match leaf {
				ProofLeaf::DidKey(key_id, key_value) => {
					keys.try_push(RevealedDidKey {
						// TODO: Avoid cloning if possible
						id: key_id.0.clone(),
						relationship: key_id.1,
						details: key_value.0.clone(),
					})
					.map_err(|_| ())?;
					Ok::<_, ()>((keys, web3_name, linked_accounts))
				}
				// TODO: Avoid cloning if possible
				ProofLeaf::Web3Name(revealed_web3_name, details) => Ok((
					keys,
					Some(RevealedWeb3Name {
						web3_name: revealed_web3_name.0.clone(),
						claimed_at: details.0.clone(),
					}),
					linked_accounts,
				)),
				ProofLeaf::LinkedAccount(account_id, _) => {
					linked_accounts.try_push(account_id.0.clone()).map_err(|_| ())?;
					Ok::<_, ()>((keys, web3_name, linked_accounts))
				}
			},
		)?;
		let verification_result = VerificationResult {
			did_keys,
			web3_name,
			linked_accounts,
		};

		// 4. Verify DID signature.
		let block_number = LocalContextProvider::block_number();
		let is_signature_fresh =
			if let Some(blocks_ago_from_now) = block_number.checked_sub(&proof.dip_proof.did_signature.block_number) {
				blocks_ago_from_now <= LocalContextProvider::SIGNATURE_VALIDITY
			} else {
				false
			};
		ensure!(is_signature_fresh, ());
		let encoded_payload = (
			call,
			&identity_details,
			submitter,
			&proof.dip_proof.did_signature.block_number,
			LocalContextProvider::genesis_hash(),
			LocalContextProvider::signed_extra(),
		)
			.encode();

		let mut proof_verification_keys = verification_result.as_ref().iter().filter_map(
			|RevealedDidKey {
			     relationship,
			     details: DidPublicKeyDetails { key, .. },
			     ..
			 }| {
				let DidPublicKey::PublicVerificationKey(key) = key else { return None };
				Some((
					key,
					DidVerificationKeyRelationship::try_from(*relationship).expect(
						"Should never
			fail to build a VerificationRelationship from the given DidKeyRelationship
			because we have already made sure the conditions hold.",
					),
				))
			},
		);
		let valid_signing_key = proof_verification_keys.find(|(verification_key, _)| {
			verification_key
				.verify_signature(&encoded_payload, &proof.dip_proof.did_signature.signature)
				.is_ok()
		});
		let Some((key, relationship)) = valid_signing_key else { return Err(()) };
		if let Some(details) = identity_details {
			details.bump();
		} else {
			*identity_details = Some(Self::IdentityDetails::default());
		}

		// 4.1 Verify call required relationship
		LocalDidCallVerifier::check_call_origin_info(call, &(key.clone(), relationship)).map_err(|_| ())?;
		Ok(verification_result)
	}
}
