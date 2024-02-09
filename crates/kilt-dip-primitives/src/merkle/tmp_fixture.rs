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

use crate::{
	DidMerkleProof, DipCommitmentStateProof, ParachainDipDidProof, ProviderHeadStateProof, RevealedAccountId,
	RevealedDidKey, RevealedWeb3Name, TimeBoundDidSignature,
};
use did::{
	did_details::{DidEncryptionKey, DidPublicKey, DidPublicKeyDetails, DidVerificationKey},
	DidVerificationKeyRelationship,
};
use hex_literal::hex;
use pallet_did_lookup::linkable_account::LinkableAccountId;
use peregrine_runtime::Runtime as PeregrineRuntime;
use sp_core::{crypto::Ss58Codec, ed25519, sr25519, ConstU32, H256};
use sp_runtime::{generic::Header, traits::BlakeTwo256, AccountId32, BoundedVec};

// Pre-generated fixture for:
// * DID subject = `5GRFonySFTkU7pNbdG48ZFLpFdJVfxPMFtd5DqpNohEWshMB`
// * Provider para ID = `2_000`
// * Call = `system.remark("Hello, world!")`
// * Relay state root =
//   `0xd9abcc76ff142acfb4bd44cc99fabd1f121b15df699b1c91bc1cf6e35afd48fe`
// * Consumer genesis hash =
//   `0x9776c2a6921124e360d8d444113d596077a7e9eab629458f8af8a92c54287577`
// * Submitter account = `5HQuQWj2YHu9jNxQSBnn4SYWdnB6WenNP4UDAC5pJktHj7gK`
pub(crate) fn test_parachain_proof(
) -> ParachainDipDidProof<u32, H256, AccountId32, u64, BoundedVec<u8, ConstU32<32>>, LinkableAccountId, u64> {
	ParachainDipDidProof {
		provider_head_proof: ProviderHeadStateProof {
			relay_block_number: 133,
			proof: vec![
				hex!("3703f5a4efb16ffa83d0070000b51fa6ac4aec5888d5a7651de99c69e8987c508c53209dbb33bdf3f7be1106be").to_vec(),
				hex!("7d037a8ecd5c1d9617d6414ebe12c8649ee2439382ce07c8c684d325b5fbebbfc02a70350f54b9ee737200d6438f75bb06727211c703b6896df29ba3282e29831cd5cbcdc1d866ce99a8ef8429bd54455dc747855c3b18892c15112fffce0eef94e8e10c066175726120f2257b0800000000045250535288aa1110c539e09aa145befd11e66635086372f219346346c6cd1a5fe60cada4449d0205617572610101c0a8361b59bd92bef5f0ea940450e49fb2422fe8f8c34bb3cc0e6d111894e42a3980b3b9ffb1de6abcd7ae4a8b0c5ed63bd549deceb917bf11d93a1050c75a82").to_vec(),
				hex!("8004648031b60c9237ed343094831987f2bec10b211621255ad0b440cf161fa820d30db480f6f6801e4b41e2e6d8ec194dba122bfb9eb33feb2545ef5144cea79551f7cc52801187a2c514af5fa677badbf02127d966ae26d909b38ae7c9459d9e807c18358c80b2b9f1cfbb40aca8faba1ac21adaf0724248ff6a212a11e9e336b8cb569574b1").to_vec(),
				hex!("80ffff80e89fa0ab954678f9793f6ecc78b508c11dd51a747716c850eb4c3007ed8e9d718018b40545390cadf65df6925d9b606ee7a96bd9c0276d6740a1f82e890348d821809f691b831c41bce1ce43b19888655d1c4484e86c8e93af0a7062c6406a39dea180c1ed537d90117b0fd7ece97455cb1e1a340bd627b9e3a3688a3e087b7b99138480ed071b9268ef5729178f35fb1187840de04a839ef132bb52b6ad679eff40bcb18096880c322e10e912229d47be232e0e959782d22c632413d44b7fa40f0a86689980c6ba3c6f9640dc8392b4cab48947087e243b9cf35b01c030659cfde882791922805b682132c52908705526057f73ab7fccab4af6d72a9805634dd8d3cc53f130d180c2d44d371e5fc1f50227d7491ad65ad049630361cefb4ab1844831237609f083805dbbd7345c705dc69aeeea3753cae5fc602d77270eaf912335ab6aef8e586cb780219ef48ff0042bb3bca7870d722f46e3c6ce61aa4dedbb8453fe507d2a87891d807e335564da75bd7790c854fd3a03088feba837c136539ba9432e848e48670494807ce7eb9130cc38c3b19e0336ce70178f60fdb8e9cdbca7e90286f8810e781dce80ed1ddd4ad533bcf9a0e20b7a34da2a0363b56f5c7d5a73661a7847886b0b740d806bfdbbf0e0bedcb993b65c9cea1e929a56d78a3b7bc53d1b7ca6fc488e2295ee801865df2e2c93dd0c04ee5ac7a78cfb0a60e604bbde616d48ea4eb29833c2b8f9").to_vec(),
				hex!("9e710b30bd2eab0352ddcc26417aa1945f43803b3441f15daa8a53147d69c48eac75356fab581febbb8030520b248c5942a14880a7f64483d6f6ad9988a5f70d7f800442f8f0c67f3464d605f652ecf16e50afcd802e2e0716043a02f2f29fdd52922704af194b98545ce5ca832255e8ec41bcdb6480324c78e69e086156698b29bd28dcc399bf27680664083596530c8fe3d8f6376b505f0e7b9012096b41c4eb3aaf947f6ea4290800004c5f03c716fb8fff3de61a883bb76adb34a20400808a3fa1efcb88075c83f538ff451258b189c1f035f25b7110c2ec1c83172649334c5f0f4993f016e2d2f8e5f43be7bb25948604008068d9a540d50271d637cc8ef5c1e7823740ff69c98397c07bfb285c4f6ea1d4e8").to_vec(),
				hex!("9f0b3c252fcb29d88eff4f3de5de4476c350008017df15222a818ac59c79fcd326bf4171d990673367e8e2dd3a535ae30c96a21e8041f5957e9832dadd679619862823fbb20923948873485de7a4d384d35d8565c7").to_vec(),
			].into_iter().into(),
		},
		dip_commitment_proof: DipCommitmentStateProof(vec![
			hex!("7f2400244c8b3698a0afc09d6588e8de09af934b06737ea1784caa3c69c2e7f0f30448a3daad2830627301a12dfa1fa4ab9a00008052972e60958a47cfb74c1c3c8b9de35d69bbb0583a74f6b13ff12dc52df45b27").to_vec(),
			hex!("800c8080da28793d083b197f8d92fc3e77f5064436f1d8eea0fbea56ddb936aba6544500803fffbd930653861eb1eea7be762a5be6211cad9f366994a7bcb6c6e7f9bafc1b806715ebb6fea7a99de30d1e97f3baa0ace8c1023f6f89eeeb84764e3def519674").to_vec(),
			hex!("80ffff80353e4d164b13c87910044f1b4e76277e404a0ab46a7cd6c33a65aaadc2375ba88007b1390da34b4dce1328430fd924a6e193517a8148dd70a912c0dc2f7f8d2d4c80ef04a3d3232df9064e2e1533591f55b8958129f4237f6b08301add3c882c5dcd80a1a190ebe1f962278b83152848f385b6cf334755c2bb5a85ed786827d962bee2807e3e49dd79670d895cffa69ddff9b4288cb359b10580130dc2b33bd60a8d18d580b83404aef4d469d23f8486b14cf20c6b5836e64938aa71416aa4a7699fecbfea8014e3e0704c9a07636322335a3c663ec9fd9df8b7bf71d6e8183fefecfbfe0e508006747e15a9ef418580b513a49e6e34a34817b510aaa7a5b06d479110a9f555ea80c58caf2059aa913d0d96e92baf6bbd5d1fc749d5d74fdfdf6705752264f884fa8022aa27a91d5c76901df3b14d21903e700f049a30a7c007e5deef99673fe9994b80d2a22db0f118e6eb30d4be956c44427cc4c6b7b75ae12b43a71dde3365c7ec34802cd11c6aa6b488df14310dabf7f76d54b695dd47d2602c47a82b9f723961bfc880c2881f1acff032edb4b7bf0d1608edd1808092944b255fea7eaa44343ee4f44480f395b7003a2eb1e39c624b9a707a6cb58c3cb6997932fc80662ae19c785a91f580b5e5172489541dfc581e116554b63de15fddf38ffed2b109394749c20b8f6ce380a84d800e9df2587c3bc0da5d4eae41f204b9a8ba74abdde8aba5c24874a365e1").to_vec(),
			hex!("9e75edf06348b4330d1e88564111cb3d3000505f0e7b9012096b41c4eb3aaf947f6ea4290800008090d625a34d8b3e81c50c42bedd620de00c65a136d924e73b4ee04681dd99c5e3").to_vec(),
			hex!("9f0bf19e4ed2927982e234d989e812f3f328008019b4e6d1acb76af7407575df6e2fef3d0dde23d810972faf7eda5c0f0271ec1c80e134dcac81b7b98a2a8f96ac17874dc271cd7d647db7865cca3b9f7f0e20afd9").to_vec(),
		].into_iter().into()),
		dip_proof: DidMerkleProof {
			blinded: vec![
				hex!("806bcb00000000000000000000").to_vec(),
				hex!("8002040000").to_vec(),
				hex!("807365000000000000000000").to_vec(),
				hex!("7f000edf371f2d1333fba36905e56733d1e7d9f5ac73eb12257f2b9e8b7df5088f0200").to_vec(),
				hex!("7f00045e18ea7dd30548dfe9cbb26d07a3dd1f6faea96bb22387a8d28dddcab71beb00").to_vec(),
				hex!("7f0001d65495de3c1b11fe30c19213685b496c37a9df8a526fcfde19eb80ed4ad70300").to_vec(),
				hex!("7f000f95383fa0bd7e353aff4e15b7b6c9a0cdb20d07c2fb00fe92add21cfdc55f4c00").to_vec(),
				hex!("8000140000").to_vec(),
				hex!("7ea7f6ac2a8c0161b6ae25369d39daefde60f76db6c9f82c64f4eb5b3a9c330500").to_vec(),
				hex!("7ee629184f476a03d41408d498638a5b80bca30185e04d0bed2d9a8a918e962000").to_vec(),
				hex!("7f0002cdec26932b726664108a9c7692a3518954643248b69663e25c4a9588c880a600").to_vec(),
				hex!("7f000c1712527b9c708576151bca932dee7448755179d71d0083679b353783022f3500").to_vec(),
				hex!("7f000ba03a30eb82f99dc4f3c5e2a31cec7e123c17c9edd76a374e1606098ab44af800").to_vec(),
				hex!("7f000408ba21b9ab08cdd79903073ceb8eaab0e270e6926a6055da6396431628e65500").to_vec(),
				hex!("7f0119831adb0c05a7f02340336e05237a6123eb5a801e700b05ec96e414c1d4310000").to_vec(),
				hex!("8080800000").to_vec(),
				hex!("7f015b6b8922a34836cc6133b1382342934cb9f66939dbcc8003b24c520112efb40000").to_vec(),
				hex!("7f033754f2a5612de4843170fd452e274002cdff2dfc103c348d829e0aa25a2e1f010000").to_vec(),
				hex!("7f02051e3e18823728847b6bdeb7b3f852e33153db8774a156ccd642c2da8cb1694a0000").to_vec(),
				hex!("6f0c303463393337386438373637653631363436626666323800").to_vec(),
				hex!("8080010000").to_vec(),
				hex!("7f01418bc51fbfa1b1b7d1dda00a85e31b5b32b7f90a168155ae95b2bd03dfc3270000").to_vec(),
				hex!("8002020000").to_vec(),
				hex!("7f00023c7ea51367ac69cf4ec1c65d3d173a869b1eb4b1909ca48785ab5bfb9dff0000").to_vec(),
				hex!("7f000eacf997d73c52a8fc1822c043bf8e9277e1c5214c1c8fcfb90241b995fc120000").to_vec(),
				hex!("7f020667d7743ecf128174c23fa8f9abd5ff2091ccfd89ae5f63b863d14f98a150070000").to_vec(),
				hex!("8004800000").to_vec(),
				hex!("7f033e1923acaee602c41d48354b82af0e7953b06f4f0fac7a306f0320e9ec2618010100").to_vec(),
				hex!("7f019ecaba66c7fbd269060e32deeedce9da662b9b079149dde9291a78841c2c820000").to_vec(),
				hex!("7f040437840b0bdbf4b53b930ee023fd2f9b3cbd52665034ab5297ff71aee7d3a274010300").to_vec(),
				hex!("7f020d227568bd1c86d3d227d7c510b76025a624cbcb2eaad82f0f6707d1164df73d0000").to_vec(),
				hex!("7f020a552bf8d89287ecfda14cb43eb6ec112c1d161b96a3921d6b3f0923d3c30ef10000").to_vec(),
			].into_iter().into(),
			revealed: vec![
				RevealedDidKey {
					id: hex!("1f3754f2a5612de4843170fd452e274002cdff2dfc103c348d829e0aa25a2e1f").into(),
					relationship: DidVerificationKeyRelationship::Authentication.into(),
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicVerificationKey(DidVerificationKey::Sr25519(sr25519::Public(hex!("c09d6588e8de09af934b06737ea1784caa3c69c2e7f0f30448a3daad28306273")))),
						block_number: 26
					}
				}.into(),
				RevealedDidKey {
					id: hex!("0a19831adb0c05a7f02340336e05237a6123eb5a801e700b05ec96e414c1d431").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("b432ae5e381e5184c39184b19461427677bd65b1215080b831d46e6fc2d9fc71"))),
						block_number: 26
					}
				}.into(),
				RevealedDidKey {
					id: hex!("175b6b8922a34836cc6133b1382342934cb9f66939dbcc8003b24c520112efb4").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("c2ee634220b3f9dea971fceb8c26a8dd5bced0b5be38809253f71d2f03d92f15"))),
						block_number: 26
					}
				}.into(),
				RevealedDidKey {
					id: hex!("351e3e18823728847b6bdeb7b3f852e33153db8774a156ccd642c2da8cb1694a").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("25b17375dbe706f2a6d40b6c3d52c2e2c910feb1491540d5c161e0248ac95b2a"))),
						block_number: 26
					}
				}.into(),
				RevealedDidKey {
					id: hex!("67418bc51fbfa1b1b7d1dda00a85e31b5b32b7f90a168155ae95b2bd03dfc327").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("8bfe06f78bdbae17cae9c8168aa893c248bf835223c623f3ab57720fa29f5a60"))),
						block_number: 26
					}
				}.into(),
				RevealedDidKey {
					id: hex!("68123c7ea51367ac69cf4ec1c65d3d173a869b1eb4b1909ca48785ab5bfb9dff").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("4b33d1675f3d004aad2a6196f02f6f7ab6fe87d851824b5e107dada61fc22031"))),
						block_number: 26
					}
				}.into(),
				RevealedDidKey {
					id: hex!("689eacf997d73c52a8fc1822c043bf8e9277e1c5214c1c8fcfb90241b995fc12").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("f4954dc22dba1b26dc7f2606d7e3714386ee5e0e026dd96ff62d17944d93f327"))),
						block_number: 26
					}
				}.into(),
				RevealedDidKey {
					id: hex!("8667d7743ecf128174c23fa8f9abd5ff2091ccfd89ae5f63b863d14f98a15007").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("bfa563dd933e3319a062b38271549c09b5584b67b971d60000194f3725197c45"))),
						block_number: 26
					}
				}.into(),
				RevealedDidKey {
					id: hex!("9f9ecaba66c7fbd269060e32deeedce9da662b9b079149dde9291a78841c2c82").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("e88eee2b1b0d6abec878c6b6dca75b9a7323e5b9fafcb9e3d21a92e3e28d8174"))),
						block_number: 26
					}
				}.into(),
				RevealedDidKey {
					id: hex!("ed227568bd1c86d3d227d7c510b76025a624cbcb2eaad82f0f6707d1164df73d").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("42ac6e9563af67b3e8c43dad68ed174776f1f5862bd80aad54f8bcb90f188e66"))),
						block_number: 26
					}
				}.into(),
				RevealedDidKey {
					id: hex!("fa552bf8d89287ecfda14cb43eb6ec112c1d161b96a3921d6b3f0923d3c30ef1").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("915924b791c34a7d063d26672aef12ad05d78f1e1bacfda49fb630d36f487f00"))),
						block_number: 26
					}
				}.into(),
				RevealedDidKey {
					id: hex!("923e1923acaee602c41d48354b82af0e7953b06f4f0fac7a306f0320e9ec2618").into(),
					relationship: DidVerificationKeyRelationship::CapabilityDelegation.into(),
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicVerificationKey(DidVerificationKey::Ed25519(ed25519::Public(hex!("693af6db28b33940820bf2d52605c873c374e555a5fe7af515f8923788b7eca0")))),
						block_number: 26
					}
				}.into(),
				RevealedDidKey {
					id: hex!("b437840b0bdbf4b53b930ee023fd2f9b3cbd52665034ab5297ff71aee7d3a274").into(),
					relationship: DidVerificationKeyRelationship::AssertionMethod.into(),
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicVerificationKey(DidVerificationKey::Ed25519(ed25519::Public(hex!("9f32c15d8eb12a4a5a8e76100e17c8f2801f2ca782e59e779543c4315c6eefcc")))),
						block_number: 26
					}
				}.into(),

				RevealedAccountId(LinkableAccountId::AccountId32(hex!("82cdec26932b726664108a9c7692a3518954643248b69663e25c4a9588c880a6").into())).into(),
				RevealedAccountId(LinkableAccountId::AccountId32(hex!("e408ba21b9ab08cdd79903073ceb8eaab0e270e6926a6055da6396431628e655").into())).into(),
				RevealedAccountId(LinkableAccountId::AccountId32(hex!("145e18ea7dd30548dfe9cbb26d07a3dd1f6faea96bb22387a8d28dddcab71beb").into())).into(),
				RevealedAccountId(LinkableAccountId::AccountId32(hex!("0edf371f2d1333fba36905e56733d1e7d9f5ac73eb12257f2b9e8b7df5088f02").into())).into(),
				RevealedAccountId(LinkableAccountId::AccountId32(hex!("6aa7f6ac2a8c0161b6ae25369d39daefde60f76db6c9f82c64f4eb5b3a9c3305").into())).into(),
				RevealedAccountId(LinkableAccountId::AccountId32(hex!("dba03a30eb82f99dc4f3c5e2a31cec7e123c17c9edd76a374e1606098ab44af8").into())).into(),
				RevealedAccountId(LinkableAccountId::AccountId32(hex!("6ce629184f476a03d41408d498638a5b80bca30185e04d0bed2d9a8a918e9620").into())).into(),
				RevealedAccountId(LinkableAccountId::AccountId32(hex!("41d65495de3c1b11fe30c19213685b496c37a9df8a526fcfde19eb80ed4ad703").into())).into(),
				RevealedAccountId(LinkableAccountId::AccountId32(hex!("ac1712527b9c708576151bca932dee7448755179d71d0083679b353783022f35").into())).into(),
				RevealedAccountId(LinkableAccountId::AccountId32(hex!("5f95383fa0bd7e353aff4e15b7b6c9a0cdb20d07c2fb00fe92add21cfdc55f4c").into())).into(),

				RevealedWeb3Name { web3_name: b"04c9378d8767e61646bff28".to_vec().try_into().unwrap(), claimed_at: 26 }.into()
			]
		},
		signature: TimeBoundDidSignature {
			signature: sr25519::Signature(hex!("ea3cc9d5980fe498b350ed0de15d3ab54288c12c03ca2441bb80f997cf8e0858cd2e86e82b636bbb9f97fa5ed47c6e48c1f3aba1bbf0cf65c0fa25371870fe89")).into(),
			valid_until: 81
		}
	}
}

