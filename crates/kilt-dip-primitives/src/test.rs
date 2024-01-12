use did::{
	did_details::{DidPublicKey, DidPublicKeyDetails},
	DidSignature,
};
use frame_support::ensure;
use parity_scale_codec::{Decode, Encode};
use sp_core::ConstU32;
use sp_runtime::{
	traits::{Hash, Header},
	BoundedVec, SaturatedConversion,
};
use sp_trie::{verify_trie_proof, LayoutV1};

use crate::{
	common::{calculate_dip_identity_commitment_storage_key_for_runtime, calculate_parachain_head_storage_key},
	did::TimeBoundDidSignature,
	merkle::{DidKeyRelationship, RevealedDidKey},
	state_proofs::{verify_storage_value_proof, MerkleProofError},
	traits::GetWithArg,
	utils::OutputOf,
	RevealedDidMerkleProofLeaf,
};

pub struct ProviderHeadProof<RelayBlockNumber, const MAX_LEAVE_COUNT: u32, const MAX_LEAVE_SIZE: u32> {
	pub(crate) relay_block_number: RelayBlockNumber,
	pub(crate) proof: BoundedVec<BoundedVec<u8, ConstU32<MAX_LEAVE_SIZE>>, ConstU32<MAX_LEAVE_COUNT>>,
}

pub struct DipCommitmentProof<const MAX_LEAVE_COUNT: u32, const MAX_LEAVE_SIZE: u32>(
	BoundedVec<BoundedVec<u8, ConstU32<MAX_LEAVE_SIZE>>, ConstU32<MAX_LEAVE_COUNT>>,
);

pub struct DidMerkleProof<
	ProviderDidKeyId,
	ProviderAccountId,
	ProviderBlockNumber,
	ProviderWeb3Name,
	ProviderLinkableAccountId,
	const MAX_BLINDED_LEAVE_COUNT: u32,
	const MAX_BLINDED_LEAVE_SIZE: u32,
	const MAX_LEAVES_REVEALED: u32,
> {
	pub blinded: BoundedVec<BoundedVec<u8, ConstU32<MAX_BLINDED_LEAVE_SIZE>>, ConstU32<MAX_BLINDED_LEAVE_COUNT>>,
	pub revealed: BoundedVec<
		RevealedDidMerkleProofLeaf<
			ProviderDidKeyId,
			ProviderAccountId,
			ProviderBlockNumber,
			ProviderWeb3Name,
			ProviderLinkableAccountId,
		>,
		ConstU32<MAX_LEAVES_REVEALED>,
	>,
}

pub enum Error {
	ProviderHeadProof(MerkleProofError),
	RelayStateRootNotFound,
}

pub struct DipDidProof<
	RelayBlockNumber,
	KiltDidKeyId,
	KiltAccountId,
	KiltBlockNumber,
	KiltWeb3Name,
	KiltLinkableAccountId,
	ConsumerBlockNumber,
	const MAX_LEAVE_COUNT: u32,
	const MAX_LEAVE_SIZE: u32,
	const MAX_DIP_LEAVES_REVEALED: u32,
> {
	pub(crate) provider_head_proof: ProviderHeadProof<RelayBlockNumber, MAX_LEAVE_COUNT, MAX_LEAVE_SIZE>,
	pub(crate) dip_commitment_proof: DipCommitmentProof<MAX_LEAVE_COUNT, MAX_LEAVE_SIZE>,
	pub(crate) dip_proof: DidMerkleProof<
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		MAX_LEAVE_COUNT,
		MAX_LEAVE_SIZE,
		MAX_DIP_LEAVES_REVEALED,
	>,
	pub(crate) signature: TimeBoundDidSignature<ConsumerBlockNumber>,
}

