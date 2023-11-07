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

use parity_scale_codec::{Decode, Encode, HasCompact};
use sp_core::{storage::StorageKey, RuntimeDebug, U256};
use sp_runtime::generic::Header;
use sp_std::{marker::PhantomData, vec::Vec};
use sp_trie::StorageProof;

use crate::utils::OutputOf;

use substrate_no_std_port::read_proof_check;

// Ported from https://github.com/paritytech/substrate/blob/b27c470eaff379f512d1dec052aff5d551ed3b03/primitives/state-machine/src/lib.rs#L1076
// Needs to be replaced with its runtime-friendly version when available, or be
// kept up-to-date with upstream.
mod substrate_no_std_port {
	use super::*;

	use hash_db::EMPTY_PREFIX;
	use parity_scale_codec::Codec;
	use sp_core::Hasher;
	use sp_state_machine::{Backend, TrieBackend, TrieBackendBuilder};
	use sp_std::collections::btree_map::BTreeMap;
	use sp_trie::{HashDBT, MemoryDB};

	pub(super) fn read_proof_check<H, I>(
		root: H::Out,
		proof: StorageProof,
		keys: I,
	) -> Result<BTreeMap<Vec<u8>, Option<Vec<u8>>>, ()>
	where
		H: Hasher,
		H::Out: Ord + Codec,
		I: IntoIterator,
		I::Item: AsRef<[u8]>,
	{
		let proving_backend = create_proof_check_backend::<H>(root, proof)?;
		let mut result = BTreeMap::new();
		for key in keys.into_iter() {
			let value = read_proof_check_on_proving_backend(&proving_backend, key.as_ref())?;
			result.insert(key.as_ref().to_vec(), value);
		}
		Ok(result)
	}

	fn read_proof_check_on_proving_backend<H>(
		proving_backend: &TrieBackend<MemoryDB<H>, H>,
		key: &[u8],
	) -> Result<Option<Vec<u8>>, ()>
	where
		H: Hasher,
		H::Out: Ord + Codec,
	{
		proving_backend.storage(key).map_err(|_| ())
	}

	fn create_proof_check_backend<H>(root: H::Out, proof: StorageProof) -> Result<TrieBackend<MemoryDB<H>, H>, ()>
	where
		H: Hasher,
		H::Out: Codec,
	{
		let db = proof.into_memory_db();

		if db.contains(&root, EMPTY_PREFIX) {
			Ok(TrieBackendBuilder::new(db, root).build())
		} else {
			Err(())
		}
	}
}

pub(super) mod relay_chain {
	use super::*;

	use sp_runtime::traits::BlakeTwo256;

	use crate::traits::{RelayChainStateInfo, RelayChainStorageInfo};

	#[derive(RuntimeDebug)]
	pub enum ParachainHeadProofVerifierError {
		InvalidMerkleProof,
		RequiredLeafNotRevealed,
		HeaderDecode,
		RelaychainStateRootNotFound,
	}

	impl From<ParachainHeadProofVerifierError> for u8 {
		fn from(value: ParachainHeadProofVerifierError) -> Self {
			match value {
				ParachainHeadProofVerifierError::InvalidMerkleProof => 0,
				ParachainHeadProofVerifierError::RequiredLeafNotRevealed => 1,
				ParachainHeadProofVerifierError::HeaderDecode => 2,
				ParachainHeadProofVerifierError::RelaychainStateRootNotFound => 3,
			}
		}
	}

	pub struct ParachainHeadProofVerifier<RelayChainState>(PhantomData<RelayChainState>);

