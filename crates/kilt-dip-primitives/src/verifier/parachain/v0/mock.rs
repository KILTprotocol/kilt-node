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

use cumulus_pallet_parachain_system::{ParachainSetCode, RelayNumberStrictlyIncreases};
use cumulus_primitives_core::ParaId;
use did::{
	did_details::{DidPublicKeyDetails, DidVerificationKey},
	DidIdentifierOf, DidVerificationKeyRelationship, KeyIdOf,
};
use frame_support::{
	construct_runtime, pallet_prelude::ValueQuery, parameter_types, storage_alias, traits::Everything, Twox64Concat,
};
use frame_system::{mocking::MockBlock, pallet_prelude::BlockNumberFor, EnsureSigned};
use hex_literal::hex;
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_relay_store::RelayParentInfo;
use pallet_web3_names::Web3NameOf;
use peregrine_runtime::Runtime as PeregrineRuntime;
use rococo_runtime::Runtime as RococoRuntime;
use sp_core::{crypto::Ss58Codec, sr25519, ConstU16, ConstU32, ConstU64, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, BoundedVec,
};

use crate::{
	parachain::v0::{mock::TestRuntime as TestConsumerRuntime, ParachainVerifier},
	traits::DipCallOriginFilter,
	DipCommitmentStateProof, ParachainDipDidProof, ProviderHeadStateProof, RelayStateRootsViaRelayStorePallet,
	RevealedDidKey, RevealedWeb3Name, TimeBoundDidSignature,
};

construct_runtime!(
	pub struct TestRuntime {
		// Same index as the DIP-consumer template runtime used to generate the cross-chain proof
		System: frame_system = 0,
		ParachainSystem: cumulus_pallet_parachain_system,
		RelayStore: pallet_relay_store,
		DipConsumer: pallet_dip_consumer,
	}
);

impl frame_system::Config for TestRuntime {
	type AccountData = ();
	type AccountId = AccountId32;
	type BaseCallFilter = Everything;
	type Block = MockBlock<Self>;
	type BlockHashCount = ConstU64<10>;
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
	type OnSetCode = ParachainSetCode<TestRuntime>;
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SS58Prefix = ConstU16<1>;
	type SystemWeightInfo = ();
	type Version = ();
}

parameter_types! {
	pub const ParachainId: ParaId = ParaId::new(2_001);
}

impl cumulus_pallet_parachain_system::Config for TestRuntime {
	type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
	type DmpMessageHandler = ();
	type OnSystemEvent = ();
	type OutboundXcmpMessageSource = ();
	type ReservedDmpWeight = ();
	type ReservedXcmpWeight = ();
	type RuntimeEvent = RuntimeEvent;
	type SelfParaId = ParachainId;
	type XcmpMessageHandler = ();
	type ConsensusHook = cumulus_pallet_parachain_system::consensus_hook::RequireParentIncluded;
}

impl pallet_relay_store::Config for TestRuntime {
	type MaxRelayBlocksStored = ConstU32<10>;
	type WeightInfo = ();
}

pub struct FilterNothing;

impl DipCallOriginFilter<RuntimeCall> for FilterNothing {
	type Error = u8;
	type OriginInfo = Vec<
		RevealedDidKey<
			KeyIdOf<PeregrineRuntime>,
			BlockNumberFor<PeregrineRuntime>,
			<PeregrineRuntime as frame_system::Config>::AccountId,
		>,
	>;
	type Success = ();

	fn check_call_origin_info(_call: &RuntimeCall, _info: &Self::OriginInfo) -> Result<Self::Success, Self::Error> {
		Ok(())
	}
}

pub(crate) const MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT: u32 = 64;
pub(crate) const MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE: u32 = 1024;
pub(crate) const MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT: u32 = 64;
pub(crate) const MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE: u32 = 1024;
pub(crate) const MAX_DID_MERKLE_PROOF_LEAVE_COUNT: u32 = 64;
pub(crate) const MAX_DID_MERKLE_PROOF_LEAVE_SIZE: u32 = 1024;
pub(crate) const MAX_DID_MERKLE_LEAVES_REVEALED: u32 = 64;
pub type Verifier = ParachainVerifier<
	RococoRuntime,
	RelayStateRootsViaRelayStorePallet<TestRuntime>,
	2_000,
	PeregrineRuntime,
	FilterNothing,
	(),
	MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT,
	MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE,
	MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT,
	MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE,
	MAX_DID_MERKLE_PROOF_LEAVE_COUNT,
	MAX_DID_MERKLE_PROOF_LEAVE_SIZE,
	MAX_DID_MERKLE_LEAVES_REVEALED,