impl<
		RelayBlockNumber,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
		const MAX_LEAVE_COUNT: u32,
		const MAX_LEAVE_SIZE: u32,
		const MAX_DIP_LEAVES_REVEALED: u32,
	>
	DipDidProof<
		RelayBlockNumber,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
		MAX_LEAVE_COUNT,
		MAX_LEAVE_SIZE,
		MAX_DIP_LEAVES_REVEALED,
	>
{
	fn verify_top_level_head_proof_for_provider_and_state_root<RelayHasher, ProviderHeader>(
		self,
		provider_para_id: u32,
		relay_state_root: &OutputOf<RelayHasher>,
	) -> Result<
		RelayVerifiedDipProof<
			OutputOf<ProviderHeader::Hashing>,
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			ConsumerBlockNumber,
			MAX_LEAVE_COUNT,
			MAX_LEAVE_SIZE,
			MAX_DIP_LEAVES_REVEALED,
		>,
		Error,
	>
	where
		RelayHasher: Hash,
		ProviderHeader: Decode + Header,
	{
		let provider_head_storage_key = calculate_parachain_head_storage_key(provider_para_id);
		let provider_header = verify_storage_value_proof::<_, RelayHasher, ProviderHeader>(
			&provider_head_storage_key,
			*relay_state_root,
			self.provider_head_proof.proof.into_iter().map(|i| i.into()),
		)
		.map_err(Error::ProviderHeadProof)?;
		Ok(RelayVerifiedDipProof {
			state_root: *provider_header.state_root(),
			dip_commitment_proof: self.dip_commitment_proof,
			dip_proof: self.dip_proof,
			signature: self.signature,
		})
	}

	fn verify_top_level_head_proof_for_provider<RelayHasher, StateRootStore, ProviderHeader>(
		self,
		provider_para_id: u32,
	) -> Result<
		RelayVerifiedDipProof<
			OutputOf<ProviderHeader::Hashing>,
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			ConsumerBlockNumber,
			MAX_LEAVE_COUNT,
			MAX_LEAVE_SIZE,
			MAX_DIP_LEAVES_REVEALED,
		>,
		Error,
	>
	where
		RelayHasher: Hash,
		StateRootStore: GetWithArg<RelayBlockNumber, Result = Option<OutputOf<RelayHasher>>>,
		ProviderHeader: Decode + Header,
	{
		let relay_state_root =
			StateRootStore::get(&self.provider_head_proof.relay_block_number).ok_or(Error::RelayStateRootNotFound)?;
		self.verify_top_level_head_proof_for_provider_and_state_root::<RelayHasher, ProviderHeader>(
			provider_para_id,
			&relay_state_root,
		)
	}
}

pub struct RelayVerifiedDipProof<
	StateRoot,
	KiltDidKeyId,
	KiltAccountId,
	KiltBlockNumber,
	KiltWeb3Name,
	KiltLinkableAccountId,
	ConsumerBlockNumber,
	const MAX_LEAVE_COUNT: u32,
	const MAX_LEAVE_SIZE: u32,
	const MAX_DIP_LEAVES_REVEALED: u32,
> {
	pub(crate) state_root: StateRoot,
	pub(crate) dip_commitment_proof: DipCommitmentProof<MAX_LEAVE_COUNT, MAX_LEAVE_SIZE>,
	pub(crate) dip_proof: DidMerkleProof<
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		MAX_LEAVE_COUNT,
		MAX_LEAVE_SIZE,
		MAX_DIP_LEAVES_REVEALED,
	>,
	pub(crate) signature: TimeBoundDidSignature<ConsumerBlockNumber>,
}

impl<
		StateRoot,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
		const MAX_LEAVE_COUNT: u32,
		const MAX_LEAVE_SIZE: u32,
		const MAX_DIP_LEAVES_REVEALED: u32,
	>
	RelayVerifiedDipProof<
		StateRoot,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
		MAX_LEAVE_COUNT,
		MAX_LEAVE_SIZE,
		MAX_DIP_LEAVES_REVEALED,
	>
{
	fn verify_dip_commitment_proof_for_subject<MerkleHasher, ProviderRuntime, Commitment>(
		self,
		subject: &ProviderRuntime::Identifier,
	) -> Result<
		CommitmentVerifiedProof<
			Commitment,
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			ConsumerBlockNumber,
			MAX_LEAVE_COUNT,
			MAX_LEAVE_SIZE,
			MAX_DIP_LEAVES_REVEALED,
		>,
		Error,
	>
	where
		MerkleHasher: Hash,
		StateRoot: Into<OutputOf<MerkleHasher>>,
		ProviderRuntime: pallet_dip_provider::Config,
		Commitment: Decode,
		OutputOf<MerkleHasher>: Into<Commitment>,
	{
		let dip_commitment_storage_key =
			calculate_dip_identity_commitment_storage_key_for_runtime::<ProviderRuntime>(subject, 0);
		let dip_commitment = verify_storage_value_proof::<_, MerkleHasher, Commitment>(
			&dip_commitment_storage_key,
			self.state_root.into(),
			self.dip_commitment_proof.0.into_iter().map(|i| i.into()),
		)
		.map_err(Error::ProviderHeadProof)?;
		Ok(CommitmentVerifiedProof {
			dip_commitment,
			dip_proof: self.dip_proof,
			signature: self.signature,
		})
	}
}

