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

use parity_scale_codec::Decode;
use sp_core::RuntimeDebug;
use sp_std::marker::PhantomData;
use sp_trie::StorageProof;

use crate::{
	state_proofs::substrate_no_std_port::read_proof_check, traits::ProviderParachainStorageInfo, utils::OutputOf,
};

#[derive(RuntimeDebug)]
pub enum DipIdentityCommitmentProofVerifierError {
	InvalidMerkleProof,
	RequiredLeafNotRevealed,
	CommitmentDecode,
}

impl From<DipIdentityCommitmentProofVerifierError> for u8 {
	fn from(value: DipIdentityCommitmentProofVerifierError) -> Self {
		match value {
			DipIdentityCommitmentProofVerifierError::InvalidMerkleProof => 0,
			DipIdentityCommitmentProofVerifierError::RequiredLeafNotRevealed => 1,
			DipIdentityCommitmentProofVerifierError::CommitmentDecode => 2,
		}
	}
}

/// Verifier of state proofs that reveal the value of the DIP commitment for
/// a given subject on the provider chain. The generic types indicate the
/// following:
/// * `ParaInfo`: defines the provider parachain runtime types relevant for
///   state proof verification, and returns the provider's runtime storage key
///   identifying the identity commitment for a subject with the given
///   identifier.
pub struct DipIdentityCommitmentProofVerifier<ParaInfo>(PhantomData<ParaInfo>);

