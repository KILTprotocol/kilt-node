// KILT Blockchain – https://botlabs.org
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

#![allow(clippy::from_over_into)]

use crate::CurrencyOf;
use frame_support::{
	parameter_types,
	storage::bounded_btree_set::BoundedBTreeSet,
	traits::{Currency, Get},
	weights::constants::RocksDbWeight,
};
use frame_system::EnsureSigned;
use kilt_primitives::constants::delegation::DELEGATION_DEPOSIT;
use sp_core::{ed25519, sr25519, Pair, H256};
use sp_keystore::{testing::KeyStore, KeystoreExt};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature, MultiSigner,
};
use sp_std::sync::Arc;

#[cfg(test)]
use codec::Encode;

use crate as delegation;
use crate::*;
use ctype::mock as ctype_mock;

pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub type Block = frame_system::mocking::MockBlock<Test>;

type TestCtypeOwner = kilt_primitives::AccountId;
type TestDelegationNodeId = kilt_primitives::Hash;
type TestDelegatorId = TestCtypeOwner;
type TestDelegateSignature = MultiSignature;
type TestBalance = kilt_primitives::Balance;
type TestCtypeHash = ctype_mock::TestCtypeHash;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Ctype: ctype::{Pallet, Call, Storage, Event<T>},
		Delegation: delegation::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const SS58Prefix: u8 = 38;
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = kilt_primitives::Hash;
	type Hashing = BlakeTwo256;
	type AccountId = <<kilt_primitives::Signature as Verify>::Signer as IdentifyAccount>::AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type DbWeight = RocksDbWeight;
	type Version = ();

	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<TestBalance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: TestBalance = 0;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type Balance = TestBalance;
	type DustRemoval = ();
	type Event = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub const Fee: TestBalance = 500;
}

impl ctype::Config for Test {
	type CtypeCreatorId = TestCtypeOwner;
	type EnsureOrigin = EnsureSigned<TestCtypeOwner>;
	type OriginSuccess = TestCtypeOwner;
	type Event = ();
	type WeightInfo = ();

	type Currency = Balances;
	type Fee = Fee;
	type FeeCollector = ();
}

parameter_types! {
	pub const MaxSignatureByteLength: u16 = 64;
	pub const MaxParentChecks: u32 = 5;
	pub const MaxRevocations: u32 = 5;
	pub const MaxRemovals: u32 = 5;
	#[derive(Clone)]
	pub const MaxChildren: u32 = 1000;
	pub const DepositMock: TestBalance = DELEGATION_DEPOSIT;
}

impl Config for Test {
	type Signature = MultiSignature;
	type DelegationSignatureVerification = DelegateSignatureVerifier;
	type DelegationEntityId = TestDelegatorId;
	type DelegationNodeId = TestDelegationNodeId;
	type EnsureOrigin = EnsureSigned<TestDelegatorId>;
	type OriginSuccess = TestDelegatorId;
	type Event = ();
	type MaxSignatureByteLength = MaxSignatureByteLength;
	type MaxParentChecks = MaxParentChecks;
	type MaxRevocations = MaxRevocations;
	type MaxRemovals = MaxRemovals;
	type MaxChildren = MaxChildren;
	type Currency = Balances;
	type Deposit = DepositMock;
	type WeightInfo = ();
}

pub struct DelegateSignatureVerifier;
impl VerifyDelegateSignature for DelegateSignatureVerifier {
	type DelegateId = TestDelegatorId;
	type Payload = Vec<u8>;
	type Signature = MultiSignature;

	// No need to retrieve delegate details as it is simply an AccountId.
	fn verify(
		delegate: &Self::DelegateId,
		payload: &Self::Payload,
		signature: &Self::Signature,
	) -> SignatureVerificationResult {
		ensure!(
			signature.verify(&payload[..], delegate),
			SignatureVerificationError::SignatureInvalid
		);

		Ok(())
	}
}

const ALICE_SEED: [u8; 32] = [0u8; 32];
const BOB_SEED: [u8; 32] = [1u8; 32];
const CHARLIE_SEED: [u8; 32] = [2u8; 32];

const DEFAULT_HIERARCHY_ID_SEED: u64 = 1u64;
const ALTERNATIVE_HIERARCHY_ID_SEED: u64 = 2u64;
const DEFAULT_DELEGATION_ID_SEED: u64 = 3u64;
const ALTERNATIVE_DELEGATION_ID_SEED: u64 = 4u64;
const DEFAULT_DELEGATION_ID_2_SEED: u64 = 5u64;
const ALTERNATIVE_DELEGATION_ID_2_SEED: u64 = 6u64;

pub fn get_origin(account: TestDelegatorId) -> Origin {
	Origin::signed(account)
}

pub fn get_ed25519_account(public_key: ed25519::Public) -> TestDelegatorId {
	MultiSigner::from(public_key).into_account()
}

pub fn get_sr25519_account(public_key: sr25519::Public) -> TestDelegatorId {
	MultiSigner::from(public_key).into_account()
}