#[test]
fn test() {
	use parity_scale_codec::Encode;

	env_logger::init();
	let proof = test_parachain_proof();
	let proof_1 = proof
		.verify_provider_head_proof_with_state_root::<BlakeTwo256, Header<u64, BlakeTwo256>>(
			2_000,
			&hex!("d9abcc76ff142acfb4bd44cc99fabd1f121b15df699b1c91bc1cf6e35afd48fe").into(),
		)
		.expect("Should not fail to verify relay state.");
	let proof_2 = proof_1
		.verify_dip_commitment_proof_for_subject::<BlakeTwo256, PeregrineRuntime>(
			&AccountId32::from_ss58check("5GRFonySFTkU7pNbdG48ZFLpFdJVfxPMFtd5DqpNohEWshMB").unwrap(),
		)
		.expect("Should not fail to verify DIP commitment value.");
	let proof_3 = proof_2.verify_dip_proof::<BlakeTwo256, 50>();
	println!("{:#?}", proof_3);
	let proof_3 = proof_3.expect("Should not fail to verify DIP DID proof.");
	let proof_4 = proof_3.verify_signature_time(&33).unwrap();
	let payload = (
		frame_system::Call::<PeregrineRuntime>::remark {
			remark: b"Hello, world!".to_vec(),
		},
		&Option::<u128>::None,
		AccountId32::from_ss58check("5HQuQWj2YHu9jNxQSBnn4SYWdnB6WenNP4UDAC5pJktHj7gK").unwrap(),
		81,
		hex!("9776c2a6921124e360d8d444113d596077a7e9eab629458f8af8a92c54287577"),
		(),
	);
	let proof_5 = proof_4
		.retrieve_signing_leaf_for_payload(&payload.encode()[..])
		.unwrap();
	println!("{:#?}", proof_5);
}