impl<ParaInfo> DipIdentityCommitmentProofVerifier<ParaInfo>
where
	ParaInfo: ProviderParachainStorageInfo,
	OutputOf<ParaInfo::Hasher>: Ord,
	ParaInfo::Commitment: Decode,
	ParaInfo::Key: AsRef<[u8]>,
{
	/// Given a parachain state root, verify a state proof for the
	/// commitment of a given subject identifier.
	#[cfg(not(feature = "runtime-benchmarks"))]
	pub fn verify_proof_for_identifier(
		identifier: &ParaInfo::Identifier,
		state_root: OutputOf<ParaInfo::Hasher>,
		proof: impl IntoIterator<Item = sp_std::vec::Vec<u8>>,
	) -> Result<ParaInfo::Commitment, DipIdentityCommitmentProofVerifierError> {
		let dip_commitment_storage_key = ParaInfo::dip_subject_storage_key(identifier, 0);
		let storage_proof = StorageProof::new(proof);
		let revealed_leaves =
			read_proof_check::<ParaInfo::Hasher, _>(state_root, storage_proof, [&dip_commitment_storage_key].iter())
				.map_err(|_| DipIdentityCommitmentProofVerifierError::InvalidMerkleProof)?;
		// TODO: Remove at some point
		{
			debug_assert!(revealed_leaves.len() == 1usize);
			debug_assert!(revealed_leaves.contains_key(dip_commitment_storage_key.as_ref()));
		}
		let Some(Some(encoded_commitment)) = revealed_leaves.get(dip_commitment_storage_key.as_ref()) else {
			return Err(DipIdentityCommitmentProofVerifierError::RequiredLeafNotRevealed);
		};
		ParaInfo::Commitment::decode(&mut &encoded_commitment[..])
			.map_err(|_| DipIdentityCommitmentProofVerifierError::CommitmentDecode)
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub fn verify_proof_for_identifier(
		identifier: &ParaInfo::Identifier,
		state_root: OutputOf<ParaInfo::Hasher>,
		proof: impl IntoIterator<Item = sp_std::vec::Vec<u8>>,
	) -> Result<ParaInfo::Commitment, DipIdentityCommitmentProofVerifierError>
	where
		ParaInfo::Commitment: Default,
	{
		let dip_commitment_storage_key = ParaInfo::dip_subject_storage_key(identifier, 0);
		let storage_proof = StorageProof::new(proof);
		let revealed_leaves =
			read_proof_check::<ParaInfo::Hasher, _>(state_root, storage_proof, [&dip_commitment_storage_key].iter())
				.unwrap_or_default();
		let encoded_commitment =
			if let Some(Some(encoded_commitment)) = revealed_leaves.get(dip_commitment_storage_key.as_ref()) {
				encoded_commitment.clone()
			} else {
				sp_std::vec::Vec::default()
			};
		let commitment = ParaInfo::Commitment::decode(&mut &encoded_commitment[..]).unwrap_or_default();
		Ok(commitment)
	}
}

#[cfg(test)]
mod spiritnet_test_event_count_value {
	use super::*;

	use hex_literal::hex;
	use pallet_dip_provider::IdentityCommitmentVersion;
	use sp_core::{storage::StorageKey, H256};
	use sp_runtime::traits::BlakeTwo256;

	// Spiritnet block n: 4_184_668,
	// hash 0x2c0746e7e9ccc6e4d27bcb4118cb6821ae53ae9bf372f4f49ac28d8598f9bed5
	struct StaticSpiritnetInfoProvider;

	// We use the `system::eventCount()` storage entry as a unit test here.
	impl ProviderParachainStorageInfo for StaticSpiritnetInfoProvider {
		type BlockNumber = u32;
		// The type of the `eventCount()` storage entry.
		type Commitment = u32;
		type Hasher = BlakeTwo256;
		// Irrelevant for this test here
		type Identifier = ();
		type Key = StorageKey;

		fn dip_subject_storage_key(_identifier: &Self::Identifier, _version: IdentityCommitmentVersion) -> Self::Key {
			// system::eventCount() raw storage key
			let storage_key = hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec();
			StorageKey(storage_key)
		}
	}

	#[test]
	fn test_spiritnet_event_count() {
		// As of RPC state_getReadProof("
		// 0x26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850",
		// "0x2c0746e7e9ccc6e4d27bcb4118cb6821ae53ae9bf372f4f49ac28d8598f9bed5")
		let spiritnet_event_count_proof_at_block = [
				hex!("800404645ea5c1b19ab7a04f536c519aca4983ac205cda3f0000000000545e98fdbe9ce6c55837576c60c7af38501005000000").to_vec(),
				hex!("80401080481e2bd8085a02c5b58987bce7a69f0b5c7fa651e8e82c5481c94707860be9078067785103d453293707ba847e21df7e35a7a57b8fb929d40465328b6642669fcc").to_vec(),
				hex!("80ffff8010623b5a3a9dbc752963d827be0bb855bf3e24258ae09341d5f762e96a836ac180c34b753605e821528756b55b4ddafb742df6e54fbc03ef401d4ebfd6dd4f3e44806f83646e0bf3ca0ac9f2092dea5b0e3caf210cc6b54c3b44a51855a133367a6580b02cde7b1fd3f8d13f698ef6e9daa29b32258d4d97a8947051070a4540aecacd80903d521961849d07ceee132617b8dde96c3ff472f5a9a089d4055ffe7ffd1e988016c29c943c106713bb8f16b776eb7daed005540165696da286cddf6b25d085448019a464010cb746b0589891f72b0eed603d4712b04af46f7bcae724564194801480a305ffe069db7eb21841f75b5939943f62c4abb3a051d530839c5dd935ccbc8a8035d8938b0c856878de1e3fe45a559588b2da52ccf195ab1e3d0aca6ac7bb079d8064019a474a283c19f46ff4652a5e1f636efd4013d3b8a91c49573045c6ff01c0801a191dcb736faddb84889a13c7aa717d260e9b635b30a9eb3907f925a2253d6880f8bc389fc62ca951609bae208b7506bae497623e647424062d1c56cb1f2d2e1c80211a9fb5f8b794f9fbfbdcd4519aa475ecaf9737b4ee513dde275d5fbbe64da080c267d0ead99634e9b9cfbf61a583877e0241ac518e62e909fbb017469de275f780b3059a7226d4b320c25e9b2f8ffe19cf93467e3b306885962c5f34b5671d15fe8092dfba9e30e1bbefab13c792755d06927e6141f7220b7485e5aa40de92401a66").to_vec(),
				hex!("9eaa394eea5630e07c48ae0c9558cef7398f8069ef420a0deb5a428c9a08563b28a78874bba09124eecc8d28bf30b0e2ddd310745f04abf5cb34d6244378cddbf18e849d962c000000000736d8e8140100505f0e7b9012096b41c4eb3aaf947f6ea4290800004c5f0684a022a34dd8bfa2baaf44f172b710040180dd3270a03a1a13fc20bcdf24d1aa4ddccc6183db2e2e153b8a68ba8540699a8a80b413dad63538a591f7f2575d287520ee44d7143aa5ec2411969861e1f55a2989804c3f0f541a13980689894db7c60c785dd29e066f213bb29b17aa740682ad7efd8026d3a50544f5c89500745aca2be36cfe076f599c5115192fb9deae227e2710c980bd04b00bf6b42756a06a4fbf05a5231c2094e48182eca95d2cff73ab907592aa").to_vec(),
			].to_vec();
		let spiritnet_state_root: H256 =
			hex!("94c23fda279cea4a4370e90f1544c8938923dfd4ac201a420c7a26fb0d3caf8c").into();
		// As of query system::eventCount() at block
		// "0x2c0746e7e9ccc6e4d27bcb4118cb6821ae53ae9bf372f4f49ac28d8598f9bed5" which
		// results in the key
		// "0x26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850"
		let expected_event_count_at_block = 5;
		let returned_event_count =
			DipIdentityCommitmentProofVerifier::<StaticSpiritnetInfoProvider>::verify_proof_for_identifier(
				&(),
				spiritnet_state_root,
				spiritnet_event_count_proof_at_block,
			)
			.unwrap();
		assert!(returned_event_count == expected_event_count_at_block, "Spiritnet event count returned from the state proof verification should not be different than the pre-computed one.");
	}
}
