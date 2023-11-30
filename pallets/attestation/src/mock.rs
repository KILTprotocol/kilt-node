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

//! Helpers functions for tests.
//!
//! Functions are split into internal functions and public functions.
//! Public functions are generic for any runtime, so that they can be reused in
//! other tests. Internal functions/structs can only be used in attestation
//! tests.

use frame_support::{dispatch::Weight, traits::Get};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::{sr25519, H256};
use sp_runtime::{traits::IdentifyAccount, DispatchError, MultiSigner};

use ctype::CtypeHashOf;
use kilt_support::{mock::SubjectId, traits::StorageDepositCollector, Deposit};

use crate::{
	pallet::AuthorizationIdOf, AccountIdOf, AttestationAccessControl, AttestationDetails, AttestationDetailsOf,
	AttesterOf, BalanceOf, ClaimHashOf, Config,
};

pub type Hash = sp_core::H256;

#[cfg(test)]
pub use crate::mock::runtime::*;

pub struct AttestationCreationDetails<T: Config> {
	pub claim_hash: ClaimHashOf<T>,
	pub ctype_hash: CtypeHashOf<T>,
	pub authorization_id: Option<AuthorizationIdOf<T>>,
}

pub fn generate_base_attestation_creation_details<T: Config>(
	claim_hash: ClaimHashOf<T>,
	attestation: AttestationDetailsOf<T>,
) -> AttestationCreationDetails<T> {
	AttestationCreationDetails {
		claim_hash,
		ctype_hash: attestation.ctype_hash,
		authorization_id: attestation.authorization_id,
	}
}

pub fn generate_base_attestation<T>(attester: AttesterOf<T>, payer: AccountIdOf<T>) -> AttestationDetailsOf<T>
where
	T: Config,
	T::Hash: From<H256>,
{
	generate_base_attestation_with_deposit::<T>(attester, payer, <T as Config>::Deposit::get())
}

pub fn generate_base_attestation_with_deposit<T>(
	attester: AttesterOf<T>,
	payer: AccountIdOf<T>,
	deposit: BalanceOf<T>,
) -> AttestationDetailsOf<T>
where
	T: Config,
	T::Hash: From<H256>,
{
	AttestationDetails {
		attester,
		authorization_id: None,
		ctype_hash: ctype::mock::get_ctype_hash::<T>(true),
		revoked: false,
		deposit: Deposit::<AccountIdOf<T>, BalanceOf<T>> {
			owner: payer,
			amount: deposit,
		},
	}
}

/// Authorize iff the subject of the origin and the provided attester id match.
#[derive(Clone, Debug, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[scale_info(skip_type_params(T))]
pub struct MockAccessControl<T: Config>(pub T::AttesterId);

impl<T> AttestationAccessControl<T::AttesterId, T::AuthorizationId, CtypeHashOf<T>, ClaimHashOf<T>>
	for MockAccessControl<T>
