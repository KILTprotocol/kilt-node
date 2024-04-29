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

use did::{DidVerificationKeyRelationship, KeyIdOf};
use dip_provider_runtime_template::{
	AccountId as ProviderAccountId, Runtime as ProviderRuntime, MAX_PUBLIC_KEYS_PER_DID, MAX_REVEALABLE_LINKED_ACCOUNTS,
};
use frame_support::traits::Contains;
use frame_system::{pallet_prelude::BlockNumberFor, EnsureSigned};
use kilt_dip_primitives::{
	parachain::{
		DEFAULT_MAX_DID_MERKLE_PROOF_LEAVE_COUNT, DEFAULT_MAX_DID_MERKLE_PROOF_LEAVE_SIZE,
		DEFAULT_MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT, DEFAULT_MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE,
		DEFAULT_MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT, DEFAULT_MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE,
	},
	traits::DipCallOriginFilter,
	KiltVersionedParachainVerifier, RelayStateRootsViaRelayStorePallet, RevealedDidKey,
};
use pallet_dip_consumer::traits::IdentityProofVerifier;
use rococo_runtime::Runtime as RelaychainRuntime;
use sp_core::ConstU32;
use sp_std::{fmt::Debug, marker::PhantomData, vec::Vec};

use crate::{weights, AccountId, DidIdentifier, Runtime, RuntimeCall, RuntimeOrigin};

// +1 for the web3name.
const MAX_PROVIDER_REVEALABLE_KEYS_COUNT: u32 = MAX_PUBLIC_KEYS_PER_DID + MAX_REVEALABLE_LINKED_ACCOUNTS + 1;

/// The verifier logic is tied to the provider template runtime definition.
pub type ProviderTemplateProofVerifier = KiltVersionedParachainVerifier<
	RelaychainRuntime,
	RelayStateRootsViaRelayStorePallet<Runtime>,
	2_000,
	ProviderRuntime,
	DipCallFilter<KeyIdOf<ProviderRuntime>, BlockNumberFor<ProviderRuntime>, ProviderAccountId>,
	(),
	DEFAULT_MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT,
	DEFAULT_MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE,
	DEFAULT_MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT,
	DEFAULT_MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE,
	DEFAULT_MAX_DID_MERKLE_PROOF_LEAVE_COUNT,
	DEFAULT_MAX_DID_MERKLE_PROOF_LEAVE_SIZE,
	MAX_PROVIDER_REVEALABLE_KEYS_COUNT,
>;
pub type MerkleProofVerifierInput = <ProviderTemplateProofVerifier as IdentityProofVerifier<Runtime>>::Proof;
pub type MerkleProofVerifierOutput =
	<ProviderTemplateProofVerifier as IdentityProofVerifier<Runtime>>::VerificationResult;
// Wrapper around the verifier to implement the `GetWorstCase` trait (required
// due to orphan rule).
pub struct ProviderTemplateProofVerifierWrapper;

// Delegate verification logic to the specialized version of
// `KiltVersionedParachainVerifier`.
impl IdentityProofVerifier<Runtime> for ProviderTemplateProofVerifierWrapper {
	type Error = <ProviderTemplateProofVerifier as IdentityProofVerifier<Runtime>>::Error;
	type Proof = MerkleProofVerifierInput;
	type VerificationResult = MerkleProofVerifierOutput;

	fn verify_proof_for_call_against_details(
		call: &pallet_dip_consumer::RuntimeCallOf<Runtime>,
		subject: &<Runtime as pallet_dip_consumer::Config>::Identifier,
		submitter: &<Runtime as frame_system::Config>::AccountId,
		identity_details: &mut Option<<Runtime as pallet_dip_consumer::Config>::LocalIdentityInfo>,
		proof: Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		<ProviderTemplateProofVerifier as IdentityProofVerifier<Runtime>>::verify_proof_for_call_against_details(
			call,
			subject,
			submitter,
			identity_details,
			proof,
		)
	}
}

// Implement worst-case logic for this specific verifier.
#[cfg(feature = "runtime-benchmarks")]
impl kilt_support::traits::GetWorstCase for ProviderTemplateProofVerifierWrapper {
	type Output = pallet_dip_consumer::benchmarking::WorstCaseOf<Runtime>;