	// Uses the provided `root` to verify the proof.
	impl<RelayChainState> ParachainHeadProofVerifier<RelayChainState>
	where
		RelayChainState: RelayChainStorageInfo,
		OutputOf<RelayChainState::Hasher>: Ord,
		RelayChainState::BlockNumber: Copy + Into<U256> + TryFrom<U256> + HasCompact,
		RelayChainState::Key: AsRef<[u8]>,
	{
		pub fn verify_proof_for_parachain_with_root(
			para_id: &RelayChainState::ParaId,
			root: &OutputOf<<RelayChainState as RelayChainStorageInfo>::Hasher>,
			proof: impl IntoIterator<Item = Vec<u8>>,
		) -> Result<Header<RelayChainState::BlockNumber, RelayChainState::Hasher>, ParachainHeadProofVerifierError> {
			let parachain_storage_key = RelayChainState::parachain_head_storage_key(para_id);
			let storage_proof = StorageProof::new(proof);
			let revealed_leaves =
				read_proof_check::<RelayChainState::Hasher, _>(*root, storage_proof, [&parachain_storage_key].iter())
					.map_err(|_| ParachainHeadProofVerifierError::InvalidMerkleProof)?;
			// TODO: Remove at some point
			{
				debug_assert!(revealed_leaves.len() == 1usize);
				debug_assert!(revealed_leaves.contains_key(parachain_storage_key.as_ref()));
			}
			let Some(Some(encoded_head)) = revealed_leaves.get(parachain_storage_key.as_ref()) else {
				return Err(ParachainHeadProofVerifierError::RequiredLeafNotRevealed);
			};
			// TODO: Figure out why RPC call returns 2 bytes in front which we don't need
			let mut unwrapped_head = &encoded_head[2..];
			Header::decode(&mut unwrapped_head).map_err(|_| ParachainHeadProofVerifierError::HeaderDecode)
		}
	}

	// Relies on the `RelayChainState::state_root_for_block` to retrieve the state
	// root for the given block.
	impl<RelayChainState> ParachainHeadProofVerifier<RelayChainState>
	where
		RelayChainState: RelayChainStateInfo,
		OutputOf<RelayChainState::Hasher>: Ord,
		RelayChainState::BlockNumber: Copy + Into<U256> + TryFrom<U256> + HasCompact,
		RelayChainState::Key: AsRef<[u8]>,
	{
		#[allow(clippy::result_unit_err)]
		pub fn verify_proof_for_parachain(
			para_id: &RelayChainState::ParaId,
			relay_height: &RelayChainState::BlockNumber,
			proof: impl IntoIterator<Item = Vec<u8>>,
		) -> Result<Header<RelayChainState::BlockNumber, RelayChainState::Hasher>, ParachainHeadProofVerifierError> {
			let relay_state_root = RelayChainState::state_root_for_block(relay_height)
				.ok_or(ParachainHeadProofVerifierError::RelaychainStateRootNotFound)?;
			Self::verify_proof_for_parachain_with_root(para_id, &relay_state_root, proof)
		}
	}

	pub struct RococoStateRootsViaRelayStorePallet<Runtime>(PhantomData<Runtime>);

	impl<Runtime> RelayChainStorageInfo for RococoStateRootsViaRelayStorePallet<Runtime>
	where
		Runtime: pallet_relay_store::Config,
	{
		type BlockNumber = u32;
		type Hasher = BlakeTwo256;
		type Key = StorageKey;
		type ParaId = u32;

		fn parachain_head_storage_key(para_id: &Self::ParaId) -> Self::Key {
			// TODO: It's not possible to access the runtime definition from here.
			let encoded_para_id = para_id.encode();
			let storage_key = [
				frame_support::storage::storage_prefix(b"Paras", b"Heads").as_slice(),
				sp_io::hashing::twox_64(&encoded_para_id).as_slice(),
				encoded_para_id.as_slice(),
			]
			.concat();
			StorageKey(storage_key)
		}
	}

	impl<Runtime> RelayChainStateInfo for RococoStateRootsViaRelayStorePallet<Runtime>
	where
		Runtime: pallet_relay_store::Config,
	{
		fn state_root_for_block(block_height: &Self::BlockNumber) -> Option<OutputOf<Self::Hasher>> {
			pallet_relay_store::Pallet::<Runtime>::latest_relay_head_for_block(block_height)
				.map(|relay_header| relay_header.relay_parent_storage_root)
		}
	}