where
	T: Config<AuthorizationId = <T as Config>::AttesterId>,
{
	fn can_attest(
		&self,
		who: &T::AttesterId,
		_ctype: &CtypeHashOf<T>,
		_claim: &ClaimHashOf<T>,
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
		_claim: &ClaimHashOf<T>,
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
		_claim: &ClaimHashOf<T>,
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

	fn can_attest_weight(&self) -> Weight {
		Weight::zero()
	}
	fn can_revoke_weight(&self) -> Weight {
		Weight::zero()
	}
	fn can_remove_weight(&self) -> Weight {
		Weight::zero()
	}
}

pub fn insert_attestation<T: Config>(claim_hash: ClaimHashOf<T>, details: AttestationDetailsOf<T>) {
	crate::AttestationStorageDepositCollector::<T>::create_deposit(
		details.deposit.owner.clone(),
		details.deposit.amount,
	)
	.expect("Should have balance");

	crate::Attestations::<T>::insert(claim_hash, details.clone());
	if let Some(delegation_id) = details.authorization_id.as_ref() {
		crate::ExternalAttestations::<T>::insert(delegation_id, claim_hash, true)
	}
}

pub fn sr25519_did_from_public_key(public_key: &[u8; 32]) -> SubjectId {
	MultiSigner::from(sr25519::Public(*public_key)).into_account().into()
}

pub fn claim_hash_from_seed(seed: u64) -> Hash {
	Hash::from_low_u64_be(seed)
}

/// Mocks that are only used internally
#[cfg(test)]
pub(crate) mod runtime {
	use super::*;

	use frame_support::{parameter_types, weights::constants::RocksDbWeight};
	use frame_system::EnsureSigned;

	use sp_core::{ed25519, Pair};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
		BuildStorage, MultiSignature, MultiSigner,
	};

	use ctype::{CtypeCreatorOf, CtypeEntryOf};
	use kilt_support::mock::{mock_origin, SubjectId};

	use crate::{self as attestation, Event};

	type Block = frame_system::mocking::MockBlock<Test>;

	pub type Balance = u128;
	pub type Signature = MultiSignature;
	pub type AccountPublic = <Signature as Verify>::Signer;
	pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

	pub const UNIT: Balance = 10u128.pow(15);
	pub const MILLI_UNIT: Balance = 10u128.pow(12);
	pub const ATTESTATION_DEPOSIT: Balance = 10 * MILLI_UNIT;

	pub(crate) fn events() -> Vec<Event<Test>> {
		System::events()
			.into_iter()
			.map(|r| r.event)
			.filter_map(|e| {
				if let RuntimeEvent::Attestation(e) = e {
					Some(e)
				} else {
					None
				}
			})
			.collect::<Vec<_>>()
	}

	frame_support::construct_runtime!(
		pub enum Test
		{
			System: frame_system,
			Attestation: attestation,
			Ctype: ctype,
			Balances: pallet_balances,
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
		type Hash = Hash;
		type Hashing = BlakeTwo256;
		type AccountId = AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Block = Block;
		type Nonce = u64;
		type RuntimeEvent = RuntimeEvent;
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
		pub const ExistentialDeposit: Balance = MILLI_UNIT;
		pub const MaxLocks: u32 = 50;
		pub const MaxReserves: u32 = 50;
		pub const MaxFreezes: u32 = 50;
		pub const MaxHolds : u32 = 50;
	}

	impl pallet_balances::Config for Test {
		type FreezeIdentifier = RuntimeFreezeReason;
		type RuntimeHoldReason = RuntimeHoldReason;
		type MaxFreezes = MaxFreezes;
		type MaxHolds = MaxHolds;
		type Balance = Balance;
		type DustRemoval = ();
		type RuntimeEvent = RuntimeEvent;
		type ExistentialDeposit = ExistentialDeposit;
		type AccountStore = System;
		type WeightInfo = ();
		type MaxLocks = MaxLocks;
		type MaxReserves = MaxReserves;
		type ReserveIdentifier = [u8; 8];
	}

	parameter_types! {
		pub const Fee: Balance = 500;
	}

	impl ctype::Config for Test {
		type CtypeCreatorId = SubjectId;
		type EnsureOrigin = mock_origin::EnsureDoubleOrigin<AccountId, Self::CtypeCreatorId>;
		type OriginSuccess = mock_origin::DoubleOrigin<AccountId, Self::CtypeCreatorId>;
		type OverarchingOrigin = EnsureSigned<AccountId>;
		type RuntimeEvent = RuntimeEvent;
		type WeightInfo = ();

		type Currency = Balances;
		type Fee = Fee;
		type FeeCollector = ();
	}

	impl mock_origin::Config for Test {
		type RuntimeOrigin = RuntimeOrigin;
		type AccountId = AccountId;
		type SubjectId = SubjectId;
	}

	parameter_types! {
		pub const MaxDelegatedAttestations: u32 = 1000;
		pub const Deposit: Balance = ATTESTATION_DEPOSIT;
	}

	impl Config for Test {
		type EnsureOrigin = mock_origin::EnsureDoubleOrigin<AccountId, AttesterOf<Self>>;
		type OriginSuccess = mock_origin::DoubleOrigin<AccountId, AttesterOf<Self>>;
		type RuntimeEvent = RuntimeEvent;
		type WeightInfo = ();
		type RuntimeHoldReason = RuntimeHoldReason;
		type Currency = Balances;
		type Deposit = Deposit;
		type MaxDelegatedAttestations = MaxDelegatedAttestations;
		type AttesterId = SubjectId;
		type AuthorizationId = SubjectId;
		type AccessControl = MockAccessControl<Self>;
		type BalanceMigrationManager = ();
	}

	pub(crate) const ACCOUNT_00: AccountId = AccountId::new([1u8; 32]);
	pub(crate) const ACCOUNT_01: AccountId = AccountId::new([2u8; 32]);

	pub(crate) const ALICE_SEED: [u8; 32] = [1u8; 32];
	pub(crate) const BOB_SEED: [u8; 32] = [2u8; 32];
	pub(crate) const CHARLIE_SEED: [u8; 32] = [3u8; 32];

	pub const CLAIM_HASH_SEED_01: u64 = 1u64;
	pub const CLAIM_HASH_SEED_02: u64 = 2u64;

	pub fn ed25519_did_from_seed(seed: &[u8; 32]) -> SubjectId {
		MultiSigner::from(ed25519::Pair::from_seed(seed).public())
			.into_account()
			.into()
	}

	#[derive(Clone, Default)]
	pub struct ExtBuilder {
		/// initial ctypes & owners
		ctypes: Vec<(CtypeHashOf<Test>, CtypeCreatorOf<Test>)>,
		/// endowed accounts with balances
		balances: Vec<(AccountIdOf<Test>, BalanceOf<Test>)>,
		attestations: Vec<(ClaimHashOf<Test>, AttestationDetailsOf<Test>)>,
	}

	impl ExtBuilder {
		#[must_use]
		pub fn with_ctypes(mut self, ctypes: Vec<(CtypeHashOf<Test>, CtypeCreatorOf<Test>)>) -> Self {
			self.ctypes = ctypes;
			self
		}

		#[must_use]
		pub fn with_balances(mut self, balances: Vec<(AccountIdOf<Test>, BalanceOf<Test>)>) -> Self {
			self.balances = balances;
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

				for (claim_hash, details) in self.attestations {
					insert_attestation::<Test>(claim_hash, details);
				}
			});

			ext
		}

		pub fn build_and_execute_with_sanity_tests(self, test: impl FnOnce()) {
			self.build().execute_with(|| {
				test();
				crate::try_state::do_try_state::<Test>().expect("Sanity test for attestation failed.");
			})
		}

		#[cfg(feature = "runtime-benchmarks")]
		pub fn build_with_keystore(self) -> sp_io::TestExternalities {
			let mut ext = self.build();

			let keystore = sp_keystore::testing::MemoryKeystore::new();
			ext.register_extension(sp_keystore::KeystoreExt(std::sync::Arc::new(keystore)));

			ext
		}
	}
}