>;
impl pallet_dip_consumer::Config for TestRuntime {
	type DipCallOriginFilter = Everything;
	type DispatchOriginCheck = EnsureSigned<AccountId32>;
	type Identifier = <PeregrineRuntime as did::Config>::DidIdentifier;
	type LocalIdentityInfo = u32;
	type ProofVerifier = Verifier;
	type RuntimeCall = RuntimeCall;
	type RuntimeOrigin = RuntimeOrigin;
	type WeightInfo = ();
}

pub(crate) const RELAY_BLOCK: u32 = 421;
pub(crate) const RELAY_STATE_ROOT: H256 =
	H256(hex!("6adf8dbf20e1b78f85f6ffe4775640f935d0d8ed38acab327be81089fd90d82d"));
pub(crate) const GENESIS_HASH: H256 = H256(hex!("74f8cd2f3764f676a5e67c45a641ce1025548c6cddcf524a663a9c0aaf7fbee2"));
pub(crate) const WRONG_GENESIS_HASH: H256 = H256([0; 32]);
pub(crate) const IDENTITY_DETAILS: Option<u32> = None;
pub(crate) const WRONG_IDENTITY_DETAILS: Option<u32> = Some(u32::MAX);
pub(crate) const SIGNATURE_VALID_UNTIL: BlockNumberFor<TestRuntime> = 199;
pub(crate) const WRONG_SIGNATURE_VALID_UNTIL: BlockNumberFor<TestRuntime> = 198;

pub(crate) fn submitter() -> AccountId32 {
	AccountId32::from_ss58check("4qgGXhqTwQmi5CaAhR5s2QpsiUzwrdeksoZG5AusPMpaYqP2").unwrap()
}
pub(crate) fn wrong_submitter() -> AccountId32 {
	AccountId32::from_ss58check("4pnAJ41mGHGDKCGBGY2zzu1hfvPasPkGAKDgPeprSkxnUmGM").unwrap()
}

pub(crate) fn subject() -> DidIdentifierOf<PeregrineRuntime> {
	DidIdentifierOf::<PeregrineRuntime>::from_ss58check("4rTs9KCbLf28yUVsMo5t39ssfW4rPsaqq2UqeZi3hwYLpg3Q").unwrap()
}

pub(crate) fn call() -> RuntimeCall {
	RuntimeCall::System(frame_system::Call::remark {
		remark: b"Hello, world!".to_vec(),
	})
}
pub(crate) fn wrong_call() -> RuntimeCall {
	RuntimeCall::System(frame_system::Call::remark {
		remark: b"Wrong payload!".to_vec(),
	})
}

// Cross-chain proof generated over the details exported above.
#[allow(clippy::type_complexity)]
pub(crate) fn cross_chain_proof_with_authentication_key_and_web3_name() -> ParachainDipDidProof<
	BlockNumberFor<RococoRuntime>,
	KeyIdOf<PeregrineRuntime>,
	<PeregrineRuntime as frame_system::Config>::AccountId,
	BlockNumberFor<PeregrineRuntime>,
	Web3NameOf<PeregrineRuntime>,
	LinkableAccountId,
	BlockNumberFor<TestConsumerRuntime>,
