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

use did::{
	did_details::{DidEncryptionKey, DidPublicKeyDetails, DidVerificationKey},
	DidSignature, DidVerificationKeyRelationship, KeyIdOf,
};
use dip_provider_runtime_template::{
	AccountId as ProviderAccountId, Runtime as ProviderRuntime, MAX_REVEALABLE_LINKED_ACCOUNTS,
	MAX_TOTAL_KEY_AGREEMENT_KEYS,
};
use frame_support::{pallet_prelude::ValueQuery, storage_alias, traits::Contains};
use frame_system::{pallet_prelude::BlockNumberFor, EnsureSigned};
use kilt_dip_primitives::{
	parachain::{
		DEFAULT_MAX_DID_MERKLE_PROOF_LEAVE_COUNT, DEFAULT_MAX_DID_MERKLE_PROOF_LEAVE_SIZE,
		DEFAULT_MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT, DEFAULT_MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE,
		DEFAULT_MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT, DEFAULT_MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE,
	},
	traits::DipCallOriginFilter,
	DidKeyRelationship, DidMerkleProof, DipCommitmentStateProof, KiltVersionedParachainVerifier, ParachainDipDidProof,
	ProviderHeadStateProof, RelayStateRootsViaRelayStorePallet, RevealedAccountId, RevealedDidKey, RevealedWeb3Name,
	TimeBoundDidSignature,
};
use pallet_dip_consumer::{benchmarking::WorstCaseOf, traits::IdentityProofVerifier};
use pallet_relay_store::RelayParentInfo;
use rococo_runtime::Runtime as RelaychainRuntime;
use sp_core::ConstU32;
use sp_runtime::AccountId32;
use sp_std::{marker::PhantomData, vec::Vec};

use crate::{weights, AccountId, DidIdentifier, Runtime, RuntimeCall, RuntimeOrigin};