pub fn get_alice_ed25519() -> ed25519::Pair {
	ed25519::Pair::from_seed(&ALICE_SEED)
}

pub fn get_alice_sr25519() -> sr25519::Pair {
	sr25519::Pair::from_seed(&ALICE_SEED)
}

pub fn get_bob_ed25519() -> ed25519::Pair {
	ed25519::Pair::from_seed(&BOB_SEED)
}

pub fn get_bob_sr25519() -> sr25519::Pair {
	sr25519::Pair::from_seed(&BOB_SEED)
}

pub fn get_charlie_ed25519() -> ed25519::Pair {
	ed25519::Pair::from_seed(&CHARLIE_SEED)
}

pub fn get_charlie_sr25519() -> sr25519::Pair {
	sr25519::Pair::from_seed(&CHARLIE_SEED)
}

pub fn get_delegation_hierarchy_id<T>(default: bool) -> T::DelegationNodeId
where
	T: Config,
	T::DelegationNodeId: From<H256>,
{
	if default {
		H256::from_low_u64_be(DEFAULT_HIERARCHY_ID_SEED).into()
	} else {
		H256::from_low_u64_be(ALTERNATIVE_HIERARCHY_ID_SEED).into()
	}
}

pub fn get_delegation_id(default: bool) -> TestDelegationNodeId {
	if default {
		TestDelegationNodeId::from_low_u64_be(DEFAULT_DELEGATION_ID_SEED)
	} else {
		TestDelegationNodeId::from_low_u64_be(ALTERNATIVE_DELEGATION_ID_SEED)
	}
}

pub fn get_delegation_id_2(default: bool) -> TestDelegationNodeId {
	if default {
		TestDelegationNodeId::from_low_u64_be(DEFAULT_DELEGATION_ID_2_SEED)
	} else {
		TestDelegationNodeId::from_low_u64_be(ALTERNATIVE_DELEGATION_ID_2_SEED)
	}
}

#[cfg(test)]
pub(crate) fn hash_to_u8<T: Encode>(hash: T) -> Vec<u8> {
	hash.encode()
}

pub fn generate_base_delegation_hierarchy_details<T>() -> DelegationHierarchyDetails<T>
where
	T: Config,
	T::Hash: From<H256>,
{
	DelegationHierarchyDetails {
		ctype_hash: ctype_mock::get_ctype_hash::<T>(true),
	}
}

pub fn generate_base_delegation_node<T: Config>(
	hierarchy_id: T::DelegationNodeId,
	owner: T::DelegationEntityId,
	parent: Option<T::DelegationNodeId>,
) -> DelegationNode<T>
where
	DelegatorIdOf<T>: Into<AccountIdOf<T>>,
{
	DelegationNode {
		details: generate_base_delegation_details(owner.clone()),
		children: BoundedBTreeSet::new(),
		hierarchy_root_id: hierarchy_id,
		parent,
		deposit: delegation::Deposit {
			owner: owner.into(),
			amount: <T as Config>::Deposit::get(),
		},
	}
}

pub fn generate_base_delegation_details<T: Config>(owner: T::DelegationEntityId) -> DelegationDetails<T> {
	DelegationDetails {
		owner,
		permissions: Permissions::DELEGATE,
		revoked: false,
	}
}

pub struct DelegationHierarchyCreationOperation<DelegationNodeId, CtypeHash> {
	pub id: DelegationNodeId,
	pub ctype_hash: CtypeHash,
}

pub fn generate_base_delegation_hierarchy_creation_operation<T>(
	id: T::DelegationNodeId,
) -> DelegationHierarchyCreationOperation<T::DelegationNodeId, CtypeHashOf<T>>
where
	T: Config,
	T::Hash: From<H256>,
{
	DelegationHierarchyCreationOperation {
		id,
		ctype_hash: ctype::mock::get_ctype_hash::<T>(true),
	}
}

pub struct DelegationCreationOperation {
	pub delegation_id: TestDelegationNodeId,
	pub hierarchy_id: TestDelegationNodeId,
	pub parent_id: TestDelegationNodeId,
	pub delegate: TestDelegatorId,
	pub permissions: Permissions,
	pub delegate_signature: TestDelegateSignature,
}

pub fn generate_base_delegation_creation_operation(
	delegation_id: TestDelegationNodeId,
	delegate_signature: TestDelegateSignature,
	delegation_node: DelegationNode<Test>,
) -> DelegationCreationOperation {
	DelegationCreationOperation {
		delegation_id,
		parent_id: delegation_node
			.parent
			.expect("Delegation node must specify a parent ID upon creation"),
		hierarchy_id: delegation_node.hierarchy_root_id,
		delegate: delegation_node.details.owner,
		delegate_signature,
		permissions: delegation_node.details.permissions,
	}
}

pub struct DelegationHierarchyRevocationOperation {
	pub id: TestDelegationNodeId,
	pub max_children: u32,
}

pub fn generate_base_delegation_hierarchy_revocation_operation(
	id: TestDelegationNodeId,
) -> DelegationHierarchyRevocationOperation {
	DelegationHierarchyRevocationOperation { id, max_children: 0u32 }
}

