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

pub(crate) const RELAY_BLOCK: u32 = 21;
pub(crate) const RELAY_STATE_ROOT: H256 =
	H256(hex!("23ed6624753dfc87f0721c867abfa77361636314a60d24e8e85b44072b89c3f6"));
pub(crate) const GENESIS_HASH: H256 = H256(hex!("fe0821e1c03846bdff40df39019205b2dce56dd0ccbff6f042d68832a56d358f"));
pub(crate) const WRONG_GENESIS_HASH: H256 = H256([0; 32]);
pub(crate) const IDENTITY_DETAILS: Option<u32> = None;
pub(crate) const WRONG_IDENTITY_DETAILS: Option<u32> = Some(u32::MAX);
pub(crate) const SIGNATURE_VALID_UNTIL: BlockNumberFor<TestRuntime> = 56;
pub(crate) const WRONG_SIGNATURE_VALID_UNTIL: BlockNumberFor<TestRuntime> = 55;

pub(crate) fn submitter() -> AccountId32 {
	AccountId32::from_ss58check("4qbGXy3VNCxRywCooPHBCiqqC8eBCi8R61FhKMhQgfe6Pi7M").unwrap()
}
pub(crate) fn wrong_submitter() -> AccountId32 {
	AccountId32::from_ss58check("4pnAJ41mGHGDKCGBGY2zzu1hfvPasPkGAKDgPeprSkxnUmGM").unwrap()
}

