use parity_scale_codec::{Decode, HasCompact};
use sp_core::{Get, U256};
use sp_runtime::{generic::Header, traits::Hash};
use sp_state_machine::read_proof_check;
use sp_std::marker::PhantomData;
use sp_trie::StorageProof;

pub trait RelayChainInfoProvider<Hash> {
	type Key;
	type ParaId;

	fn storage_key(para_id: Self::ParaId) -> Self::Key;
	fn state_root() -> Hash;
}

pub struct ParachainHeadProofVerifier<RelayHasher, RelayBlockNumber, ParaId, RelayInfoProvider>(
	PhantomData<(RelayHasher, RelayBlockNumber, ParaId, RelayInfoProvider)>,
);

impl<RelayHasher, RelayBlockNumber, ParaId, RelayInfoProvider>
	ParachainHeadProofVerifier<RelayHasher, RelayBlockNumber, ParaId, RelayInfoProvider>
where
	RelayHasher: Hash + 'static,
	RelayHasher::Output: Ord,
	RelayBlockNumber: Copy + Into<U256> + TryFrom<U256> + HasCompact,
	ParaId: Get<u32>,
	RelayInfoProvider: RelayChainInfoProvider<RelayHasher::Output>,
	RelayInfoProvider::ParaId: From<u32>,
	RelayInfoProvider::Key: AsRef<[u8]>,
{
	pub fn verify_proof(proof: impl IntoIterator<Item = Vec<u8>>) -> Result<Header<RelayBlockNumber, RelayHasher>, ()> {
		let relay_state_root = RelayInfoProvider::state_root();
		let parachain_storage_key: <RelayInfoProvider as RelayChainInfoProvider<<RelayHasher as Hash>::Output>>::Key =
			RelayInfoProvider::storage_key(ParaId::get().into());
		let storage_proof = StorageProof::new(proof);
		let revealed_leaves =
			read_proof_check::<RelayHasher, _>(relay_state_root, storage_proof, [&parachain_storage_key].iter())
				.map_err(|_| ())?;
		// TODO: Remove at some point
		debug_assert!(revealed_leaves.len() == 1);
		debug_assert!(revealed_leaves.contains_key(parachain_storage_key.as_ref()));
		let Some(Some(encoded_head)) = revealed_leaves.get(parachain_storage_key.as_ref()) else { return Err(()) };
		// TODO: Figure out why RPC call returns 2 bytes in front which we don't need
		let mut unwrapped_head = &encoded_head[2..];
		Header::<RelayBlockNumber, _>::decode(&mut unwrapped_head).map_err(|_| ())
	}
}

#[cfg(test)]
mod parachain_head_proof_verifier_tests {
	use super::*;

	use hex_literal::hex;
	use parity_scale_codec::Encode;
	use polkadot_primitives::BlakeTwo256;
	use sp_core::{storage::StorageKey, ConstU32, H256};

	// Polkadot block n: 16_363_919,
	// hash 0x18e90e9aa8e3b063f60386ba1b0415111798e72d01de58b1438d620d42f58e39
	struct StaticPolkadotBlockProvider;

	impl RelayChainInfoProvider<H256> for StaticPolkadotBlockProvider {
		type Key = StorageKey;
		type ParaId = u32;

		fn state_root() -> H256 {
			hex!("81b75d95075d16005ee0a987a3f061d3011ada919b261e9b02961b9b3725f3fd").into()
		}

		fn storage_key(para_id: Self::ParaId) -> Self::Key {
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
		// which results in the key
		// "0xcd710b30bd2eab0352ddcc26417aa1941b3c252fcb29d88eff4f3de5de4476c32c0cfd6c23b92a7826080000"
		//
		let expected_spiritnet_head_at_block = hex!("65541097fb02782e14f43074f0b00e44ae8e9fe426982323ef1d329739740d37f252ff006d1156941db1bccd58ce3a1cac4f40cad91f692d94e98f501dd70081a129b69a3e2ef7e1ff84ba3d86dab4e95f2c87f6b1055ebd48519c185360eae58f05d1ea08066175726120dcdc6308000000000561757261010170ccfaf3756d1a8dd8ae5c89094199d6d32e5dd9f0920f6fe30f986815b5e701974ea0e0e0a901401f2c72e3dd8dbdf4aa55d59bf3e7021856cdb8038419eb8c").to_vec();
		let returned_head =
			ParachainHeadProofVerifier::<BlakeTwo256, u32, ConstU32<2_086>, StaticPolkadotBlockProvider>::verify_proof(
				spiritnet_head_proof_at_block,
			)
			.expect("Parachain head proof verification should not fail.");
		assert!(returned_head.encode() == expected_spiritnet_head_at_block, "Parachain head returned from the state proof verification should not be different than the pre-computed one.");
	}
}