	fn worst_case(_context: ()) -> Self::Output {
		use did::{
			did_details::{DidEncryptionKey, DidPublicKeyDetails, DidVerificationKey},
			DidSignature,
		};
		use frame_support::{pallet_prelude::ValueQuery, storage_alias, Twox64Concat};
		use hex_literal::hex;
		use kilt_dip_primitives::{
			DidKeyRelationship, DidMerkleProof, DipCommitmentStateProof, ParachainDipDidProof, ProviderHeadStateProof,
			RevealedAccountId, RevealedWeb3Name, TimeBoundDidSignature,
		};
		use pallet_dip_consumer::benchmarking::WorstCaseOf;
		use pallet_relay_store::RelayParentInfo;
		use sp_core::{ed25519, sr25519, H256};
		use sp_runtime::AccountId32;
		use sp_std::vec;

		#[storage_alias]
		type BlockHash = StorageMap<
			System,
			Twox64Concat,
			BlockNumberFor<Runtime>,
			<Runtime as frame_system::Config>::Hash,
			ValueQuery,
		>;
		#[storage_alias]
		type LatestRelayHeads = StorageMap<RelayStore, Twox64Concat, u32, RelayParentInfo<H256>>;

		const PROOF_RELAY_BLOCK: u32 = 589;

		let provider_head_state_proof = ProviderHeadStateProof::new(PROOF_RELAY_BLOCK, vec![
			hex!("3703f5a4efb16ffa83d007000088e2fdf5c9b8f94277579ae683ead98aae1e06facab1d301144a0271157399ee").to_vec(),
			hex!("8004648031b60c9237ed343094831987f2bec10b211621255ad0b440cf161fa820d30db480f6f6801e4b41e2e6d8ec194dba122bfb9eb33feb2545ef5144cea79551f7cc5280bfc5f17f5701ebcd8a25d9e08a90343321779f9c335471d0b22c2686bd57d9c0800ad654b674c2cd45843018e4083f71d892f9463aab9920f166d47499aecc3e1d").to_vec(),
			hex!("80ffff8002af9d53d0fe38d916e77086562a2af535ec94a36494384d66273d2339604f2380c5801068e98806370ad5939ba17df962ca6c5e7a7b06b34def0bd9a286f3349780173d3299944e3f85dac5b2c2ecb3f1f13f26df47f38c25d937077d0f344caa0780521bb76b6b176fa67e1f40de0f8cdf439ef8dc3e6ca5e483055eeba380bc7b7a809a6d265f539abd682eb0f593cd3c0006367a48bc4a4bd7eb6b755bc48b187c9f8039c39d126632e3b9af053befe643119111fb627077145ebf8ab8277f4e791f6a80646452cc2e74ebf3311ffcdfcaa4bdbd0b31c19d6fc777be9f7a4f5808d96cc2805b682132c52908705526057f73ab7fccab4af6d72a9805634dd8d3cc53f130d180c2d44d371e5fc1f50227d7491ad65ad049630361cefb4ab1844831237609f08380fe93a2a86fd60f3c6b30051cec7e5e72d331b6f4b6fc142834eac567e6a3aca1808b058239988689d9aa6cb8721760371dce42f384ec0f95771555e195320d3e18800ff113df26dc01f6916caf9728ad8ddf2d278362ab63312ec47e40e63ac7ac9880603666dff2710c1262571a56902a9bae2066026f57a9499c98ef56a700abf94c808c1356abbbc74f6009a7b95604976482b2b4573ba58a14072283b5ca5b37bafa806bfdbbf0e0bedcb993b65c9cea1e929a56d78a3b7bc53d1b7ca6fc488e2295ee801f25bd16505bdd55875b871aa63dc73faff8929e8010ca2b535868849af770ed").to_vec(),
			hex!("810338345b941f7b5396da7c8a8ee4a561ea107105bc488887d68344fa716bc271a691030290b49b480f77d69ceb87a8b854012d2f704508d735c7a5e03df7a098869e2b6391949b5d132b311d09614c6bcf46b282c8dee37128faef7f10353ec1f310c40c066175726120f01e7e0800000000045250535288db2cb81d66fb9974bab34f6375abcb3531524bec8f7f20c09cea921e9eae84092d09056175726101017a967f2b9621cdaaf8860e8887f82f950580bd90f366382092452b019a71217039afdf809b48809cfe00b68a958ff74bb38b070216f90d10ffad2d1aa665bc82").to_vec(),
			hex!("9e710b30bd2eab0352ddcc26417aa1945f43803b3441f15daa8a53147d69c48eac75356fab581febbb8030520b248c5942a148804dbeecdd4792782a820b4f713c58dece06bd69e8fcb9b506fe052a24eb7eb0eb802e2e0716043a02f2f29fdd52922704af194b98545ce5ca832255e8ec41bcdb6480a0718fee6fd849f63aebd00a6e9d09e984d70549c0b5475b16c244090876e628505f0e7b9012096b41c4eb3aaf947f6ea4290800004c5f03c716fb8fff3de61a883bb76adb34a20400808aefdc67024312a782a33b24ee2d1bfa728e3842db64274191fa9a4f0f7a56744c5f0f4993f016e2d2f8e5f43be7bb259486040080949e352413ff8a43f35e73a6077d7a87a2de45fb6ce9bc40ad3717bdbf7a5708").to_vec(),
			hex!("9f0b3c252fcb29d88eff4f3de5de4476c3500080d94a128016b9dd6dce1aca270b09fa36eeb8227e134f24b89ca7bde76723c44c808f4595bab11d5f07ca595107dbae994cd82279c9b5e437230d387e2be69c49bd").to_vec(),
		]);
		let dip_commitment_proof = DipCommitmentStateProof::new(vec![
			hex!("7f340f4ac20413f4e00f0a9eaae0343e8e56e68a94309d0adee950b6a63a0a141a3166c15e8ef25c301531f75e25086fe05a01a12dfa1fa4ab9a000080dc96078e1aa097ade1ee470e32ebdd2e6f5808cbfa62f1a625cc21b88677c272").to_vec(),
			hex!("800c8080da28793d083b197f8d92fc3e77f5064436f1d8eea0fbea56ddb936aba654450080a1588f087b233f3494cdc5cdf5147d6dbfa9651bd2974f5e82b3b00dcbfdc0f18081acf868c884e3bbeb26e53acb3a2e4eca7bd36b22d30e0cffb36ed0c48305bb").to_vec(),
			hex!("80ffff80353e4d164b13c87910044f1b4e76277e404a0ab46a7cd6c33a65aaadc2375ba88007b1390da34b4dce1328430fd924a6e193517a8148dd70a912c0dc2f7f8d2d4c80f0a8aefb3eb62ec86937a4e49657b03d7de0588ad6bb795a4ebb0b5654d9e63880fef940f449f15a0e6fa92eeac30b55e5a69d939d85dfc9e6b75545ea5fb5b6f680048e707e1b93570f4506833da06205a54a4e7ff36092237904359e7461fd44d38020f7b28bc23361dcfa7b988a30f92202a9fd05d783f27b89f304c41eeca500958014e3e0704c9a07636322335a3c663ec9fd9df8b7bf71d6e8183fefecfbfe0e5080259cb4cd05acf09d7a9d7e9935585ff95670dc498cfc365c25b217d66b985f28802943cf2440d02afc0e2a7fb7567af93eba7d83bba93e8e8d7dc4abe20544cb53802b320a7ee167d52bd32ffcd3d94312f8d17c0eadeb2840c0b448de09ac54e3e7803ee8e59b8b261a960b3d00106c36cecfa42b5f72dac70d37530f1958a080da8080b6d9076aa2cd7e8700fb5a5b3ef182975c6515077195c911748da5d21434220e800eaec11c028f926112839db018f6de72c505168b910bbe589d8c83ebdc4fca8d80f395b7003a2eb1e39c624b9a707a6cb58c3cb6997932fc80662ae19c785a91f580b5e5172489541dfc581e116554b63de15fddf38ffed2b109394749c20b8f6ce380949007eae7367a82ee80988be32c8f8f6d936593122a6576d186ba6be490b5bb").to_vec(),
			hex!("9e75edf06348b4330d1e88564111cb3d3000505f0e7b9012096b41c4eb3aaf947f6ea42908000080f20e8f088dd913ee6a53e72e9de980ad8256cb48c3718f8080c0efeeb43e64cd").to_vec(),
			hex!("9f0bf19e4ed2927982e234d989e812f3f348408028e4e828a83fd632d6d17fa940bb289ef8d04c1c154ecbf583d677460bef22128048f06290dfec2596fa70eaca62ea496d3dc0cd2f51fd40c61b58d7e5b476eebd80c898c636c42ebafd67de87f4ebe2e79a6de88441d420e423dba761169752355b").to_vec(),
		]);
		let dip_proof = DidMerkleProof::new(
			vec![
				hex!("80bf2f000000000000000000000000").to_vec(),
				hex!("800281000000").to_vec(),
				hex!("809697000000000000000000").to_vec(),
				hex!("7f000207da77a11b67f17653408a8d6cf85d10b3f366c7e7be82f3b30a8eb935c66c00").to_vec(),
				hex!("7f0000acfd57871165f2330ca49a4ddafabc52698bc894c899d6368107056ee90c2200").to_vec(),
				hex!("7f000afc8c7501fd42bd62db9953e4c54bdf154ff9f5255ebd362b2b795a271b3a7b00").to_vec(),
				hex!("8002020000").to_vec(),
				hex!("7edb492c2503f35d8b783e6d077875aedf473c502c3f641c5c87dad957e3f98b00").to_vec(),
				hex!("7e1dfe90617727b1c2c2d4a570b6e7d042b228c62eba1aeb0f1d43a99d2ee88300").to_vec(),
				hex!("7f0006ad76d64191ec2a4bfee79fadbb7085fa8ccfb7a590cb91b0f3ebd7ec943df900").to_vec(),
				hex!("7f0008fd1a85f17803a48501005e8fc59bb69ede7407062f83f1a950b917951f9bba00").to_vec(),
				hex!("7f0007beb4c2f6b8b2143dcec8771011006b6380ab3a65530ebc849a6a518e4f586000").to_vec(),
				hex!("7f000ceb4ca89584fa1bbb95318204596d8f883101dfeb6b8ebfa61f3a2d081789fe00").to_vec(),
				hex!("7f0007a3e3a7ffb4e10170a73b41039e7298b67ae2fe1d7b8cbfbcb9a19122c51e4b00").to_vec(),
				hex!("7f01c204f1ff9fb3da19442271d014cf3fafa761d4f624d718e729efba11065e300000").to_vec(),
				hex!("7f014c3e671d00ed67683177268a1aae0f7faf290e4754730bb8b0fff18243cc600000").to_vec(),
				hex!("802120000000").to_vec(),
				hex!("7f036e6b1fa2d0ac6b81387fce6eb985b760f70a43a6d8e0c3f9e78c8a9d9e548e010100").to_vec(),
				hex!("7f01ffb682b21cd48217b4010102721378f80e0463cbfbd5a39b0f08b4801d57520000").to_vec(),
				hex!("7f01bf0e5f1c3a6536b9b6c7cd2da10e0dfaba631f50ed16a115b6dc53ba1ff2060000").to_vec(),
				hex!("7f020af4abba9639e828f74df06a5729504ac2ab50e417065f717ed66ee85d1ff88f0000").to_vec(),
				hex!("8000110000").to_vec(),
				hex!("7f01ca9fef5649916accf658e00f703dc2d66bca2fe39b3daa24bbbb096a18bdbf0000").to_vec(),
				hex!("7f017939667bd5080dd837a5187381b02e5944960f73a364fc3499d39ed10ba47d0000").to_vec(),
				hex!("7f020f5333e95049a79201ad8be14bc94440590a41402384ee141b4f17be5b94e57f0000").to_vec(),
				hex!("800250000000").to_vec(),
				hex!("7f010abbb2522022332bec89495323df12567b5abe2f8fcc2e3da40756bfbb7b5e0000").to_vec(),
				hex!("6e353639396133633537343834316234653335336433373700").to_vec(),
				hex!("7f0314c0826d524d79a17cb5bc5fd61f9b2d364c9af73a5db87408f389e83afcdf010300").to_vec(),
				hex!("8000030000").to_vec(),
				hex!("7f03e54fc7807f8c1cbd6e3dac9f3291096e7a2d8ab879934edb402f320a3d46a0010000").to_vec(),
				hex!("7f0194423645f905c2ecc8d07b89babe374ebf761c2b4676c95a749ae7f3f840720000").to_vec(),
				hex!("7f02058867de4a252085d0a8a1078b6d72b8adf1912565bac7733be05b4bf3cbb4ae0000").to_vec(),
				hex!("805800000000").to_vec(),
				hex!("7f0131acdacf05ed4d81448e501ff82a979ddfe90342b62c2d439df8b2bdf6f56a0000").to_vec(),
				hex!("7f0198d9c99157dc9b19fbe30fa8057f0337cfa0b2e23181081b20137f0a2bba5d0000").to_vec(),
				hex!("7f01091a1d3dbf3b1f12c41cf1b4e9c7cf59039aa6407e3010f32d5079ceba07a30000").to_vec(),
				hex!("7f020e59e300e930fa773ef7b8ed42a07c77c2913650a8323caa2fd143ecbb75bfac0000").to_vec(),
				hex!("7f020a4664e1571d22e18e0d45969da0479cbfb8b2bc5fb37850719f7f7fa506267d0000").to_vec(),
				hex!("7f020522e1a7e2ae92f98383e9ff7eb0fbb5baaf09999db7c4161652d3e233af0d140000").to_vec(),
			],
			vec![
				RevealedDidKey {
					id: hex!("78e54fc7807f8c1cbd6e3dac9f3291096e7a2d8ab879934edb402f320a3d46a0").into(),
					relationship: DidVerificationKeyRelationship::Authentication.into(),
					details: DidPublicKeyDetails {
						key: DidVerificationKey::Sr25519(sr25519::Public(hex!(
							"e68a94309d0adee950b6a63a0a141a3166c15e8ef25c301531f75e25086fe05a"
						)))
						.into(),
						block_number: 227u64,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("08c204f1ff9fb3da19442271d014cf3fafa761d4f624d718e729efba11065e30").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"bd09a314a5f66ad2c56639140862bfaad56071044c78e41ba4756ab21147b824"
						))
						.into(),
						block_number: 227,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("0f4c3e671d00ed67683177268a1aae0f7faf290e4754730bb8b0fff18243cc60").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"e503588f6016e08c7ff79c7e74817ecd264b2a97707998748527ef7766819e27"
						))
						.into(),
						block_number: 227,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("15ffb682b21cd48217b4010102721378f80e0463cbfbd5a39b0f08b4801d5752").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"ac95eb8c17f951bb9ae41d19fa9dac75342c6b9b901be6da6a5f42265b491635"
						))
						.into(),
						block_number: 227,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("1dbf0e5f1c3a6536b9b6c7cd2da10e0dfaba631f50ed16a115b6dc53ba1ff206").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"c102dfa2aa8b9ed85e5a67c0612bcf6a3b702ad10fab937881bf57e8a344eb5c"
						))
						.into(),
						block_number: 227,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("2af4abba9639e828f74df06a5729504ac2ab50e417065f717ed66ee85d1ff88f").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"89d45b096b0cd8163dc18bd0bf74399c933116f9de79bda845f00d27b3f2c657"
						))
						.into(),
						block_number: 227,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("38ca9fef5649916accf658e00f703dc2d66bca2fe39b3daa24bbbb096a18bdbf").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"5ce39370f803bea2f945a82e97a06bdea1d340a210dba62f865077018c45cb16"
						))
						.into(),
						block_number: 227,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("3c7939667bd5080dd837a5187381b02e5944960f73a364fc3499d39ed10ba47d").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"f675a09e224219c63b3e33b067ff0b2dc1584c504f1908ed2518d5cceae20347"
						))
						.into(),
						block_number: 227,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("4f5333e95049a79201ad8be14bc94440590a41402384ee141b4f17be5b94e57f").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"7849c1371c98d2bc940df61b22b5094124eaa82151c85068b97f2a37e5abd713"
						))
						.into(),
						block_number: 227,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("510abbb2522022332bec89495323df12567b5abe2f8fcc2e3da40756bfbb7b5e").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"581f2d1e3988a7cf7a695bf77485aa06473b9a67b077df3171dcc15e4d88f521"
						))
						.into(),
						block_number: 227,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("7994423645f905c2ecc8d07b89babe374ebf761c2b4676c95a749ae7f3f84072").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"82545402deeffdc4a6c8d53f8b2442f54e4ec5ed0c26f59d8089300699b7a40a"
						))
						.into(),
						block_number: 227,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("858867de4a252085d0a8a1078b6d72b8adf1912565bac7733be05b4bf3cbb4ae").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"f201c6ca5bb324698e6e1fcebeef42f34f66fd62cd183df1149892a7dcf7cb48"
						))
						.into(),
						block_number: 227,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("9331acdacf05ed4d81448e501ff82a979ddfe90342b62c2d439df8b2bdf6f56a").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"baa2ab5a4663e440b13417864b043ce18af6990f5d1563afdb0e2fec040aed3f"
						))
						.into(),
						block_number: 227,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("9498d9c99157dc9b19fbe30fa8057f0337cfa0b2e23181081b20137f0a2bba5d").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"3f6fb782a6809668998634e264abe3ecc97f15b8f726b86c6a5024fec1d39e53"
						))
						.into(),
						block_number: 227,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("96091a1d3dbf3b1f12c41cf1b4e9c7cf59039aa6407e3010f32d5079ceba07a3").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"de2f7b17ca8a01055027dc2d424ef9b01c0df98ae42fea41a462f84e447ca230"
						))
						.into(),
						block_number: 227,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("ae59e300e930fa773ef7b8ed42a07c77c2913650a8323caa2fd143ecbb75bfac").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"755bdbb3dc4f43d8b3a8c8b19f07bc362ab9015fe3276b3f64439f9ec67e8b0f"
						))
						.into(),
						block_number: 227,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("ba4664e1571d22e18e0d45969da0479cbfb8b2bc5fb37850719f7f7fa506267d").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"bdeaf31cab91d67cea1c6b3f64fa9e9e66826e271e01690f181851d8831e4317"
						))
						.into(),
						block_number: 227,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("d522e1a7e2ae92f98383e9ff7eb0fbb5baaf09999db7c4161652d3e233af0d14").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"cb7cb8c59b2784b87d2270ab4e41f661b4626590954dbd3400208eb1f958f77e"
						))
						.into(),
						block_number: 227,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("106e6b1fa2d0ac6b81387fce6eb985b760f70a43a6d8e0c3f9e78c8a9d9e548e").into(),
					relationship: DidVerificationKeyRelationship::CapabilityDelegation.into(),
					details: DidPublicKeyDetails {
						key: DidVerificationKey::Ed25519(ed25519::Public(hex!(
							"39985b639d8d21629190f2a310b0e2b935894a6261e45ba58f0fbf2bd6c0c832"
						)))
						.into(),
						block_number: 227,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("5e14c0826d524d79a17cb5bc5fd61f9b2d364c9af73a5db87408f389e83afcdf").into(),
					relationship: DidVerificationKeyRelationship::AssertionMethod.into(),
					details: DidPublicKeyDetails {
						key: DidVerificationKey::Ed25519(ed25519::Public(hex!(
							"6c89991144954da6d916f88e59ce0c52bc2dcea2e7edd065e750234ebbb8d8eb"
						)))
						.into(),
						block_number: 227,
					},
				}
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("a7beb4c2f6b8b2143dcec8771011006b6380ab3a65530ebc849a6a518e4f5860")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("86ad76d64191ec2a4bfee79fadbb7085fa8ccfb7a590cb91b0f3ebd7ec943df9")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("791dfe90617727b1c2c2d4a570b6e7d042b228c62eba1aeb0f1d43a99d2ee883")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("71db492c2503f35d8b783e6d077875aedf473c502c3f641c5c87dad957e3f98b")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("f7a3e3a7ffb4e10170a73b41039e7298b67ae2fe1d7b8cbfbcb9a19122c51e4b")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("20acfd57871165f2330ca49a4ddafabc52698bc894c899d6368107056ee90c22")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("1207da77a11b67f17653408a8d6cf85d10b3f366c7e7be82f3b30a8eb935c66c")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("98fd1a85f17803a48501005e8fc59bb69ede7407062f83f1a950b917951f9bba")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("cceb4ca89584fa1bbb95318204596d8f883101dfeb6b8ebfa61f3a2d081789fe")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("4afc8c7501fd42bd62db9953e4c54bdf154ff9f5255ebd362b2b795a271b3a7b")).into(),
				)
				.into(),
				RevealedWeb3Name {
					web3_name: b"5699a3c574841b4e353d377".to_vec().try_into().unwrap(),
					claimed_at: 227,
				}
				.into(),
			],
		);
		let signature = TimeBoundDidSignature::new(DidSignature::Sr25519(sr25519::Signature(hex!("1ca20d39357dba602862e6b6371887c6b1ec46c86ead3c92178cca814e3ff45f7fd6a58395d422b53b6e1d1ab7be5944dbc2c6e640ecfac67c02a218607cc881"))), 282 as BlockNumberFor<Runtime>);
		let proof = ParachainDipDidProof::new(provider_head_state_proof, dip_commitment_proof, dip_proof, signature);

		BlockHash::insert(
			0,
			H256(hex!("74f8cd2f3764f676a5e67c45a641ce1025548c6cddcf524a663a9c0aaf7fbee2")),
		);
		LatestRelayHeads::insert(
			PROOF_RELAY_BLOCK,
			RelayParentInfo {
				relay_parent_storage_root: H256(hex!(
					"29575e65f298648588bc53a45346098e89a99c7330f53d93a899efbb24ddfb69"
				)),
			},
		);

		WorstCaseOf {
			proof: proof.into(),
			call: pallet_postit::Call::post {
				text: b"Hello, world!".to_vec().try_into().unwrap(),
			}
			.into(),
			// 4t8M197K3r1xygdVNoRLRBCpWf6G58VcWTKQUiv5kbJiQhvs
			subject: DidIdentifier::new(hex!("e68a94309d0adee950b6a63a0a141a3166c15e8ef25c301531f75e25086fe05a")),
			// 4rBcMBgT7HzH9NaTpgcBT8AfDUmJjRWiiYGpsqa19CJTSHL3
			submitter: AccountId::new(hex!("908f818bebf2db6d64d86cce811d2133e2d9c9ac447c6c5cc61b23ab04e1fc30")),
		}
	}
}