pub(crate) fn subject() -> DidIdentifierOf<PeregrineRuntime> {
	DidIdentifierOf::<PeregrineRuntime>::from_ss58check("4p9S4FrPp4HATybUu6FoBaveQynGWzp8oTpJ5KYyfmYZ9RH4").unwrap()
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
		hex!("3703f5a4efb16ffa83d00700005c5197306d02680fa1d14a3b19ba0fa41b17e8949911dda103b1b0476bfc980e").to_vec(),
		hex!("790309fd7e1fbcde7136109a7c9d435fac9bd912d8857a7eb6b5a02ada5eef14effd14c9d5f469ad91a7ce17998925ed087b1b0e82d2b213eacdf87eda9bd14bafc7bbbdcd2a3423d2648d844f668a1de5f409dbfbe1c529b6fdf8efa5b8b94c919dcd0c0661757261201d607d080000000004525053528484b480424aa62b5ec40d592c52a3f36bc06afa6b1e8fcf6806dd50c6147304944c05617572610101f4a4dc233d8ddd805ae2e53f987926dd55609fce234019e60bb2b0cd8b70805c5888f3f408cd7c5e39385adef76223445e2473ddeb23760b1863d592281c7182").to_vec(),
		hex!("80046480a1736fb82eeef3ae99c2d1dfc79ca72de61d32d379e5accb53bf99203c9c3b2880f6f6801e4b41e2e6d8ec194dba122bfb9eb33feb2545ef5144cea79551f7cc5280d8416fa071a12a1632a04f2cfe01cd9c7beeacc9d90f647cb93d235dd8870e73808c2f1b77b9294abc1a55fc8432f862b4abfa90f9af3a47f138e4d8dfdfee9468").to_vec(),
		hex!("80ffff80e4b470c8e610803be35fb30c2297c88daefe2fb9984db06c45b68c441d989f6680fce4c77e35ddc74b02c611a5436c98b6d2fec67ef1d9eb0c706ac06570913aa580594aafb93d9618327a4d0723e4e6ae1c34de455716c3205e665493a88303e3c4809d3100527438cdc0c7b8a19b932fc76e25d7e22b5ef9ca0a0dbcdfeefec9e9238085ab5177d435d816c3143c5a7ffc4bd8929328ec3ec9a8fb6b8ad1ff9eaf08aa80739be177872c5beb6da57440ce6941849b20f0bc344170a48312fa761fa45b3280275ba9412df014f6c2bd421a42b64052417d01defc479406b157ef5733dbf280805b682132c52908705526057f73ab7fccab4af6d72a9805634dd8d3cc53f130d180c2d44d371e5fc1f50227d7491ad65ad049630361cefb4ab1844831237609f08380134bd63183fb7e62530dd82964c89077ec083b5117f24842f8453f6f9fe3d83080afddf55b94871699b66eb154e0b6495121e88337c7b80f86058ddf54ad9e25c3804b438f963950b0230a6bdbe6664bf5a492d1c05a62343dabf14b377024995a1880490ee6b2b446a32bf0bd68d8cdc905688bdc036a5f349ee84deb700f0bcc95a9803b225accc70e19d48fd9b2e3fdec7b185a451556cf50362b056951abf2df89f4806bfdbbf0e0bedcb993b65c9cea1e929a56d78a3b7bc53d1b7ca6fc488e2295ee80d6513cd4e03e5d4dfda76ba48fefe60422081e4f885128b01400ae254fbc48a1").to_vec(),
		hex!("9e710b30bd2eab0352ddcc26417aa1945f43803b3441f15daa8a53147d69c48eac75356fab581febbb8030520b248c5942a148801f09f47c0d4767dc1ff9ae54ba8f174d9e5fa06b8242368a39481f5fe5a078f3802e2e0716043a02f2f29fdd52922704af194b98545ce5ca832255e8ec41bcdb6480935f8561d684b40c45e36088c7daa1575cc60b54080e3e023ae43db4092287ba505f0e7b9012096b41c4eb3aaf947f6ea4290800004c5f03c716fb8fff3de61a883bb76adb34a204008092e3fee779c209e5562dd0679d5fcb3876ce9ea0b126e14f1f801a50d8c1d8a44c5f0f4993f016e2d2f8e5f43be7bb259486040080cfad4870b13343cea64432d5dc64a59f0a5c6da43817f25d8a72a3900c9cee17").to_vec(),
		hex!("9f0b3c252fcb29d88eff4f3de5de4476c350008072c23a8d4d26e772d0e0e0877b3fa481025ba0f8695a5537b7771587bbe5ca60808e11df642368fb86db2a9cd579f9a3bedf50546a1a325f3c4052c037683e3656").to_vec(),
	] }, dip_commitment_proof: DipCommitmentStateProof(vec![
		hex!("7f440bf19e4ed2927982e234d989e812f3f32da9da135714ded7366de71f9a6bd6620f03ac92421fea3539e7b80a01bc14cc200265029563162101a12dfa1fa4ab9a00008032e9f6961b6f2915ebb3b3fff7ecdee4d11c1dc7c326c7890cd098498da51df1").to_vec(),
		hex!("800c8080da28793d083b197f8d92fc3e77f5064436f1d8eea0fbea56ddb936aba6544500806105b92c7c2c540155c67a2782607dace59d3093432f81564d5ada8bff4be04180b2aafe11c416356c5a97e233670962facb2a18944c3bdc4b9e27f1fa67a5bafe").to_vec(),
		hex!("80ffff80353e4d164b13c87910044f1b4e76277e404a0ab46a7cd6c33a65aaadc2375ba88007b1390da34b4dce1328430fd924a6e193517a8148dd70a912c0dc2f7f8d2d4c80ade4fe11f1179c11ffdcbfa22755ecb2b1a904b42a8e61838ac5d61b50527e5180e12d12e0e160241a582c5068f11f66364c4421b3444fc3a69da31576a46e93d180e32fd413c5f3f35cf140619d01c260348df629c9581ddb2ffa3ed3a4454611bc80e73af1cd43b13af0d4726e252583bfc4b0e4f159cacfbedeb14669fec54f16d28014e3e0704c9a07636322335a3c663ec9fd9df8b7bf71d6e8183fefecfbfe0e508089e0d83f324b3a94a57e6c9ca7517f7829acf273e063c3b86e876f5f5000dfad808237efee33d7cbf612b36cf8e72b49b7a7ee4d48085dcaf5ffa8b163261a495b80591a4868cf7eafa20b043d709923044e17e7cde25ee7a35b9732af83d346ddf8808ddf2174553f85bc1836060e6ed175ba06730cecc706a30493e8bcfd9823eeca80e36ae624a00ef6eed407fd4d97dfe9980549cc00adeb2f9454c79d73032e10e48085c95ba8d0c7c8734e14270f873eefada04c1c71d6d99d9236772f890c8a74fa80f395b7003a2eb1e39c624b9a707a6cb58c3cb6997932fc80662ae19c785a91f580b5e5172489541dfc581e116554b63de15fddf38ffed2b109394749c20b8f6ce3805256998e8d08896289d5756f1f96ec6d8f4be237654682f91f559a511bf50a75").to_vec(),
		hex!("9e75edf06348b4330d1e88564111cb3d3000505f0e7b9012096b41c4eb3aaf947f6ea429080000801109e5a50d25358a1bcff63c57103c8eb73b80885bb28ba9b666503b8669953e").to_vec(),
	]), dip_proof: crate::DidMerkleProof { blinded: vec![
		hex!("8022000000").to_vec(),
		hex!("7f04069d06a63af2662632789148708798b64f753eb007f162a641efbbe572f20e33010000").to_vec(),
		hex!("6f0c623964373239616630626365346664303738313630393800").to_vec(),
	], revealed: vec![
		RevealedDidKey {
			id: hex!("169d06a63af2662632789148708798b64f753eb007f162a641efbbe572f20e33").into(),
			relationship: DidVerificationKeyRelationship::Authentication.into(),
			details: DidPublicKeyDetails {
				key: DidVerificationKey::Sr25519(sr25519::Public(hex!("366de71f9a6bd6620f03ac92421fea3539e7b80a01bc14cc2002650295631621"))).into(),
				block_number: 4
			}
		}.into(),
		RevealedWeb3Name {
			web3_name: b"b9d729af0bce4fd07816098".to_vec().try_into().unwrap(),
			claimed_at: 4
		}.into()
	] }, signature: TimeBoundDidSignature::new(did::DidSignature::Sr25519(sr25519::Signature(hex!("faf3508b0075d8570bb1a79f7aeba4b08e9ae88f16bb9fc44eaf6f203bad842f75dfc17b114e015c7ccdaa672c359bb066961ba2cbaccf3308dc44e0fee3b28c"))), SIGNATURE_VALID_UNTIL) }
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
