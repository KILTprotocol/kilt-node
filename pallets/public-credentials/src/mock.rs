// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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

use attestation::ClaimHashOf;
use ctype::CtypeHashOf;

use crate::{Claim, ClaimerSignatureInfo, Config, InputClaimsContentOf, InputCredentialOf, InputSubjectIdOf};

pub(crate) type ClaimerSignatureInfoOf<Test> =
	ClaimerSignatureInfo<<Test as Config>::ClaimerIdentifier, <Test as Config>::ClaimerSignature>;

// Generate a public credential using a many Default::default() as possible.
pub fn generate_base_public_credential_creation_op<T: Config>(
	subject_id: InputSubjectIdOf<T>,
	claim_hash: ClaimHashOf<T>,
	ctype_hash: CtypeHashOf<T>,
	contents: InputClaimsContentOf<T>,
	claimer_signature: Option<ClaimerSignatureInfoOf<T>>,
) -> InputCredentialOf<T> {
	InputCredentialOf::<T> {
		claim: Claim {
			ctype_hash,
			subject: subject_id,
			contents,
		},
		claim_hash,
		claimer_signature,
		nonce: Default::default(),
		authorization_info: Default::default(),
	}
}

#[cfg(test)]
pub use crate::mock::runtime::*;

// Mocks that are only used internally
#[cfg(test)]
pub(crate) mod runtime {
	use super::*;

