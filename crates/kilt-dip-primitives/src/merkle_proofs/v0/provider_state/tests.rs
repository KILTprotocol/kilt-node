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

// These test cases are, for now, the same as the ones in
// [`super::relay_state::relay_dip_did_proof_with_verified_relay_state_root`],
// since the functions in there are a wrapper for functions in here.
// Nevertheless, these two components can diverge in the future.
mod parachain_dip_did_proof {
	use frame_support::assert_err;
	use frame_system::pallet_prelude::HeaderFor;
	use hex_literal::hex;
	use sp_core::H256;
	use sp_runtime::traits::{BlakeTwo256, Keccak256};
	use spiritnet_runtime::Runtime as SpiritnetRuntime;

	use crate::{state_proofs::MerkleProofError, Error, ParachainDipDidProof, ProviderHeadStateProof};

	impl<
			RelayBlockNumber,
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			ConsumerBlockNumber,
		>
		ParachainDipDidProof<
			RelayBlockNumber,
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			ConsumerBlockNumber,
		> where
		KiltDidKeyId: Default,
		KiltBlockNumber: Default,
		ConsumerBlockNumber: Default,
	{
		fn with_provider_head_proof(provider_head_proof: ProviderHeadStateProof<RelayBlockNumber>) -> Self {
			Self {
				provider_head_proof,
				dip_commitment_proof: Default::default(),
				dip_proof: Default::default(),
				signature: Default::default(),
			}
		}
	}

	// Storage proof generated at Polkadot block `19_663_508` with hash
	// `0x6e87866fb4f412e1e691e25d294019a7695d5a756ee7bc8d012c25177b5e1e13` for
	// storage key
	// `0xcd710b30bd2eab0352ddcc26417aa1941b3c252fcb29d88eff4f3de5de4476c32c0cfd6c23b92a7826080000`
	// (`paras::heads(2_086)`)
	fn get_provider_head_proof() -> (H256, ProviderHeadStateProof<u32>) {
		(hex!("623b36bddae282e9fefab4707697171a594fdb27e90fd4ada4ebcc356438b070").into(), ProviderHeadStateProof {
			relay_block_number: 19_663_508,
			proof: vec![
				hex!("560cfd6c23b92a7826080000f102e902fb1bd938b2f4fcea70641da8e64e0e11098b92b767279227cdfdc0ae9500da99d27e5f012937179bfe939750c770f2aa6e84c6b8cf9d0aa9ab852243ceb78e3eeb93fc56eacc28c5503a155c4d8bc7ee4797c38e212428cefff0a7ad19b28ebbab793e64080661757261204a207d0800000000056175726101017cda19117c87384aaebfd2ae546771bcfbfe7011a91119932883382cc62be3050d745c9734f422228c7c43d87e6172519019829b14b2d3b64afafb1fb7d3a683").to_vec(),
				hex!("80021080f0c4027f5eba380b623a2d3382ab03961b2b7e753e62d3475a6940207db367cc80fde8c5a37120e2f1d987f5302783d22f8ac1b213c580030a7f5b15e706df6262").to_vec(),
				hex!("8004648019885bbc2709cbd3a89f9a8813867f322e5663c99fdea9af3ff2ba0010455d5d8091eb4873541a81f69d22ccd903b864c36c747910eec7433d947e5a61f87eb7db80ca85a51ad63cbcb7a988023f3b082492a6937f9957c029eb34d6d618279d232d8048dcb685fcf3963e7697a630c6ce64ff06f325063bb05e34e679eb01a4e2b644").to_vec(),
				hex!("80ffff8003ff6c42a935aca27c743910dbb62aae8009854a21741d74080abb406c26b1f58084a7c1351a2986c948c9a111c955d0f8635e4bd305c24f9b6680405fdce955a180ed003737744c7fba94d0c2cb57f96e7bf3310d9c7a285ae789e25af8b79091b38017a0734a39f27a75f6f648bca2facca2381325b529d32bcf82e75aaf6b7d82dd8042e2e666a38ce9fdbe60164d0c3a351ce06c931931d2cd6650378c1ad691c21480d0cc4967746360ee3895a6937608d7f36674426928790cb8ca7426289ad74469804e8940ff6b30dfb0f92341c3a738f262bed9ca03de9b868eb99cfc282aa7786780acda22345d4597dfe6fd831509b944254e26a00fd56e77bc2cb780c0775a520a808c0dae720727cec94dbc853812332bfd6d5f2cc5e287bcd1e5efc530053dbd2280a16a8184b9f2e555d4991995fd479b1ee7b35653f2215f74f822436dbbb2331580984648137ae9c8ecf33f878cedffdba73fb4282ba3ec033102aa6d7442466517801132afeed824c180373b2450b32c72c84a21cdfddbe0f1bf8e76d6958963669580357f2107df0a82f2605f90e39c5665bdf69e1d6222bc425f8390bde67c1d414780c4e048c8dc0ea614a190375a1b215c8e8ff5f5098cd43a93d59be907a2258a74807ad4cd868c49acc40e389d45a1e7e7629e666972ed747c67b607b07f637c1f0b8021bbaa444a77faac92b771c0e1b19162ace64b5ce745892d3ce59f820cba2dc7").to_vec(),
				hex!("9e710b30bd2eab0352ddcc26417aa1945fd3802284b6ec6d4b3138fca93d003a58421ba947ecbc14c39e76572061105bbc568b809f8b23e74053dd98b58424e102ba5ac16f028714ec16a61522011fe6e16771ff80ed3e43ac278948816e8c9e8adda2dbeefe552702cf8144fd9b50e0b8db99bfcf80694abb8b23315ab79cdb22ca6826e867a9157415a832ad38f376dd819107d3ea80b9aee043e378f8313e68a6030679ccf3880fa1e7ab19b6244b5c262b7a152f004c5f03c716fb8fff3de61a883bb76adb34a2040080f15f37adeb10597dac54c2c65393277b2ca62aa27b2d16a23a78a4cc55ef15bb8008a0c609ab4888f02c2545c002153297c2641c5a7b4f3d8e25c634e721f80bea80b6617c764df278313c426c46961ccde8ee7a03f9007b74bc8bc6c49d1583cf7d801c9a4a3457ad4a568dd4c9abe231304689c9bec78be932ef0a2d30690ca428848059ef8bbe3a06c98792f41b3e0a6cdf1f157d9be85e12a7c1daf9c30f969daba4").to_vec(),
				hex!("9f0b3c252fcb29d88eff4f3de5de4476c3ffbf805254dc9131b269f3bbbb71f58a76a5034b2bc2faaab0d1cf45c3819dc6e69740804bc059c3d96f627e09a3b6c0f9851d902f84ac68006617289ac0b7d0a272b36280d97e2394406f94be4266da29b6fe7f3178059525eaf3c9b540064389af020bf180636959b43018d3ff8a55246d5874a16c93e85bd2a58c82ebfc1b54dd9b2a7d0780d3c1a10188200f31459d722f7efc693736d1a36af5644fd949b2e411d7942597800328f24d0485b9701135913a569f6ccbf261a05d055183abf3e4ecb4e4375b7c80f3229cd59de7b1e604f110cbcf814466f2d2973e9bdb6c106a662c576e0820e480b66b29cbd45f93602dbc9f1175407c6f69bd686d23dd22a8f0dfe9cff08843ad80ddb2d426c0c546068b429e77253e0a8a32e818151f5fc031e899a0f6acad157580ea7fb3cad8e128cc295194658016f4865ef37501e5759fb4f15cb2ecb689e85e80e9f3cac1b25842da7fbaf947952dc30329a1d19037ab21baed3851acbee629f6800d898e2a4a6ee9969a233c4741e4441c0fe393104b3cfc5adcf348f3ef20fc7480ac6c622536e593ae3c9d423a461faafc7abbf01ecb129e69d66f3382eaf484dc80c7ba3cadffaea5acd013dba51c96129ae93ea6cd45f3930e4302f5b100f6deae806f29f805e30029363e42381d6609ecb6837411bd6fd676c0a37621a3b5588101").to_vec()
			],
		})
	}