#[cfg(all(test, feature = "runtime-benchmarks"))]
mod worst_case_tests {
	use kilt_dip_primitives::VersionedDipParachainStateProof;
	use kilt_support::traits::GetWorstCase;
	use pallet_dip_consumer::benchmarking::WorstCaseOf;

	use crate::{dip::MAX_PROVIDER_REVEALABLE_KEYS_COUNT, ProviderTemplateProofVerifierWrapper};

	// Test that the worst case actually refers to the worst case that the provider
	// can generate.
	#[test]
	fn worst_case_max_limits() {
		sp_io::TestExternalities::default().execute_with(|| {
			let WorstCaseOf { proof, .. } = <ProviderTemplateProofVerifierWrapper as GetWorstCase>::worst_case(());
			let VersionedDipParachainStateProof::V0(proof) = proof;
			// We test that the worst case reveals the maximum number of leaves revealable.
			// This is required since the worst case is generated elsewhere and used here as
			// a fixture.
			sp_io::TestExternalities::default().execute_with(|| {
				assert_eq!(
					proof.dip_proof().revealed().len(),
					MAX_PROVIDER_REVEALABLE_KEYS_COUNT as usize
				);
			});
		});
	}
}

impl pallet_dip_consumer::Config for Runtime {
	type DipCallOriginFilter = PreliminaryDipOriginFilter;
	// Any signed origin can submit a cross-chain DIP tx, since subject
	// authentication (and optional binding to the tx submitter) is performed in the
	// DIP proof verification step.
	type DispatchOriginCheck = EnsureSigned<AccountId>;
	type Identifier = DidIdentifier;
	// Local identity info contains a simple `u128` representing a nonce. This means
	// that two cross-chain operations targeting the same chain and with the same
	// nonce cannot be both successfully evaluated.
	type LocalIdentityInfo = u128;
	type ProofVerifier = ProviderTemplateProofVerifierWrapper;
	type RuntimeCall = RuntimeCall;
	type RuntimeOrigin = RuntimeOrigin;
	type WeightInfo = weights::pallet_dip_consumer::WeightInfo<Runtime>;
}

