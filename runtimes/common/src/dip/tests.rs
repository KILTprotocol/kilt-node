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

use did::{
	did_details::{DidCreationDetails, DidEncryptionKey},
	DidVerificationKeyRelationship, KeyIdOf,
};
use dip_support::latest::Proof;
use frame_support::{
	assert_err, assert_ok, construct_runtime, parameter_types, traits::Everything, weights::constants::RocksDbWeight,
};
use frame_system::{
	mocking::{MockBlock, MockUncheckedExtrinsic},
	EnsureSigned, RawOrigin,
};
use pallet_dip_receiver::traits::IdentityProofVerifier;
use parity_scale_codec::Encode;
use sp_core::{ecdsa, ed25519, sr25519, ConstU16, ConstU32, ConstU64, Hasher, Pair};
use sp_io::TestExternalities;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup},
	AccountId32,
};
use sp_std::collections::btree_set::BTreeSet;

use crate::dip::{
	receiver::DidMerkleProofVerifier,
	sender::{CompleteMerkleProof, DidMerkleRootGenerator},
	ProofLeaf,
};

pub(crate) type AccountId = AccountId32;
pub(crate) type Balance = u128;
pub(crate) type Block = MockBlock<TestRuntime>;
pub(crate) type BlockNumber = u64;
pub(crate) type Hashing = BlakeTwo256;
pub(crate) type Index = u64;
pub(crate) type UncheckedExtrinsic = MockUncheckedExtrinsic<TestRuntime>;

construct_runtime!(
	pub enum TestRuntime where
	Block = Block,
	NodeBlock = Block,
	UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Balances: pallet_balances,
		Did: did,
	}
);

impl frame_system::Config for TestRuntime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type BaseCallFilter = Everything;
	type BlockHashCount = ConstU64<250>;
	type BlockLength = ();
	type BlockNumber = BlockNumber;
	type BlockWeights = ();
	type DbWeight = RocksDbWeight;
	type Hash = <Hashing as Hasher>::Out;
	type Hashing = Hashing;
	type Header = Header;
	type Index = Index;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaxConsumers = ConstU32<16>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SS58Prefix = ConstU16<38>;
	type SystemWeightInfo = ();
	type Version = ();
}

parameter_types! {
	pub ExistentialDeposit: Balance = 500u64.into();
}