> {
	ParachainDipDidProof { provider_head_proof: ProviderHeadStateProof { relay_block_number: RELAY_BLOCK, proof: vec![
		hex!("3703f5a4efb16ffa83d0070000da00d3541403539d4256de2db65d713afc8aedc8abede84d5dc4014019605d94").to_vec(),
		hex!("8004648031b60c9237ed343094831987f2bec10b211621255ad0b440cf161fa820d30db480f6f6801e4b41e2e6d8ec194dba122bfb9eb33feb2545ef5144cea79551f7cc52801287b410de904c199ac477f0d317d3a4b9a45b5424236719bbe2b2f0736a505a80c2160c2830b22a1eb05c14f6a9e20639de8f9e21dcd0e621ca18540027f89ba5").to_vec(),
		hex!("80ffff80d8655205caee5e0a6b74cdf5b1adc20aea610833ad71da05d3143031b5744be58015f6db81af2768203cf235fa69602e86dd51d963cbaf2e93e3d08a7a71436ac280f48da460759e201ca3c3b9127a366e235ecdbb721c7fdd02673544b39c1a0e7180095af3328f28eb7c21cf96129f628930323efd14acb42e674600f4542a2347e980e277a338d70d91f2da9e3fcdd516aadfaa1e9aa3c91080a74d1580bf033d524d80eb47b9f01723d00dbf42a4227b4b217f2bf928240d54e2f57b32b73f088158fa80c8b5d1d00527c8ba24530642a1f9049ee21ccd7d2923158e31bdb16b1e16b9ed805b682132c52908705526057f73ab7fccab4af6d72a9805634dd8d3cc53f130d180c2d44d371e5fc1f50227d7491ad65ad049630361cefb4ab1844831237609f08380a6a172370370c5b197e769e205270d4e0a36d5d8c300384ad3a04b97f7167a188036f5935fe1e0440c815666c5d68304f0723de7be305845935ab7220dd222ef868040f4d528d1dcbfb62dbc70e0c242402975b6b4009001aec75a1239f23d5650b9809d95d41f288555f74a76e2d8ec9691d240a8d9a9a57851c85e2e390d0fba659780f5528af32ddc75ed1e91e25b644e0d9fc506d1828fe5876beca37860c51b884a806bfdbbf0e0bedcb993b65c9cea1e929a56d78a3b7bc53d1b7ca6fc488e2295ee80e9810f66374c83abcac91f2c4e0b6592dab9bea79e432c469a65efc0488e93d0").to_vec(),
		hex!("8103bc05984bd8e93876468ac91f85d3a6afca02e9729db00c0214032d745bcf0d5a4502b7819c39bf85ecf59d380c6b36e0542d8f5f587756fdadc94058c345804ef2f92f35250cbd9abe38f049145ec553be0b232b6b705a7bd95fe6f0134106e0b5440c0661757261209c1e7e0800000000045250535288dba69c63177375777ef2360d7023a05e5af585aa6a1be07aac94cfb0c3979cc88d0605617572610101266c2da415cf67bc39e13f754f212eaba3839d7d5aea0c42376e00e3c376572c1ba3cb1e156ea8b3a3a6dae589a1d62a861f0247487391452b2d0f10862ea780").to_vec(),
		hex!("9e710b30bd2eab0352ddcc26417aa1945f43803b3441f15daa8a53147d69c48eac75356fab581febbb8030520b248c5942a14880ec1d5ee4349a9c6f534ce103adef97bc85a794ba786d51bd75d2fe2bc9826134802e2e0716043a02f2f29fdd52922704af194b98545ce5ca832255e8ec41bcdb6480a0718fee6fd849f63aebd00a6e9d09e984d70549c0b5475b16c244090876e628505f0e7b9012096b41c4eb3aaf947f6ea4290800004c5f03c716fb8fff3de61a883bb76adb34a20400808aefdc67024312a782a33b24ee2d1bfa728e3842db64274191fa9a4f0f7a56744c5f0f4993f016e2d2f8e5f43be7bb259486040080949e352413ff8a43f35e73a6077d7a87a2de45fb6ce9bc40ad3717bdbf7a5708").to_vec(),
		hex!("9f0b3c252fcb29d88eff4f3de5de4476c3500080680174266144346b8929a369c75acac037ac3b4edfde15b308cddfa28b7def8e805e009b7041665711ae523d4eab10f4cc0b3c7d8f0283381899b208aa42435ba3").to_vec(),
	] }, dip_commitment_proof: DipCommitmentStateProof(vec![
		hex!("7f3200658e5d6cfde41fac5eadc5b800e29cf53cf19360e5cac6055254c77d91a79701381c47e03e17c3284aa85edc851e01a12dfa1fa4ab9a0000802e1cdab36fe7e9ffaa624f5d86fa18b9809536271f60d4363b0bbf672c240f68").to_vec(),
		hex!("800c8080da28793d083b197f8d92fc3e77f5064436f1d8eea0fbea56ddb936aba654450080738fe375d48815633f8040a1f7c6311aba813d535b0f23b37e5139c85c6b4f0880b2aafe11c416356c5a97e233670962facb2a18944c3bdc4b9e27f1fa67a5bafe").to_vec(),
		hex!("80ffff80353e4d164b13c87910044f1b4e76277e404a0ab46a7cd6c33a65aaadc2375ba88007b1390da34b4dce1328430fd924a6e193517a8148dd70a912c0dc2f7f8d2d4c806322b31235b002ea35614d4eaa1282246a5cefe4a625e5265170d93f2adb9a64802407df9dc8f440a6f8ee7cde4b162e5406ffb5c2a4c99de693bcd20350cc74e0806b313ff9ef1a351bfcc5351cc3b42f8a4fdf1b4a612c350a970677ab3adc91308077be4f344b7438aec6d87a6a29089d64db3dcab9fcec7b91ee4eb37b8afc56c98014e3e0704c9a07636322335a3c663ec9fd9df8b7bf71d6e8183fefecfbfe0e508021d9b25eb4eb0be974f964ba45a39182e42c74034baa38b7499bf7eab8253533804d54f9f6624640788154e78f39dd9b535ba37be663a4ff7b9423bc28637c9f0c8035f638d2e64e75369bba87d0c8aaecf3374294d77818a299ce98dd7d7ff208ee8070de6b035f859c70b5df439f7793eaeca3d858b04dfe70d0a08b1ca06571e87d804aaaff272d09c1b5593870282b1f09e12e8ad325794662edc4a12c02bfd853a880f341940454f25e0b2c93b674eaca644f4ecdd9ace1c3955197f222fda677eebf80f395b7003a2eb1e39c624b9a707a6cb58c3cb6997932fc80662ae19c785a91f580b5e5172489541dfc581e116554b63de15fddf38ffed2b109394749c20b8f6ce3801c763d73cb3d67092a0f18f421080b782403b0a95b6c92bd8dc60e80baf2b1a5").to_vec(),
		hex!("810210108082cadebddb74d7ea90430a7205294eedaafa180228e73bd9849228dbabbf32698025b425162cc535f40a255ba9be090e12d1b50aa8a6ec0b108acb60ec048268da").to_vec(),
		hex!("9e75edf06348b4330d1e88564111cb3d3000505f0e7b9012096b41c4eb3aaf947f6ea429080000803e5de95874c4bbe730354a3a777b39be54d6141653673cda856be1d5a8893c78").to_vec(),
		hex!("9f0bf19e4ed2927982e234d989e812f3f348008028e4e828a83fd632d6d17fa940bb289ef8d04c1c154ecbf583d677460bef22128048f06290dfec2596fa70eaca62ea496d3dc0cd2f51fd40c61b58d7e5b476eebd").to_vec(),
	]), dip_proof: crate::DidMerkleProof { blinded: vec![
		hex!("8020040000").to_vec(),
		hex!("6f0c396636316435353033376335383836623033393636633900").to_vec(),
		hex!("7f04099e99fc7ce5529bc72a0846778d0f62137ddcbab51a1af2d3e91752962d91b4010000").to_vec(),
	], revealed: vec![
		RevealedDidKey {
			id: hex!("a99e99fc7ce5529bc72a0846778d0f62137ddcbab51a1af2d3e91752962d91b4").into(),
			relationship: DidVerificationKeyRelationship::Authentication.into(),
			details: DidPublicKeyDetails {
				key: DidVerificationKey::Sr25519(sr25519::Public(hex!("9cf53cf19360e5cac6055254c77d91a79701381c47e03e17c3284aa85edc851e"))).into(),
				block_number: 144
			}
		}.into(),
		RevealedWeb3Name {
			web3_name: b"9f61d55037c5886b03966c9".to_vec().try_into().unwrap(),
			claimed_at: 144
		}.into()
	] }, signature: TimeBoundDidSignature::new(did::DidSignature::Sr25519(sr25519::Signature(hex!("3cd5e72f04d248e5155bfdabb94c308a88368db63a8a0cafc15fb3204a709b07da028cf85bd450d9a2bdb6679f2b07ac69188101185ab3acd9f41419cbfb3c81"))), SIGNATURE_VALID_UNTIL) }
}