	#[test]
	fn verify_provider_head_proof_with_state_root_successful() {
		let (relay_state_root, provider_head_proof) = get_provider_head_proof();
		// Only interested in the parachain head verification part, we skip everything
		// else.
		let proof = ParachainDipDidProof::<_, (), (), _, (), (), ()>::with_provider_head_proof(provider_head_proof);
		let proof_verification_result = proof
			.verify_provider_head_proof_with_state_root::<BlakeTwo256, HeaderFor<SpiritnetRuntime>>(
				2_086,
				&relay_state_root,
			)
			.unwrap();
		assert_eq!(
			proof_verification_result.state_root,
			hex!("2937179bfe939750c770f2aa6e84c6b8cf9d0aa9ab852243ceb78e3eeb93fc56").into()
		);
	}

	#[test]
	fn verify_provider_head_proof_with_state_root_multi_storage() {
		// Storage proof generated at Polkadot block `19_663_508` with hash
		// `0x6e87866fb4f412e1e691e25d294019a7695d5a756ee7bc8d012c25177b5e1e13` for
		// storage keys
		// [`0xcd710b30bd2eab0352ddcc26417aa1941b3c252fcb29d88eff4f3de5de4476c32c0cfd6c23b92a7826080000`, `0xcd710b30bd2eab0352ddcc26417aa1941b3c252fcb29d88eff4f3de5de4476c3b6ff6f7d467b87a9e8030000`]
		// ([`paras::heads(2_086)`, `paras::heads(1_000)]`)
		let relay_state_root: H256 = hex!("623b36bddae282e9fefab4707697171a594fdb27e90fd4ada4ebcc356438b070").into();
		let provider_head_proof = ProviderHeadStateProof {
			relay_block_number: 19_663_508,
			proof: vec![
				hex!("560cfd6c23b92a7826080000f102e902fb1bd938b2f4fcea70641da8e64e0e11098b92b767279227cdfdc0ae9500da99d27e5f012937179bfe939750c770f2aa6e84c6b8cf9d0aa9ab852243ceb78e3eeb93fc56eacc28c5503a155c4d8bc7ee4797c38e212428cefff0a7ad19b28ebbab793e64080661757261204a207d0800000000056175726101017cda19117c87384aaebfd2ae546771bcfbfe7011a91119932883382cc62be3050d745c9734f422228c7c43d87e6172519019829b14b2d3b64afafb1fb7d3a683").to_vec(),
				hex!("56ff6f7d467b87a9e80300009903910331d9f8f427be99ba3b36ad6f66c49b5448e16745fc3cbe08821204e2e94c9abe1ef65e01dc0f3c52d9aaef735eae1aa679e6c3020e993fb3eef8fab9c32cc7b55dfc85362c771d9a07e5b6cf9f8e30b67598c513ca087574ecce0b3e205eb4de8783a4630c0661757261204a207d08000000000452505352906ca2562dd3ddae0ecb7076465e223753e76792653f739d5dfb00ad76a6b3607d4a2ab00405617572610101dadcb6f606d8a71dc6d0d4d20ccc3bd67bae8816c86491b14fa899242cd872f3bf5fe9635d4414f4329a578a0627cf367dcaa3e86beca64a9aaef9afd124c701").to_vec(),
				hex!("80021080f0c4027f5eba380b623a2d3382ab03961b2b7e753e62d3475a6940207db367cc80fde8c5a37120e2f1d987f5302783d22f8ac1b213c580030a7f5b15e706df6262").to_vec(),
				hex!("8004648019885bbc2709cbd3a89f9a8813867f322e5663c99fdea9af3ff2ba0010455d5d8091eb4873541a81f69d22ccd903b864c36c747910eec7433d947e5a61f87eb7db80ca85a51ad63cbcb7a988023f3b082492a6937f9957c029eb34d6d618279d232d8048dcb685fcf3963e7697a630c6ce64ff06f325063bb05e34e679eb01a4e2b644").to_vec(),
				hex!("80d510805396188100731505c3fe5f51e7d4a9c6e6e4cd2c50ff6d122f5f091a186b2f9780e69515c0c399ad09a7b5da0afb5a8bbd22c6873b69f9f2da18e26a8bd04c6e9d80d647e804958d947c20337a2ac3714b3eca41be52847542b065da3614230decab806dbb5b1913c89acb68a2e85013c4b7adf37ab010cf9b9d7346348d0ca9aafd4a80702af779edd6e8d659600cbe342947238af804d41589116a3dd7fb48905aeab18076a51b70378cbf602d939a885bbad80c94ee9325398105ec2173324bd7f59b55").to_vec(),
				hex!("80ffff8003ff6c42a935aca27c743910dbb62aae8009854a21741d74080abb406c26b1f58084a7c1351a2986c948c9a111c955d0f8635e4bd305c24f9b6680405fdce955a180ed003737744c7fba94d0c2cb57f96e7bf3310d9c7a285ae789e25af8b79091b38017a0734a39f27a75f6f648bca2facca2381325b529d32bcf82e75aaf6b7d82dd8042e2e666a38ce9fdbe60164d0c3a351ce06c931931d2cd6650378c1ad691c21480d0cc4967746360ee3895a6937608d7f36674426928790cb8ca7426289ad74469804e8940ff6b30dfb0f92341c3a738f262bed9ca03de9b868eb99cfc282aa7786780acda22345d4597dfe6fd831509b944254e26a00fd56e77bc2cb780c0775a520a808c0dae720727cec94dbc853812332bfd6d5f2cc5e287bcd1e5efc530053dbd2280a16a8184b9f2e555d4991995fd479b1ee7b35653f2215f74f822436dbbb2331580984648137ae9c8ecf33f878cedffdba73fb4282ba3ec033102aa6d7442466517801132afeed824c180373b2450b32c72c84a21cdfddbe0f1bf8e76d6958963669580357f2107df0a82f2605f90e39c5665bdf69e1d6222bc425f8390bde67c1d414780c4e048c8dc0ea614a190375a1b215c8e8ff5f5098cd43a93d59be907a2258a74807ad4cd868c49acc40e389d45a1e7e7629e666972ed747c67b607b07f637c1f0b8021bbaa444a77faac92b771c0e1b19162ace64b5ce745892d3ce59f820cba2dc7").to_vec(),
				hex!("9e710b30bd2eab0352ddcc26417aa1945fd3802284b6ec6d4b3138fca93d003a58421ba947ecbc14c39e76572061105bbc568b809f8b23e74053dd98b58424e102ba5ac16f028714ec16a61522011fe6e16771ff80ed3e43ac278948816e8c9e8adda2dbeefe552702cf8144fd9b50e0b8db99bfcf80694abb8b23315ab79cdb22ca6826e867a9157415a832ad38f376dd819107d3ea80b9aee043e378f8313e68a6030679ccf3880fa1e7ab19b6244b5c262b7a152f004c5f03c716fb8fff3de61a883bb76adb34a2040080f15f37adeb10597dac54c2c65393277b2ca62aa27b2d16a23a78a4cc55ef15bb8008a0c609ab4888f02c2545c002153297c2641c5a7b4f3d8e25c634e721f80bea80b6617c764df278313c426c46961ccde8ee7a03f9007b74bc8bc6c49d1583cf7d801c9a4a3457ad4a568dd4c9abe231304689c9bec78be932ef0a2d30690ca428848059ef8bbe3a06c98792f41b3e0a6cdf1f157d9be85e12a7c1daf9c30f969daba4").to_vec(),
				hex!("9f0b3c252fcb29d88eff4f3de5de4476c3ffbf805254dc9131b269f3bbbb71f58a76a5034b2bc2faaab0d1cf45c3819dc6e69740804bc059c3d96f627e09a3b6c0f9851d902f84ac68006617289ac0b7d0a272b36280d97e2394406f94be4266da29b6fe7f3178059525eaf3c9b540064389af020bf180636959b43018d3ff8a55246d5874a16c93e85bd2a58c82ebfc1b54dd9b2a7d0780d3c1a10188200f31459d722f7efc693736d1a36af5644fd949b2e411d7942597800328f24d0485b9701135913a569f6ccbf261a05d055183abf3e4ecb4e4375b7c80f3229cd59de7b1e604f110cbcf814466f2d2973e9bdb6c106a662c576e0820e480b66b29cbd45f93602dbc9f1175407c6f69bd686d23dd22a8f0dfe9cff08843ad80ddb2d426c0c546068b429e77253e0a8a32e818151f5fc031e899a0f6acad157580ea7fb3cad8e128cc295194658016f4865ef37501e5759fb4f15cb2ecb689e85e80e9f3cac1b25842da7fbaf947952dc30329a1d19037ab21baed3851acbee629f6800d898e2a4a6ee9969a233c4741e4441c0fe393104b3cfc5adcf348f3ef20fc7480ac6c622536e593ae3c9d423a461faafc7abbf01ecb129e69d66f3382eaf484dc80c7ba3cadffaea5acd013dba51c96129ae93ea6cd45f3930e4302f5b100f6deae806f29f805e30029363e42381d6609ecb6837411bd6fd676c0a37621a3b5588101").to_vec()
			],
		};
		// Only interested in the parachain head verification part, we skip everything
		// else.
		let proof = ParachainDipDidProof::<_, (), (), _, (), (), ()>::with_provider_head_proof(provider_head_proof);
		let proof_verification_result = proof
			.verify_provider_head_proof_with_state_root::<BlakeTwo256, HeaderFor<SpiritnetRuntime>>(
				2_086,
				&relay_state_root,
			)
			.unwrap();
		assert_eq!(
			proof_verification_result.state_root,
			hex!("2937179bfe939750c770f2aa6e84c6b8cf9d0aa9ab852243ceb78e3eeb93fc56").into()
		);
	}

