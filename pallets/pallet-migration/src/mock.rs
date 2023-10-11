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

use crate::{Config, EntriesToMigrate};

#[cfg(test)]
pub use runtime::*;

impl<T: Config> Default for EntriesToMigrate<T> {
	fn default() -> Self {
		EntriesToMigrate {
			attestation: Default::default(),
			delegation: Default::default(),
			did: Default::default(),
			lookup: Default::default(),
			w3n: Default::default(),
			public_credentials: Default::default(),
		}
	}
}

#[cfg(test)]
pub mod runtime {
	use attestation::mock::MockAccessControl;
	use ctype::{CtypeEntryOf, CtypeHashOf, Ctypes};
	use did::{
		DeriveDidCallAuthorizationVerificationKeyRelationship, DeriveDidCallKeyRelationshipResult,
		RelationshipDeriveError,
	};
	use frame_support::{
		ord_parameter_types, parameter_types,
		traits::fungible::{Inspect, MutateHold},
		weights::constants::RocksDbWeight,
	};
	use frame_system::{EnsureRoot, EnsureSigned, EnsureSignedBy};
	use kilt_support::{
		mock::{mock_origin, SubjectId},
		signature::EqualVerify,
	};
	use pallet_did_lookup::linkable_account::LinkableAccountId;
	use pallet_web3_names::web3_name::AsciiWeb3Name;
	use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
	use public_credentials::InputSubjectIdOf;
	use scale_info::TypeInfo;
	use sp_core::{ed25519, ConstU128, ConstU32};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
		AccountId32, BuildStorage, MultiSignature, MultiSigner, Perquintill,
	};

	use crate::{self as migration, Config, MigratedKeys};

	type BalanceOf<T> = <<T as ctype::Config>::Currency as Inspect<AccountId>>::Balance;
	pub type Block = frame_system::mocking::MockBlock<Test>;
	pub type Hash = sp_core::H256;
	pub type Balance = u128;
	pub type Signature = MultiSignature;
	pub type AccountPublic = <Signature as Verify>::Signer;
	pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
	type AttesterOf<T> = <T as attestation::Config>::AttesterId;
	type DidIdentifier = AccountId;

	pub const MICRO_KILT: Balance = 10u128.pow(9);
	pub const KILT: Balance = 10u128.pow(15);

	frame_support::construct_runtime!(
		pub enum Test
		{
			System: frame_system,
			Ctype: ctype,
			Balances: pallet_balances,
			MockOrigin: mock_origin,
			Attestation: attestation,
			Delegation: delegation,
			Did: did,
			DidLookup: pallet_did_lookup,
			W3n: pallet_web3_names,
			PublicCredentials: public_credentials,
			Migration: migration,
		}
	);

	parameter_types! {
		pub const MaxMigrationsPerPallet: u8 = 42;
	}

	impl Config for Test {
		type MaxMigrationsPerPallet = MaxMigrationsPerPallet;
		type RuntimeEvent = ();
		type WeightInfo = ();
		type Currency = Balances;
	}

	parameter_types! {
		pub const SS58Prefix: u8 = 38;
		pub const BlockHashCount: u64 = 250;
	}

	impl frame_system::Config for Test {
		type RuntimeOrigin = RuntimeOrigin;
		type RuntimeCall = RuntimeCall;
		type Nonce = u64;
		type Block = Block;
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
		pub const MaxDelegatedAttestations: u32 = 1000;
		pub const Deposit: Balance = MICRO_KILT;
	}

	impl attestation::Config for Test {
		type EnsureOrigin = mock_origin::EnsureDoubleOrigin<AccountId, AttesterOf<Self>>;
		type OriginSuccess = mock_origin::DoubleOrigin<AccountId, AttesterOf<Self>>;
		type RuntimeEvent = ();
		type WeightInfo = ();
		type RuntimeHoldReason = RuntimeHoldReason;
		type Currency = Balances;
		type Deposit = Deposit;
		type MaxDelegatedAttestations = MaxDelegatedAttestations;
		type AttesterId = SubjectId;
		type AuthorizationId = SubjectId;
		type AccessControl = MockAccessControl<Self>;
		type BalanceMigrationManager = Migration;
	}

	parameter_types! {
		pub const ExistentialDeposit: Balance = 500;
		pub const MaxLocks: u32 = 50;
		pub const MaxReserves: u32 = 50;
		pub const MaxFreezes: u32 = 50;
		pub const MaxHolds: u32 = 50;
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

	ord_parameter_types! {
		pub const OverarchingOrigin: AccountId = ACCOUNT_00;
		pub const Fee: Balance = 0;
	}

	impl ctype::Config for Test {
		type CtypeCreatorId = SubjectId;
		type EnsureOrigin = mock_origin::EnsureDoubleOrigin<AccountId, SubjectId>;
		type OverarchingOrigin = EnsureSignedBy<OverarchingOrigin, AccountId>;
		type OriginSuccess = mock_origin::DoubleOrigin<AccountId, SubjectId>;
		type RuntimeEvent = ();
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
		#[derive(Clone, TypeInfo)]
		pub const MaxChildren: u32 = 1000;
		pub const DepositMock: Balance = MICRO_KILT;
	}

	impl delegation::Config for Test {
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
		type BalanceMigrationManager = Migration;
	}

	parameter_types! {
		#[derive(Clone, TypeInfo, Debug, PartialEq, Eq, Encode, Decode)]
		pub const MaxNewKeyAgreementKeys: u32 = 10u32;
		#[derive(Debug, Clone, Eq, PartialEq)]
		pub const MaxTotalKeyAgreementKeys: u32 = 10u32;
		// IMPORTANT: Needs to be at least MaxTotalKeyAgreementKeys + 3 (auth, delegation, attestation keys) for benchmarks!
		#[derive(Debug, Clone)]
		pub const MaxPublicKeysPerDid: u32 = 13u32;
		pub const MaxBlocksTxValidity: u64 = 300u64;
		pub const DidFee: Balance = 0;
		pub const MaxNumberOfServicesPerDid: u32 = 25u32;
		pub const MaxServiceIdLength: u32 = 50u32;
		pub const MaxServiceTypeLength: u32 = 50u32;
		pub const MaxServiceUrlLength: u32 = 100u32;
		pub const MaxNumberOfTypesPerService: u32 = 1u32;
		pub const MaxNumberOfUrlsPerService: u32 = 1u32;
		pub const KeyDeposit :Balance = 32 * MICRO_KILT;
		pub const ServiceEndpointDeposit :Balance = 50 * MICRO_KILT;
		pub const BaseDeposit: Balance = 100 * MICRO_KILT;
	}

	impl DeriveDidCallAuthorizationVerificationKeyRelationship for RuntimeCall {
		fn derive_verification_key_relationship(&self) -> DeriveDidCallKeyRelationshipResult {
			Err(RelationshipDeriveError::NotCallableByDid)
		}

		// Always return a System::remark() extrinsic call
		#[cfg(feature = "runtime-benchmarks")]
		fn get_call_for_did_call_benchmark() -> Self {
			RuntimeCall::System(frame_system::Call::remark { remark: sp_std::vec![] })
		}
	}

	impl did::Config for Test {
		type DidIdentifier = DidIdentifier;
		type RuntimeOrigin = RuntimeOrigin;
		type RuntimeCall = RuntimeCall;
		type EnsureOrigin = EnsureSigned<DidIdentifier>;
		type KeyDeposit = KeyDeposit;
		type RuntimeHoldReason = RuntimeHoldReason;
		type ServiceEndpointDeposit = KeyDeposit;
		type OriginSuccess = AccountId;
		type RuntimeEvent = ();
		type Currency = Balances;
		type BaseDeposit = BaseDeposit;
		type Fee = DidFee;
		type FeeCollector = ();
		type MaxNewKeyAgreementKeys = MaxNewKeyAgreementKeys;
		type MaxTotalKeyAgreementKeys = MaxTotalKeyAgreementKeys;
		type MaxPublicKeysPerDid = MaxPublicKeysPerDid;
		type MaxBlocksTxValidity = MaxBlocksTxValidity;
		type WeightInfo = ();
		type MaxNumberOfServicesPerDid = MaxNumberOfServicesPerDid;
		type MaxServiceIdLength = MaxServiceIdLength;
		type MaxServiceTypeLength = MaxServiceTypeLength;
		type MaxServiceUrlLength = MaxServiceUrlLength;
		type MaxNumberOfTypesPerService = MaxNumberOfTypesPerService;
		type MaxNumberOfUrlsPerService = MaxNumberOfUrlsPerService;
		type BalanceMigrationManager = Migration;
	}

	parameter_types! {
		pub const DidLookupDeposit: Balance = MICRO_KILT;
	}

	impl pallet_did_lookup::Config for Test {
		type RuntimeEvent = ();
		type RuntimeHoldReason = RuntimeHoldReason;
		type Currency = Balances;
		type Deposit = DidLookupDeposit;
		type EnsureOrigin = mock_origin::EnsureDoubleOrigin<AccountId, SubjectId>;
		type OriginSuccess = mock_origin::DoubleOrigin<AccountId, SubjectId>;
		type DidIdentifier = SubjectId;
		type WeightInfo = ();
		type BalanceMigrationManager = Migration;
	}

	pub(crate) type TestWeb3Name = AsciiWeb3Name<Test>;
	pub(crate) type TestWeb3NameOwner = SubjectId;
	pub(crate) type TestWeb3NamePayer = AccountId;
	pub(crate) type TestOwnerOrigin = mock_origin::EnsureDoubleOrigin<TestWeb3NamePayer, TestWeb3NameOwner>;
	pub(crate) type TestOriginSuccess = mock_origin::DoubleOrigin<TestWeb3NamePayer, TestWeb3NameOwner>;
	pub(crate) type TestBanOrigin = EnsureRoot<AccountId>;

	parameter_types! {
		pub const MaxNameLength: u32 = 32;
		pub const MinNameLength: u32 = 3;
		// Easier to setup insufficient funds for deposit but still above existential deposit
		pub const Web3NameDeposit: Balance = MICRO_KILT;
	}

	impl pallet_web3_names::Config for Test {
		type BanOrigin = TestBanOrigin;
		type OwnerOrigin = TestOwnerOrigin;
		type OriginSuccess = TestOriginSuccess;
		type Currency = Balances;
		type RuntimeHoldReason = RuntimeHoldReason;
		type Deposit = Web3NameDeposit;
		type RuntimeEvent = ();
		type MaxNameLength = MaxNameLength;
		type MinNameLength = MinNameLength;
		type Web3Name = TestWeb3Name;
		type Web3NameOwner = TestWeb3NameOwner;
		type WeightInfo = ();
		type BalanceMigrationManager = Migration;
	}

	#[derive(
		Default,
		Clone,
		Copy,
		Encode,
		Decode,
		MaxEncodedLen,
		sp_runtime::RuntimeDebug,
		Eq,
		PartialEq,
		Ord,
		PartialOrd,
		TypeInfo,
	)]
	pub struct TestSubjectId([u8; 32]);

	impl TryFrom<Vec<u8>> for TestSubjectId {
		type Error = public_credentials::Error<Test>;

		// Test failure for subject input. Fails if the input vector is too long or if
		// the first byte is 255.
		fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
			let inner: [u8; 32] = value
				.try_into()
				.map_err(|_| public_credentials::Error::<Test>::InvalidInput)?;
			if inner[0] == 255 {
				Err(public_credentials::Error::<Test>::InvalidInput)
			} else {
				Ok(Self(inner))
			}
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl kilt_support::traits::GetWorstCase for TestSubjectId {
		// Only used for benchmark testing, not really relevant.
		fn worst_case() -> Self {
			crate::mock::TestSubjectId::default()
		}
	}

	impl From<TestSubjectId> for Vec<u8> {
		fn from(value: TestSubjectId) -> Self {
			value.0.into()
		}
	}

	impl From<TestSubjectId> for InputSubjectIdOf<Test> {
		fn from(value: TestSubjectId) -> Self {
			value
				.0
				.to_vec()
				.try_into()
				.expect("Test subject ID should fit into the expected input subject ID of for the test runtime.")
		}
	}

	impl From<[u8; 32]> for TestSubjectId {
		fn from(value: [u8; 32]) -> Self {
			Self(value)
		}
	}

	impl public_credentials::Config for Test {
		type AccessControl = public_credentials::mock::MockAccessControl<Self>;
		type AttesterId = SubjectId;
		type AuthorizationId = SubjectId;
		type CredentialId = Hash;
		type RuntimeHoldReason = RuntimeHoldReason;
		type CredentialHash = BlakeTwo256;
		type Currency = Balances;
		type Deposit = ConstU128<{ 10 * MICRO_KILT }>;
		type EnsureOrigin = mock_origin::EnsureDoubleOrigin<AccountId, Self::AttesterId>;
		type RuntimeEvent = ();
		type MaxEncodedClaimsLength = ConstU32<500>;
		type MaxSubjectIdLength = ConstU32<100>;
		type OriginSuccess = mock_origin::DoubleOrigin<AccountId, Self::AttesterId>;
		type SubjectId = TestSubjectId;
		type WeightInfo = ();
		type BalanceMigrationManager = Migration;
	}

	pub(crate) type BlockNumber = u64;
	pub(crate) const BLOCKS_PER_ROUND: BlockNumber = 5;

	parameter_types! {
		pub const MinBlocksPerRound: BlockNumber = 3;
		pub const StakeDuration: u32 = 2;
		pub const ExitQueueDelay: u32 = 2;
		pub const DefaultBlocksPerRound: BlockNumber = BLOCKS_PER_ROUND;
		pub const MinCollators: u32 = 2;
		pub const MaxDelegationsPerRound: u32 = 2;
		#[derive(Debug, Eq, PartialEq)]
		pub const MaxDelegatorsPerCollator: u32 = 4;
		pub const MinCollatorStake: Balance = 10;
		#[derive(Debug, Eq, PartialEq)]
		pub const MaxCollatorCandidates: u32 = 10;
		pub const MinDelegatorStake: Balance = 5;
		pub const MaxUnstakeRequests: u32 = 6;
		pub const NetworkRewardRate: Perquintill = Perquintill::from_percent(10);
		pub const NetworkRewardStart: BlockNumber = 5 * 5 * 60 * 24 * 36525 / 100;
	}

	parameter_types! {
		pub const MinimumPeriod: u64 = 1;
	}

	pub(crate) const DID_00: SubjectId = SubjectId(AccountId32::new([0u8; 32]));
	pub(crate) const ACCOUNT_00: AccountId = AccountId::new([1u8; 32]);
	pub(crate) const ALICE_SEED: [u8; 32] = [1u8; 32];
	pub(crate) const BOB_SEED: [u8; 32] = [2u8; 32];
	pub const CLAIM_HASH_SEED_12: u64 = 12u64;
	pub const DELEGATION_ID_SEED_3: u64 = 3u64;
	pub(crate) const LINKABLE_ACCOUNT_00: LinkableAccountId = LinkableAccountId::AccountId32(ACCOUNT_00);
	pub(crate) const WEB3_NAME_00_INPUT: &[u8; 12] = b"web3_name_00";
	pub(crate) const SUBJECT_ID_00: TestSubjectId = TestSubjectId([100u8; 32]);

	pub fn get_did_identifier_from_ed25519_key(public_key: ed25519::Public) -> DidIdentifier {
		MultiSigner::from(public_key).into_account()
	}

	#[derive(Clone, Default)]
	pub(crate) struct ExtBuilder {
		ctypes_stored: Vec<(CtypeHashOf<Test>, SubjectId)>,
		balances: Vec<(AccountId, BalanceOf<Test>)>,
		attestations: Vec<(attestation::ClaimHashOf<Test>, attestation::AttestationDetailsOf<Test>)>,
		delegations: Vec<(delegation::DelegationNodeIdOf<Test>, delegation::DelegationNodeOf<Test>)>,
		delegation_hierarchies: delegation::mock::DelegationHierarchyInitialization<Test>,
		dids_stored: Vec<(DidIdentifier, did::did_details::DidDetails<Test>)>,
		connections: Vec<(
			AccountId,
			SubjectId,
			pallet_did_lookup::linkable_account::LinkableAccountId,
		)>,
		claimed_web3_names: Vec<(TestWeb3NameOwner, TestWeb3Name, TestWeb3NamePayer)>,
		public_credentials: Vec<(
			<Test as public_credentials::Config>::SubjectId,
			public_credentials::CredentialIdOf<Test>,
			public_credentials::CredentialEntryOf<Test>,
		)>,
	}

	impl ExtBuilder {
		pub(crate) fn with_ctypes(mut self, ctypes: Vec<(CtypeHashOf<Test>, SubjectId)>) -> Self {
			self.ctypes_stored = ctypes;
			self
		}

		pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, BalanceOf<Test>)>) -> Self {
			self.balances = balances;
			self
		}

		#[must_use]
		pub fn with_connections(
			mut self,
			connections: Vec<(
				AccountId,
				SubjectId,
				pallet_did_lookup::linkable_account::LinkableAccountId,
			)>,
		) -> Self {
			self.connections = connections;
			self
		}

		#[must_use]
		pub fn with_web3_names(
			mut self,
			web3_names: Vec<(TestWeb3NameOwner, TestWeb3Name, TestWeb3NamePayer)>,
		) -> Self {
			self.claimed_web3_names = web3_names;
			self
		}

		#[must_use]
		pub fn with_dids(mut self, dids: Vec<(DidIdentifier, did::did_details::DidDetails<Test>)>) -> Self {
			self.dids_stored = dids;
			self
		}

		#[must_use]
		pub fn with_attestations(
			mut self,
			attestations: Vec<(attestation::ClaimHashOf<Test>, attestation::AttestationDetailsOf<Test>)>,
		) -> Self {
			self.attestations = attestations;
			self
		}

		#[must_use]
		pub fn with_delegation_hierarchies(
			mut self,
			delegation_hierarchies: delegation::mock::DelegationHierarchyInitialization<Test>,
		) -> Self {
			self.delegation_hierarchies = delegation_hierarchies;
			self
		}

		#[must_use]
		pub fn with_delegations(
			mut self,
			delegations: Vec<(delegation::DelegationNodeIdOf<Test>, delegation::DelegationNodeOf<Test>)>,
		) -> Self {
			self.delegations = delegations;
			self
		}

		#[must_use]
		pub fn with_public_credentials(
			mut self,
			credentials: Vec<(
				<Test as public_credentials::Config>::SubjectId,
				public_credentials::CredentialIdOf<Test>,
				public_credentials::CredentialEntryOf<Test>,
			)>,
		) -> Self {
			self.public_credentials = credentials;
			self
		}

		pub(crate) fn build(self) -> sp_io::TestExternalities {
			let mut storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
			pallet_balances::GenesisConfig::<Test> {
				balances: self.balances.clone(),
			}
			.assimilate_storage(&mut storage)
			.expect("assimilate should not fail");
			let mut ext = sp_io::TestExternalities::new(storage);

			ext.execute_with(|| {
				for (ctype_hash, owner) in self.ctypes_stored.iter() {
					Ctypes::<Test>::insert(
						ctype_hash,
						CtypeEntryOf::<Test> {
							creator: owner.clone(),
							created_at: System::block_number(),
						},
					);
				}

				delegation::mock::initialize_pallet::<Test>(self.delegations, self.delegation_hierarchies);

				for (claim_hash, details) in self.attestations {
					attestation::mock::insert_attestation::<Test>(claim_hash, details);
				}

				for did in self.dids_stored.iter() {
					did::Did::<Test>::insert(&did.0, did.1.clone());
					pallet_balances::Pallet::<Test>::hold(
						&did::HoldReason::Deposit.into(),
						&did.1.deposit.owner,
						did.1.deposit.amount,
					)
					.expect("Deposit owner should have enough balance");
				}

				for (sender, did, account) in self.connections {
					pallet_did_lookup::Pallet::<Test>::add_association(sender, did, account)
						.expect("Should create connection");
				}

				for (owner, web3_name, payer) in self.claimed_web3_names {
					pallet_web3_names::Pallet::<Test>::register_name(web3_name, owner, payer)
						.expect("Could not register name");
				}

				for (subject_id, credential_id, credential_entry) in self.public_credentials {
					public_credentials::mock::insert_public_credentials::<Test>(
						subject_id,
						credential_id,
						credential_entry,
					);
				}
			});

			ext
		}

		#[cfg(feature = "runtime-benchmarks")]
		pub(crate) fn build_with_keystore(self) -> sp_io::TestExternalities {
			use sp_keystore::{testing::MemoryKeystore, KeystoreExt};
			use sp_std::sync::Arc;

			let mut ext = self.build();

			let keystore = MemoryKeystore::new();
			ext.register_extension(KeystoreExt(Arc::new(keystore)));

			ext
		}
	}

	pub(crate) fn translate_all_holds_to_reserves() {
		let deposit_reasons: Vec<RuntimeHoldReason> = vec![
			attestation::HoldReason::Deposit.into(),
			delegation::HoldReason::Deposit.into(),
			did::HoldReason::Deposit.into(),
			pallet_did_lookup::HoldReason::Deposit.into(),
			pallet_web3_names::HoldReason::Deposit.into(),
			public_credentials::HoldReason::Deposit.into(),
		];

		deposit_reasons
			.iter()
			.for_each(|hold_id| kilt_support::migration::translate_holds_to_reserve::<Test>(*hold_id))
	}

	pub(crate) fn clear_storage() {
		let count = MigratedKeys::<Test>::iter().count() as u32;
		let cursor = MigratedKeys::<Test>::clear(count, None).maybe_cursor;
		assert!(cursor.is_none());
	}
}
