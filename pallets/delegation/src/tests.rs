// KILT Blockchain – https://botlabs.o index: (), error: (), message: ()rg
// Copyright (C) 2019-2021 BOTLabs GmbH

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

use crate as pallet_delegation;
use crate::*;

use codec::Encode;
use frame_support::{
	assert_noop, assert_ok,
	dispatch::Weight,
	parameter_types,
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
		DispatchClass,
	},
};
use frame_system::limits::{BlockLength, BlockWeights};
use kilt_primitives::Signature;
use sp_core::{ed25519, Pair, H256, H512};
use sp_io::TestExternalities;
use sp_keystore::{testing::KeyStore, KeystoreExt};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup, Verify},
	MultiSignature, MultiSigner, Perbill,
};
use sp_std::sync::Arc;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Delegation: pallet_delegation::{Module, Call, Storage, Event<T>},
		CType: ctype::{Module, Call, Storage, Event<T>},
	}
);

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
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 38;
}

impl frame_system::Config for Test {
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type DbWeight = RocksDbWeight;
	type Version = ();

	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	type SS58Prefix = SS58Prefix;
}

impl ctype::Config for Test {
	type Event = ();
	type WeightInfo = ();
}

impl Config for Test {
	type Event = ();
	type Signature = MultiSignature;
	type Signer = <Self::Signature as Verify>::Signer;
	type DelegationNodeId = H256;
	type WeightInfo = ();
}

fn hash_to_u8<T: Encode>(hash: T) -> Vec<u8> {
	hash.encode()
}

pub struct ExtBuilder;