	#[cfg(test)]
	mod polkadot_parachain_head_proof_verifier_tests {
		use super::*;

		use hex_literal::hex;
		use sp_runtime::traits::BlakeTwo256;

		// Polkadot block n: 16_363_919,
		// hash 0x18e90e9aa8e3b063f60386ba1b0415111798e72d01de58b1438d620d42f58e39
		struct StaticPolkadotInfoProvider;

		impl RelayChainStorageInfo for StaticPolkadotInfoProvider {
			type BlockNumber = u32;
			type Hasher = BlakeTwo256;
			type Key = StorageKey;
			type ParaId = u32;

			fn parachain_head_storage_key(para_id: &Self::ParaId) -> Self::Key {
				// Adapted from https://github.com/polytope-labs/substrate-ismp/blob/7fb09da6c7b818a98c25c962fee0ddde8e737306/parachain/src/consensus.rs#L369
				// Used for testing. In production this would be generated from the relay
				// runtime definition of the `paras` storage map.
				let encoded_para_id = para_id.encode();
				let storage_key = [
					frame_support::storage::storage_prefix(b"Paras", b"Heads").as_slice(),
					sp_io::hashing::twox_64(&encoded_para_id).as_slice(),
					encoded_para_id.as_slice(),
				]
				.concat();
				StorageKey(storage_key)
			}
		}

		impl RelayChainStateInfo for StaticPolkadotInfoProvider {
			fn state_root_for_block(_block_height: &Self::BlockNumber) -> Option<OutputOf<Self::Hasher>> {
				Some(hex!("81b75d95075d16005ee0a987a3f061d3011ada919b261e9b02961b9b3725f3fd").into())
			}
		}