pub struct DelegationRevocationOperation {
	pub delegation_id: TestDelegationNodeId,
	pub max_parent_checks: u32,
	pub max_revocations: u32,
}

pub fn generate_base_delegation_revocation_operation(
	delegation_id: TestDelegationNodeId,
) -> DelegationRevocationOperation {
	DelegationRevocationOperation {
		delegation_id,
		max_parent_checks: 0u32,
		max_revocations: 0u32,
	}
}

pub fn initialize_pallet<T: Config>(
	delegations: Vec<(T::DelegationNodeId, DelegationNode<T>)>,
	delegation_hierarchies: Vec<(
		T::DelegationNodeId,
		DelegationHierarchyDetails<T>,
		DelegatorIdOf<T>,
		// AccountIdOf<T>,
	)>,
) where
	DelegatorIdOf<T>: Into<AccountIdOf<T>>,
{
	for delegation_hierarchy in delegation_hierarchies {
		// manually mint to enable deposit reserving
		let deposit_owner = delegation_hierarchy.2.clone().into();
		let balance = CurrencyOf::<T>::free_balance(&deposit_owner.clone());
		CurrencyOf::<T>::make_free_balance_be(&deposit_owner.clone(), balance + <T as Config>::Deposit::get());

		// reserve deposit and store
		delegation::Pallet::<T>::create_and_store_new_hierarchy(
			delegation_hierarchy.0,
			delegation_hierarchy.1.clone(),
			delegation_hierarchy.2.clone(),
			deposit_owner.clone(),
		)
		.expect("Each deposit owner should have sufficient balance to create a hierarchy");
	}

	for del in delegations {
		let parent_node_id = del
			.1
			.parent
			.expect("Delegation node that is not a root must have a parent ID specified.");
		let parent_node = delegation::DelegationNodes::<T>::get(parent_node_id).unwrap();

		// manually mint to enable deposit reserving
		let deposit_owner = del.1.deposit.owner.clone();
		let balance = CurrencyOf::<T>::free_balance(&deposit_owner.clone());
		CurrencyOf::<T>::make_free_balance_be(&deposit_owner.clone(), balance + <T as Config>::Deposit::get());

		// reserve deposit and store
		delegation::Pallet::<T>::store_delegation_under_parent(
			del.0,
			del.1.clone(),
			parent_node_id,
			parent_node.clone(),
			deposit_owner,
		)
		.expect("Should not exceed max children");
	}
}

#[derive(Clone, Default)]
pub struct ExtBuilder {
	/// endowed accounts with balances
	balances: Vec<(AccountIdOf<Test>, BalanceOf<Test>)>,
	/// initial ctypes & owners
	ctypes: Vec<(TestCtypeHash, AccountIdOf<Test>)>,
	delegation_hierarchies_stored: Vec<(
		TestDelegationNodeId,
		DelegationHierarchyDetails<Test>,
		DelegatorIdOf<Test>,
		// AccountIdOf<Test>,
	)>,
	delegations_stored: Vec<(TestDelegationNodeId, DelegationNode<Test>)>,
	storage_version: DelegationStorageVersion,
}

impl ExtBuilder {
	pub fn with_delegation_hierarchies(
		mut self,
		delegation_hierarchies: Vec<(
			TestDelegationNodeId,
			DelegationHierarchyDetails<Test>,
			DelegatorIdOf<Test>,
			// AccountIdOf<Test>,
		)>,
	) -> Self {
		self.delegation_hierarchies_stored = delegation_hierarchies;
		self
	}

	pub fn with_balances(mut self, balances: Vec<(AccountIdOf<Test>, BalanceOf<Test>)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn with_ctypes(mut self, ctypes: Vec<(TestCtypeHash, TestCtypeOwner)>) -> Self {
		self.ctypes = ctypes;
		self
	}

	pub fn with_delegations(mut self, delegations: Vec<(TestDelegationNodeId, DelegationNode<Test>)>) -> Self {
		self.delegations_stored = delegations;
		self
	}

	pub fn with_storage_version(mut self, storage_version: DelegationStorageVersion) -> Self {
		self.storage_version = storage_version;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		pallet_balances::GenesisConfig::<Test> {
			balances: self.balances.clone(),
		}
		.assimilate_storage(&mut storage)
		.expect("assimilate should not fail");

		let mut ext = sp_io::TestExternalities::new(storage);

		ext.execute_with(|| {
			for (ctype_hash, owner) in self.ctypes.iter() {
				ctype::Ctypes::<Test>::insert(ctype_hash, owner);
			}

			initialize_pallet(self.delegations_stored, self.delegation_hierarchies_stored);

			delegation::StorageVersion::<Test>::set(self.storage_version);
		});

		ext
	}

	pub fn build_with_keystore(self) -> sp_io::TestExternalities {
		let mut ext = self.build();

		let keystore = KeyStore::new();
		ext.register_extension(KeystoreExt(Arc::new(keystore)));

		ext
	}
}