	use codec::{Decode, Encode, MaxEncodedLen};
	use frame_support::{
		traits::{ConstU128, ConstU16, ConstU32, ConstU64, Get},
		weights::constants::RocksDbWeight,
	};
	use scale_info::TypeInfo;
	use sp_core::{sr25519, Pair};
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
		MultiSignature, MultiSigner,
	};

	use kilt_support::{
		deposit::Deposit,
		mock::{mock_origin, SubjectId},
		signature::EqualVerify,
	};

	use attestation::{mock::MockAccessControl, AttestationDetails, ClaimHashOf};
	use ctype::{CtypeCreatorOf, CtypeHashOf};

	use crate::{
		BalanceOf, CredentialEntryOf, Credentials, CredentialsUnicityIndex, CurrencyOf, Error, InputSubjectIdOf,
	};

	pub(crate) type BlockNumber = u64;
	pub(crate) type Balance = u128;
	pub(crate) type Hash = sp_core::H256;
	pub(crate) type AccountPublic = <MultiSignature as Verify>::Signer;
	pub(crate) type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

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
		type Error = Error<Test>;

		fn try_from(value:Vec<u8>) -> Result<Self, Self::Error> {
			let inner: [u8; 32] = value.try_into().map_err(|_| Error::<Test>::InvalidInput)?;
			Ok(Self(inner))
		}
	}

	impl From<TestSubjectId> for Vec<u8> {
		fn from(value: TestSubjectId) -> Self {
			value.0.into()
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl kilt_support::traits::GetWorstCase for TestSubjectId {
		// Only used for benchmark testing, not really relevant.
		fn worst_case() -> Self {
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

	pub(crate) fn generate_base_credential_entry<T: Config>(
		payer: T::AccountId,
		block_number: <T as frame_system::Config>::BlockNumber,
	) -> CredentialEntryOf<T> {
		CredentialEntryOf::<T> {
			block_number,
			deposit: Deposit::<T::AccountId, BalanceOf<T>> {
				owner: payer,
				amount: <T as Config>::Deposit::get(),
			},
		}
	}

	fn insert_public_credentials<T: Config>(
		subject_id: T::SubjectId,
		claim_hash: ClaimHashOf<T>,
		credential_entry: CredentialEntryOf<T>,
	) {
		kilt_support::reserve_deposit::<T::AccountId, CurrencyOf<T>>(
			credential_entry.deposit.owner.clone(),
			credential_entry.deposit.amount,
		)
		.expect("Attester should have enough balance");

		Credentials::<T>::insert(&subject_id, &claim_hash, credential_entry);
		CredentialsUnicityIndex::<T>::insert(claim_hash, subject_id);
	}

	pub(crate) const MILLI_UNIT: Balance = 10u128.pow(12);

	frame_support::construct_runtime!(
		pub enum Test where
			Block = frame_system::mocking::MockBlock<Test>,
			NodeBlock = frame_system::mocking::MockBlock<Test>,
			UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			Attestation: attestation::{Pallet, Call, Storage, Event<T>},
			Ctype: ctype::{Pallet, Call, Storage, Event<T>},
			Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
			MockOrigin: mock_origin::{Pallet, Origin<T>},
			PublicCredentials: crate::{Pallet, Call, Storage, Event<T>},
		}
	);

	impl mock_origin::Config for Test {
		type Origin = Origin;
		type AccountId = AccountId;
		type SubjectId = SubjectId;
	}

	impl frame_system::Config for Test {
		type Origin = Origin;
		type Call = Call;
		type Index = u64;
		type BlockNumber = BlockNumber;
		type Hash = Hash;
		type Hashing = BlakeTwo256;
		type AccountId = AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
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
	}

	impl pallet_balances::Config for Test {
		type Balance = Balance;
		type DustRemoval = ();
		type Event = ();
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
		type Event = ();
		type WeightInfo = ();

		type Currency = Balances;
		type Fee = ConstU128<500>;
		type FeeCollector = ();
	}

	impl attestation::Config for Test {
		type EnsureOrigin = mock_origin::EnsureDoubleOrigin<AccountId, Self::AttesterId>;
		type OriginSuccess = mock_origin::DoubleOrigin<AccountId, Self::AttesterId>;
		type Event = ();
		type WeightInfo = ();

		type Currency = Balances;
		type Deposit = ConstU128<{ 100 * MILLI_UNIT }>;
		type MaxDelegatedAttestations = ConstU32<0>;
		type AttesterId = SubjectId;
		type AuthorizationId = SubjectId;
		type AccessControl = MockAccessControl<Self>;
	}

	impl Config for Test {
		type ClaimerIdentifier = SubjectId;
		type ClaimerSignature = (Self::ClaimerIdentifier, Vec<u8>);
		type ClaimerSignatureVerification = EqualVerify<Self::ClaimerIdentifier, Vec<u8>>;
		type Deposit = ConstU128<{ 10 * MILLI_UNIT }>;
		type EnsureOrigin = <Self as attestation::Config>::EnsureOrigin;
		type Event = ();
		type MaxEncodedClaimsLength = ConstU32<500>;
		type MaxSubjectIdLength = ConstU32<100>;
		type OriginSuccess = <Self as attestation::Config>::OriginSuccess;
		type SubjectId = TestSubjectId;
		type WeightInfo = ();
	}

	pub(crate) const ACCOUNT_00: AccountId = AccountId::new([1u8; 32]);
	pub(crate) const ACCOUNT_01: AccountId = AccountId::new([2u8; 32]);

	pub(crate) const ALICE_SEED: [u8; 32] = [0u8; 32];
	pub(crate) const BOB_SEED: [u8; 32] = [1u8; 32];
	pub(crate) const CHARLIE_SEED: [u8; 32] = [2u8; 32];

	pub(crate) const SUBJECT_ID_00: TestSubjectId = TestSubjectId([100u8; 32]);

	pub(crate) const CLAIM_HASH_SEED_01: u64 = 1u64;
	pub(crate) const CLAIM_HASH_SEED_02: u64 = 2u64;

	pub(crate) fn claim_hash_from_seed(seed: u64) -> Hash {
		Hash::from_low_u64_be(seed)
	}

	pub(crate) fn sr25519_did_from_seed(seed: &[u8; 32]) -> SubjectId {
		MultiSigner::from(sr25519::Pair::from_seed(seed).public())
			.into_account()
			.into()
	}

	pub(crate) fn hash_to_u8<Hash: Encode>(hash: Hash) -> Vec<u8> {
		hash.encode()
	}

	#[derive(Clone, Default)]
	pub(crate) struct ExtBuilder {
		/// initial ctypes & owners
		ctypes: Vec<(CtypeHashOf<Test>, CtypeCreatorOf<Test>)>,
		/// endowed accounts with balances
		balances: Vec<(AccountId, Balance)>,
		attestations: Vec<(ClaimHashOf<Test>, AttestationDetails<Test>)>,
		public_credentials: Vec<(<Test as Config>::SubjectId, ClaimHashOf<Test>, CredentialEntryOf<Test>)>,
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
		pub fn with_attestations(mut self, attestations: Vec<(ClaimHashOf<Test>, AttestationDetails<Test>)>) -> Self {
			self.attestations = attestations;
			self
		}

		#[must_use]
		pub fn with_public_credentials(
			mut self,
			credentials: Vec<(<Test as Config>::SubjectId, ClaimHashOf<Test>, CredentialEntryOf<Test>)>,
		) -> Self {
			self.public_credentials = credentials;
			self
		}

		pub(crate) fn build(self) -> sp_io::TestExternalities {
			let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
			pallet_balances::GenesisConfig::<Test> {
				balances: self.balances.clone(),
			}
			.assimilate_storage(&mut storage)
			.expect("assimilate should not fail");

			let mut ext = sp_io::TestExternalities::new(storage);

			ext.execute_with(|| {
				for ctype in self.ctypes {
					ctype::Ctypes::<Test>::insert(ctype.0, ctype.1.clone());
				}

				for (claim_hash, details) in self.attestations {
					attestation::mock::insert_attestation(claim_hash, details);
				}

				for (subject_id, claim_hash, credential_entry) in self.public_credentials {
					insert_public_credentials::<Test>(subject_id, claim_hash, credential_entry);
				}
			});

			ext
		}

		#[cfg(feature = "runtime-benchmarks")]
		pub(crate) fn build_with_keystore(self) -> sp_io::TestExternalities {
			let mut ext = self.build();

			let keystore = sp_keystore::testing::KeyStore::new();
			ext.register_extension(sp_keystore::KeystoreExt(std::sync::Arc::new(keystore)));

			ext
		}
	}
}