	#[test]
	fn verify_provider_head_proof_with_state_root_wrong_relay_hasher() {
		let (relay_state_root, provider_head_proof) = get_provider_head_proof();
		let proof = ParachainDipDidProof::<_, (), (), _, (), (), ()>::with_provider_head_proof(provider_head_proof);
		assert_err!(
			// Using a different hasher for verification
			proof.verify_provider_head_proof_with_state_root::<Keccak256, HeaderFor<SpiritnetRuntime>>(
				2_086,
				&relay_state_root
			),
			Error::ParaHeadMerkleProof(MerkleProofError::InvalidProof)
		);
	}

	#[test]
	fn verify_provider_head_proof_with_state_root_wrong_para_id() {
		let (relay_state_root, provider_head_proof) = get_provider_head_proof();
		let proof = ParachainDipDidProof::<_, (), (), _, (), (), ()>::with_provider_head_proof(provider_head_proof);
		assert_err!(
			proof.verify_provider_head_proof_with_state_root::<BlakeTwo256, HeaderFor<SpiritnetRuntime>>(
				1_000,
				&relay_state_root
			),
			Error::ParaHeadMerkleProof(MerkleProofError::InvalidProof)
		);
	}

	#[test]
	fn verify_provider_head_proof_with_state_root_invalid_proof() {
		let (relay_state_root, provider_head_proof) = get_provider_head_proof();
		// Remove last part of the blinded component to get an invalid proof
		let (_, invalid_blinded_proof) = provider_head_proof.proof.split_last().unwrap();
		let invalid_provider_head_proof = ProviderHeadStateProof {
			proof: invalid_blinded_proof.to_owned(),
			..provider_head_proof
		};
		let proof =
			ParachainDipDidProof::<_, (), (), _, (), (), ()>::with_provider_head_proof(invalid_provider_head_proof);
		assert_err!(
			proof.verify_provider_head_proof_with_state_root::<BlakeTwo256, HeaderFor<SpiritnetRuntime>>(
				2_086,
				&relay_state_root
			),
			Error::ParaHeadMerkleProof(MerkleProofError::InvalidProof)
		);
	}
}

