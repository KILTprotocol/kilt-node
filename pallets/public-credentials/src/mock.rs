// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

use frame_support::{traits::Get, weights::Weight};
use frame_system::pallet_prelude::BlockNumberFor;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{traits::Hash, DispatchError};

use kilt_support::{traits::StorageDepositCollector, Deposit};

use crate::{
	AttesterOf, BalanceOf, Config, CredentialEntryOf, CredentialIdOf, CredentialSubjects, Credentials, CtypeHashOf,
	InputClaimsContentOf, InputCredentialOf, InputSubjectIdOf, PublicCredentialDepositCollector,
	PublicCredentialsAccessControl,
};

// Generate a public credential using a many Default::default() as possible.
pub fn generate_base_public_credential_creation_op<T: Config>(
	subject_id: InputSubjectIdOf<T>,
	ctype_hash: CtypeHashOf<T>,
	claims: InputClaimsContentOf<T>,
) -> InputCredentialOf<T> {
	InputCredentialOf::<T> {
		ctype_hash,
		subject: subject_id,
		claims,
		authorization: None,
	}
}

pub fn generate_credential_id<T: Config>(
	input_credential: &InputCredentialOf<T>,
	attester: &AttesterOf<T>,
) -> CredentialIdOf<T> {
	T::CredentialHash::hash(&[&input_credential.encode()[..], &attester.encode()[..]].concat()[..])
}

/// Generates a basic credential entry using the provided input parameters
/// and the default value for the other ones. The credential will be marked
/// as non-revoked and with no authorization ID associated with it.
pub fn generate_base_credential_entry<T: Config>(
	payer: T::AccountId,
	block_number: BlockNumberFor<T>,
	attester: T::AttesterId,
	ctype_hash: Option<CtypeHashOf<T>>,
	deposit: Option<Deposit<T::AccountId, BalanceOf<T>>>,
) -> CredentialEntryOf<T> {
	CredentialEntryOf::<T> {
		ctype_hash: ctype_hash.unwrap_or_default(),
		revoked: false,
		attester,
		block_number,
		deposit: deposit.unwrap_or(Deposit::<T::AccountId, BalanceOf<T>> {
			owner: payer,
			amount: <T as Config>::Deposit::get(),
		}),
		authorization_id: None,
	}
}

pub fn insert_public_credentials<T: Config>(
	subject_id: T::SubjectId,
	credential_id: CredentialIdOf<T>,
	credential_entry: CredentialEntryOf<T>,
) {
	PublicCredentialDepositCollector::<T>::create_deposit(
		credential_entry.deposit.owner.clone(),
		credential_entry.deposit.amount,
	)
	.expect("Attester should have enough balance");

	Credentials::<T>::insert(&subject_id, &credential_id, credential_entry);
	CredentialSubjects::<T>::insert(credential_id, subject_id);
}

/// Authorize iff the subject of the origin and the provided attester id
/// match.
#[derive(Clone, Debug, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[scale_info(skip_type_params(T))]
pub struct MockAccessControl<T: Config>(pub T::AttesterId);

impl<T> PublicCredentialsAccessControl<T::AttesterId, T::AuthorizationId, CtypeHashOf<T>, CredentialIdOf<T>>
	for MockAccessControl<T>
where
	T: Config<AuthorizationId = <T as Config>::AttesterId>,
{
	fn can_issue(
		&self,
		who: &T::AttesterId,
		_ctype: &CtypeHashOf<T>,
		_credential_id: &CredentialIdOf<T>,
	) -> Result<Weight, DispatchError> {
		if who == &self.0 {
			Ok(Weight::zero())
		} else {
			Err(DispatchError::Other("Unauthorized"))
		}
	}

	fn can_revoke(
		&self,
		who: &T::AttesterId,
		_ctype: &CtypeHashOf<T>,
		_credential_id: &CredentialIdOf<T>,
		authorization_id: &T::AuthorizationId,
	) -> Result<Weight, DispatchError> {
		if authorization_id == who {
			Ok(Weight::zero())
		} else {
			Err(DispatchError::Other("Unauthorized"))
		}
	}

	fn can_unrevoke(
		&self,
		who: &T::AttesterId,
		_ctype: &CtypeHashOf<T>,
		_credential_id: &CredentialIdOf<T>,
		authorization_id: &T::AuthorizationId,
	) -> Result<Weight, DispatchError> {
		if authorization_id == who {
			Ok(Weight::zero())
		} else {
			Err(DispatchError::Other("Unauthorized"))
		}
	}

	fn can_remove(
		&self,
		who: &T::AttesterId,
		_ctype: &CtypeHashOf<T>,
		_credential_id: &CredentialIdOf<T>,
		authorization_id: &T::AuthorizationId,
	) -> Result<Weight, DispatchError> {
		if authorization_id == who {
			Ok(Weight::zero())
		} else {
			Err(DispatchError::Other("Unauthorized"))
		}
	}

	fn authorization_id(&self) -> T::AuthorizationId {
		self.0.clone()
	}

	fn can_issue_weight(&self) -> Weight {
		Weight::zero()
	}
	fn can_revoke_weight(&self) -> Weight {
		Weight::zero()
	}
	fn can_unrevoke_weight(&self) -> Weight {
		Weight::zero()
	}
	fn can_remove_weight(&self) -> Weight {
		Weight::zero()
	}
}