pub(crate) struct CommitmentVerifiedProof<
	Commitment,
	KiltDidKeyId,
	KiltAccountId,
	KiltBlockNumber,
	KiltWeb3Name,
	KiltLinkableAccountId,
	ConsumerBlockNumber,
	const MAX_LEAVE_COUNT: u32,
	const MAX_LEAVE_SIZE: u32,
	const MAX_DIP_LEAVES_REVEALED: u32,
> {
	pub(crate) dip_commitment: Commitment,
	pub(crate) dip_proof: DidMerkleProof<
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		MAX_LEAVE_COUNT,
		MAX_LEAVE_SIZE,
		MAX_DIP_LEAVES_REVEALED,
	>,
	pub(crate) signature: TimeBoundDidSignature<ConsumerBlockNumber>,
}

impl<
		Commitment,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
		const MAX_LEAVE_COUNT: u32,
		const MAX_LEAVE_SIZE: u32,
		const MAX_DIP_LEAVES_REVEALED: u32,
	>
	CommitmentVerifiedProof<
		Commitment,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
		MAX_LEAVE_COUNT,
		MAX_LEAVE_SIZE,
		MAX_DIP_LEAVES_REVEALED,
	> where
	KiltDidKeyId: Encode,
	KiltAccountId: Encode,
	KiltBlockNumber: Encode,
	KiltWeb3Name: Encode,
	KiltLinkableAccountId: Encode,
{
	fn verify_dip_proof<MerkleHasher>(
		self,
	) -> Result<
		DipVerifiedProof<
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			ConsumerBlockNumber,
			MAX_DIP_LEAVES_REVEALED,
		>,
		Error,
	>
	where
		MerkleHasher: Hash<Output = Commitment>,
	{
		let mut revealed_keys = self
			.dip_proof
			.revealed
			.iter()
			.take(MAX_DIP_LEAVES_REVEALED.saturated_into());

		// If there are more keys than MAX_LEAVES_REVEALED, bail out.
		ensure!(
			revealed_keys.next().is_none(),
			// TODO: Change
			Error::RelayStateRootNotFound,
		);

		let proof_leaves_key_value_pairs: Vec<(Vec<u8>, Option<Vec<u8>>)> = revealed_keys
			.by_ref()
			.map(|revealed_leaf| (revealed_leaf.encoded_key(), Some(revealed_leaf.encoded_value())))
			.collect();

		verify_trie_proof::<LayoutV1<MerkleHasher>, _, _, _>(
			&self.dip_commitment,
			self.dip_proof
				.blinded
				.into_iter()
				.map(|l| l.into_inner())
				.collect::<Vec<_>>()
				.as_slice(),
			&proof_leaves_key_value_pairs,
		)
		// TODO: Change
		.map_err(|_| Error::RelayStateRootNotFound)?;

		Ok(DipVerifiedProof {
			revealed_leaves: self.dip_proof.revealed,
			signature: self.signature,
		})
	}
}

pub(crate) struct DipVerifiedProof<
	KiltDidKeyId,
	KiltAccountId,
	KiltBlockNumber,
	KiltWeb3Name,
	KiltLinkableAccountId,
	ConsumerBlockNumber,
	const MAX_DIP_LEAVES_REVEALED: u32,