mod dip_did_proof_with_verified_relay_state_root {
	use frame_support::{assert_err, construct_runtime, traits::Everything};
	use frame_system::{mocking::MockBlock, EnsureSigned};
	use hex_literal::hex;
	use pallet_dip_provider::{DefaultIdentityCommitmentGenerator, DefaultIdentityProvider};
	use sp_core::{crypto::Ss58Codec, ConstU16, ConstU32, ConstU64, H256};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentityLookup, Keccak256},
		AccountId32,
	};

	use crate::{state_proofs::MerkleProofError, DipCommitmentStateProof, DipDidProofWithVerifiedStateRoot, Error};

	impl<
			StateRoot,
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			ConsumerBlockNumber,
		>
		DipDidProofWithVerifiedStateRoot<
			StateRoot,
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			ConsumerBlockNumber,
		> where
		KiltDidKeyId: Default,
		KiltBlockNumber: Default,
		ConsumerBlockNumber: Default,
	{
		fn with_state_root_and_dip_commitment_proof(
			provider_state_root: StateRoot,
			dip_commitment_proof: DipCommitmentStateProof,
		) -> Self {
			Self {
				state_root: provider_state_root,
				dip_commitment_proof,
				dip_proof: Default::default(),
				signature: Default::default(),
			}
		}
	}

	construct_runtime!(
		pub enum TestProviderRuntime {
			System: frame_system,
			DipProvider: pallet_dip_provider,
		}
	);

	impl frame_system::Config for TestProviderRuntime {
		type AccountData = ();
		type AccountId = AccountId32;
		type BaseCallFilter = Everything;
		type Block = MockBlock<TestProviderRuntime>;
		type BlockHashCount = ConstU64<256>;
		type BlockLength = ();
		type BlockWeights = ();
		type DbWeight = ();
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type Lookup = IdentityLookup<Self::AccountId>;
		type MaxConsumers = ConstU32<16>;
		type Nonce = u64;
		type OnKilledAccount = ();
		type OnNewAccount = ();
		type OnSetCode = ();
		type PalletInfo = PalletInfo;
		type RuntimeCall = RuntimeCall;
		type RuntimeEvent = RuntimeEvent;
		type RuntimeOrigin = RuntimeOrigin;
		type SS58Prefix = ConstU16<1>;
		type SystemWeightInfo = ();
		type Version = ();
	}

	impl pallet_dip_provider::Config for TestProviderRuntime {
		type CommitOrigin = AccountId32;
		type CommitOriginCheck = EnsureSigned<AccountId32>;
		type Identifier = AccountId32;
		type IdentityCommitmentGenerator = DefaultIdentityCommitmentGenerator<H256>;
		type IdentityProvider = DefaultIdentityProvider;
		type ProviderHooks = ();
		type RuntimeEvent = RuntimeEvent;
		type WeightInfo = ();
	}

	// Storage proof generated at local Peregrine instance (with new storage hasher)
	// for storage key
	// `0xb375edf06348b4330d1e88564111cb3d5bf19e4ed2927982e234d989e812f3f366a25a7fa9282d4c8e07cfeb5ec4b0f44cec8bb650a6e6ff111f30916b9ca56a4542f70764e95d7ceb6736d981b2d95d01a12dfa1fa4ab9a0000`
	// (`dipProvider::identityCommitments(4pevjN6chwUqWPVaoUF6naRmZyrA4XWfdK8nLQLEjufgW55c, 0)`)
	fn get_dip_commitment_proof() -> (H256, DipCommitmentStateProof) {
		(hex!("0757487b9dda09be65eae2e4ffeff8de52e66d5187d064f31e24fac44be9f4f7").into(), DipCommitmentStateProof(vec![
			hex!("7f540bf19e4ed2927982e234d989e812f3f366a25a7fa9282d4c8e07cfeb5ec4b0f44cec8bb650a6e6ff111f30916b9ca56a4542f70764e95d7ceb6736d981b2d95d01a12dfa1fa4ab9a00008051b175db0cd3a4071aaa1cdde8f3cc562b9618961d8a80ed77981ec98b91da45").to_vec(),
			hex!("800c8080da28793d083b197f8d92fc3e77f5064436f1d8eea0fbea56ddb936aba654450080667f196f66a258b7e851925a9fca0e787fa2080ade3ec203fe940a85a4ef68b080b2aafe11c416356c5a97e233670962facb2a18944c3bdc4b9e27f1fa67a5bafe").to_vec(),
			hex!("80ffff80353e4d164b13c87910044f1b4e76277e404a0ab46a7cd6c33a65aaadc2375ba88007b1390da34b4dce1328430fd924a6e193517a8148dd70a912c0dc2f7f8d2d4c803d402a5fdb0bf83f4f6da28178dc3d3b61d639a4c5733d8eaa79b3a159d9a79f80303acb9eafad3fe6028cf2abca4c824bf48af2b7241920ddf31b37d7921ee932802fd5e075dd0ae75eb64c49c178294214311140bc7c62763c839bedfac51cfa3180048fcfdbc81e0bb059327959a95b003093bf9b04e3918eebe0ac05aae2af93bb8014e3e0704c9a07636322335a3c663ec9fd9df8b7bf71d6e8183fefecfbfe0e50808e788bf3aaaea24abc0ee6d00eb102be955c07bd2b134e24cde6bdfbd922fdcf80deb1dbe09dc8972faeb0de3f080bfbb9d688dcf63906c91db762cc20cbf1e761804ee6ab85272b59bf8715509ccdcdbc038eb7ab7c13552f0eedbdc64bb1ccbacf808d42b27ca13475581cb35914e531fc84820bac04a5c6260b18adc6403c9d78d3807282321f53526da2c8f33500a0c90c75f95972c3c28366033c3f2c38beaaadc2804b74323792921a9cd34cd56a233f5768e3732bb41c157789371b110c5248446f80f395b7003a2eb1e39c624b9a707a6cb58c3cb6997932fc80662ae19c785a91f580b5e5172489541dfc581e116554b63de15fddf38ffed2b109394749c20b8f6ce3805a64908ec1ee443f9cff1793aa18d683308ae1bbd100498b5420e34c9a3c59de").to_vec(),
			hex!("9e75edf06348b4330d1e88564111cb3d3000505f0e7b9012096b41c4eb3aaf947f6ea429080000806ad99dcfd0f2738b39c05d53a22890f969ba700ab74676cde1b3658e6a1d3b28").to_vec(),
		]))
	}

	#[test]
	fn verify_dip_commitment_proof_for_subject_successful() {
		let (parachain_state_root, dip_commitment_proof) = get_dip_commitment_proof();
		// Only interested in the DIP commitment verification part, we skip everything
		// else.
		let proof =
			DipDidProofWithVerifiedStateRoot::<_, (), (), (), (), (), ()>::with_state_root_and_dip_commitment_proof(
				parachain_state_root,
				dip_commitment_proof,
			);
		let proof_verification_result = proof
			.verify_dip_commitment_proof_for_subject::<BlakeTwo256, TestProviderRuntime>(
				&AccountId32::from_ss58check("4pevjN6chwUqWPVaoUF6naRmZyrA4XWfdK8nLQLEjufgW55c").unwrap(),
			)
			.unwrap();
		assert_eq!(
			proof_verification_result.dip_commitment,
			hex!("51b175db0cd3a4071aaa1cdde8f3cc562b9618961d8a80ed77981ec98b91da45").into()
		);
	}

	#[test]
	fn verify_dip_commitment_proof_for_subject_multi_storage() {
		// Storage proof generated at local Peregrine instance (with new storage hasher)
		// for storage keys
		// [`0xb375edf06348b4330d1e88564111cb3d5bf19e4ed2927982e234d989e812f3f366a25a7fa9282d4c8e07cfeb5ec4b0f44cec8bb650a6e6ff111f30916b9ca56a4542f70764e95d7ceb6736d981b2d95d01a12dfa1fa4ab9a0000`, '0xb375edf06348b4330d1e88564111cb3d5bf19e4ed2927982e234d989e812f3f3324b39c02c5b89191d516a1cb2438497d68f8ab82a2af4df66983a1fd0992711686c0fbf8ff8437552365e26f488c17c01a12dfa1fa4ab9a0000]
		// ([`dipProvider::identityCommitments(4pevjN6chwUqWPVaoUF6naRmZyrA4XWfdK8nLQLEjufgW55c, 0)`, `dipProvider::identityCommitments(4smPiDNt9eLaJCe6uq1hGG3kWEmB3ooMpbGbSp1VF9D2vwEg, 0)`])
		let parachain_state_root: H256 =
			hex!("506f0aa6af2e04874ab94835b359ab97a9cca1d1773777b5004da93ffd08a088").into();
		let dip_commitment_proof = DipCommitmentStateProof(vec![
			hex!("7f34024b39c02c5b89191d516a1cb2438497d68f8ab82a2af4df66983a1fd0992711686c0fbf8ff8437552365e26f488c17c01a12dfa1fa4ab9a0000806e5f8a286a025f2631fc7e903f16f4732de04623a411da2abda7c81eb7a42e31").to_vec(),
			hex!("7f3406a25a7fa9282d4c8e07cfeb5ec4b0f44cec8bb650a6e6ff111f30916b9ca56a4542f70764e95d7ceb6736d981b2d95d01a12dfa1fa4ab9a00008051b175db0cd3a4071aaa1cdde8f3cc562b9618961d8a80ed77981ec98b91da45").to_vec(),
			hex!("800c8080da28793d083b197f8d92fc3e77f5064436f1d8eea0fbea56ddb936aba6544500802ac01dbcb6bbdd9e784796b03fa804e22e6c7d552e6432d2e782c78f1fd62ed080b2aafe11c416356c5a97e233670962facb2a18944c3bdc4b9e27f1fa67a5bafe").to_vec(),
			hex!("80ffff80353e4d164b13c87910044f1b4e76277e404a0ab46a7cd6c33a65aaadc2375ba88007b1390da34b4dce1328430fd924a6e193517a8148dd70a912c0dc2f7f8d2d4c8056524aa90f9d9955e0f11cb65a3301b3feec8236f6c506f60851cd9182f6dabe809ce9739227a80b74d58ca2731bb7c95fef30c44badeaed70d4cae8ece37b875180f42657aef3c7a9da89d7fa2ead23197e6c7a0d9a56224c30a23d5e72af213b568011c66e7235c652b25a2599a23850ab0b2c45aa6adc8d0340956aeb06f677780e8014e3e0704c9a07636322335a3c663ec9fd9df8b7bf71d6e8183fefecfbfe0e50801f8da04e41cb0e58c2899f258c3606a4f8a9029ce7dd2831fe2f18748714e1b08063a92863797f3ff47d446998ab380abf3e19f9ad052378a890c969e3665ccfc480ac8fc0b324e4a48b6995b1ace16c16896f31a7d342fdd8c2812aebc74b3b1b2080f567d19109fd00674a7d71a364d5036670bd8413170968a2cd7e204ee9762b1d809183d04fbdc18d7dd79fe20d07131563bef1b21aa5cb6861a2dab4fb6173cbe3802588aac7065dd9e759283fcbf53a0c3696e5669564f92ee17d0ea9fabfe9e82880f395b7003a2eb1e39c624b9a707a6cb58c3cb6997932fc80662ae19c785a91f580b5e5172489541dfc581e116554b63de15fddf38ffed2b109394749c20b8f6ce38019dd7bc351b0ddab367f9c10a27d0dad1669e16af8a8a58c577ce0b2fb26ce8d").to_vec(),
			hex!("9e75edf06348b4330d1e88564111cb3d3000505f0e7b9012096b41c4eb3aaf947f6ea429080000803cfa8887e3f3605330a40b74e99d031b21aeba65d2ef7f35c24a5cefab5291f1").to_vec(),
			hex!("9f0bf19e4ed2927982e234d989e812f3f3480080f3fd8dffe32bd8f539044baf30efd07801d87ea5280154588c3abd3e325f578d8048f06290dfec2596fa70eaca62ea496d3dc0cd2f51fd40c61b58d7e5b476eebd").to_vec(),
		]);
		// Only interested in the DIP commitment verification part, we skip everything
		// else.
		let proof =
			DipDidProofWithVerifiedStateRoot::<_, (), (), (), (), (), ()>::with_state_root_and_dip_commitment_proof(
				parachain_state_root,
				dip_commitment_proof,
			);
		let proof_verification_result = proof
			.verify_dip_commitment_proof_for_subject::<BlakeTwo256, TestProviderRuntime>(
				&AccountId32::from_ss58check("4pevjN6chwUqWPVaoUF6naRmZyrA4XWfdK8nLQLEjufgW55c").unwrap(),
			)
			.unwrap();
		assert_eq!(
			proof_verification_result.dip_commitment,
			hex!("51b175db0cd3a4071aaa1cdde8f3cc562b9618961d8a80ed77981ec98b91da45").into()
		);
	}

	#[test]
	fn verify_dip_commitment_proof_for_subject_wrong_provider_hasher() {
		let (parachain_state_root, dip_commitment_proof) = get_dip_commitment_proof();
		// Only interested in the DIP commitment verification part, we skip everything
		// else.
		let proof =
			DipDidProofWithVerifiedStateRoot::<_, (), (), (), (), (), ()>::with_state_root_and_dip_commitment_proof(
				parachain_state_root,
				dip_commitment_proof,
			);
		assert_err!(
			proof.verify_dip_commitment_proof_for_subject::<Keccak256, TestProviderRuntime>(
				// We try
				&AccountId32::from_ss58check("4qVtUbkD2xqp9cqGDjViPpFPesJNdfoJvGeSUgturBxAPyBK").unwrap(),
			),
			Error::DipCommitmentMerkleProof(MerkleProofError::InvalidProof)
		);
	}

	#[test]
	fn verify_dip_commitment_proof_for_subject_different_subject() {
		let (parachain_state_root, dip_commitment_proof) = get_dip_commitment_proof();
		let proof =
			DipDidProofWithVerifiedStateRoot::<_, (), (), (), (), (), ()>::with_state_root_and_dip_commitment_proof(
				parachain_state_root,
				dip_commitment_proof,
			);
		assert_err!(
			proof.verify_dip_commitment_proof_for_subject::<BlakeTwo256, TestProviderRuntime>(
				&AccountId32::from_ss58check("4smPiDNt9eLaJCe6uq1hGG3kWEmB3ooMpbGbSp1VF9D2vwEg").unwrap(),
			),
			Error::DipCommitmentMerkleProof(MerkleProofError::RequiredLeafNotRevealed)
		);
	}

	#[test]
	fn verify_dip_commitment_proof_for_subject_invalid_proof() {
		let (parachain_state_root, dip_commitment_proof) = get_dip_commitment_proof();
		// Remove last part of the blinded component to get an invalid proof.
		let (_, invalid_blinded_proof) = dip_commitment_proof.0.split_last().unwrap();
		let invalid_dip_commitment_proof = DipCommitmentStateProof(invalid_blinded_proof.to_owned());
		let proof =
			DipDidProofWithVerifiedStateRoot::<_, (), (), (), (), (), ()>::with_state_root_and_dip_commitment_proof(
				parachain_state_root,
				invalid_dip_commitment_proof,
			);
		assert_err!(
			proof.verify_dip_commitment_proof_for_subject::<BlakeTwo256, TestProviderRuntime>(
				&AccountId32::from_ss58check("4qVtUbkD2xqp9cqGDjViPpFPesJNdfoJvGeSUgturBxAPyBK").unwrap(),
			),
			Error::DipCommitmentMerkleProof(MerkleProofError::InvalidProof)
		);
	}
}