// 3 is the attestation, delegation, and authentication key.
// 1 is the web3name.
const MAX_PROVIDER_REVEALABLE_KEYS_COUNT: u32 = MAX_TOTAL_KEY_AGREEMENT_KEYS + 3 + 1 + MAX_REVEALABLE_LINKED_ACCOUNTS;

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
// Wrapper around the verifier to implement the `GetWorstCase` trait.
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
	type Output = WorstCaseOf<Runtime>;

	fn worst_case(_context: ()) -> Self::Output {
		use frame_support::Twox64Concat;
		use hex_literal::hex;
		use sp_core::{ed25519, sr25519, H256};
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

		const PROOF_RELAY_BLOCK: u32 = 97;

		let provider_head_state_proof = ProviderHeadStateProof::new(PROOF_RELAY_BLOCK, vec![
			hex!("3703f5a4efb16ffa83d007000008cee9c1c963e73af7fc207dbebe6f1bbb4021ef306d72178c1f0ed9f697fb5d").to_vec(),
			hex!("7d0379fca928dfb717b6f057a1e219ded3c9f0161411d8862fb0d35db5fa0658dfd66cbfe88cbd5e3c6e23b59ee284f49179453481973e23cdbaedae536563e9cb0ec8383b1c68c67a1756d37a56b1a9a4f8b6a1d2b4e85454d6b017f44fd5c17b0f620c066175726120b9e87d08000000000452505352884a92d2daf0f346d3fe051b8842ef5ee3ac796feea708ff44e2b35c662698b80e7d010561757261010180278e6ab8cd65e94d8c8264cb34ccac5e2a0fc797fc2d0cf7be8b602ca6061044c440d12404de5c595454bec27de12cb1cd631342e6c4090b556828533b3483").to_vec(),
			hex!("8004648031b60c9237ed343094831987f2bec10b211621255ad0b440cf161fa820d30db480f6f6801e4b41e2e6d8ec194dba122bfb9eb33feb2545ef5144cea79551f7cc5280800f085545989f203978befd777d0db4963134effddc31c8d991ed83eff86e598024001dd7f8ab2d65332af5f168e33329c3363d38619ae10579556f7c568d8cfa").to_vec(),
			hex!("80ffff80da3b13b6f4d00e78e5b4f9febe6ebdd907cb93f888684ae98beb244b23c2bfe9808eedebcb5207842de9ca534d8417b005715106e4c89481e45436606b0e7a51a2801fec1d90c30b290a110a2e6abdadc8243d13ab9974e64f618478704c3ab3d26180965db03a23a9608d689bc991515dabf5929cf73813cb71a330a84a61465116aa800db60e6cc6a1bf8cfdf30b0898db1e570400f22dd342a2e395e9f65dd322f838808a72642beadb74458557cfac7f0a094f164726efc64a41664f89d1c6676fed4e80792a15dbb3b1a6756b605c0785572dc15de8342c534890665add81be037faae1805b682132c52908705526057f73ab7fccab4af6d72a9805634dd8d3cc53f130d180c2d44d371e5fc1f50227d7491ad65ad049630361cefb4ab1844831237609f08380613ee4da8696b75f4409975f271bc6a2d8a1b4acdfd4dd617af34e1488d4f66b802701a3a8687f58f9de5c4f1fa21fa40d0a731b3954d58b02ea286c83b1a98d4180721c6d88d612c4dd11022fc52ee952342f6ba9630e4be9307dfa00988f22060180294443dd616e0b3e21617f766ba58dac88ce58224d7caa4dae29fea8ae629f7180e0d0559c10499f45c1162c73a5f19228779cc75762b99c7d9266dfd58ac4d4bd806bfdbbf0e0bedcb993b65c9cea1e929a56d78a3b7bc53d1b7ca6fc488e2295ee80c306d51e0505b64fb65c094b1c003c2188145a47acb2c1a3ad7cf3c239cc08b3").to_vec(),
			hex!("9e710b30bd2eab0352ddcc26417aa1945f43803b3441f15daa8a53147d69c48eac75356fab581febbb8030520b248c5942a148803efd73d3f24bd2b3ca3916f50072b3cedcec47eb3c61ebb3318d44b7aa9bde3f802e2e0716043a02f2f29fdd52922704af194b98545ce5ca832255e8ec41bcdb64805777400f9579aadc0324691757c92714c245ff336464684b4e1c166ec4fd25eb505f0e7b9012096b41c4eb3aaf947f6ea4290800004c5f03c716fb8fff3de61a883bb76adb34a2040080bfafab45a320864b3a0037065b596599b16bfb6eb2a837e1c4839577045684654c5f0f4993f016e2d2f8e5f43be7bb2594860400801412850013bc8a13c78289220554c9aabb218bd2d3919b9d55a9cc434d337272").to_vec(),
			hex!("9f0b3c252fcb29d88eff4f3de5de4476c350008008c8d0b4cf32fddcde0ace00fa1d052669181c9658ddb0a0e8d967edd7766ed680746cb579dee48fc7153d83cca35a07c60b37660b193012a49a85155d252daa0c").to_vec(),
		]);
		let dip_commitment_proof = DipCommitmentStateProof::new(vec![
			hex!("7f24086f0d72fb64cf00aea16625a4c4ad138e416da9a3fa2302a4d0f367029d1784cbc85d226705684e01a12dfa1fa4ab9a00008030957a9ddde54ceb05cf3b96c348b458026e42b05f2159bf35011bc3957717aa").to_vec(),
			hex!("800c8080da28793d083b197f8d92fc3e77f5064436f1d8eea0fbea56ddb936aba65445008027c4fa196c2981e3f0e695d6b20f54626eda4c6b7c048b8e78f538c3003fe594804c1639cc5554613cba6090de44badd65709713a9efac68281cd883904507722f").to_vec(),
			hex!("80ffff80353e4d164b13c87910044f1b4e76277e404a0ab46a7cd6c33a65aaadc2375ba88007b1390da34b4dce1328430fd924a6e193517a8148dd70a912c0dc2f7f8d2d4c8008a563235f40d290d669842210155bf39823051d55778d15b44285b58b56db468040f42b645208abd84a2f00e27a227d2dcbc04d593d428b8aaf424494ca9fc3898049e4b5d562b41f98388487036c83400c59c09ae23091f6ea2e37f248f409ed5e801ca59f119820510bcd535f508127ca3fe9c38765c143dbb44be9f556ad3e4a4c8014e3e0704c9a07636322335a3c663ec9fd9df8b7bf71d6e8183fefecfbfe0e50808387a771b3f40eeeca915cd56599105120bd236e9da44122032155f9961c69638073f910fc27a7ebad806ebcea959a18debc401163da29832d9e8d87c13fbd270b80450ee2563f72918bfa60fbee9eeb1523b5c0c03ca4dced71d9896fc30c57157680e83c74d69e5cd6dee4cd34bed845929f515fd99ea8b0f2c9f4f41ee1108695e680ecfada42d946722ae4377aa62c262fe1e7e33b300c0eaa0c6d9451defbc740c580369fc68368609c90c18c4378933b282c57c2973beabb7baad0aef61ef09a7ba480f395b7003a2eb1e39c624b9a707a6cb58c3cb6997932fc80662ae19c785a91f580b5e5172489541dfc581e116554b63de15fddf38ffed2b109394749c20b8f6ce380646ba914c6984d5d37b95687a778e2b80628f739ee223273fdc51e179393e581").to_vec(),
			hex!("9e75edf06348b4330d1e88564111cb3d3000505f0e7b9012096b41c4eb3aaf947f6ea42908000080f55a9532867b91cdfb7c67084730fa19f121659a7050b9cf3dce004be452bea4").to_vec(),
			hex!("9f0bf19e4ed2927982e234d989e812f3f30401809aeada952657ae61b8e86d7c3cf64315e71658d378e1e0d9a418a97e3fe19c4d8028d2665c55ce38abb47566d37912f1947f6d126e68b86a8656d1ab187a82553f").to_vec(),
		]);
		let dip_proof = DidMerkleProof::new(
			vec![
				hex!("80f7eb00000000000000000000000000").to_vec(),
				hex!("81016d8b000000000000000000").to_vec(),
				hex!("7f0005e69aa86ca4ef945c050f71df4018c40043e57994b70f1ce8d30723b52e78f300").to_vec(),
				hex!("7f0003125b5671d675e64bf07c911e6aed00eed230bf5d89b4063faf4d8b2738660000").to_vec(),
				hex!("8020020000").to_vec(),
				hex!("7ec2a225d9f89d836a1a0918ac43ea276717c82e6555dd2d935c77b55e6a2af700").to_vec(),
				hex!("7ebaf365662e138e79e9f20d4b811470421fb5456533f27cb9d86f74ff58338500").to_vec(),
				hex!("7f00054c64e7c349536628c7b8dc8bd4f69d23f409ab6b8a2586a0441252b6eec13b00").to_vec(),
				hex!("7f0003bfdd8989572fafed46db59551a54167229f94e4c399a8a45680e7f716ff7bb00").to_vec(),
				hex!("7f000ef8883295cd31bd47965d3d291ae8f4b8b12b2e7c0346f804ac3c87c9fad44c00").to_vec(),
				hex!("7f000068f812b08c085e9c13ebdac3e3e02ca5e4d72f825730ce62c1860133af6ed300").to_vec(),
				hex!("8002400000").to_vec(),
				hex!("7e5e2ec431e7b58042d14f77082c639609873ef15986d7162ecfb8de44e3d28700").to_vec(),
				hex!("7eed7077ac0d92683c6200b724fbdc305854691c80a29bdffefb5ec4a40dac3900").to_vec(),
				hex!("7f0005cb1d1b41caf49a644faa643ed53235da11e27bb32dea75a997bf519bb24ef300").to_vec(),
				hex!("7f020529c46d67b890ac140fb81c80b4db478ed7f6bcef7de075410ff6e418906bb70000").to_vec(),
				hex!("7f0208eae731f8e1832afdfbffbdba974985ddc243c4b21a0b26ea491e537d0a8a230000").to_vec(),
				hex!("7f02096cf53b2d24cbf261c63666dc5264a223014143eb53b3068b8a22712f027c670000").to_vec(),
				hex!("6f0c663831623736356330643338386364666663636537386500").to_vec(),
				hex!("7f0202d18e551945503ab4819410b00a03f9d0837b1ec94dadfbed10a329026b15990000").to_vec(),
				hex!("7f020e22736773636ca149240d65840a7a9e8f730c7fb49595924160c10a999a133a0000").to_vec(),
				hex!("7f040e2e5a8f885d4e9cf36e7b712dccbc02370a1b9832b086f7546b9b19b8e16a76010300").to_vec(),
				hex!("7f020bb048fe494d7a31ee94481770cc65b413767310558907cc12da89c63847dd680000").to_vec(),
				hex!("7f020ed56fa195ae408557da06b1fda161bbb3d59decbdf8da11b62d94ba4a97a5050000").to_vec(),
				hex!("7f0403a9176af554951693286560dff02ac1839fc78dfd541ae5420fe4add3bd651e010100").to_vec(),
				hex!("8000420000").to_vec(),
				hex!("7f0178b1f175a02e5e32150ae7467eff3364833fecc8a5854c64cd394ca06348320000").to_vec(),
				hex!("7f018df100a81fa9c82787ec56d3abc219dba95d49d4bad8339776dc1ab2bcf4560000").to_vec(),
				hex!("7f0402b08a60751c017f70c3d006dc34d94210942562bc5b27af012534d9627dc3d8010000").to_vec(),
			],
			vec![
				RevealedDidKey {
					id: hex!("f2b08a60751c017f70c3d006dc34d94210942562bc5b27af012534d9627dc3d8").into(),
					relationship: DidVerificationKeyRelationship::Authentication.into(),
					details: DidPublicKeyDetails {
						key: DidVerificationKey::Sr25519(sr25519::Public(hex!(
							"aea16625a4c4ad138e416da9a3fa2302a4d0f367029d1784cbc85d226705684e"
						)))
						.into(),
						block_number: 26,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("0123125b5671d675e64bf07c911e6aed00eed230bf5d89b4063faf4d8b273866").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"01588da8661499cdcdcdd8cb2cad3f025ccfd81dc68e89ea271a72f4a2632a74"
						))
						.into(),
						block_number: 26,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("1529c46d67b890ac140fb81c80b4db478ed7f6bcef7de075410ff6e418906bb7").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"6dbf487eb115549616188f18b065ed0b8250eec0932b6dc12af47365215e833d"
						))
						.into(),
						block_number: 26,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("28eae731f8e1832afdfbffbdba974985ddc243c4b21a0b26ea491e537d0a8a23").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"7a6c0ad3cf5f3121f8ee1dfb98543c8e39e1a71bda6ca8b94af9f384d7bfb44f"
						))
						.into(),
						block_number: 26,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("496cf53b2d24cbf261c63666dc5264a223014143eb53b3068b8a22712f027c67").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"143cdc8c7ac305ca85385d1fd47277775b9a6d5a3595a98cfc8676a9d2274449"
						))
						.into(),
						block_number: 26,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("62d18e551945503ab4819410b00a03f9d0837b1ec94dadfbed10a329026b1599").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"e2d9f974b8d5483ca6c2f82750889bcde7c7c5b1f5ed5c871dab09c2518fd75c"
						))
						.into(),
						block_number: 26,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("7e22736773636ca149240d65840a7a9e8f730c7fb49595924160c10a999a133a").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"095de33b750ba03738cc2ca30920b1758318145e7f96b6ce952177b1f5714229"
						))
						.into(),
						block_number: 26,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("9bb048fe494d7a31ee94481770cc65b413767310558907cc12da89c63847dd68").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"014154bcc7195582e2ad289604082a3b0f23af6ba70bc24ddbc36c5e9c622f21"
						))
						.into(),
						block_number: 26,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("bed56fa195ae408557da06b1fda161bbb3d59decbdf8da11b62d94ba4a97a505").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"6922c04846b47fcc23a1936e95457183ed3c4f155624a23f50bfeb14e8647466"
						))
						.into(),
						block_number: 26,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("e978b1f175a02e5e32150ae7467eff3364833fecc8a5854c64cd394ca0634832").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"f5437452eef8fa791b1bd66c24ad01bae3da4f04ec93a7525f5d1100d3cb4e14"
						))
						.into(),
						block_number: 26,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("ee8df100a81fa9c82787ec56d3abc219dba95d49d4bad8339776dc1ab2bcf456").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"72225bb468e6dfead4afe19d7b5971c6934896fb0563e9a03224c27c5c12ca72"
						))
						.into(),
						block_number: 26,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("d3a9176af554951693286560dff02ac1839fc78dfd541ae5420fe4add3bd651e").into(),
					relationship: DidVerificationKeyRelationship::CapabilityDelegation.into(),
					details: DidPublicKeyDetails {
						key: DidVerificationKey::Ed25519(ed25519::Public(hex!(
							"d790fd1b9eb633be20d16d381fdecf6cc826357164059a0fa2e44a4964551088"
						)))
						.into(),
						block_number: 26,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("8e2e5a8f885d4e9cf36e7b712dccbc02370a1b9832b086f7546b9b19b8e16a76").into(),
					relationship: DidVerificationKeyRelationship::AssertionMethod.into(),
					details: DidPublicKeyDetails {
						key: DidVerificationKey::Ed25519(ed25519::Public(hex!(
							"71b66f3e5594cff1df7373e7795739219a0acdeebe6bd7f691783fee120b5747"
						)))
						.into(),
						block_number: 26,
					},
				}
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("9068f812b08c085e9c13ebdac3e3e02ca5e4d72f825730ce62c1860133af6ed3")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("554c64e7c349536628c7b8dc8bd4f69d23f409ab6b8a2586a0441252b6eec13b")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("8ef8883295cd31bd47965d3d291ae8f4b8b12b2e7c0346f804ac3c87c9fad44c")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("63bfdd8989572fafed46db59551a54167229f94e4c399a8a45680e7f716ff7bb")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("b15e2ec431e7b58042d14f77082c639609873ef15986d7162ecfb8de44e3d287")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("39baf365662e138e79e9f20d4b811470421fb5456533f27cb9d86f74ff583385")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("beed7077ac0d92683c6200b724fbdc305854691c80a29bdffefb5ec4a40dac39")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("f5cb1d1b41caf49a644faa643ed53235da11e27bb32dea75a997bf519bb24ef3")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("05e69aa86ca4ef945c050f71df4018c40043e57994b70f1ce8d30723b52e78f3")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("35c2a225d9f89d836a1a0918ac43ea276717c82e6555dd2d935c77b55e6a2af7")).into(),
				)
				.into(),
				RevealedWeb3Name {
					web3_name: b"f81b765c0d388cdffcce78e".to_vec().try_into().unwrap(),
					claimed_at: 26,
				}
				.into(),
			],
		);
		let signature = TimeBoundDidSignature::new(DidSignature::Sr25519(sr25519::Signature(hex!("804e4e48dc9dc920e14edb8d3590c1bbc3523c60088d7a71250c3e30a265b77704d7e244bfed86a42126a326c7ed15ae707c888b788202c8e9c488071859228a"))), 79 as BlockNumberFor<Runtime>);
		let proof = ParachainDipDidProof::new(provider_head_state_proof, dip_commitment_proof, dip_proof, signature);

		BlockHash::insert(
			0,
			H256(hex!("8704d60f04d95d6cd6b774a84582b251a3129bb8f88b5c564447a76f31d0857b")),
		);
		LatestRelayHeads::insert(
			97,
			RelayParentInfo {
				relay_parent_storage_root: H256(hex!(
					"5977f2b96c1982d205055cfd1ce41592e9cd435770f62d7e11b9583d25fa67a8"
				)),
			},
		);

		WorstCaseOf {
			proof: proof.into(),
			call: pallet_postit::Call::post {
				text: b"Hello, world!".to_vec().try_into().unwrap(),
			}
			.into(),
			// 4rs36nx5DuPgvJMs5bd2C8X8ySnGnc8KLsP2BUuJbxtYWEBn
			subject: DidIdentifier::new(hex!("aea16625a4c4ad138e416da9a3fa2302a4d0f367029d1784cbc85d226705684e")),
			// 4pMywUEABML35y2feheNsQDJFqYVTrUZbCww7XLRMkZxfmbm
			submitter: AccountId::new(hex!("40002ce6270685b06ea56b9f5594efc9422ae8e498ee202ef3d886d84c4b343e")),
		}
	}
}

#[cfg(all(test, feature = "runtime-benchmarks"))]
mod worst_case_tests {
	use kilt_dip_primitives::VersionedDipParachainStateProof;
	use kilt_support::traits::GetWorstCase;
	use pallet_dip_consumer::benchmarking::WorstCaseOf;

	use crate::{dip::MAX_PROVIDER_REVEALABLE_KEYS_COUNT, ProviderTemplateProofVerifierWrapper};

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