impl ExtBuilder {
	pub fn build_with_keystore() -> TestExternalities {
		let storage = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		let mut ext = TestExternalities::from(storage);
		// register keystore
		let keystore = KeyStore::new();
		ext.register_extension(KeystoreExt(Arc::new(keystore)));
		// events are not emitted on default block number 0
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub fn new_test_ext() -> TestExternalities {
	frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}

#[test]
fn check_add_and_revoke_delegations() {
	new_test_ext().execute_with(|| {
		let pair_alice = ed25519::Pair::from_seed(&*b"Alice                           ");
		let account_hash_alice = MultiSigner::from(pair_alice.public()).into_account();
		let pair_bob = ed25519::Pair::from_seed(&*b"Bob                             ");
		let account_hash_bob = MultiSigner::from(pair_bob.public()).into_account();
		let pair_charlie = ed25519::Pair::from_seed(&*b"Charlie                         ");
		let account_hash_charlie = MultiSigner::from(pair_charlie.public()).into_account();

		let ctype_hash = H256::from_low_u64_be(1);
		let id_level_0 = H256::from_low_u64_be(1);
		let id_level_1 = H256::from_low_u64_be(2);
		let id_level_2_1 = H256::from_low_u64_be(21);
		let id_level_2_2 = H256::from_low_u64_be(22);
		let id_level_2_2_1 = H256::from_low_u64_be(221);
		assert_ok!(CType::add(
			Origin::signed(account_hash_alice.clone()),
			ctype_hash
		));

		assert_ok!(Delegation::create_root(
			Origin::signed(account_hash_alice.clone()),
			id_level_0,
			ctype_hash
		));
		assert_noop!(
			Delegation::create_root(
				Origin::signed(account_hash_alice.clone()),
				id_level_0,
				ctype_hash
			),
			Error::<Test>::RootAlreadyExists
		);
		assert_noop!(
			Delegation::create_root(
				Origin::signed(account_hash_alice.clone()),
				id_level_1,
				H256::from_low_u64_be(2)
			),
			ctype::Error::<Test>::NotFound
		);

		assert_ok!(Delegation::add_delegation(
			Origin::signed(account_hash_alice.clone()),
			id_level_1,
			id_level_0,
			None,
			account_hash_bob.clone(),
			Permissions::DELEGATE,
			MultiSignature::from(pair_bob.sign(&hash_to_u8(Delegation::calculate_hash(
				id_level_1,
				id_level_0,
				None,
				Permissions::DELEGATE
			))))
		));
		assert_noop!(
			Delegation::add_delegation(
				Origin::signed(account_hash_alice.clone()),
				id_level_1,
				id_level_0,
				None,
				account_hash_bob.clone(),
				Permissions::DELEGATE,
				MultiSignature::from(pair_bob.sign(&hash_to_u8(Delegation::calculate_hash(
					id_level_1,
					id_level_0,
					None,
					Permissions::DELEGATE
				))))
			),
			Error::<Test>::AlreadyExists
		);
		assert_noop!(
			Delegation::add_delegation(
				Origin::signed(account_hash_bob.clone()),
				id_level_2_1,
				id_level_0,
				Some(id_level_1),
				account_hash_charlie.clone(),
				Permissions::ATTEST,
				MultiSignature::from(ed25519::Signature::from_h512(H512::from_low_u64_be(0)))
			),
			Error::<Test>::BadSignature,
		);
		assert_noop!(
			Delegation::add_delegation(
				Origin::signed(account_hash_charlie.clone()),
				id_level_2_1,
				id_level_0,
				None,
				account_hash_bob.clone(),
				Permissions::DELEGATE,
				MultiSignature::from(pair_bob.sign(&hash_to_u8(Delegation::calculate_hash(
					id_level_2_1,
					id_level_0,
					None,
					Permissions::DELEGATE
				))))
			),
			Error::<Test>::NotOwnerOfRoot,
		);
		assert_noop!(
			Delegation::add_delegation(
				Origin::signed(account_hash_alice.clone()),
				id_level_2_1,
				id_level_1,
				None,
				account_hash_bob.clone(),
				Permissions::DELEGATE,
				MultiSignature::from(pair_bob.sign(&hash_to_u8(Delegation::calculate_hash(
					id_level_2_1,
					id_level_1,
					None,
					Permissions::DELEGATE
				))))
			),
			Error::<Test>::RootNotFound
		);

		assert_ok!(Delegation::add_delegation(
			Origin::signed(account_hash_bob.clone()),
			id_level_2_1,
			id_level_0,
			Some(id_level_1),
			account_hash_charlie.clone(),
			Permissions::ATTEST,
			MultiSignature::from(pair_charlie.sign(&hash_to_u8(Delegation::calculate_hash(
				id_level_2_1,
				id_level_0,
				Some(id_level_1),
				Permissions::ATTEST
			))))
		));
		assert_noop!(
			Delegation::add_delegation(
				Origin::signed(account_hash_alice.clone()),
				id_level_2_2,
				id_level_0,
				Some(id_level_1),
				account_hash_charlie.clone(),
				Permissions::ATTEST,
				MultiSignature::from(pair_charlie.sign(&hash_to_u8(Delegation::calculate_hash(
					id_level_2_2,
					id_level_0,
					Some(id_level_1),
					Permissions::ATTEST
				))))
			),
			Error::<Test>::NotOwnerOfParent
		);
		assert_noop!(
			Delegation::add_delegation(
				Origin::signed(account_hash_charlie.clone()),
				id_level_2_2,
				id_level_0,
				Some(id_level_2_1),
				account_hash_alice.clone(),
				Permissions::ATTEST,
				MultiSignature::from(pair_alice.sign(&hash_to_u8(Delegation::calculate_hash(
					id_level_2_2,
					id_level_0,
					Some(id_level_2_1),
					Permissions::ATTEST
				))))
			),
			Error::<Test>::UnauthorizedDelegation
		);
		assert_noop!(
			Delegation::add_delegation(
				Origin::signed(account_hash_bob.clone()),
				id_level_2_2,
				id_level_0,
				Some(id_level_0),
				account_hash_charlie.clone(),
				Permissions::ATTEST,
				MultiSignature::from(pair_charlie.sign(&hash_to_u8(Delegation::calculate_hash(
					id_level_2_2,
					id_level_0,
					Some(id_level_0),
					Permissions::ATTEST
				))))
			),
			Error::<Test>::ParentNotFound
		);

		assert_ok!(Delegation::add_delegation(
			Origin::signed(account_hash_bob.clone()),
			id_level_2_2,
			id_level_0,
			Some(id_level_1),
			account_hash_charlie.clone(),
			Permissions::ATTEST | Permissions::DELEGATE,
			MultiSignature::from(pair_charlie.sign(&hash_to_u8(Delegation::calculate_hash(
				id_level_2_2,
				id_level_0,
				Some(id_level_1),
				Permissions::ATTEST | Permissions::DELEGATE
			))))
		));

		assert_ok!(Delegation::add_delegation(
			Origin::signed(account_hash_charlie.clone()),
			id_level_2_2_1,
			id_level_0,
			Some(id_level_2_2),
			account_hash_alice.clone(),
			Permissions::ATTEST,
			MultiSignature::from(pair_alice.sign(&hash_to_u8(Delegation::calculate_hash(
				id_level_2_2_1,
				id_level_0,
				Some(id_level_2_2),
				Permissions::ATTEST
			))))
		));

		let root = {
			let opt = Delegation::root(id_level_0);
			assert!(opt.is_some());
			opt.unwrap()
		};
		assert_eq!(root.ctype_hash, ctype_hash);
		assert_eq!(root.owner, account_hash_alice);
		assert_eq!(root.revoked, false);

		let delegation_1 = {
			let opt = Delegation::delegation(id_level_1);
			assert!(opt.is_some());
			opt.unwrap()
		};
		assert_eq!(delegation_1.root_id, id_level_0);
		assert_eq!(delegation_1.parent, None);
		assert_eq!(delegation_1.owner, account_hash_bob);
		assert_eq!(delegation_1.permissions, Permissions::DELEGATE);
		assert_eq!(delegation_1.revoked, false);

		let delegation_2 = {
			let opt = Delegation::delegation(id_level_2_2);
			assert!(opt.is_some());
			opt.unwrap()
		};
		assert_eq!(delegation_2.root_id, id_level_0);
		assert_eq!(delegation_2.parent, Some(id_level_1));
		assert_eq!(delegation_2.owner, account_hash_charlie);
		assert_eq!(
			delegation_2.permissions,
			Permissions::ATTEST | Permissions::DELEGATE
		);
		assert_eq!(delegation_2.revoked, false);

		let children = Delegation::children(id_level_1);
		assert_eq!(children.len(), 2);
		assert_eq!(children[0], id_level_2_1);
		assert_eq!(children[1], id_level_2_2);

		// check is_delgating
		assert_eq!(
			Delegation::is_delegating(&account_hash_alice, &id_level_1, 3),
			Ok(true)
		);
		assert_eq!(
			Delegation::is_delegating(&account_hash_alice, &id_level_2_1, 3),
			Ok(true)
		);
		assert_eq!(
			Delegation::is_delegating(&account_hash_bob, &id_level_2_1, 3),
			Ok(true)
		);
		assert_eq!(
			Delegation::is_delegating(&account_hash_charlie, &id_level_2_1, 1),
			Ok(true)
		);
		let res = Delegation::is_delegating(&account_hash_charlie, &id_level_0, 1);
		assert!(res.is_err(), "Expected error got {:?}", res);
		assert_eq!(
			Delegation::is_delegating(&account_hash_charlie, &id_level_1, 3),
			Ok(false)
		);
		assert_noop!(
			Delegation::is_delegating(&account_hash_charlie, &id_level_0, 3),
			Error::<Test>::DelegationNotFound
		);
		assert_noop!(
			Delegation::revoke_delegation(
				Origin::signed(account_hash_charlie.clone()),
				H256::from_low_u64_be(999),
				10,
				1
			),
			Error::<Test>::DelegationNotFound
		);
		assert_noop!(
			Delegation::revoke_delegation(
				Origin::signed(account_hash_charlie.clone()),
				id_level_1,
				10,
				1
			),
			Error::<Test>::UnauthorizedRevocation,
		);
		assert_ok!(Delegation::revoke_delegation(
			Origin::signed(account_hash_charlie),
			id_level_2_2,
			10,
			2
		));

		assert_eq!(Delegation::delegation(id_level_2_2).unwrap().revoked, true);
		assert_eq!(
			Delegation::delegation(id_level_2_2_1).unwrap().revoked,
			true
		);
		assert_noop!(
			Delegation::revoke_root(
				Origin::signed(account_hash_bob.clone()),
				H256::from_low_u64_be(999),
				1
			),
			Error::<Test>::RootNotFound
		);
		assert_noop!(
			Delegation::revoke_root(Origin::signed(account_hash_bob), id_level_0, 1),
			Error::<Test>::UnauthorizedRevocation,
		);
		assert_noop!(
			Delegation::revoke_root(Origin::signed(account_hash_alice.clone()), id_level_0, 0),
			crate::Error::<Test>::ExceededRevocationBounds,
		);

		assert_ok!(Delegation::revoke_root(
			Origin::signed(account_hash_alice),
			id_level_0,
			3
		));
		assert_eq!(Delegation::root(id_level_0).unwrap().revoked, true);
		assert_eq!(Delegation::delegation(id_level_1).unwrap().revoked, true);
		assert_eq!(Delegation::delegation(id_level_2_1).unwrap().revoked, true);
	});
}