mod dip_did_proof_with_verified_subject_commitment {
	use did::{
		did_details::{DidPublicKeyDetails, DidVerificationKey},
		DidVerificationKeyRelationship,
	};
	use frame_support::assert_err;
	use hex_literal::hex;
	use pallet_did_lookup::linkable_account::LinkableAccountId;
	use sp_core::{ed25519, ConstU32, H256};
	use sp_runtime::{
		traits::{BlakeTwo256, Keccak256},
		AccountId32, BoundedVec,
	};

	use crate::{
		DidMerkleProof, DipDidProofWithVerifiedSubjectCommitment, Error, RevealedDidKey, TimeBoundDidSignature,
	};

	impl<
			Commitment,
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			ConsumerBlockNumber,
		>
		DipDidProofWithVerifiedSubjectCommitment<
			Commitment,
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			ConsumerBlockNumber,
		> where
		ConsumerBlockNumber: Default,
	{
		fn with_commitment_and_dip_proof(
			commitment: Commitment,
			dip_proof: DidMerkleProof<
				KiltDidKeyId,
				KiltAccountId,
				KiltBlockNumber,
				KiltWeb3Name,
				KiltLinkableAccountId,
			>,
		) -> Self {
			Self {
				dip_commitment: commitment,
				dip_proof,
				signature: TimeBoundDidSignature::default(),
			}
		}
	}

