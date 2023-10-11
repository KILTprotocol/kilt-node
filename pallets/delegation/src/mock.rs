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

use ctype::{mock as ctype_mock, CtypeHashOf};
use frame_support::{
	storage::bounded_btree_set::BoundedBTreeSet,
	traits::{
		fungible::{Inspect, Mutate},
		Get,
	},
};
use kilt_support::Deposit;
use sp_core::H256;

use crate::{
	self as delegation, AccountIdOf, Config, CurrencyOf, DelegationDetails, DelegationHierarchyDetails, DelegationNode,
	DelegationNodeOf, DelegatorIdOf, Permissions,
};

#[cfg(test)]
pub use self::runtime::*;

const DEFAULT_HIERARCHY_ID_SEED: u64 = 1u64;
const ALTERNATIVE_HIERARCHY_ID_SEED: u64 = 2u64;

pub const DELEGATION_ID_SEED_1: u64 = 3u64;
pub const DELEGATION_ID_SEED_2: u64 = 4u64;
pub const DELEGATION_ID_SEED_3: u64 = 5u64;
pub const DELEGATION_ID_SEED_4: u64 = 6u64;

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

pub fn delegation_id_from_seed<T>(seed: u64) -> T::DelegationNodeId
where
	T: Config,
	T::DelegationNodeId: From<H256>,
{
	H256::from_low_u64_be(seed).into()
}

pub type DelegationHierarchyInitialization<T> = Vec<(
	<T as Config>::DelegationNodeId,
	DelegationHierarchyDetails<CtypeHashOf<T>>,
	DelegatorIdOf<T>,
	AccountIdOf<T>,
)>;

pub fn initialize_pallet<T>(
	delegations: Vec<(T::DelegationNodeId, DelegationNodeOf<T>)>,
	delegation_hierarchies: DelegationHierarchyInitialization<T>,
) where
	T: Config,
	<T as Config>::Currency: Mutate<AccountIdOf<T>>,
{
	for (root_id, details, hierarchy_owner, deposit_owner) in delegation_hierarchies {
		// manually mint to enable deposit reserving

		let balance = CurrencyOf::<T>::balance(&deposit_owner);
		CurrencyOf::<T>::set_balance(&deposit_owner, balance + <T as Config>::Deposit::get());

		// reserve deposit and store
		delegation::Pallet::<T>::create_and_store_new_hierarchy(root_id, details, hierarchy_owner, deposit_owner)
			.expect("Should not exceed max children");
	}

	for del in delegations {
		let parent_node_id = del
			.1
			.parent
			.expect("Delegation node that is not a root must have a parent ID specified.");
		let parent_node = delegation::DelegationNodes::<T>::get(parent_node_id).unwrap();

		// manually mint to enable deposit reserving
		let deposit_owner = del.1.deposit.owner.clone();
		let balance = CurrencyOf::<T>::balance(&deposit_owner);
		CurrencyOf::<T>::set_balance(&deposit_owner, balance + <T as Config>::Deposit::get());

		// reserve deposit and store

		delegation::Pallet::<T>::store_delegation_under_parent(
			del.0,
			del.1.clone(),
			parent_node_id,
			parent_node.clone(),
			deposit_owner.clone(),
		)
		.expect("Should not exceed max children");
	}
}

pub fn generate_base_delegation_hierarchy_details<T>() -> DelegationHierarchyDetails<CtypeHashOf<T>>
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
	deposit_owner: <T as frame_system::Config>::AccountId,
) -> DelegationNodeOf<T> {
	DelegationNode {
		details: generate_base_delegation_details::<T>(owner),
		children: BoundedBTreeSet::new(),
		hierarchy_root_id: hierarchy_id,
		parent,
		deposit: Deposit {
			owner: deposit_owner,
			amount: <T as Config>::Deposit::get(),
		},
	}
}