		#[test]
		fn test_spiritnet_head_proof() {
			// As of RPC state_getReadProof("0xcd710b30bd2eab0352ddcc26417aa1941b3c252fcb29d88eff4f3de5de4476c32c0cfd6c23b92a7826080000", "0x18e90e9aa8e3b063f60386ba1b0415111798e72d01de58b1438d620d42f58e39")
			let spiritnet_head_proof_at_block = [
				hex!("570c0cfd6c23b92a7826080000f102e90265541097fb02782e14f43074f0b00e44ae8e9fe426982323ef1d329739740d37f252ff006d1156941db1bccd58ce3a1cac4f40cad91f692d94e98f501dd70081a129b69a3e2ef7e1ff84ba3d86dab4e95f2c87f6b1055ebd48519c185360eae58f05d1ea08066175726120dcdc6308000000000561757261010170ccfaf3756d1a8dd8ae5c89094199d6d32e5dd9f0920f6fe30f986815b5e701974ea0e0e0a901401f2c72e3dd8dbdf4aa55d59bf3e7021856cdb8038419eb8c").to_vec(),
				hex!("80046480186b1513c5112466ada33da3c65558979906ca9fb82510b62f6ea01f550a4807808bc90ded5636f31c8395a315b5f27a1a25a2ceebd36921a518669ce7e52f80e680993c5e952e6e4f72f295ba04951ace9029b23e9a87887b41895c16f77bec42ee80b798b224c5ee3d668519e75ca98504116f645fb969a5e2653a298b0181f9a694").to_vec(),
				hex!("80ffff806ecd86e87715a007ee9b216d8a99a604773014260d51f6552b6fbd7c21786d9c80e23ef51809d6c80c01a6e264ff0d298cce01c1addfdbb0789597b9a6b3f3e4fd80c9c5f0f29d777e2cebcdbd06ddf1c2cfa8ee83524b37ace99d8b7a3aeff039b380da013185503cfefa6c9cc88751993f1f2bf4b8fa4918e876f499fb9405e3206c803a89668f636552a0fb93619913dcc46cf3e087363d532b76a345155a44a46b5180c2e7fc654720b7dcc0316ae1591fde4beb8b853a343b7e5e3ee564d2692c2ee280840f9c4ae7c16ae948828bf50faf062264402e6134d2d6144a5e3ecb0a1e1d9c80f93c2be1ef51fb2032445cc7fbc2023b9e3b8cf8c0d832b464ae48a020bfaa8c8010c63537c9bf58d50c8c0e13c154fd88b2f683e13701901bdc64565aa9b756d580f0b60eaf17fb680827e8a8938c717ac943b85ff373c0fc911e06c34a3a30327280ccb29f1efa59fd7c80a730cb88789a5a256b01fee7e83ac9a3c90da330adc7a480c8e57c547edb33d4b712f017f09d2de2e055f18815669c83eef2f7f3e5dcafae80b7b7e7ffc91a7dd4c4902f7f15cd7598d1258a75433ea953565661d114e2dcca80ebc3a2df819c7c2fd1a33eb1d484beaf7b71114d6a6db240d8b07dc10bfdc49b80a71f21aa3fa5d7475bf134d50f25e2515c797d0a4c2e93998888150c1f969ab8801e32613f54e70c95e9b16a14f5797522ef5e2ef7867868ff663e33e8880994ed").to_vec(),
				hex!("9e710b30bd2eab0352ddcc26417aa1945fd380d49ebc7ca5c1b751c2badb5e5a326d3ba9e331d8b7c6cf279ed7fd71a8882b6c8038088652f73dc8a22336d10f492f0ef8836beaba0ccfeb0f8fabdc9df1d17e2d807f88402cbbed7fa3307e07044200b572d5e8e12913b41e1923dcb2c0799bc2be804d57e9a8e4934fab698a9db50682052ee9459c666a075d1bfc471da8e5da14da80b9aee043e378f8313e68a6030679ccf3880fa1e7ab19b6244b5c262b7a152f004c5f03c716fb8fff3de61a883bb76adb34a2040080f282bc12648ffb197ffc257edc7ff3a3fdda452daa51091ccbd2dfb91d8aa9518008a0c609ab4888f02c2545c002153297c2641c5a7b4f3d8e25c634e721f80bea80b6617c764df278313c426c46961ccde8ee7a03f9007b74bc8bc6c49d1583cf7d8077b493d45eb153353026cc330307e0753ac41a5cb8e843ceb1efdc46655f33a0808bdaa43fc5dc0e928e2da0ce8ed02096b0b74c61feaba2546980ed9c6174f71d").to_vec(),
				hex!("9f0b3c252fcb29d88eff4f3de5de4476c3ffbf8013c601cc93de3437f9d415bd52c48d794b341f218b9d0020a4b646746c24d0ca80348b8e2c39c479a146933297f62b7051df82e92e1bca761432c3e6f64c74033f80220131e7cd7a08b97f8aa06225f7aefbbca8118fb436c07689c552ed3f577145806d974dd9e4db5e407e29f84c4121ccc58f9c6adc3933afc1bcaef52defe77de5801e9e1a21db053de56365fdee57998488ddae7d664c0430da90469dde17936c1f80c5c11751bbfc99a1ad805c58a65b9704e0bad58e694023e9cc57ce6ef84cdb0b8038f6c242700eaea04ffad5c25ca9a9b1cc2af7303655a32eb59e84b6bb927cd3802575469e76e104b0db8b18dbc762b997a78aa666432a44c4b955ced044a4691f80a81408b856272feeec08845af515e27d033efd3ff8b46de6bc706c38e600086a809ee78332c2a38a3918070942421e651e0b9a43e4b8b2c92e87a2552cede73e8380c9d79f411f742cad0c6f2b070aa08703a04cb7db840c3821a6762837dd8d00e9807dcfbc7f2fcc9415e2cb40eef7f718758d76193f325b3f8b7180e3e5e7d6b81e8036252cae6d24a531a151ce1ee223a07bf71cf82a7fdf49090e4ca345d27d68ca80e3f08ef11671f8f1defa66fa2af71e1a871430e9352df9b3f1d427b5a5dabfb280b51d28c9b99030d050fc1578cd23b5506e769b86c8f4ccc6aded4b8d7c1a73b7").to_vec(),
			].to_vec();
			// As of query paras::heads(2_086) at block
			// "0x18e90e9aa8e3b063f60386ba1b0415111798e72d01de58b1438d620d42f58e39"
			// (16_363_919) which results in the key
			// "0xcd710b30bd2eab0352ddcc26417aa1941b3c252fcb29d88eff4f3de5de4476c32c0cfd6c23b92a7826080000"
			//
			let expected_spiritnet_head_at_block = hex!("65541097fb02782e14f43074f0b00e44ae8e9fe426982323ef1d329739740d37f252ff006d1156941db1bccd58ce3a1cac4f40cad91f692d94e98f501dd70081a129b69a3e2ef7e1ff84ba3d86dab4e95f2c87f6b1055ebd48519c185360eae58f05d1ea08066175726120dcdc6308000000000561757261010170ccfaf3756d1a8dd8ae5c89094199d6d32e5dd9f0920f6fe30f986815b5e701974ea0e0e0a901401f2c72e3dd8dbdf4aa55d59bf3e7021856cdb8038419eb8c").to_vec();
			let returned_head = ParachainHeadProofVerifier::<StaticPolkadotInfoProvider>::verify_proof_for_parachain(
				&2_086,
				&16_363_919,
				spiritnet_head_proof_at_block,
			)
			.expect("Parachain head proof verification should not fail.");
			assert!(returned_head.encode() == expected_spiritnet_head_at_block, "Parachain head returned from the state proof verification should not be different than the pre-computed one.");
		}
	}
}