	// DIP proof generated on Peregrine via the `dipProvider::generateProof` runtime
	// API.
	#[allow(clippy::type_complexity)]
	fn get_dip_proof() -> (
		H256,
		DidMerkleProof<H256, AccountId32, u64, BoundedVec<u8, ConstU32<32>>, LinkableAccountId>,
	) {
		(
			hex!("1997d38bec607be35cab175edc55e2119e0138976021e1f938942c10f9f7b329").into(),
			DidMerkleProof {
				blinded: vec![
					hex!(
						"8027f4809d06d6e9516f8bcbe97b3e1fa94f294b2606a11d00f1162c90bbdbaa0cbc77d480421f140adb34
								53138eb8c4512f9cff60ee9a62502cbb0ddd30355235c12dbd318001ba7e874784b7c79fdc37d1584ff254
								efb6d167087dcb1227c704fd9f6c21d40080a92c5bdfcfbb286551bc43fb263980bc9148f3645f6bc0743c
								4292b88dc4039f8011e7fd2693a380b14bd3dd83736bec3bbcb7f70c7b7e0aaf30a03d2bbf96bd3b80c5a2
								1afb7e16c0f8869ca44efbafddef083c89104fe153d0a77698a5aa1eef7d808cf84bd4fa37829f7229d507
								3cbb504832fc88766def7b06930c5c27f7bf12a080dab6661eac3da9d306e8bbfdffb8ccc901239d8c1664
								220062a4384224babea0"
					)
					.to_vec(),
					hex!("7f0400da6646d21f19b4d7d9f80d5beb103fbef7f4bb95eb94e0c02552175b1bff3a010000").to_vec(),
				],
				revealed: vec![RevealedDidKey {
					id: hex!("50da6646d21f19b4d7d9f80d5beb103fbef7f4bb95eb94e0c02552175b1bff3a").into(),
					relationship: DidVerificationKeyRelationship::Authentication.into(),
					details: DidPublicKeyDetails {
						key: DidVerificationKey::Ed25519(ed25519::Public(hex!(
							"43a72e714401762df66b68c26dfbdf2682aaec9f2474eca4613e424a0fbafd3c"
						)))
						.into(),
						block_number: 0,
					},
				}
				.into()],
			},
		)
	}