pub fn generate_base_delegation_details<T: Config>(
	owner: T::DelegationEntityId,
) -> DelegationDetails<DelegatorIdOf<T>> {
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

#[cfg(test)]
pub(crate) mod runtime {
	use crate::{BalanceOf, DelegateSignatureTypeOf, DelegationAc, DelegationNodeIdOf, DelegationNodeOf};

	use super::*;

	use frame_support::{parameter_types, weights::constants::RocksDbWeight};
	use frame_system::EnsureSigned;
	use parity_scale_codec::Encode;
	use scale_info::TypeInfo;
	use sp_core::{ed25519, sr25519, Pair};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
		BuildStorage, MultiSignature, MultiSigner,
	};

	use attestation::{mock::insert_attestation, AttestationDetailsOf, ClaimHashOf};
	use ctype::CtypeEntryOf;
	use kilt_support::{
		mock::{mock_origin, SubjectId},
		signature::EqualVerify,
	};

	pub(crate) type Block = frame_system::mocking::MockBlock<Test>;

	pub(crate) type Hash = sp_core::H256;
	pub(crate) type Balance = u128;
	pub(crate) type Signature = MultiSignature;
	pub(crate) type AccountPublic = <Signature as Verify>::Signer;
	pub(crate) type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

	pub(crate) const MILLI_UNIT: Balance = 10u128.pow(12);
	pub(crate) const DELEGATION_DEPOSIT: Balance = 10 * MILLI_UNIT;
	pub(crate) const ATTESTATION_DEPOSIT: Balance = 10 * MILLI_UNIT;

	frame_support::construct_runtime!(
		pub enum Test
		{
			System: frame_system,
			Balances: pallet_balances,
			Attestation: attestation,
			Ctype: ctype,
			Delegation: delegation,
			MockOrigin: mock_origin,
		}
	);

	parameter_types! {
		pub const SS58Prefix: u8 = 38;
		pub const BlockHashCount: u64 = 250;
	}

	impl frame_system::Config for Test {
		type RuntimeOrigin = RuntimeOrigin;
		type RuntimeCall = RuntimeCall;
		type Block = Block;
		type Nonce = u64;

		type Hash = Hash;
		type Hashing = BlakeTwo256;
		type AccountId = AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;

		type RuntimeEvent = ();
		type BlockHashCount = BlockHashCount;
		type DbWeight = RocksDbWeight;
		type Version = ();

		type PalletInfo = PalletInfo;
		type AccountData = pallet_balances::AccountData<Balance>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type BaseCallFilter = frame_support::traits::Everything;
		type SystemWeightInfo = ();
		type BlockWeights = ();
		type BlockLength = ();
		type SS58Prefix = SS58Prefix;
		type OnSetCode = ();
		type MaxConsumers = frame_support::traits::ConstU32<16>;
	}

	parameter_types! {
		pub const ExistentialDeposit: Balance = 1;
		pub const MaxLocks: u32 = 50;
		pub const MaxReserves: u32 = 50;
		pub const MaxHolds: u32 = 50;
		pub const MaxFreezes: u32 = 50;
	}

	impl pallet_balances::Config for Test {
		type FreezeIdentifier = RuntimeFreezeReason;
		type RuntimeHoldReason = RuntimeHoldReason;
		type MaxFreezes = MaxFreezes;
		type MaxHolds = MaxHolds;
		type Balance = Balance;
		type DustRemoval = ();
		type RuntimeEvent = ();
		type ExistentialDeposit = ExistentialDeposit;
		type AccountStore = System;
		type WeightInfo = ();
		type MaxLocks = MaxLocks;
		type MaxReserves = MaxReserves;
		type ReserveIdentifier = [u8; 8];
	}

	impl mock_origin::Config for Test {
		type RuntimeOrigin = RuntimeOrigin;
		type AccountId = AccountId;
		type SubjectId = SubjectId;
	}

	parameter_types! {
		pub const Fee: Balance = 500;
	}

	impl ctype::Config for Test {
		type CtypeCreatorId = SubjectId;
		type EnsureOrigin = mock_origin::EnsureDoubleOrigin<AccountId, Self::CtypeCreatorId>;
		type OriginSuccess = mock_origin::DoubleOrigin<AccountId, Self::CtypeCreatorId>;
		type OverarchingOrigin = EnsureSigned<AccountId>;
		type RuntimeEvent = ();
		type WeightInfo = ();

		type Currency = Balances;
		type Fee = Fee;
		type FeeCollector = ();
	}

	parameter_types! {
		pub const MaxDelegatedAttestations: u32 = 1000;
		pub const Deposit: Balance = ATTESTATION_DEPOSIT;
	}

	impl attestation::Config for Test {
		type EnsureOrigin = mock_origin::EnsureDoubleOrigin<AccountId, DelegatorIdOf<Self>>;
		type OriginSuccess = mock_origin::DoubleOrigin<AccountId, DelegatorIdOf<Self>>;
		type RuntimeHoldReason = RuntimeHoldReason;
		type RuntimeEvent = ();
		type WeightInfo = ();

		type Currency = Balances;
		type Deposit = Deposit;
		type MaxDelegatedAttestations = MaxDelegatedAttestations;
		type AttesterId = SubjectId;
		type AuthorizationId = DelegationNodeIdOf<Self>;
		type AccessControl = DelegationAc<Self>;
		type BalanceMigrationManager = ();
	}

	parameter_types! {
		pub const MaxSignatureByteLength: u16 = 64;
		pub const MaxParentChecks: u32 = 5;
		pub const MaxRevocations: u32 = 5;
		pub const MaxRemovals: u32 = 5;
		#[derive(Clone, TypeInfo, PartialEq, Debug)]
		pub const MaxChildren: u32 = 1000;
		pub const DepositMock: Balance = DELEGATION_DEPOSIT;
	}

	impl Config for Test {
		type Signature = (SubjectId, Vec<u8>);
		type RuntimeHoldReason = RuntimeHoldReason;
		type DelegationSignatureVerification = EqualVerify<Self::DelegationEntityId, Vec<u8>>;
		type DelegationEntityId = SubjectId;
		type DelegationNodeId = Hash;
		type EnsureOrigin = mock_origin::EnsureDoubleOrigin<AccountId, Self::DelegationEntityId>;
		type OriginSuccess = mock_origin::DoubleOrigin<AccountId, Self::DelegationEntityId>;
		type RuntimeEvent = ();
		type MaxSignatureByteLength = MaxSignatureByteLength;
		type MaxParentChecks = MaxParentChecks;
		type MaxRevocations = MaxRevocations;
		type MaxRemovals = MaxRemovals;
		type MaxChildren = MaxChildren;
		type Currency = Balances;
		type Deposit = DepositMock;
		type WeightInfo = ();
		type BalanceMigrationManager = ();
	}

	pub(crate) const ACCOUNT_00: AccountId = AccountId::new([1u8; 32]);
	pub(crate) const ACCOUNT_01: AccountId = AccountId::new([2u8; 32]);
	pub(crate) const ACCOUNT_02: AccountId = AccountId::new([3u8; 32]);

	pub(crate) const ALICE_SEED: [u8; 32] = [0u8; 32];
	pub(crate) const BOB_SEED: [u8; 32] = [1u8; 32];
	pub(crate) const CHARLIE_SEED: [u8; 32] = [2u8; 32];

	pub(crate) const CLAIM_HASH_SEED_01: u64 = 1u64;

	pub(crate) fn claim_hash_from_seed(seed: u64) -> Hash {
		Hash::from_low_u64_be(seed)
	}

	pub(crate) fn ed25519_did_from_seed(seed: &[u8; 32]) -> SubjectId {
		MultiSigner::from(ed25519::Pair::from_seed(seed).public())
			.into_account()
			.into()
	}

	pub(crate) fn sr25519_did_from_seed(seed: &[u8; 32]) -> SubjectId {
		MultiSigner::from(sr25519::Pair::from_seed(seed).public())
			.into_account()
			.into()
	}

	pub(crate) fn hash_to_u8<Hash: Encode>(hash: Hash) -> Vec<u8> {
		hash.encode()
	}

	pub(crate) struct DelegationCreationOperation {
		pub delegation_id: DelegationNodeIdOf<Test>,
		pub hierarchy_id: DelegationNodeIdOf<Test>,
		pub parent_id: DelegationNodeIdOf<Test>,
		pub delegate: SubjectId,
		pub permissions: Permissions,
		pub delegate_signature: DelegateSignatureTypeOf<Test>,
	}

	pub(crate) fn generate_base_delegation_creation_operation(
		delegation_id: DelegationNodeIdOf<Test>,
		delegate_signature: DelegateSignatureTypeOf<Test>,
		delegation_node: DelegationNodeOf<Test>,
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

	pub(crate) struct DelegationHierarchyRevocationOperation {
		pub id: DelegationNodeIdOf<Test>,
		pub max_children: u32,
	}

	pub(crate) fn generate_base_delegation_hierarchy_revocation_operation(
		id: DelegationNodeIdOf<Test>,
	) -> DelegationHierarchyRevocationOperation {
		DelegationHierarchyRevocationOperation { id, max_children: 0u32 }
	}

	pub(crate) struct DelegationRevocationOperation {
		pub delegation_id: DelegationNodeIdOf<Test>,
		pub max_parent_checks: u32,
		pub max_revocations: u32,
	}

	pub(crate) struct DelegationDepositClaimOperation {
		pub delegation_id: DelegationNodeIdOf<Test>,
		pub max_removals: u32,
	}

	pub(crate) fn generate_base_delegation_revocation_operation(
		delegation_id: DelegationNodeIdOf<Test>,
	) -> DelegationRevocationOperation {
		DelegationRevocationOperation {
			delegation_id,
			max_parent_checks: 0u32,
			max_revocations: 0u32,
		}
	}

	pub(crate) fn generate_base_delegation_deposit_claim_operation(
		delegation_id: DelegationNodeIdOf<Test>,
	) -> DelegationDepositClaimOperation {
		DelegationDepositClaimOperation {
			delegation_id,
			max_removals: 0u32,
		}
	}

	#[derive(Clone, Default)]
	pub(crate) struct ExtBuilder {
		/// endowed accounts with balances
		balances: Vec<(AccountIdOf<Test>, BalanceOf<Test>)>,
		/// initial ctypes & owners
		ctypes: Vec<(CtypeHashOf<Test>, SubjectId)>,
		delegation_hierarchies: DelegationHierarchyInitialization<Test>,
		delegations: Vec<(DelegationNodeIdOf<Test>, DelegationNodeOf<Test>)>,
		attestations: Vec<(ClaimHashOf<Test>, AttestationDetailsOf<Test>)>,
	}

	impl ExtBuilder {
		#[must_use]
		pub fn with_delegation_hierarchies(
			mut self,
			delegation_hierarchies: DelegationHierarchyInitialization<Test>,
		) -> Self {
			self.delegation_hierarchies = delegation_hierarchies;
			self
		}

		#[must_use]
		pub fn with_balances(mut self, balances: Vec<(AccountIdOf<Test>, BalanceOf<Test>)>) -> Self {
			self.balances = balances;
			self
		}

		#[must_use]
		pub fn with_ctypes(mut self, ctypes: Vec<(CtypeHashOf<Test>, SubjectId)>) -> Self {
			self.ctypes = ctypes;
			self
		}

		#[must_use]
		pub fn with_delegations(
			mut self,
			delegations: Vec<(DelegationNodeIdOf<Test>, DelegationNodeOf<Test>)>,
		) -> Self {
			self.delegations = delegations;
			self
		}

		#[must_use]
		pub fn with_attestations(mut self, attestations: Vec<(ClaimHashOf<Test>, AttestationDetailsOf<Test>)>) -> Self {
			self.attestations = attestations;
			self
		}

		pub fn build(self) -> sp_io::TestExternalities {
			let mut storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
			pallet_balances::GenesisConfig::<Test> {
				balances: self.balances.clone(),
			}
			.assimilate_storage(&mut storage)
			.expect("assimilate should not fail");

			let mut ext = sp_io::TestExternalities::new(storage);

			ext.execute_with(|| {
				for (ctype_hash, owner) in self.ctypes.iter() {
					ctype::Ctypes::<Test>::insert(
						ctype_hash,
						CtypeEntryOf::<Test> {
							creator: owner.clone(),
							created_at: System::block_number(),
						},
					);
				}

				initialize_pallet::<Test>(self.delegations, self.delegation_hierarchies);

				for (claim_hash, details) in self.attestations {
					insert_attestation::<Test>(claim_hash, details)
				}
			});

			ext
		}

		pub fn build_and_execute_with_sanity_tests(self, test: impl FnOnce()) {
			self.build().execute_with(|| {
				test();
				crate::try_state::do_try_state::<Test>().expect("Sanity test for delegation failed.");
			})
		}

		#[cfg(feature = "runtime-benchmarks")]
		pub fn build_with_keystore(self) -> sp_io::TestExternalities {
			let mut ext = self.build();

			let keystore = sp_keystore::testing::MemoryKeystore::new();
			ext.register_extension(sp_keystore::KeystoreExt(sp_std::sync::Arc::new(keystore)));

			ext
		}
	}
}