/// A preliminary DID call filter that only allows dispatching of extrinsics
/// from the [`pallet_postit::Pallet`] pallet.
pub struct PreliminaryDipOriginFilter;

impl Contains<RuntimeCall> for PreliminaryDipOriginFilter {
	#[cfg(not(feature = "runtime-benchmarks"))]
	fn contains(t: &RuntimeCall) -> bool {
		matches!(
			t,
			RuntimeCall::PostIt { .. }
				| RuntimeCall::Utility(pallet_utility::Call::batch { .. })
				| RuntimeCall::Utility(pallet_utility::Call::batch_all { .. })
				| RuntimeCall::Utility(pallet_utility::Call::force_batch { .. })
		)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn contains(_t: &RuntimeCall) -> bool {
		true
	}
}

/// Calls to the [`pallet_postit::Pallet`] pallet or batches containing only
/// calls to the [`pallet_postit::Pallet`] pallet will go through if authorized
/// by a DID's authentication key. Everything else will fail.
fn derive_verification_key_relationship(call: &RuntimeCall) -> Option<DidVerificationKeyRelationship> {
	match call {
		RuntimeCall::PostIt { .. } => Some(DidVerificationKeyRelationship::Authentication),
		#[cfg(feature = "runtime-benchmarks")]
		RuntimeCall::System(frame_system::Call::remark { .. }) => Some(DidVerificationKeyRelationship::Authentication),
		RuntimeCall::Utility(pallet_utility::Call::batch { calls }) => single_key_relationship(calls.iter()).ok(),
		RuntimeCall::Utility(pallet_utility::Call::batch_all { calls }) => single_key_relationship(calls.iter()).ok(),
		RuntimeCall::Utility(pallet_utility::Call::force_batch { calls }) => single_key_relationship(calls.iter()).ok(),
		_ => None,
	}
}

// Taken and adapted from `impl
// did::DeriveDidCallAuthorizationVerificationKeyRelationship for RuntimeCall`
// in Spiritnet/Peregrine runtime.
fn single_key_relationship<'a>(
	calls: impl Iterator<Item = &'a RuntimeCall>,
) -> Result<DidVerificationKeyRelationship, ()> {
	let mut calls = calls.peekable();
	let first_call_relationship = calls
		.peek()
		.and_then(|k| derive_verification_key_relationship(k))
		.ok_or(())?;
	calls
		.map(derive_verification_key_relationship)
		.try_fold(first_call_relationship, |acc, next| {
			if next == Some(acc) {
				Ok(acc)
			} else {
				Err(())
			}
		})
}