	#[test]
	fn verify_dip_proof_successful() {
		let (dip_commitment, dip_proof) = get_dip_proof();
		let proof = DipDidProofWithVerifiedSubjectCommitment::<_, _, _, _, _, _, ()>::with_commitment_and_dip_proof(
			dip_commitment,
			dip_proof.clone(),
		);
		let proof_verification_result = proof.verify_dip_proof::<BlakeTwo256, 1>().unwrap();
		assert_eq!(
			proof_verification_result.revealed_leaves.into_inner(),
			vec![dip_proof.revealed.first().unwrap().to_owned()]
		);
	}

	#[test]
	fn verify_dip_proof_wrong_merkle_hasher() {
		let (dip_commitment, dip_proof) = get_dip_proof();
		let proof = DipDidProofWithVerifiedSubjectCommitment::<_, _, _, _, _, _, ()>::with_commitment_and_dip_proof(
			dip_commitment,
			dip_proof,
		);
		// Different hasher used for verification
		assert_err!(proof.verify_dip_proof::<Keccak256, 1>(), Error::InvalidDidMerkleProof);
	}

	#[test]
	fn verify_dip_proof_too_many_leaves() {
		let (dip_commitment, dip_proof) = get_dip_proof();
		let proof = DipDidProofWithVerifiedSubjectCommitment::<_, _, _, _, _, _, ()>::with_commitment_and_dip_proof(
			dip_commitment,
			dip_proof,
		);
		// We set 0 as the maximum limit.
		assert_err!(proof.verify_dip_proof::<BlakeTwo256, 0>(), Error::TooManyLeavesRevealed);
	}

	#[test]
	fn verify_dip_proof_invalid_proof() {
		let proof =
			DipDidProofWithVerifiedSubjectCommitment::<_, (), (), (), (), (), ()>::with_commitment_and_dip_proof(
				H256([100; 32]),
				DidMerkleProof {
					blinded: vec![vec![100; 32]],
					revealed: Default::default(),
				},
			);
		assert_err!(proof.verify_dip_proof::<BlakeTwo256, 1>(), Error::InvalidDidMerkleProof);
	}
}
