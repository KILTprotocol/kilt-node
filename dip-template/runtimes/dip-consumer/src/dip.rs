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
use sp_std::{marker::PhantomData, vec::Vec};

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

		const PROOF_RELAY_BLOCK: u32 = 193;

		let provider_head_state_proof = ProviderHeadStateProof::new(PROOF_RELAY_BLOCK, vec![
			hex!("3703f5a4efb16ffa83d00700007589ffcbe7fe666f76c721443cf633e6ee45a06f439cb3637c7791cf31b0cdf1").to_vec(),
			hex!("8004648031b60c9237ed343094831987f2bec10b211621255ad0b440cf161fa820d30db480f6f6801e4b41e2e6d8ec194dba122bfb9eb33feb2545ef5144cea79551f7cc5280a1e39f80557cd7da7aa27045494d8bafc93f1d1fff00b77bfc4dc87078155a248038b7ab2b0c7e94565e832199accca003b74c41e8f4d881d8034ed8b3c1f08e22").to_vec(),
			hex!("80ffff80437bee387434e6e8f91a0739adfdc95ad239020339bc3e99e001b88992670b98804982fd732f232253fcaa75c350ef6e2ad7b587b0a9ffcd3c6be95d25f556bee980de36611e633e4f59d89fe9f3f216fa52bc054b56137e8f55a0092ada207377b9803baf41139df1886d135151e6e64604b4405033b62038878c3f7609c5fab69cdc80a1e407fc0eb00a05fde19c35367d5f6f1ed76d36a4630ae73fb964fc19ec4a6e808b4d66a7c2324664d29962ff7930152e708fdef4213acaa76601b99fb55fb3fa80413bb6d7abf53c1bd3d1adef322b493310c67b82ad106001d06a96211802b723805b682132c52908705526057f73ab7fccab4af6d72a9805634dd8d3cc53f130d180c2d44d371e5fc1f50227d7491ad65ad049630361cefb4ab1844831237609f0838027dbae280a97bff856ec780ad629b86a828f8235af6c212963a83e25b143c0ac80af6bf8534d659672b96174d96a90e7ca58acd3fae4141991b953a5c61b5ce8ab80811f63fc4d3997a103105f26e3dbf4c89911a5aa3e47d52f35b8db960d08680a8015ca3a7de67e0f69f05047887eb2da1673bf8952dbb572659713f7a68e381237800802ced193c37d688d8da534e615665a56713e1846eeca9a92d25ed5691ce9e0806bfdbbf0e0bedcb993b65c9cea1e929a56d78a3b7bc53d1b7ca6fc488e2295ee804e2a451dd1bed45187a8b7d21a1c9f8f0093eac2fb5f6a9cede45bfb7b892a53").to_vec(),
			hex!("81039cb1953d2d87454e865602d3631099122bacda46587c417641803ef5445e5812090119bd7ebf90604600d62330c172239485aa5d790b6f5b5278b5664f21e8a5e0db342c83b3ac5cea35aff1d4f9490a6d32e68b95455899ca468cb3c6d66548ed990c066175726120ce027e08000000000452505352886b769249b85767f45cf6dd369674daa53edf47b52af36c92166ea0037890ce2bfd02056175726101013288e0ad4796fba9fbcd75e92c4901561a1e8c728585fa8fb1f4fc6e090d0c2c941024b25a4b029871fb1f74cf71ffdc8a0cf3965ef52b208e6f8b5abca6c685").to_vec(),
			hex!("9e710b30bd2eab0352ddcc26417aa1945fc3803b3441f15daa8a53147d69c48eac75356fab581febbb8030520b248c5942a14880027bd77389a90d1bacdc428100d19fe4f8cf389277fc9a99a92dcc013e600fe980f9f5f25fa95e76ac835b9c7dc442eddbe86404477019dbe3c451762011638bd28048d16f61bbf0c00dafa9f54513d948c2f6276f3c89f02a3035fde4a2191e60fd505f0e7b9012096b41c4eb3aaf947f6ea4290800004c5f03c716fb8fff3de61a883bb76adb34a20400800f179c5eb0be8c4a17a77d173c53aa48253116c1bbda3440fd89aa60f7225aaa4c5f0f4993f016e2d2f8e5f43be7bb2594860400806c7b0c0ed8ee4ae71865fb04382fe8667ab03884ae1938bee35817f9e48b34bc80f17b86d932260aaa0f44c42fbf77d966025518e043c509542dfb8fadaaa743a1").to_vec(),
			hex!("9f0b3c252fcb29d88eff4f3de5de4476c3500080fb95bb5f127efa46e85746ac0fc594c04b183c773ed7ae2dc9799dc614ad582e807f15a6dde1226da17ff9bef88a51fdd844ab6039e2f2de7703d55e7919a38753").to_vec(),
		]);
		let dip_commitment_proof = DipCommitmentStateProof::new(vec![
			hex!("7f2403f9d16616f8b2725e4943c7d52ca3f47645d53dadef3a2fff9347958d6c896a9d8c93d2c77b844f01a12dfa1fa4ab9a00008006c5fe6f90767dabc1300511e799824561398773f9e87db30f62c49ad5820d7f").to_vec(),
			hex!("800c8080da28793d083b197f8d92fc3e77f5064436f1d8eea0fbea56ddb936aba654450080844045a19371db591cd5a3a2208e48377af097957887812434f9427425d94d72808d726a582260bcfa0bff4b499c80733fd9435e69d1468c58d365e1a09140af00").to_vec(),
			hex!("80ffff80353e4d164b13c87910044f1b4e76277e404a0ab46a7cd6c33a65aaadc2375ba88007b1390da34b4dce1328430fd924a6e193517a8148dd70a912c0dc2f7f8d2d4c80f6530c738de13d0b7d40dacd86b60a8208a9ed8b830213897554539a7d0810bf80a7fcb88bab872bcaa970cd9d2d39b374e879824e0386ca22683a7cceef7288df8027d23a487eecbbe30faec61592e3ac212ea5f0f7ed4a4d26a7ebb13c0ea85ad28019604cca6005d5080b81e4a4947bc7342403bb63f0f2710ed797b77da161e5b58014e3e0704c9a07636322335a3c663ec9fd9df8b7bf71d6e8183fefecfbfe0e5080dbb36019911f2e9e4814cdf9f70000801292c80c454ab64e582c360e6f44879e8081844e130a7097df746d0d33f73217d7c593d7cda07cc51611434937a769711480cabbbb0a75f296a2d28581ddce96b140870ae0323da94c391b5d10645c05d1748010ca42e1b971aa31afb2d49feab11453846fb934130207f8d7414f7db080c5c58079e06e2642b7f259494863c940bae5b86cfbe5ce0c072a12f370f422cac257ab80e34bdb523ca7c77cd0250774f9fdc5c93283fd9ddabcd3670ef85d1130ec21ca80f395b7003a2eb1e39c624b9a707a6cb58c3cb6997932fc80662ae19c785a91f580b5e5172489541dfc581e116554b63de15fddf38ffed2b109394749c20b8f6ce3806c855b252bf486677c75b093345eeb02fb689a5de290e94cf3b5c9a6ec04b140").to_vec(),
			hex!("9e75edf06348b4330d1e88564111cb3d3000505f0e7b9012096b41c4eb3aaf947f6ea42908000080d215e80254fa4203351180e8847c71d6394706f65d797084e1700776561165a7").to_vec(),
			hex!("9f0bf19e4ed2927982e234d989e812f3f320028071027c3ebff96767415d8e07c7140a0153cd58cd756ccfe4379440db4313741680971fd57cff7255f21a73629696972524c89573193928eaf19b6255f3c96c8f16").to_vec(),
		]);
		let dip_proof = DidMerkleProof::new(
			vec![
				hex!("80ffff00000000000000000000000000000000").to_vec(),
				hex!("81016fe30000000000000000000000").to_vec(),
				hex!("8000090000").to_vec(),
				hex!("7e57b214552de5fe2406c4cde2ffadc265478d8f8e45dc78868c256a8b4866ea00").to_vec(),
				hex!("7edfe406e27bf7ed8f7c4b06f24679a138799a88cf300fd15a052ae2d53c7eb100").to_vec(),
				hex!("8000220000").to_vec(),
				hex!("7ed5f08f01da4d5d838c3c0c6426436013f75862be45194ab93e70387bf5095b00").to_vec(),
				hex!("7e6843609b10ca26da79c387cc536eb690955855ef4f134114ff23285c5982ba00").to_vec(),
				hex!("810f02200000").to_vec(),
				hex!("7d01993e9f04f1b8e6f18a5601f44e5103224c4d9765947ceba0a3d1346e123500").to_vec(),
				hex!("7d0291ae1f08bfade39e5e36051c8cca6e68b750385d07cc53da6428998e602900").to_vec(),
				hex!("801022000000").to_vec(),
				hex!("7e41ee4a3d7b38791bb0559cd35ee40b2b156f5ff38a1a6ea687a64457c36f7200").to_vec(),
				hex!("7e60adecda557e10c2cd0ba412c2e349302ed097d094ffeb16f4701b2aea618700").to_vec(),
				hex!("7e34f655e0fbe25baede3a76e3bcdefb3accb2793c743741eabc5d78a745417700").to_vec(),
				hex!("8001040000").to_vec(),
				hex!("7e565d8afa2d3b25ab6b46dff8bb283f8c6fe06b316c6e0b1ab049953176727c00").to_vec(),
				hex!("7e51018a6697855845a053ee9a4a913254a184bc18afe0dc8e7aaad5a9d1583900").to_vec(),
				hex!("7f000ec15fc3b3cc3bd5562654d8feccd64f551a2a66c2723851334fd7c85af6934500").to_vec(),
				hex!("7f000bad02837aed37702de843a75f8c00e708aa56c9763f940d9836b25f16fdebeb00").to_vec(),
				hex!("7f000647b93db48080f756347726a74aa6fefdc21236cc489846288e16526f172a3b00").to_vec(),
				hex!("7f00003c45ee0b9ef5c42548a39c82373e2783e0bd1f5eb14cc26f75c3ae29758d0e00").to_vec(),
				hex!("801220000000").to_vec(),
				hex!("7eb6affef1466702f4a481ce4131b268602bc30abef3707d15be36ad35088fb300").to_vec(),
				hex!("7ee57ddc7dbc5fd1c15deaf1bff17a0b71859b1257ab9be7b065f0c256821c9400").to_vec(),
				hex!("7e29b0ee6ed03d272cd55547d3caab97737df03e355d5c64f64554852dd1fdb900").to_vec(),
				hex!("8000810000").to_vec(),
				hex!("7edd1c9a8fcdcffec717722fe5880e22791e6272a8225542cbd6015ba3a6ce1200").to_vec(),
				hex!("7ed7820134975ff44fa50fa08a30cf10dbb7a735916abe422bbdfbe9eda6e12a00").to_vec(),
				hex!("8010400000").to_vec(),
				hex!("7f01ad53785d4ae768679975217fcbd542eda1ff8db297a50ec8dc27898e05049e0000").to_vec(),
				hex!("7f0109af0bb791f497f4dfdc13d4dff877003616017d131edea0aefe3e502850b00000").to_vec(),
				hex!("8030000000").to_vec(),
				hex!("7f0122395d5823abcdfd65e218f72bb0562cd776ebeca015f69ca5c236cc5b6f340000").to_vec(),
				hex!("7f01c21c151275046296c998833f0cd4662dd80fc579789efb04aa6bdd70b0c09f0000").to_vec(),
				hex!("8024000000").to_vec(),
				hex!("7f0185dabb534a1dbac4048505d6a5a08a3dd3588c75216ba69739e81a08e2b0190000").to_vec(),
				hex!("7f01e07a74d48fd0c5fea1e0516488c5c008ea5ecb121f9e585120a7cb4910f8790000").to_vec(),
				hex!("8000b400000000").to_vec(),
				hex!("7f01373b4e8626dd7207c2d63c210e8e77783d2996c3642112bf6c2a4c545af8790000").to_vec(),
				hex!("7f015545772ced78f5aca5f79efc23ad4e053052e013a836933afd0b791c1e8b910000").to_vec(),
				hex!("7f01f52a574d0309da3f252087e418a38c8a65e8a3f1f1a98410fff133fa3be95a0000").to_vec(),
				hex!("7f0196d60c1d3e3852a27b78078ec12231e7ea4d8104b8729e79b345e01affecc80000").to_vec(),
				hex!("805010000000").to_vec(),
				hex!("7f0139efc9192269ebad118bbffa93357f6bd93f933280f2ad4e5411a735ed53070000").to_vec(),
				hex!("7f01f94df3bfae48938b90ceabff13739ba295fa629ee05b694d310f33f98aa9fa0000").to_vec(),
				hex!("8040400000").to_vec(),
				hex!("8008800000").to_vec(),
				hex!("6c3764623339376438353366633061376437316334633400").to_vec(),
				hex!("7e73b5d2054d1db7c4ba5e6249535ab732df806eb8a2d08008e8a452b449260000").to_vec(),
				hex!("7f00032aa8e1e570c1affae398a770a1337273742b171dd68a67b9ffc7aba0bf850000").to_vec(),
				hex!("8009000000").to_vec(),
				hex!("7f032057470eb4fe7c5db309eb60252683b20a50812bf10287dc9aad50af6e2d8b010000").to_vec(),
				hex!("7f01d6f6a7d1cce502c8a89b1d8f6b80e96233f5e9238362873fd2f2c64d141c440000").to_vec(),
				hex!("8014000000").to_vec(),
				hex!("7f01c13b2eae63f486d514d13f5cf0bffc7c6855135fb9b2a9830747f216b59b0b0000").to_vec(),
				hex!("7f01254d7ae15cf0c2716ba93140763e2d376b1b3393e7b3ca7432b35f811ceaaa0000").to_vec(),
				hex!("802500000000").to_vec(),
				hex!("80e000000000").to_vec(),
				hex!("8080200000").to_vec(),
				hex!("7e65a65164ae787d2cd7a0427ee7877949f7faa361134fccee205205566f360000").to_vec(),
				hex!("7e4b0216d27ebace4c78e9f6ef667fc461d17ba3ca17eb772cd6ff107913690000").to_vec(),
				hex!("7f0009c44e121b255dfb2d555718f7a57813cc7a2d0d9ac8759771f01ef7e6af150000").to_vec(),
				hex!("7f00019f07b226114d1f46beee265ae1d353ddcd089a7ca5de6543bdf83458b37d0000").to_vec(),
				hex!("7f01dda56493aac9445d47d3bcf1cd41caed3155d177bec6f13c058d5358a95df00000").to_vec(),
				hex!("7f0190a4f5d159310e40a12ec4e537d2dcfe8888a4a00a6552813aaaf50ef34e850000").to_vec(),
				hex!("802014000000").to_vec(),
				hex!("7f0146e967c30f86f51eccceef9a80b7296467433dd5498162a9a4b163b6d1560d0000").to_vec(),
				hex!("7f01fdf7bc3f9ebadd02ccfefb9ec43b4224a45aa09a78ccb045f1dde5152a87c60000").to_vec(),
				hex!("8000500000").to_vec(),
				hex!("7f00097f45183745a2380e1617eb8b28408676557162610b83dce4b58e3f2d78110000").to_vec(),
				hex!("7f000f6e2304900b1fe24a34364ee86cd8fb538deb341b38b7664dad4e1a98752a0000").to_vec(),
				hex!("80a21000000000").to_vec(),
				hex!("7f0105c3bf07b7576c64cf1b9badadb7369288be3199d5704b0c234082b7e99ca40000").to_vec(),
				hex!("8002200000").to_vec(),
				hex!("7f0008c7c53f036b88ff2bf3a34cc3787d4e821180ea4ebcb925223e065b4eeb6c0000").to_vec(),
				hex!("7f000fe34120b267f1ed19b878e2b1b638ae8e686a31b8c18c31d3f0444f0156a30000").to_vec(),
				hex!("7f01b03baef5dfddc8a288f54cfd0950ee1517d9825f04d3e0fd5fc284453393520000").to_vec(),
				hex!("7f0146d945efbc4811794e5d0f6a1c9a650d7edea807053095ab1c164cc00d51410000").to_vec(),
				hex!("8010800000").to_vec(),
				hex!("7f010932bdb2cc79743bf17dc42d23db09ee707e727282f45e4e2cd690e12c03810000").to_vec(),
				hex!("7f017db1acaaa2340dcf5e35319168afc89254ed996b09cd4dfebdeccbba82d8770000").to_vec(),
				hex!("800c130000000000").to_vec(),
				hex!("7f010b5cded04e606f5c22c9fc5a0493e431932b3bae31ec202e4d4ac95a1303e80000").to_vec(),
				hex!("7f014835eb41231b08afbbc0ee552e6a5a9c6e4acec73b9a86b790d3e7e1b89dc80000").to_vec(),
				hex!("7f01a3c84cb233a90760bbf2e38cd37d0a84e7c8c17f4a2f473edb492746f536080000").to_vec(),
				hex!("7f010e7019faab3641dca2868a4e7dcab49b57b29c681f291d796c798bf6f841540000").to_vec(),
				hex!("7f0153282ed059a09a60806750a25da29e2b36456c87493f3db0b5ce0cc28a08f10000").to_vec(),
				hex!("806c0000000000").to_vec(),
				hex!("7f03a11be47d883068a1e29c68fcbd9a73a6662c8ab530e076d64120482aa54774010300").to_vec(),
				hex!("7f013f6c1c5c18136d873c6e13c4f0f750f038bee17f5743dd84a56d02ec4720910000").to_vec(),
				hex!("7f019aa42ff358de40f6124c2ee7bd9b81abcf91542f372685c7bb5bf8882dd7740000").to_vec(),
				hex!("7f0130fd748ce3e1d64895244625a084f9b978cac67e3625c791283348a11f50220000").to_vec(),
				hex!("802110000000").to_vec(),
				hex!("7f03ba0c668b63f955f65585f2986a049582d3e4ae1ac5e55a95907b61ca91c4e5010100").to_vec(),
				hex!("7f01c35c1674360f8a969b42fa1e7df3cacdb0266dfb8f95a5b88f61ed576fe7120000").to_vec(),
				hex!("8000900000").to_vec(),
				hex!("7f00053320f8f37ffb3d441f24b416a3275c304e6fde6ecda2831e951deb94e7c50000").to_vec(),
				hex!("7f000183acf9f02c4f1493bb81621d8b2d62a2afd0f4e80b4479bc7a6ea1ee2f4c0000").to_vec(),
				hex!("80442800000000").to_vec(),
				hex!("7f014ccca248d7952f3f007f42b2e6cb436db45c2bb929668249eac79868039eed0000").to_vec(),
				hex!("7f01a63ade95a0104c3b77b7becdcb6d4c0a3de7de703a774988cd38a8ca3d6a420000").to_vec(),
				hex!("7f015cd198788a5625ee35d32eb0d51e9f18e8f7ee520bf4cb587a813b2802506d0000").to_vec(),
				hex!("8000840000").to_vec(),
				hex!("7f0007cdc5272176810379f1ef7c49cc15cf5493a2d41a1d0110bf9a96127c44720000").to_vec(),
				hex!("7f000914ff74aff344cd0891c27000c06d83be335175e86586ccf3b8d0dc513c830000").to_vec(),
			],
			vec![
				RevealedDidKey {
					id: hex!("602057470eb4fe7c5db309eb60252683b20a50812bf10287dc9aad50af6e2d8b").into(),
					relationship: DidVerificationKeyRelationship::Authentication.into(),
					details: DidPublicKeyDetails {
						key: DidVerificationKey::Sr25519(sr25519::Public(hex!(
							"5e4943c7d52ca3f47645d53dadef3a2fff9347958d6c896a9d8c93d2c77b844f"
						)))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("14ad53785d4ae768679975217fcbd542eda1ff8db297a50ec8dc27898e05049e").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"b592fcc2bfb5e53dbd40a9997dbf0e0842e4d2487764fbae065656d2a39cf602"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("1e09af0bb791f497f4dfdc13d4dff877003616017d131edea0aefe3e502850b0").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"40c406ffa3475fa5b52446b4b94232aca86e9c50daef7dbc5a4b96a64337df38"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("2422395d5823abcdfd65e218f72bb0562cd776ebeca015f69ca5c236cc5b6f34").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"3f18b1e4225bb527981502dc337a2966f73541bef4fe9975a2a285bc192bc61f"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("25c21c151275046296c998833f0cd4662dd80fc579789efb04aa6bdd70b0c09f").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"14a576035c1b71c705f1158bea900e5317dd140b64d161fbe9b9f849b3e38647"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("3285dabb534a1dbac4048505d6a5a08a3dd3588c75216ba69739e81a08e2b019").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"449e1d268626a9b3abd8a09077a0fc2414af2e33bc88e5a13f6cd4e487ef8359"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("35e07a74d48fd0c5fea1e0516488c5c008ea5ecb121f9e585120a7cb4910f879").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"979b9f342464eb32b53ad5471020e5476d798b23fa48c8ead7d8a2e63adee276"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("4a373b4e8626dd7207c2d63c210e8e77783d2996c3642112bf6c2a4c545af879").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"f5e7dd973db0ebd5669e5c9d83ec8bd55043df5abc0eb9a7c34ae96e29dcda6f"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("4c5545772ced78f5aca5f79efc23ad4e053052e013a836933afd0b791c1e8b91").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"fc96617a9f5e81e0b5c03edc321247909c39e8d7ffaac7635b8e39110faea061"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("4df52a574d0309da3f252087e418a38c8a65e8a3f1f1a98410fff133fa3be95a").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"27777600efa700166c1055649db55c6a87afeb5f2edf4800e9f81815c3108d50"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("4f96d60c1d3e3852a27b78078ec12231e7ea4d8104b8729e79b345e01affecc8").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"24acf15ac6929d13fe510e1758a39d685533e4c95761339420584c4565be1875"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("5439efc9192269ebad118bbffa93357f6bd93f933280f2ad4e5411a735ed5307").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"aa6650e9d42479acb868e673bc129321548a2eed3f9a70304dfbd03e74e0640d"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("56f94df3bfae48938b90ceabff13739ba295fa629ee05b694d310f33f98aa9fa").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"fd49a178ab2711d9b9954334eafca36f21c8eb703a45a776efa9b2d15540c92c"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("5c6f73b5d2054d1db7c4ba5e6249535ab732df806eb8a2d08008e8a452b44926").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"96ac75cb8b69d3914a729e80093b178614057a8c69ccf35a96b091703391e978"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("5ce32aa8e1e570c1affae398a770a1337273742b171dd68a67b9ffc7aba0bf85").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"38671bfbe4da1c55cca5efadbfdeede7e67c801a99f8eff995fdf734d3d39a37"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("63d6f6a7d1cce502c8a89b1d8f6b80e96233f5e9238362873fd2f2c64d141c44").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"dbe3fc55ce46b8e0e41bbfd9ec9e33fd62bfc838ad03e5d17042f83d88305e77"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("72c13b2eae63f486d514d13f5cf0bffc7c6855135fb9b2a9830747f216b59b0b").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"bcbddd68269c2172e242837cc587bd95b7d6326ba1a21507777cc5cbf81ebf52"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("74254d7ae15cf0c2716ba93140763e2d376b1b3393e7b3ca7432b35f811ceaaa").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"f9b0e62da4bc84bd266b90ff4b5d00aa5d1c910eab72f4a401e4db64d6aef614"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("805765a65164ae787d2cd7a0427ee7877949f7faa361134fccee205205566f36").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"1ce9eacca63e71090633182e317781fc4ce24e4a07e884da787eabbc52f90c5d"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("805d4b0216d27ebace4c78e9f6ef667fc461d17ba3ca17eb772cd6ff10791369").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"a117665f28ede7a0df6cdefb9c33753d667d5b65909611bd576798fe3de35360"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("8069c44e121b255dfb2d555718f7a57813cc7a2d0d9ac8759771f01ef7e6af15").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"8507c4a02f173edc4f85317e9b765ac4c15a682cde800e5d853adf8c345aba3e"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("80719f07b226114d1f46beee265ae1d353ddcd089a7ca5de6543bdf83458b37d").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"b4b3f9dd5fe448cd31a1cc2276f02934c5b4b3287bf2cb0ba466be4db0e41656"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("82dda56493aac9445d47d3bcf1cd41caed3155d177bec6f13c058d5358a95df0").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"7b88d2a3a7dd19a4c2a2ef766facbc59136c77744c9a7625b80446a1ee63770f"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("8590a4f5d159310e40a12ec4e537d2dcfe8888a4a00a6552813aaaf50ef34e85").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"2cc6176fecee453f1403277b5b4c56597f8b1d585f970243249c17b3b2b61038"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("9546e967c30f86f51eccceef9a80b7296467433dd5498162a9a4b163b6d1560d").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"f3daf4c51548d0542efa8880e5ea7ff0a6669d02cf7910c576cdfcd9ee278115"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("9afdf7bc3f9ebadd02ccfefb9ec43b4224a45aa09a78ccb045f1dde5152a87c6").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"6c146a8d09e29c042a1790ae2a26e1d59d6ba9f43d130008db58347fc3284453"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("9cc97f45183745a2380e1617eb8b28408676557162610b83dce4b58e3f2d7811").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"01da52dfd25f59f1938974e127ab6a61f3f66b64c4d47e2a1bd1e4db45a8fc5a"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("9cef6e2304900b1fe24a34364ee86cd8fb538deb341b38b7664dad4e1a98752a").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"c93aa6c996c80f74195b7b32b79f4b33668857ef477b29f9e60ee9531ab85800"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("a105c3bf07b7576c64cf1b9badadb7369288be3199d5704b0c234082b7e99ca4").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"0326177df0d4a3a0b520682e8a1f07a3c1bdc05d102744e31a7ef16c4224b36a"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("a518c7c53f036b88ff2bf3a34cc3787d4e821180ea4ebcb925223e065b4eeb6c").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"73bd63787af4bd759b9705fe0427bdffa9213c0ea7ec6ea6441e6ce2a2f1aa33"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("a5dfe34120b267f1ed19b878e2b1b638ae8e686a31b8c18c31d3f0444f0156a3").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"97592a0630ba772ff25d8e44f0923e74a407e20588faac07e2e99f3fc7569f40"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("a7b03baef5dfddc8a288f54cfd0950ee1517d9825f04d3e0fd5fc28445339352").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"99b5d7f045c444f5c7b10d9a8c4a35e9fcf570e9dac2a313ec719ad10182b838"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("ac46d945efbc4811794e5d0f6a1c9a650d7edea807053095ab1c164cc00d5141").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"0bd02dcc95c971dd82e0df2ef093b6dd35dba13b87de1f784e2ecdbd6f0b4018"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("b40932bdb2cc79743bf17dc42d23db09ee707e727282f45e4e2cd690e12c0381").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"0aeea7d0d826cf245427cae46e9936cd980676a9e7a67c8f2c6abe44ecd15031"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("bf7db1acaaa2340dcf5e35319168afc89254ed996b09cd4dfebdeccbba82d877").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"86544d9dc2e9e040fa863c2931b9cf19bf2ccfdea0763b0b43c10d9b315a7a07"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("c20b5cded04e606f5c22c9fc5a0493e431932b3bae31ec202e4d4ac95a1303e8").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"b33918a4bd6691a2fdcc5851ad098a096eed6037c60fd2b7744973fc0362475a"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("c34835eb41231b08afbbc0ee552e6a5a9c6e4acec73b9a86b790d3e7e1b89dc8").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"5cb3d63b83b81bb32beb7812207838df72e251d51546f1f1aedd8b5eeefae301"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("c8a3c84cb233a90760bbf2e38cd37d0a84e7c8c17f4a2f473edb492746f53608").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"aa4d43c1816f8f31cbdb669449917779b38249bf554fc76cec25316b33f5aa26"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("c90e7019faab3641dca2868a4e7dcab49b57b29c681f291d796c798bf6f84154").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"daf05f0523aa08837a2ca4697bef006c0a361813db80061f9d75d7607036e90b"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("cc53282ed059a09a60806750a25da29e2b36456c87493f3db0b5ce0cc28a08f1").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"1e382ba8ead527d8377e5ebf3a3bee6b3d4b6fee2f71f1381be79148c2b8457a"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("d33f6c1c5c18136d873c6e13c4f0f750f038bee17f5743dd84a56d02ec472091").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"b75d34b52629725ca611f05c92e7b8ab2c012081d83ba2b493cec265a23c5b5d"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("d59aa42ff358de40f6124c2ee7bd9b81abcf91542f372685c7bb5bf8882dd774").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"50e9d2ba03660e5386263c25b68a35222d450677e9faab7245f0981455345c21"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("d630fd748ce3e1d64895244625a084f9b978cac67e3625c791283348a11f5022").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"a89fd540aa526d06c4217cc503664c9e8d8b668634befc304f0b455234cee73f"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("e5c35c1674360f8a969b42fa1e7df3cacdb0266dfb8f95a5b88f61ed576fe712").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"546bca48d552ee30e75384a891fb0a0fad3c0508c0488d3e274152d1cd1b8f7e"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("ecc53320f8f37ffb3d441f24b416a3275c304e6fde6ecda2831e951deb94e7c5").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"994b26c2282d4fa54257dd772c7946099b484be85447a86e000ac9e8955b9013"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("ecf183acf9f02c4f1493bb81621d8b2d62a2afd0f4e80b4479bc7a6ea1ee2f4c").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"5da8b96bd0dfc4309ab0bedfe81d026c7737cd1af648004abd621e6b17578228"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("f24ccca248d7952f3f007f42b2e6cb436db45c2bb929668249eac79868039eed").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"3f3fa3a63df3a1ca6d1669f5d08e5f036cb7a8b0f6b51b82e55c921f85a9ee32"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("f6a63ade95a0104c3b77b7becdcb6d4c0a3de7de703a774988cd38a8ca3d6a42").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"433cde68cf1af4785decf8e6c8c2caccc9ec2c3386d7fd9bc94368a7f8d4453f"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("fb5cd198788a5625ee35d32eb0d51e9f18e8f7ee520bf4cb587a813b2802506d").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"bc9db68c0ef1feb18ae9d4cb129a6af40308d9eb134e1c27fa3ed37ac836b15b"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("fda7cdc5272176810379f1ef7c49cc15cf5493a2d41a1d0110bf9a96127c4472").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"05806aed54131797a26027561c3a4932cf03f1fe55fd26371bf696029ad88e38"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("fdf914ff74aff344cd0891c27000c06d83be335175e86586ccf3b8d0dc513c83").into(),
					relationship: DidKeyRelationship::Encryption,
					details: DidPublicKeyDetails {
						key: DidEncryptionKey::X25519(hex!(
							"5d68a6b1cf91e0e69a17c2e2a8151f65bbb1f0594c879dfc1a86c0a7d6274f6e"
						))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("e0ba0c668b63f955f65585f2986a049582d3e4ae1ac5e55a95907b61ca91c4e5").into(),
					relationship: DidVerificationKeyRelationship::CapabilityDelegation.into(),
					details: DidPublicKeyDetails {
						key: DidVerificationKey::Ed25519(ed25519::Public(hex!(
							"fe08d9dfaf751d9bc1ab6d38884055f913680f1bf5e9cd4ed118b7534ce89a13"
						)))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedDidKey {
					id: hex!("d2a11be47d883068a1e29c68fcbd9a73a6662c8ab530e076d64120482aa54774").into(),
					relationship: DidVerificationKeyRelationship::AssertionMethod.into(),
					details: DidPublicKeyDetails {
						key: DidVerificationKey::Ed25519(ed25519::Public(hex!(
							"b3d80300165e3b9d46528c0c0f37edcd1d80e0c31c3ce2b2f248765b755292b7"
						)))
						.into(),
						block_number: 65,
					},
				}
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("6ec15fc3b3cc3bd5562654d8feccd64f551a2a66c2723851334fd7c85af69345")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("e4e57ddc7dbc5fd1c15deaf1bff17a0b71859b1257ab9be7b065f0c256821c94")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("2f11993e9f04f1b8e6f18a5601f44e5103224c4d9765947ceba0a3d1346e1235")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("19d5f08f01da4d5d838c3c0c6426436013f75862be45194ab93e70387bf5095b")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("9647b93db48080f756347726a74aa6fefdc21236cc489846288e16526f172a3b")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("50565d8afa2d3b25ab6b46dff8bb283f8c6fe06b316c6e0b1ab049953176727c")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("0bdfe406e27bf7ed8f7c4b06f24679a138799a88cf300fd15a052ae2d53c7eb1")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("3960adecda557e10c2cd0ba412c2e349302ed097d094ffeb16f4701b2aea6187")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("1d6843609b10ca26da79c387cc536eb690955855ef4f134114ff23285c5982ba")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("f8dd1c9a8fcdcffec717722fe5880e22791e6272a8225542cbd6015ba3a6ce12")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("ed29b0ee6ed03d272cd55547d3caab97737df03e355d5c64f64554852dd1fdb9")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("ffd7820134975ff44fa50fa08a30cf10dbb7a735916abe422bbdfbe9eda6e12a")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("e1b6affef1466702f4a481ce4131b268602bc30abef3707d15be36ad35088fb3")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("0857b214552de5fe2406c4cde2ffadc265478d8f8e45dc78868c256a8b4866ea")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("2fd291ae1f08bfade39e5e36051c8cca6e68b750385d07cc53da6428998e6029")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("3441ee4a3d7b38791bb0559cd35ee40b2b156f5ff38a1a6ea687a64457c36f72")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("8bad02837aed37702de843a75f8c00e708aa56c9763f940d9836b25f16fdebeb")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("5a51018a6697855845a053ee9a4a913254a184bc18afe0dc8e7aaad5a9d15839")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("d03c45ee0b9ef5c42548a39c82373e2783e0bd1f5eb14cc26f75c3ae29758d0e")).into(),
				)
				.into(),
				RevealedAccountId(
					AccountId32::new(hex!("3d34f655e0fbe25baede3a76e3bcdefb3accb2793c743741eabc5d78a7454177")).into(),
				)
				.into(),
				RevealedWeb3Name {
					web3_name: b"c7db397d853fc0a7d71c4c4".to_vec().try_into().unwrap(),
					claimed_at: 65,
				}
				.into(),
			],
		);
		let signature = TimeBoundDidSignature::new(DidSignature::Sr25519(sr25519::Signature(hex!("3004ba5f86d048439a9abdf36eeaea90decf5391d9ad4a2b4c4ba11137c7447c92f841018defc2ffa1054e5da6656ad1ddd7ac015fdec902bf98fd4f707b4b80"))), 57 as BlockNumberFor<Runtime>);
		let proof = ParachainDipDidProof::new(provider_head_state_proof, dip_commitment_proof, dip_proof, signature);

		BlockHash::insert(
			0,
			H256(hex!("c4a31d219fa5fe2dfa9160a2e664f33965f019732f2dd5249168066c1bfb6aae")),
		);
		LatestRelayHeads::insert(
			PROOF_RELAY_BLOCK,
			RelayParentInfo {
				relay_parent_storage_root: H256(hex!(
					"baab06b3fca8881e14a954e81fac724bd3967e30f24a0eb234602180516cb164"
				)),
			},
		);

		WorstCaseOf {
			proof: proof.into(),
			call: pallet_postit::Call::post {
				text: b"Hello, world!".to_vec().try_into().unwrap(),
			}
			.into(),
			// 4q3h66CC45jSL5dpcY4B9BWUeJtPFgwVQr4BAW7HEEmy5iZp
			subject: DidIdentifier::new(hex!("5e4943c7d52ca3f47645d53dadef3a2fff9347958d6c896a9d8c93d2c77b844f")),
			// 4oq393G4AHrbTR33D4t45HXm4myUYiXJwiSXQZETAsFiJqYW
			submitter: AccountId::new(hex!("286656971deb16389ba37da9d8dd8ee331ccfb8780f05c705c2c938f1f6b030b")),
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