pub(super) mod parachain {
	use super::*;

	use crate::traits::ProviderParachainStateInfo;

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

	pub struct DipIdentityCommitmentProofVerifier<ParaInfo>(PhantomData<ParaInfo>);

	impl<ParaInfo> DipIdentityCommitmentProofVerifier<ParaInfo>
	where
		ParaInfo: ProviderParachainStateInfo,
		OutputOf<ParaInfo::Hasher>: Ord,
		ParaInfo::Commitment: Decode,
		ParaInfo::Key: AsRef<[u8]>,
	{
		#[allow(clippy::result_unit_err)]
		pub fn verify_proof_for_identifier(
			identifier: &ParaInfo::Identifier,
			state_root: OutputOf<ParaInfo::Hasher>,
			proof: impl IntoIterator<Item = Vec<u8>>,
		) -> Result<ParaInfo::Commitment, DipIdentityCommitmentProofVerifierError> {
			let dip_commitment_storage_key = ParaInfo::dip_subject_storage_key(identifier, 0);
			let storage_proof = StorageProof::new(proof);
			let revealed_leaves = read_proof_check::<ParaInfo::Hasher, _>(
				state_root,
				storage_proof,
				[&dip_commitment_storage_key].iter(),
			)
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
	}

	#[cfg(test)]
	mod spiritnet_test_event_count_value {
		use super::*;

		use hex_literal::hex;
		use pallet_dip_provider::IdentityCommitmentVersion;
		use sp_core::H256;
		use sp_runtime::traits::BlakeTwo256;

		// Spiritnet block n: 4_184_668,
		// hash 0x2c0746e7e9ccc6e4d27bcb4118cb6821ae53ae9bf372f4f49ac28d8598f9bed5
		struct StaticSpiritnetInfoProvider;

		// We use the `system::eventCount()` storage entry as a unit test here.
		impl ProviderParachainStateInfo for StaticSpiritnetInfoProvider {
			type BlockNumber = u32;
			// The type of the `eventCount()` storage entry.
			type Commitment = u32;
			type Hasher = BlakeTwo256;
			// Irrelevant for this test here
			type Identifier = ();
			type Key = StorageKey;

			fn dip_subject_storage_key(
				_identifier: &Self::Identifier,
				_version: IdentityCommitmentVersion,
			) -> Self::Key {
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
}