#[derive(Debug)]
/// Errors generated by calls that do not pass the filter.
pub enum DipCallFilterError {
	/// The call cannot be dispatched with the provided origin.
	BadOrigin,
	/// The call could be dispatched with the provided origin, but it has been
	/// authorized with the wrong DID key.
	WrongVerificationRelationship,
}

impl From<DipCallFilterError> for u8 {
	fn from(value: DipCallFilterError) -> Self {
		match value {
			// DO NOT USE 0
			// Errors of different sub-parts are separated by a `u8::MAX`.
			// A value of 0 would make it confusing whether it's the previous sub-part error (u8::MAX)
			// or the new sub-part error (u8::MAX + 0).
			DipCallFilterError::BadOrigin => 1,
			DipCallFilterError::WrongVerificationRelationship => 2,
		}
	}
}

/// A call filter that requires calls to the [`pallet_postit::Pallet`] pallet to
/// be authorized with a DID signature generated with a key of a given
/// verification relationship.
pub struct DipCallFilter<ProviderDidKeyId, ProviderBlockNumber, ProviderAccountId>(
	PhantomData<(ProviderDidKeyId, ProviderBlockNumber, ProviderAccountId)>,
);

impl<ProviderDidKeyId, ProviderBlockNumber, ProviderAccountId> DipCallOriginFilter<RuntimeCall>
	for DipCallFilter<ProviderDidKeyId, ProviderBlockNumber, ProviderAccountId>
{
	type Error = DipCallFilterError;
	type OriginInfo = Vec<RevealedDidKey<ProviderDidKeyId, ProviderBlockNumber, ProviderAccountId>>;
	type Success = ();

	// Accepts only a DipOrigin for the DidLookup pallet calls.
	fn check_call_origin_info(call: &RuntimeCall, info: &Self::OriginInfo) -> Result<Self::Success, Self::Error> {
		let expected_key_relationship =
			single_key_relationship([call].into_iter()).map_err(|_| DipCallFilterError::BadOrigin)?;
		// If any of the keys revealed is of the right relationship, it's ok.
		if info
			.iter()
			.any(|did_key| did_key.relationship == expected_key_relationship.into())
		{
			Ok(())
		} else {
			Err(DipCallFilterError::WrongVerificationRelationship)
		}
	}
}

impl pallet_relay_store::Config for Runtime {
	// The pallet stores the last 100 relaychain state roots, making state proofs
	// valid for at most 100 * 6 = 600 seconds.
	type MaxRelayBlocksStored = ConstU32<100>;
	type WeightInfo = weights::pallet_relay_store::WeightInfo<Runtime>;
}