#[cfg(test)]
pub use crate::mock::runtime::*;

// Mocks that are only used internally
#[cfg(test)]
pub(crate) mod runtime {
	use super::*;

	use frame_support::{
		traits::{ConstU128, ConstU16, ConstU32, ConstU64},
		weights::constants::RocksDbWeight,
	};
	use frame_system::EnsureSigned;
	use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
	use scale_info::TypeInfo;
	use sp_core::{sr25519, Pair};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
		BuildStorage, MultiSignature, MultiSigner, RuntimeDebug,
	};

	use kilt_support::mock::{mock_origin, SubjectId};

	use ctype::{CtypeCreatorOf, CtypeEntryOf, CtypeHashOf};

	use crate::{self as public_credentials, Config, CredentialEntryOf, Error, InputSubjectIdOf};

	pub(crate) type Balance = u128;
	pub(crate) type Hash = sp_core::H256;
	pub(crate) type AccountPublic = <MultiSignature as Verify>::Signer;
	pub(crate) type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

	#[derive(
		Default, Clone, Copy, Encode, Decode, MaxEncodedLen, RuntimeDebug, Eq, PartialEq, Ord, PartialOrd, TypeInfo,
	)]
	pub struct TestSubjectId(pub [u8; 32]);

	impl TryFrom<Vec<u8>> for TestSubjectId {
		type Error = Error<Test>;

		// Test failure for subject input. Fails if the input vector is too long or if
		// the first byte is 255.
		fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
			let inner: [u8; 32] = value.try_into().map_err(|_| Error::<Test>::InvalidInput)?;
			if inner[0] == 255 {
				Err(Error::<Test>::InvalidInput)
			} else {
				Ok(Self(inner))
			}
		}
	}

	impl From<TestSubjectId> for Vec<u8> {
		fn from(value: TestSubjectId) -> Self {
			value.0.into()
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<Context> kilt_support::traits::GetWorstCase<Context> for TestSubjectId {
		// Only used for benchmark testing, not really relevant.
		type Output = Self;

		fn worst_case(_context: Context) -> Self::Output {
			crate::mock::TestSubjectId::default()
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

	pub(crate) const MILLI_UNIT: Balance = 10u128.pow(12);
	type Block = frame_system::mocking::MockBlock<Test>;

	frame_support::construct_runtime!(
		pub enum Test
		{
			System: frame_system,
			Ctype: ctype,
			Balances: pallet_balances,
			MockOrigin: mock_origin,
			PublicCredentials: public_credentials,
		}
	);

	impl mock_origin::Config for Test {
		type RuntimeOrigin = RuntimeOrigin;
		type AccountId = AccountId;
		type SubjectId = SubjectId;
	}

	impl frame_system::Config for Test {
		type RuntimeTask = ();
		type RuntimeOrigin = RuntimeOrigin;
		type RuntimeCall = RuntimeCall;
		type Block = Block;
		type Nonce = u64;
		type Hash = Hash;
		type Hashing = BlakeTwo256;
		type AccountId = AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
		type RuntimeEvent = RuntimeEvent;
		type BlockHashCount = ConstU64<250>;
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
		type SS58Prefix = ConstU16<38>;
		type OnSetCode = ();
		type MaxConsumers = ConstU32<16>;
		type MultiBlockMigrator = ();
		type SingleBlockMigrations = ();
		type PostInherents = ();
		type PostTransactions = ();
		type PreInherents = ();
	}

	impl pallet_balances::Config for Test {
		type RuntimeFreezeReason = RuntimeFreezeReason;
		type FreezeIdentifier = RuntimeFreezeReason;
		type RuntimeHoldReason = RuntimeHoldReason;
		type MaxFreezes = ConstU32<10>;
		type Balance = Balance;
		type DustRemoval = ();
		type RuntimeEvent = RuntimeEvent;
		type ExistentialDeposit = ConstU128<MILLI_UNIT>;
		type AccountStore = System;
		type WeightInfo = ();
		type MaxLocks = ConstU32<5>;
		type MaxReserves = ConstU32<5>;
		type ReserveIdentifier = [u8; 8];
	}

	impl ctype::Config for Test {
		type CtypeCreatorId = SubjectId;
		type EnsureOrigin = mock_origin::EnsureDoubleOrigin<AccountId, Self::CtypeCreatorId>;
		type OriginSuccess = mock_origin::DoubleOrigin<AccountId, Self::CtypeCreatorId>;
		type OverarchingOrigin = EnsureSigned<AccountId>;
		type RuntimeEvent = RuntimeEvent;
		type WeightInfo = ();

		type Currency = Balances;
		type Fee = ConstU128<500>;
		type FeeCollector = ();
	}

	impl Config for Test {
		type AccessControl = MockAccessControl<Self>;
		type AttesterId = SubjectId;
		type AuthorizationId = SubjectId;
		type CredentialId = Hash;
		type RuntimeHoldReason = RuntimeHoldReason;
		type CredentialHash = BlakeTwo256;
		type Currency = Balances;
		type Deposit = ConstU128<{ 10 * MILLI_UNIT }>;
		type EnsureOrigin = mock_origin::EnsureDoubleOrigin<AccountId, Self::AttesterId>;
		type RuntimeEvent = RuntimeEvent;
		type MaxEncodedClaimsLength = ConstU32<500>;
		type MaxSubjectIdLength = ConstU32<100>;
		type OriginSuccess = mock_origin::DoubleOrigin<AccountId, Self::AttesterId>;
		type SubjectId = TestSubjectId;
		type WeightInfo = ();
		type BalanceMigrationManager = ();
	}

	pub(crate) const ACCOUNT_00: AccountId = AccountId::new([1u8; 32]);
	pub(crate) const ACCOUNT_01: AccountId = AccountId::new([2u8; 32]);
	// Min Balance has to be >= [ExistentialDeposit]
	pub(crate) const MIN_BALANCE: Balance = MILLI_UNIT;

	pub(crate) const ALICE_SEED: [u8; 32] = [0u8; 32];
	pub(crate) const BOB_SEED: [u8; 32] = [1u8; 32];

	pub(crate) const SUBJECT_ID_00: TestSubjectId = TestSubjectId([100u8; 32]);
	pub(crate) const SUBJECT_ID_01: TestSubjectId = TestSubjectId([1u8; 32]);
	pub(crate) const INVALID_SUBJECT_ID: TestSubjectId = TestSubjectId([255u8; 32]);

	pub(crate) fn sr25519_did_from_seed(seed: &[u8; 32]) -> SubjectId {
		MultiSigner::from(sr25519::Pair::from_seed(seed).public())
			.into_account()
			.into()
	}

	#[derive(Clone, Default)]
	pub(crate) struct ExtBuilder {
		/// initial ctypes & owners
		ctypes: Vec<(CtypeHashOf<Test>, CtypeCreatorOf<Test>)>,
		/// endowed accounts with balances
		balances: Vec<(AccountId, Balance)>,
		public_credentials: Vec<(
			<Test as Config>::SubjectId,
			CredentialIdOf<Test>,
			CredentialEntryOf<Test>,
		)>,
	}

	impl ExtBuilder {
		#[must_use]
		pub fn with_ctypes(mut self, ctypes: Vec<(CtypeHashOf<Test>, CtypeCreatorOf<Test>)>) -> Self {
			self.ctypes = ctypes;
			self
		}

		#[must_use]
		pub fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
			self.balances = balances;
			self
		}

		#[must_use]
		pub fn with_public_credentials(
			mut self,
			credentials: Vec<(
				<Test as Config>::SubjectId,
				CredentialIdOf<Test>,
				CredentialEntryOf<Test>,
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
				// ensure that we are not at the genesis block. Events are not registered for
				// the genesis block.
				System::set_block_number(System::block_number() + 1);

				for ctype in self.ctypes {
					ctype::Ctypes::<Test>::insert(
						ctype.0,
						CtypeEntryOf::<Test> {
							creator: ctype.1.clone(),
							created_at: System::block_number(),
						},
					);
				}

				for (subject_id, credential_id, credential_entry) in self.public_credentials {
					insert_public_credentials::<Test>(subject_id, credential_id, credential_entry);
				}
			});

			ext
		}

		pub fn build_and_execute_with_sanity_tests(self, test: impl FnOnce()) {
			self.build().execute_with(|| {
				test();
				crate::try_state::do_try_state::<Test>().expect("Sanity test for public credential failed.");
			})
		}

		#[cfg(feature = "runtime-benchmarks")]
		pub(crate) fn build_with_keystore(self) -> sp_io::TestExternalities {
			let mut ext = self.build();

			let keystore = sp_keystore::testing::MemoryKeystore::new();
			ext.register_extension(sp_keystore::KeystoreExt(std::sync::Arc::new(keystore)));

			ext
		}
	}
}
