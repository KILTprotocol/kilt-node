// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019  BOTLabs GmbH

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

use crate::*;

use codec::Encode;
use frame_support::{
	assert_err, assert_ok,
	dispatch::Weight,
	impl_outer_origin, parameter_types,
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
		DispatchClass,
	},
};
use frame_system::limits::{BlockLength, BlockWeights};
use kilt_primitives::{AccountId, Signature};
use sp_core::{ed25519, Pair, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature, MultiSigner, Perbill,
};

impl_outer_origin! {
	pub enum Origin for Test {}
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Test;

/// We assume that ~10% of the block weight is consumed by `on_initalize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

parameter_types! {
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u8 = 38;
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
	type Lookup = IdentityLookup<AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type DbWeight = RocksDbWeight;
	type Version = ();

	type PalletInfo = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	type SS58Prefix = SS58Prefix;
}

impl ctype::Trait for Test {
	type Event = ();
}

impl error::Trait for Test {
	type ErrorCode = u16;
	type Event = ();
}

impl delegation::Trait for Test {
	type Event = ();
	type Signature = Signature;
	type Signer = <Self::Signature as Verify>::Signer;
	type DelegationNodeId = H256;
}

impl Trait for Test {
	type Event = ();
}

type AttestationModule = Module<Test>;
type CType = ctype::Module<Test>;
type Delegation = delegation::Module<Test>;

fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}

fn hash_to_u8<T: Encode>(hash: T) -> Vec<u8> {
	hash.encode()
}

#[test]
fn check_add_attestation() {
	new_test_ext().execute_with(|| {
		let pair = ed25519::Pair::from_seed(&*b"Alice                           ");
		let hash = H256::from_low_u64_be(1);
		let account_hash = MultiSigner::from(pair.public()).into_account();
		assert_ok!(CType::add(Origin::signed(account_hash.clone()), hash));
		assert_ok!(AttestationModule::add(
			Origin::signed(account_hash.clone()),
			hash,
			hash,
			None
		));
		let Attestation {
			ctype_hash,
			attester,
			revoked,
			delegation_id,
		} = {
			let opt = AttestationModule::attestations(hash);
			assert!(opt.is_some());
			opt.unwrap()
		};
		assert_eq!(ctype_hash, hash);
		assert_eq!(attester, account_hash);
		assert_eq!(delegation_id, None);
		assert_eq!(revoked, false);
	});
}

#[test]
fn check_revoke_attestation() {
	new_test_ext().execute_with(|| {
		let pair = ed25519::Pair::from_seed(&*b"Alice                           ");
		let hash = H256::from_low_u64_be(1);
		let account_hash = MultiSigner::from(pair.public()).into_account();
		assert_ok!(CType::add(Origin::signed(account_hash.clone()), hash));
		assert_ok!(AttestationModule::add(
			Origin::signed(account_hash.clone()),
			hash,
			hash,
			None
		));
		assert_ok!(AttestationModule::revoke(
			Origin::signed(account_hash.clone()),
			hash,
			10
		));
		let Attestation {
			ctype_hash,
			attester,
			revoked,
			delegation_id,
		} = {
			let opt = AttestationModule::attestations(hash);
			assert!(opt.is_some());
			opt.unwrap()
		};
		assert_eq!(ctype_hash, hash);
		assert_eq!(attester, account_hash);
		assert_eq!(delegation_id, None);
		assert_eq!(revoked, true);
	});
}

#[test]
fn check_double_attestation() {
	new_test_ext().execute_with(|| {
		let pair = ed25519::Pair::from_seed(&*b"Alice                           ");
		let hash = H256::from_low_u64_be(1);
		let account_hash = MultiSigner::from(pair.public()).into_account();
		assert_ok!(CType::add(Origin::signed(account_hash.clone()), hash));
		assert_ok!(AttestationModule::add(
			Origin::signed(account_hash.clone()),
			hash,
			hash,
			None
		));
		assert_err!(
			AttestationModule::add(Origin::signed(account_hash), hash, hash, None),
			AttestationModule::ERROR_ALREADY_ATTESTED.1
		);
	});
}

#[test]
fn check_double_revoke_attestation() {
	new_test_ext().execute_with(|| {
		let pair = ed25519::Pair::from_seed(&*b"Alice                           ");
		let hash = H256::from_low_u64_be(1);
		let account_hash = MultiSigner::from(pair.public()).into_account();
		assert_ok!(CType::add(Origin::signed(account_hash.clone()), hash));
		assert_ok!(AttestationModule::add(
			Origin::signed(account_hash.clone()),
			hash,
			hash,
			None
		));
		assert_ok!(AttestationModule::revoke(
			Origin::signed(account_hash.clone()),
			hash,
			10
		));
		assert_err!(
			AttestationModule::revoke(Origin::signed(account_hash), hash, 10),
			AttestationModule::ERROR_ALREADY_REVOKED.1,
		);
	});
}

#[test]
fn check_revoke_unknown() {
	new_test_ext().execute_with(|| {
		let pair = ed25519::Pair::from_seed(&*b"Alice                           ");
		let hash = H256::from_low_u64_be(1);
		let account_hash = MultiSigner::from(pair.public()).into_account();
		assert_err!(
			AttestationModule::revoke(Origin::signed(account_hash), hash, 10),
			AttestationModule::ERROR_ATTESTATION_NOT_FOUND.1
		);
	});
}

