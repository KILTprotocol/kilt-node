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

use super::*;

use crate::{
	AccountId, AvailableBlockRatio, BlockHashCount, MaximumBlockLength, MaximumBlockWeight,
	MaximumExtrinsicWeight, Signature,
};
use codec::Encode;
use sp_core::{ed25519, Pair, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature, MultiSigner,
};
use support::{
	assert_err, assert_ok, impl_outer_origin,
	weights::constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight},
};

impl_outer_origin! {
	pub enum Origin for Test {}
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Test;

impl system::Trait for Test {
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
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = RocksDbWeight;
	type BlockExecutionWeight = BlockExecutionWeight;
	type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
	type MaximumExtrinsicWeight = MaximumExtrinsicWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();

	type ModuleToIndex = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type BaseCallFilter = ();
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

type Attestation = Module<Test>;
type CType = ctype::Module<Test>;
type Delegation = delegation::Module<Test>;

fn new_test_ext() -> runtime_io::TestExternalities {
	system::GenesisConfig::default()
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
		assert_ok!(Attestation::add(
			Origin::signed(account_hash.clone()),
			hash,
			hash,
			None
		));
		let existing_attestation_for_claim = {
			let opt = Attestation::attestations(hash);
			assert!(opt.is_some());
			opt.unwrap()
		};
		assert_eq!(existing_attestation_for_claim.0, hash);
		assert_eq!(existing_attestation_for_claim.1, account_hash);
		assert_eq!(existing_attestation_for_claim.3, false);
	});
}

#[test]
fn check_revoke_attestation() {
	new_test_ext().execute_with(|| {
		let pair = ed25519::Pair::from_seed(&*b"Alice                           ");
		let hash = H256::from_low_u64_be(1);
		let account_hash = MultiSigner::from(pair.public()).into_account();
		assert_ok!(CType::add(Origin::signed(account_hash.clone()), hash));
		assert_ok!(Attestation::add(
			Origin::signed(account_hash.clone()),
			hash,
			hash,
			None
		));
		assert_ok!(Attestation::revoke(
			Origin::signed(account_hash.clone()),
			hash
		));
		let existing_attestation_for_claim = {
			let opt = Attestation::attestations(hash);
			assert!(opt.is_some());
			opt.unwrap()
		};
		assert_eq!(existing_attestation_for_claim.0, hash);
		assert_eq!(existing_attestation_for_claim.1, account_hash);
		assert_eq!(existing_attestation_for_claim.3, true);
	});
}

#[test]
fn check_double_attestation() {
	new_test_ext().execute_with(|| {
		let pair = ed25519::Pair::from_seed(&*b"Alice                           ");
		let hash = H256::from_low_u64_be(1);
		let account_hash = MultiSigner::from(pair.public()).into_account();
		assert_ok!(CType::add(Origin::signed(account_hash.clone()), hash));
		assert_ok!(Attestation::add(
			Origin::signed(account_hash.clone()),
			hash,
			hash,
			None
		));
		assert_err!(
			Attestation::add(Origin::signed(account_hash), hash, hash, None),
			Attestation::ERROR_ALREADY_ATTESTED.1
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
		assert_ok!(Attestation::add(
			Origin::signed(account_hash.clone()),
			hash,
			hash,
			None
		));
		assert_ok!(Attestation::revoke(
			Origin::signed(account_hash.clone()),
			hash
		));
		assert_err!(
			Attestation::revoke(Origin::signed(account_hash), hash),
			Attestation::ERROR_ALREADY_REVOKED.1
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
			Attestation::revoke(Origin::signed(account_hash), hash),
			Attestation::ERROR_ATTESTATION_NOT_FOUND.1
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
		assert_ok!(Attestation::add(
			Origin::signed(account_hash_alice),
			hash,
			hash,
			None
		));
		assert_err!(
			Attestation::revoke(Origin::signed(account_hash_bob), hash),
			Attestation::ERROR_NOT_PERMITTED_TO_REVOKE_ATTESTATION.1
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
			Attestation::add(
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
			Attestation::add(
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
			Attestation::add(
				Origin::signed(account_hash_bob.clone()),
				claim_hash,
				other_ctype_hash,
				Some(delegation_2)
			),
			Attestation::ERROR_CTYPE_OF_DELEGATION_NOT_MATCHING.1
		);
		assert_err!(
			Attestation::add(
				Origin::signed(account_hash_alice.clone()),
				claim_hash,
				ctype_hash,
				Some(delegation_2)
			),
			Attestation::ERROR_NOT_DELEGATED_TO_ATTESTER.1
		);
		assert_err!(
			Attestation::add(
				Origin::signed(account_hash_bob.clone()),
				claim_hash,
				ctype_hash,
				Some(delegation_1)
			),
			Attestation::ERROR_DELEGATION_NOT_AUTHORIZED_TO_ATTEST.1
		);
		assert_ok!(Attestation::add(
			Origin::signed(account_hash_bob.clone()),
			claim_hash,
			ctype_hash,
			Some(delegation_2)
		));

		let existing_attestations_for_delegation =
			Attestation::delegated_attestations(delegation_2);
		assert_eq!(existing_attestations_for_delegation.len(), 1);
		assert_eq!(existing_attestations_for_delegation[0], claim_hash);

		assert_ok!(Delegation::revoke_root(
			Origin::signed(account_hash_alice.clone()),
			delegation_root
		));
		assert_err!(
			Attestation::add(
				Origin::signed(account_hash_bob),
				claim_hash,
				ctype_hash,
				Some(delegation_2)
			),
			Attestation::ERROR_DELEGATION_REVOKED.1
		);

		assert_err!(
			Attestation::revoke(Origin::signed(account_hash_charlie), claim_hash),
			Attestation::ERROR_NOT_PERMITTED_TO_REVOKE_ATTESTATION.1
		);
		assert_ok!(Attestation::revoke(
			Origin::signed(account_hash_alice),
			claim_hash
		));
	});
}