> {
	pub(crate) revealed_leaves: BoundedVec<
		RevealedDidMerkleProofLeaf<KiltDidKeyId, KiltAccountId, KiltBlockNumber, KiltWeb3Name, KiltLinkableAccountId>,
		ConstU32<MAX_DIP_LEAVES_REVEALED>,
	>,
	pub(crate) signature: TimeBoundDidSignature<ConsumerBlockNumber>,
}

impl<
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
		const MAX_DIP_LEAVES_REVEALED: u32,
	>
	DipVerifiedProof<
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
		MAX_DIP_LEAVES_REVEALED,
	> where
	ConsumerBlockNumber: PartialOrd,
{
	fn verify_signature_time(
		self,
		block_number: &ConsumerBlockNumber,
	) -> Result<
		DipSignatureTimeVerifiedProof<
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			MAX_DIP_LEAVES_REVEALED,
		>,
		Error,
	> {
		ensure!(
			self.signature.valid_until >= *block_number,
			Error::RelayStateRootNotFound
		);
		Ok(DipSignatureTimeVerifiedProof {
			revealed_leaves: self.revealed_leaves,
			signature: self.signature.signature,
		})
	}
}

pub(crate) struct DipSignatureTimeVerifiedProof<
	KiltDidKeyId,
	KiltAccountId,
	KiltBlockNumber,
	KiltWeb3Name,
	KiltLinkableAccountId,
	const MAX_DIP_LEAVES_REVEALED: u32,
> {
	pub(crate) revealed_leaves: BoundedVec<
		RevealedDidMerkleProofLeaf<KiltDidKeyId, KiltAccountId, KiltBlockNumber, KiltWeb3Name, KiltLinkableAccountId>,
		ConstU32<MAX_DIP_LEAVES_REVEALED>,
	>,
	pub(crate) signature: DidSignature,
}

impl<
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		const MAX_DIP_LEAVES_REVEALED: u32,
	>
	DipSignatureTimeVerifiedProof<
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		MAX_DIP_LEAVES_REVEALED,
	>
{
	fn retrieve_signing_leaf_for_payload<'a>(
		self,
		payload: &[u8],
	) -> Result<
		DipSignatureVerifiedProof<
			'a,
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			MAX_DIP_LEAVES_REVEALED,
		>,
		Error,
	> {
		let revealed_verification_keys = self.revealed_leaves.iter().filter(|leaf| {
			matches!(
				leaf,
				RevealedDidMerkleProofLeaf::DidKey(RevealedDidKey {
					relationship: DidKeyRelationship::Verification(verification_relationship),
					..
				})
			)
		});
		let signing_key = revealed_verification_keys
			.find(|revealed_verification_key| {
				let RevealedDidMerkleProofLeaf::DidKey(RevealedDidKey {
					details:
						DidPublicKeyDetails {
							key: DidPublicKey::PublicVerificationKey(verification_key),
							..
						},
					..
				}) = revealed_verification_key
				else {
					return false;
				};
				verification_key.verify_signature(payload, &self.signature).is_ok()
				// TODO: Change
			})
			.ok_or(Error::RelayStateRootNotFound)?;
		Ok(DipSignatureVerifiedProof {
			revealed_leaves: self.revealed_leaves,
			// TODO: Fix this compilation issue, and then we are golden!
			signing_leaf: RevealedDidMerkleProofLeaf::DidKey(signing_key),
		})
	}
}

pub(crate) struct DipSignatureVerifiedProof<
	'a,
	KiltDidKeyId,
	KiltAccountId,
	KiltBlockNumber,
	KiltWeb3Name,
	KiltLinkableAccountId,
	const MAX_DIP_LEAVES_REVEALED: u32,
> {
	pub(crate) revealed_leaves: BoundedVec<
		RevealedDidMerkleProofLeaf<KiltDidKeyId, KiltAccountId, KiltBlockNumber, KiltWeb3Name, KiltLinkableAccountId>,
		ConstU32<MAX_DIP_LEAVES_REVEALED>,
	>,
	pub(crate) signing_leaf: &'a RevealedDidKey<KiltDidKeyId, KiltAccountId, KiltBlockNumber>,
}