#[test]
fn check_revoke_not_permitted() {
	new_test_ext().execute_with(|| {
		let pair_alice = ed25519::Pair::from_seed(&*b"Alice                           ");
		let account_hash_alice = MultiSigner::from(pair_alice.public()).into_account();
		let pair_bob = ed25519::Pair::from_seed(&*b"Bob                             ");
		let account_hash_bob = MultiSigner::from(pair_bob.public()).into_account();
		let hash = H256::from_low_u64_be(1);
		assert_ok!(CType::add(Origin::signed(account_hash_alice.clone()), hash));
		assert_ok!(AttestationModule::add(
			Origin::signed(account_hash_alice),
			hash,
			hash,
			None
		));
		assert_err!(
			AttestationModule::revoke(Origin::signed(account_hash_bob), hash, 10),
			AttestationModule::ERROR_NOT_PERMITTED_TO_REVOKE_ATTESTATION.1
		);
	});
}

#[test]
fn check_add_attestation_with_delegation() {
	new_test_ext().execute_with(|| {
		let pair_alice = ed25519::Pair::from_seed(&*b"Alice                           ");
		let account_hash_alice = MultiSigner::from(pair_alice.public()).into_account();
		let pair_bob = ed25519::Pair::from_seed(&*b"Bob                             ");
		let account_hash_bob = MultiSigner::from(pair_bob.public()).into_account();
		let pair_charlie = ed25519::Pair::from_seed(&*b"Charlie                         ");
		let account_hash_charlie = MultiSigner::from(pair_charlie.public()).into_account();

		let ctype_hash = H256::from_low_u64_be(1);
		let other_ctype_hash = H256::from_low_u64_be(2);
		let claim_hash = H256::from_low_u64_be(1);

		let delegation_root = H256::from_low_u64_be(0);
		let delegation_1 = H256::from_low_u64_be(1);
		let delegation_2 = H256::from_low_u64_be(2);

		assert_ok!(CType::add(
			Origin::signed(account_hash_alice.clone()),
			ctype_hash
		));

		assert_err!(
			AttestationModule::add(
				Origin::signed(account_hash_alice.clone()),
				claim_hash,
				ctype_hash,
				Some(delegation_1)
			),
			Delegation::ERROR_DELEGATION_NOT_FOUND.1
		);

		assert_ok!(Delegation::create_root(
			Origin::signed(account_hash_alice.clone()),
			delegation_root,
			ctype_hash
		));
		assert_ok!(Delegation::add_delegation(
			Origin::signed(account_hash_alice.clone()),
			delegation_1,
			delegation_root,
			None,
			account_hash_bob.clone(),
			delegation::Permissions::DELEGATE,
			MultiSignature::from(pair_bob.sign(&hash_to_u8(Delegation::calculate_hash(
				delegation_1,
				delegation_root,
				None,
				delegation::Permissions::DELEGATE
			))))
		));
		assert_ok!(Delegation::add_delegation(
			Origin::signed(account_hash_alice.clone()),
			delegation_2,
			delegation_root,
			None,
			account_hash_bob.clone(),
			delegation::Permissions::ATTEST,
			MultiSignature::from(pair_bob.sign(&hash_to_u8(Delegation::calculate_hash(
				delegation_2,
				delegation_root,
				None,
				delegation::Permissions::ATTEST
			))))
		));

		assert_err!(
			AttestationModule::add(
				Origin::signed(account_hash_bob.clone()),
				claim_hash,
				other_ctype_hash,
				Some(delegation_2)
			),
			CType::ERROR_CTYPE_NOT_FOUND.1
		);
		assert_ok!(CType::add(
			Origin::signed(account_hash_alice.clone()),
			other_ctype_hash
		));
		assert_err!(
			AttestationModule::add(
				Origin::signed(account_hash_bob.clone()),
				claim_hash,
				other_ctype_hash,
				Some(delegation_2)
			),
			AttestationModule::ERROR_CTYPE_OF_DELEGATION_NOT_MATCHING.1
		);
		assert_err!(
			AttestationModule::add(
				Origin::signed(account_hash_alice.clone()),
				claim_hash,
				ctype_hash,
				Some(delegation_2)
			),
			AttestationModule::ERROR_NOT_DELEGATED_TO_ATTESTER.1
		);
		assert_err!(
			AttestationModule::add(
				Origin::signed(account_hash_bob.clone()),
				claim_hash,
				ctype_hash,
				Some(delegation_1)
			),
			AttestationModule::ERROR_DELEGATION_NOT_AUTHORIZED_TO_ATTEST.1
		);
		assert_ok!(AttestationModule::add(
			Origin::signed(account_hash_bob.clone()),
			claim_hash,
			ctype_hash,
			Some(delegation_2)
		));

		let existing_attestations_for_delegation =
			AttestationModule::delegated_attestations(delegation_2);
		assert_eq!(existing_attestations_for_delegation.len(), 1);
		assert_eq!(existing_attestations_for_delegation[0], claim_hash);

		assert_ok!(Delegation::revoke_root(
			Origin::signed(account_hash_alice.clone()),
			delegation_root
		));
		assert_err!(
			AttestationModule::add(
				Origin::signed(account_hash_bob),
				claim_hash,
				ctype_hash,
				Some(delegation_2)
			),
			AttestationModule::ERROR_DELEGATION_REVOKED.1
		);

		assert_err!(
			AttestationModule::revoke(Origin::signed(account_hash_charlie), claim_hash, 10),
			AttestationModule::ERROR_NOT_PERMITTED_TO_REVOKE_ATTESTATION.1
		);
		assert_ok!(AttestationModule::revoke(
			Origin::signed(account_hash_alice),
			claim_hash,
			10
		));
	});
}