impl pallet_balances::Config for TestRuntime {
	type AccountStore = System;
	type Balance = Balance;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

parameter_types! {
	pub Deposit: Balance = 500u64.into();
	pub Fee: Balance = 500u64.into();
	pub MaxBlocksTxValidity: BlockNumber = 10u64;
	#[derive(Debug, Clone, Eq, PartialEq)]
	pub const MaxTotalKeyAgreementKeys: u32 = 2;
}

impl did::DeriveDidCallAuthorizationVerificationKeyRelationship for RuntimeCall {
	fn derive_verification_key_relationship(&self) -> did::DeriveDidCallKeyRelationshipResult {
		Ok(DidVerificationKeyRelationship::Authentication)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn get_call_for_did_call_benchmark() -> Self {
		RuntimeCall::System(frame_system::Call::remark { remark: vec![] })
	}
}

impl did::Config for TestRuntime {
	type Currency = Balances;
	type Deposit = Deposit;
	type DidIdentifier = AccountId;
	type EnsureOrigin = EnsureSigned<AccountId>;
	type Fee = Fee;
	type FeeCollector = ();
	type MaxBlocksTxValidity = MaxBlocksTxValidity;
	type MaxNewKeyAgreementKeys = ConstU32<2>;
	type MaxNumberOfServicesPerDid = ConstU32<1>;
	type MaxNumberOfTypesPerService = ConstU32<1>;
	type MaxNumberOfUrlsPerService = ConstU32<1>;
	type MaxPublicKeysPerDid = ConstU32<5>;
	type MaxServiceIdLength = ConstU32<100>;
	type MaxServiceTypeLength = ConstU32<100>;
	type MaxServiceUrlLength = ConstU32<100>;
	type MaxTotalKeyAgreementKeys = MaxTotalKeyAgreementKeys;
	type OriginSuccess = AccountId;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type WeightInfo = ();
}

fn base_ext() -> TestExternalities {
	TestExternalities::new(
		frame_system::GenesisConfig::default()
			.build_storage::<TestRuntime>()
			.unwrap(),
	)
}

const ALICE: AccountId = AccountId::new([1u8; 32]);

#[test]
fn minimal_did_merkle_proof() {
	base_ext().execute_with(|| {
		// Give Alice some balance
		assert_ok!(Balances::set_balance(RawOrigin::Root.into(), ALICE, 1_000_000_000, 0));
		// Generate a DID for alice
		let did_auth_key = ed25519::Pair::from_seed(&[100u8; 32]);
		let did: AccountId = did_auth_key.public().into_account().into();
		let create_details = DidCreationDetails {
			did: did.clone(),
			submitter: ALICE,
			new_attestation_key: None,
			new_delegation_key: None,
			new_key_agreement_keys: BTreeSet::new().try_into().unwrap(),
			new_service_details: vec![],
		};
		// Create Alice's DID with only authentication key
		assert_ok!(Did::create(
			RawOrigin::Signed(ALICE).into(),
			Box::new(create_details.clone()),
			did_auth_key.sign(&create_details.encode()).into()
		));
		let did_details = Did::get_did(&did).expect("DID should be present");

		// 1. Create the DID merkle proof revealing only the authentication key
		let CompleteMerkleProof { root, proof } = DidMerkleRootGenerator::<TestRuntime>::generate_proof(
			&did_details,
			[did_details.authentication_key].iter(),
		)
		.expect("Merkle proof generation should not fail.");
		println!("{:?} - {:?} - {:?} bytes", root, proof, proof.encoded_size());
		// Verify the generated merkle proof
		assert_ok!(
			DidMerkleProofVerifier::<KeyIdOf<TestRuntime>, BlockNumber, Hashing>::verify_proof_against_digest(
				proof.clone().into(),
				root
			)
		);

		// 2. Fail to generate a Merkle proof for a key that does not exist
		assert_err!(
			DidMerkleRootGenerator::<TestRuntime>::generate_proof(
				&did_details,
				[<<Hashing as Hasher>::Out>::default()].iter()
			),
			()
		);

		// 3. Fail to verify a merkle proof with a compromised merkle root
		let new_root = <<Hashing as Hasher>::Out>::default();
		assert_err!(
			DidMerkleProofVerifier::<KeyIdOf<TestRuntime>, BlockNumber, Hashing>::verify_proof_against_digest(
				proof.into(),
				new_root
			),
			()
		);
	})
}

#[test]
fn complete_did_merkle_proof() {
	base_ext().execute_with(|| {
		// Give Alice some balance
		assert_ok!(Balances::set_balance(RawOrigin::Root.into(), ALICE, 1_000_000_000, 0));
		// Generate a DID for alice
		let did_auth_key = ed25519::Pair::from_seed(&[100u8; 32]);
		let did_att_key = sr25519::Pair::from_seed(&[150u8; 32]);
		let did_del_key = ecdsa::Pair::from_seed(&[200u8; 32]);
		let enc_keys = BTreeSet::from_iter(vec![
			DidEncryptionKey::X25519([250u8; 32]),
			DidEncryptionKey::X25519([251u8; 32]),
		]);
		let did: AccountId = did_auth_key.public().into_account().into();
		let create_details = DidCreationDetails {
			did: did.clone(),
			submitter: ALICE,
			new_attestation_key: Some(did_att_key.public().into()),
			new_delegation_key: Some(did_del_key.public().into()),
			new_key_agreement_keys: enc_keys
				.try_into()
				.expect("BTreeSet to BoundedBTreeSet should not fail"),
			new_service_details: vec![],
		};
		// Create Alice's DID with only authentication key
		assert_ok!(Did::create(
			RawOrigin::Signed(ALICE).into(),
			Box::new(create_details.clone()),
			did_auth_key.sign(&create_details.encode()).into()
		));
		let did_details = Did::get_did(&did).expect("DID should be present");

		// 1. Create the DID merkle proof revealing only the authentication key
		let CompleteMerkleProof { root, proof } = DidMerkleRootGenerator::<TestRuntime>::generate_proof(
			&did_details,
			[did_details.authentication_key].iter(),
		)
		.expect("Merkle proof generation should not fail.");
		// Verify the generated merkle proof
		assert_ok!(
			DidMerkleProofVerifier::<KeyIdOf<TestRuntime>, BlockNumber, Hashing>::verify_proof_against_digest(
				proof.into(),
				root
			)
		);

		// 2. Create the DID merkle proof revealing all the keys
		let CompleteMerkleProof { root, proof } = DidMerkleRootGenerator::<TestRuntime>::generate_proof(
			&did_details,
			[
				did_details.authentication_key,
				did_details.attestation_key.unwrap(),
				did_details.delegation_key.unwrap(),
			]
			.iter()
			.chain(did_details.key_agreement_keys.iter()),
		)
		.expect("Merkle proof generation should not fail.");
		// Verify the generated merkle proof
		assert_ok!(
			DidMerkleProofVerifier::<KeyIdOf<TestRuntime>, BlockNumber, Hashing>::verify_proof_against_digest(
				proof.into(),
				root
			)
		);

		// 2. Create the DID merkle proof revealing only the key reference and not the
		// key ID
		let CompleteMerkleProof { root, proof } = DidMerkleRootGenerator::<TestRuntime>::generate_proof(
			&did_details,
			[did_details.authentication_key].iter(),
		)
		.expect("Merkle proof generation should not fail.");
		let reference_only_authentication_leaf: Vec<_> = proof
			.revealed
			.into_iter()
			.filter(|l| !matches!(l, ProofLeaf::KeyDetails(_, _)))
			.collect();
		// Fail to verify the generated merkle proof
		assert_err!(
			DidMerkleProofVerifier::<KeyIdOf<TestRuntime>, BlockNumber, Hashing>::verify_proof_against_digest(
				Proof {
					blinded: proof.blinded,
					revealed: reference_only_authentication_leaf
				}
				.into(),
				root
			),
			()
		);
	})
}
