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
// * DID subject = `5F7Q4Tv8A2Wob14H6V7eGqhhcFEXzjZXSDptYrhxdxATe5qV`
// * Provider para ID = `2_000`
// * Call = `system.remark("Hello, world!")`
// * Relay state root =
//   `0x4a8c971e646cee7c5bc37b1568acfdc5efd4c7ee5dd449946eddc43b86ad44e3`
pub(crate) fn test_parachain_proof(
) -> ParachainDipDidProof<u32, H256, AccountId32, u64, BoundedVec<u8, ConstU32<32>>, LinkableAccountId, u64> {
	ParachainDipDidProof {
		provider_head_proof: ProviderHeadStateProof {
			relay_block_number: 133,
			proof: vec![
				hex!("3703f5a4efb16ffa83d007000093d85e0dad808ee5f36da4db4b3121b2a86ef1f8a5a733f115aaf31ae9a9af70").to_vec(),
				hex!("7d0382df17ef0c3f7c9655c0a27b80381bfec6b4c5f9d2a5ec6c2921abb537d996327c350abbed7b44e2ee1c4346a59527379271d362c111019fc54e7dbbd00ded18b53ee8d614c30e4ec7a2d5ff38b594ce53d713e13f5c524a193dd0754e74e8cc980c0661757261206cd27a0800000000045250535288ddaee22afbac7e7203eb2fa2ea527c1bd4a949876726edb58f2c9303c15f3fe50d0205617572610101b89407654c769f5841d62a07c7d64beaff531cf4b257bf66175b3181cd6d7653c1562f734b6772fccb9c2e1de9846ddad1051212a8f6e4712f538993d668db87").to_vec(),
				hex!("8004648031b60c9237ed343094831987f2bec10b211621255ad0b440cf161fa820d30db480f6f6801e4b41e2e6d8ec194dba122bfb9eb33feb2545ef5144cea79551f7cc52804decd4c93c57d7947b88e0d4f24c20a9972752e5823006b0b30cc5767b46028c80aab0134e3fd3141b21dc8bf5742be1bf3d02b897fd9d972635698bf628a79728").to_vec(),
				hex!("80ffff80992aa1d896c4dbe65d1129ef0330779cf1c9f048d3ca12ab5576417c4386df8a80c81755b3ea602b9fad21a5baa5b7e4ca0fe40cd51f1eeec330006d8c937426d78055849a7cbcb04b9f84b7de948a20c0524d33a4811b83fe1d3070279b54abe9a8805d6d97510801ae466ce026f778864148268b4a9aebd01fd7dce5e4cc0967efa48043615615d8cc32d5c71c0a68a0669134f9f9a7506e052f2f5b39e8eccc5f29e7809d5725dd1de5192503a7153fbc0329782b87a28032983160e5b56d3f67d4b7a5802c01fe7da5c29dfff1a668f85f54a5cb721ac5dd805dafcfce180cd967a09eeb805b682132c52908705526057f73ab7fccab4af6d72a9805634dd8d3cc53f130d180c2d44d371e5fc1f50227d7491ad65ad049630361cefb4ab1844831237609f08380532fb6cf30d0ecda87f1af18123a8d80efc9749940066fb2e99d9afa930d925a803190111ca643947166119ace77ccee2c6310d653e77c8b18e85838519612a6dc80400f26558d8e918f6fd14ac5b07cf998aa0a693cdb5bf01778ae8ff2a703379c8008df63d899d47defd0619ed1f16e2ea7d25e25a26e8eeefc582f4ed00f37318b808d76109701165927974dd013a53869bfb3f5f948558a2aa7d7a7579040bd25a8806bfdbbf0e0bedcb993b65c9cea1e929a56d78a3b7bc53d1b7ca6fc488e2295ee8039e62a01279cafb588bacecc9e810508588cdf5a4bf654b8e8089d00d8ebde22").to_vec(),
				hex!("9e710b30bd2eab0352ddcc26417aa1945f43803b3441f15daa8a53147d69c48eac75356fab581febbb8030520b248c5942a14880759320262d01a41b11b13b2198b3fc53ef6c0eeb71df226ac6c44fced8d9e059802e2e0716043a02f2f29fdd52922704af194b98545ce5ca832255e8ec41bcdb64800785fa7e4a70b078743c377104d989829614b9298a806fec65095c96460c02bd505f0e7b9012096b41c4eb3aaf947f6ea4290800004c5f03c716fb8fff3de61a883bb76adb34a204008045eb3b829726ba0c40216213fe4552c294095cd1f060d9828ef4b9defd9101de4c5f0f4993f016e2d2f8e5f43be7bb259486040080cf12f7bfee9584ae9324a8773f19f6bc6c16b881d05ffdc19d00f345e82a689e").to_vec(),
				hex!("9f0b3c252fcb29d88eff4f3de5de4476c35000800cbdde632eeb04dbf9b1628fb1b4fa8bcf40f321cdf41b6547563f5012e277ea80f14a31f821270aa703562af1584d60aba2c6d482a8f56793933ce493152f00ef").to_vec(),
				hex!("9e710b30bd2eab0352ddcc26417aa1945f43803b3441f15daa8a53147d69c48eac75356fab581febbb8030520b248c5942a14880759320262d01a41b11b13b2198b3fc53ef6c0eeb71df226ac6c44fced8d9e059802e2e0716043a02f2f29fdd52922704af194b98545ce5ca832255e8ec41bcdb64800785fa7e4a70b078743c377104d989829614b9298a806fec65095c96460c02bd505f0e7b9012096b41c4eb3aaf947f6ea4290800004c5f03c716fb8fff3de61a883bb76adb34a204008045eb3b829726ba0c40216213fe4552c294095cd1f060d9828ef4b9defd9101de4c5f0f4993f016e2d2f8e5f43be7bb259486040080cf12f7bfee9584ae9324a8773f19f6bc6c16b881d05ffdc19d00f345e82a689e").to_vec(),
				hex!("9e710b30bd2eab0352ddcc26417aa1945f43803b3441f15daa8a53147d69c48eac75356fab581febbb8030520b248c5942a14880759320262d01a41b11b13b2198b3fc53ef6c0eeb71df226ac6c44fced8d9e059802e2e0716043a02f2f29fdd52922704af194b98545ce5ca832255e8ec41bcdb64800785fa7e4a70b078743c377104d989829614b9298a806fec65095c96460c02bd505f0e7b9012096b41c4eb3aaf947f6ea4290800004c5f03c716fb8fff3de61a883bb76adb34a204008045eb3b829726ba0c40216213fe4552c294095cd1f060d9828ef4b9defd9101de4c5f0f4993f016e2d2f8e5f43be7bb259486040080cf12f7bfee9584ae9324a8773f19f6bc6c16b881d05ffdc19d00f345e82a689e").to_vec(),
				hex!("9e710b30bd2eab0352ddcc26417aa1945f43803b3441f15daa8a53147d69c48eac75356fab581febbb8030520b248c5942a14880759320262d01a41b11b13b2198b3fc53ef6c0eeb71df226ac6c44fced8d9e059802e2e0716043a02f2f29fdd52922704af194b98545ce5ca832255e8ec41bcdb64800785fa7e4a70b078743c377104d989829614b9298a806fec65095c96460c02bd505f0e7b9012096b41c4eb3aaf947f6ea4290800004c5f03c716fb8fff3de61a883bb76adb34a204008045eb3b829726ba0c40216213fe4552c294095cd1f060d9828ef4b9defd9101de4c5f0f4993f016e2d2f8e5f43be7bb259486040080cf12f7bfee9584ae9324a8773f19f6bc6c16b881d05ffdc19d00f345e82a689e").to_vec(),
			].into_iter().into(),
		},
		dip_commitment_proof: DipCommitmentStateProof(vec![
			hex!("7f440bf19e4ed2927982e234d989e812f3f34654cc5e40e6060086c2871ed4042fc2a4c1399619c4ca17b7c9585768d90b77376045a39d9a702f01a12dfa1fa4ab9a00008030abd7efa72c7cbdb7967be6423b4ac91cf2d2e16b09a92865d21942d7104a81").to_vec(),
			hex!("800c8080da28793d083b197f8d92fc3e77f5064436f1d8eea0fbea56ddb936aba6544500809a8c60b0711d522d3b2a45eaf025e3cde78d1e58f7f790df1ab0b5d0457abb5f8079390a5c412f5df194b86134c8e9b467fda78227f2c86bc52a8edaebc613d2b1").to_vec(),
			hex!("80ffff80353e4d164b13c87910044f1b4e76277e404a0ab46a7cd6c33a65aaadc2375ba88007b1390da34b4dce1328430fd924a6e193517a8148dd70a912c0dc2f7f8d2d4c803b71030515390857c36a33ae72e31365f66aa173d3fce3febd495234b7d0c02e80a303de001167a8ee485de6e2ca02dd28fe7ca0f93f1a17e407d05666766fce4080e97bf24c29677b139d5e68ca3d63ee2ad354badaf8f04b9736f58658f2894ac780687d05beac9e00b21f9703b6de6c09717b56f39aa8a3758f5d9b9651ff56a43c8014e3e0704c9a07636322335a3c663ec9fd9df8b7bf71d6e8183fefecfbfe0e50800f85b4ce0ddce489b4d71597ea06b5367d373381c17227dace2d957ef9d0d98880a5a428d4eeb23d9ef89919c96f6a9062f9733f74e49344122bd16fa928d51123805a7b3e628e3ae6c62e450ad1a3263d600f8950e27f060e2607ea8197cabfa22980debf2fbad7b225ccb8baa0290925169230acda383eb1dce6790270623fdd5e08802e1fb6e72d49c002a378f3a21fbc5e77b4da4defed0cccc7a82e14ca9462cdfc80143fa131937e1df9bf669d09c8f230596638cca865ffefdd94f7ec14098adf9780f395b7003a2eb1e39c624b9a707a6cb58c3cb6997932fc80662ae19c785a91f580b5e5172489541dfc581e116554b63de15fddf38ffed2b109394749c20b8f6ce38067518d32b11790b8ee713aa7181dc4d3e1a17bcb44cd3ed082c9a069a21f8044").to_vec(),
			hex!("9e75edf06348b4330d1e88564111cb3d3000505f0e7b9012096b41c4eb3aaf947f6ea42908000080b2203fb94fc21b7a3e319f23c6d18cfa9c1eaeba945166035b6c85f97381dec7").to_vec(),
		].into_iter().into()),
		dip_proof: DidMerkleProof {
			blinded: vec![
				hex!("80ab93000000000000000000").to_vec(),
				hex!("8012000000").to_vec(),
				hex!("80294700000000000000").to_vec(),
				hex!("7f000753e95eac51474dc20653d86195cd11657b3af8f9af52d03f6b42a6cbe78efa00").to_vec(),
				hex!("7f0006dfc8d778b1b15835d8bc7953b1d782b38aee53ef7785340ec451a1dcc0cdc100").to_vec(),
				hex!("7f000f97e76eefbd998e4886319066e29d7646f7de9b812cef5654536b6a39e257a200").to_vec(),
				hex!("7f0008dd8180874f51ca369e5a970fc2d09789af161a8ae187df94ff1658884441a400").to_vec(),
				hex!("7f0008812f5ef930d26a12e8ebcef5b5ce6f5458af9028694d49902172d969e8373800").to_vec(),
				hex!("7f00036777a228a2e0651764c7de0be063f9a1cc0281aedececc12cfdc69e048b7cf00").to_vec(),
				hex!("80430100000000").to_vec(),
				hex!("7ee591ff4d23b8b5cc3655e59aa140c600565c1c21f27960cfb980a4c74b6b0300").to_vec(),
				hex!("7e54fe906fd0c0227be2967528a972215ceac09ede6167421a761507cea3f1b700").to_vec(),
				hex!("7e0b14479bb4cee4f526ae31541bd3dbc79f541d01cf0a9691c1dbb1bfeb42d400").to_vec(),
				hex!("7e08d3d3fe19d0fd336cff473acbc2cf93074c9e163a5d04a9ea33a918e8910500").to_vec(),
				hex!("7f016171527563d8e98a5222de4a9b4141b088da96377e8efc3388c0538a36fe680000").to_vec(),
				hex!("7f040b4f12a6cc3a3d8c3d2d508aadbfa6b71edbb9ac7ec3da2f0448ca8035f95c22010100").to_vec(),
				hex!("7f02023486d437a096fd79ff975506e5829beace1e88d0348d6e25a6e465a8b89ab60000").to_vec(),
				hex!("6f0c623064383332663863396231343561343535333765383600").to_vec(),
				hex!("8080200000").to_vec(),
				hex!("7f03d0160b28ad4d2f5db38e2192867914fec04eff50dd860bc46df10b36bf3b7b010300").to_vec(),
				hex!("7f018c9f4a66a0e93717dddcadba70889c4424f338f589d09714f239043591440d0000").to_vec(),
				hex!("810c00110000").to_vec(),
				hex!("7f0006c882487160789d32d32e697b2d8e520f1563cb1e4164225753aa28a5ab2f0000").to_vec(),
				hex!("7f0001078ba56113813991934ae0ffabbd5cb562cb6a2fc699f9bb8b4aabb8f6160000").to_vec(),
				hex!("8001040000").to_vec(),
				hex!("7f01fd143d81ddef983d62ce1aa1639168489ff5da0f6702efd258da6c55b0777e0000").to_vec(),
				hex!("7f01eeb6b118090da14b14065df0bfa29a394e3c57574c2cf72fec38027953f2ca0000").to_vec(),
				hex!("800181000000").to_vec(),
				hex!("7f01339da5d721ecf5a599e588a74363db45688e2356ad15bf80f226017bd102480000").to_vec(),
				hex!("7f03585a08f8de24ca2fe9ac237846bfa50ae0be99bc8d1dcd1bbf2d5dcb5469de010000").to_vec(),
				hex!("7f01dcf6e5989bdee27413af55c187c96387dfd2e4f3cde973b250acfac6af6cad0000").to_vec(),
				hex!("7f020da35a39cce44fbfb012b0f6a0a1dfa0b866b0ed8a2f7efb3c7bc944b8b494930000").to_vec(),
			].into_iter().into(),
			revealed: vec![
				RevealedDidKey {
					id: hex!("c8585a08f8de24ca2fe9ac237846bfa50ae0be99bc8d1dcd1bbf2d5dcb5469de").into(),
					relationship: DidVerificationKeyRelationship::Authentication.into(),
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicVerificationKey(DidVerificationKey::Sr25519(sr25519::Public(hex!("86c2871ed4042fc2a4c1399619c4ca17b7c9585768d90b77376045a39d9a702f")))),
						block_number: 30
					}
				}.into(),
				RevealedDidKey {
					id: hex!("046171527563d8e98a5222de4a9b4141b088da96377e8efc3388c0538a36fe68").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("fe2070c665fa802a3263fc8a89321321184918e584b9499cb84fa38911d11f7f"))),
						block_number: 30
					}
				}.into(),
				RevealedDidKey {
					id: hex!("323486d437a096fd79ff975506e5829beace1e88d0348d6e25a6e465a8b89ab6").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("142d0e8808dd5a1287256cbbd64d06aef686606d872e824cf56b492902000a79"))),
						block_number: 30
					}
				}.into(),
				RevealedDidKey {
					id: hex!("7d8c9f4a66a0e93717dddcadba70889c4424f338f589d09714f239043591440d").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("4c0aa2e8f3cf029e08759e8b61d244a6192a2271642bb5da7b5d29990b5da00b"))),
						block_number: 30
					}
				}.into(),
				RevealedDidKey {
					id: hex!("8c86c882487160789d32d32e697b2d8e520f1563cb1e4164225753aa28a5ab2f").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("9f97d434ca3cb7b727928beb46cf49f27da871b63ae5447e3ec3b5abb08c9c0e"))),
						block_number: 30
					}
				}.into(),
				RevealedDidKey {
					id: hex!("8cc1078ba56113813991934ae0ffabbd5cb562cb6a2fc699f9bb8b4aabb8f616").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("024b98aac3d5ec1b786293c2a50e7b3ac993919c492f4da5a93f94f0f9cdb241"))),
						block_number: 30
					}
				}.into(),
				RevealedDidKey {
					id: hex!("90fd143d81ddef983d62ce1aa1639168489ff5da0f6702efd258da6c55b0777e").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("4a53daee31bc9cdce8896026c31d7621ad90854bdec56f077c2b135aa36f7a18"))),
						block_number: 30
					}
				}.into(),
				RevealedDidKey {
					id: hex!("9aeeb6b118090da14b14065df0bfa29a394e3c57574c2cf72fec38027953f2ca").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("d523029cf92cbf98206572e11d2c315f9750cf467f7be745034776c8c7552e6f"))),
						block_number: 30
					}
				}.into(),
				RevealedDidKey {
					id: hex!("c0339da5d721ecf5a599e588a74363db45688e2356ad15bf80f226017bd10248").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("4ac14eed420bc9f97db98b198133d094dd9c2ebcbe00cc6dd4576b4da6515c65"))),
						block_number: 30
					}
				}.into(),
				RevealedDidKey {
					id: hex!("cfdcf6e5989bdee27413af55c187c96387dfd2e4f3cde973b250acfac6af6cad").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("cabf0743ea77e7a4aec9ab7135700482079a9b4eff89a46add7608192680a413"))),
						block_number: 30
					}
				}.into(),
				RevealedDidKey {
					id: hex!("fda35a39cce44fbfb012b0f6a0a1dfa0b866b0ed8a2f7efb3c7bc944b8b49493").into(),
					relationship: crate::DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(hex!("fe1eb53fd59a1a27b893f0d663df84cc33fc5923d146c26eb57d8158480c7e52"))),
						block_number: 30
					}
				}.into(),
				RevealedDidKey {
					id: hex!("1b4f12a6cc3a3d8c3d2d508aadbfa6b71edbb9ac7ec3da2f0448ca8035f95c22").into(),
					relationship: DidVerificationKeyRelationship::CapabilityDelegation.into(),
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicVerificationKey(DidVerificationKey::Ed25519(ed25519::Public(hex!("61d5bd79fe0095640a7bb05e791cae4317d4575e817f41628648fc4ff5271f2d")))),
						block_number: 30
					}
				}.into(),
				RevealedDidKey {
					id: hex!("77d0160b28ad4d2f5db38e2192867914fec04eff50dd860bc46df10b36bf3b7b").into(),
					relationship: DidVerificationKeyRelationship::AssertionMethod.into(),
					details: DidPublicKeyDetails {
						key: DidPublicKey::PublicVerificationKey(DidVerificationKey::Ed25519(ed25519::Public(hex!("34c4685c61e5d6a7ff4e42d1594735285f6c116f1c6e8f70f12a3942655a7c34")))),
						block_number: 30
					}
				}.into(),

				RevealedAccountId(LinkableAccountId::AccountId32(hex!("a36777a228a2e0651764c7de0be063f9a1cc0281aedececc12cfdc69e048b7cf").into())).into(),
				RevealedAccountId(LinkableAccountId::AccountId32(hex!("0753e95eac51474dc20653d86195cd11657b3af8f9af52d03f6b42a6cbe78efa").into())).into(),
				RevealedAccountId(LinkableAccountId::AccountId32(hex!("e154fe906fd0c0227be2967528a972215ceac09ede6167421a761507cea3f1b7").into())).into(),
				RevealedAccountId(LinkableAccountId::AccountId32(hex!("e0e591ff4d23b8b5cc3655e59aa140c600565c1c21f27960cfb980a4c74b6b03").into())).into(),
				RevealedAccountId(LinkableAccountId::AccountId32(hex!("98812f5ef930d26a12e8ebcef5b5ce6f5458af9028694d49902172d969e83738").into())).into(),
				RevealedAccountId(LinkableAccountId::AccountId32(hex!("e60b14479bb4cee4f526ae31541bd3dbc79f541d01cf0a9691c1dbb1bfeb42d4").into())).into(),
				RevealedAccountId(LinkableAccountId::AccountId32(hex!("36dfc8d778b1b15835d8bc7953b1d782b38aee53ef7785340ec451a1dcc0cdc1").into())).into(),
				RevealedAccountId(LinkableAccountId::AccountId32(hex!("e808d3d3fe19d0fd336cff473acbc2cf93074c9e163a5d04a9ea33a918e89105").into())).into(),
				RevealedAccountId(LinkableAccountId::AccountId32(hex!("88dd8180874f51ca369e5a970fc2d09789af161a8ae187df94ff1658884441a4").into())).into(),
				RevealedAccountId(LinkableAccountId::AccountId32(hex!("5f97e76eefbd998e4886319066e29d7646f7de9b812cef5654536b6a39e257a2").into())).into(),

				RevealedWeb3Name { web3_name: b"b0d832f8c9b145a45537e86".to_vec().try_into().unwrap(), claimed_at: 30 }.into()
			]
		},
		signature: TimeBoundDidSignature {
			signature: sr25519::Signature(hex!("d061cf97e661c4f3e51e42e38973580f0076393a225781bd063a3476fb98dd03d55ca49b20caa18dd62cb88af4da81fc05b9299ed3b243bdbe92881ec31f2182")).into(),
			valid_until: 82
		}
	}
}

#[test]
fn test() {
	env_logger::init();
	let proof = test_parachain_proof();
	let proof_1 = proof
		.verify_provider_head_proof_with_state_root::<BlakeTwo256, Header<u64, BlakeTwo256>>(
			2_000,
			&hex!("4a8c971e646cee7c5bc37b1568acfdc5efd4c7ee5dd449946eddc43b86ad44e3").into(),
		)
		.expect("Should not fail to verify relay state.");
	let proof_2 = proof_1
		.verify_dip_commitment_proof_for_subject::<BlakeTwo256, PeregrineRuntime>(
			&AccountId32::from_ss58check("5F7Q4Tv8A2Wob14H6V7eGqhhcFEXzjZXSDptYrhxdxATe5qV").unwrap(),
		)
		.expect("Should not fail to verify DIP commitment value.");
	let proof_3 = proof_2.verify_dip_proof::<BlakeTwo256, 50>();
	println!("{:#?}", proof_3);
	let proof_3 = proof_3.expect("Should not fail to verify DIP DID proof.");
}