// Aliases requires because the pallet does not expose anything public.
#[storage_alias]
type LatestRelayHeads = StorageMap<RelayStore, Twox64Concat, u32, RelayParentInfo<H256>>;
#[storage_alias]
type LatestBlockHeights = StorageValue<RelayStore, BoundedVec<u32, ConstU32<10>>, ValueQuery>;

#[derive(Default)]
pub(crate) struct ExtBuilder(Option<H256>, Vec<(u32, H256)>, Option<BlockNumberFor<TestRuntime>>);

impl ExtBuilder {
	pub(crate) fn with_genesis_hash(mut self, hash: H256) -> Self {
		self.0 = Some(hash);
		self
	}
	pub(crate) fn with_relay_roots(mut self, relay_roots: Vec<(u32, H256)>) -> Self {
		self.1 = relay_roots;
		self
	}
	pub(crate) fn with_block_number(mut self, block_number: BlockNumberFor<TestRuntime>) -> Self {
		self.2 = Some(block_number);
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut ext = sp_io::TestExternalities::default();

		ext.execute_with(|| {
			if let Some(genesis_hash) = self.0 {
				frame_system::BlockHash::<TestRuntime>::insert(0, genesis_hash);
			}
			for (relay_block, relay_root) in self.1 {
				LatestRelayHeads::insert(
					relay_block,
					RelayParentInfo {
						relay_parent_storage_root: relay_root,
					},
				);
				LatestBlockHeights::mutate(|v| {
					v.try_push(relay_block).unwrap_or_else(|_| {
						panic!("Failed to push relay block ({:#?}, {:#?})", relay_block, relay_root)
					});
				});
			}
			if let Some(block_number) = self.2 {
				System::set_block_number(block_number);
			}
		});

		ext
	}
}
