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

//! Helpers functions for tests.
//!
//! Functions are split into internal functions and public functions.
//! Public functions are generic for any runtime, so that they can be reused in
//! other tests. Internal functions/structs can only be used in attestation
//! tests.

use codec::{Decode, Encode};
use frame_support::{dispatch::Weight, traits::Get};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::DispatchError;

use ctype::CtypeHashOf;
use kilt_support::deposit::Deposit;

use crate::{
	pallet::AuthorizationIdOf, AccountIdOf, AttestationAccessControl, AttestationDetails, AttesterOf, BalanceOf,
	ClaimHashOf, Config,
};

#[cfg(test)]
pub use crate::mock::runtime::*;

pub struct AttestationCreationDetails<T: Config> {
	pub claim_hash: ClaimHashOf<T>,
	pub ctype_hash: CtypeHashOf<T>,
	pub authorization_id: Option<AuthorizationIdOf<T>>,
}

pub fn generate_base_attestation_creation_details<T: Config>(
	claim_hash: ClaimHashOf<T>,
	attestation: AttestationDetails<T>,
) -> AttestationCreationDetails<T> {
	AttestationCreationDetails {
		claim_hash,
		ctype_hash: attestation.ctype_hash,
		authorization_id: attestation.authorization_id,
	}
}

pub fn generate_base_attestation<T>(attester: AttesterOf<T>, payer: AccountIdOf<T>) -> AttestationDetails<T>
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
			amount: <T as Config>::Deposit::get(),
		},
	}
}

#[derive(Clone, Debug, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[scale_info(skip_type_params(T))]
pub struct MockAccessControl<T: Config>(pub T::AttesterId);
impl<T: Config> AttestationAccessControl<T> for MockAccessControl<T>
where
	T: Config<AuthorizationId = <T as Config>::AttesterId>,
{
	fn can_attest(&self, who: &T::AttesterId) -> Result<Weight, DispatchError> {
		if who == &self.0 {
			Ok(0)
		} else {
			Err(DispatchError::Other("Unauthorized"))
		}
	}

	fn can_revoke(&self, who: &T::AttesterId, attestation: &AttestationDetails<T>) -> Result<Weight, DispatchError> {
		if attestation.authorization_id.as_ref().map_or(false, |auth| auth == who) {
			Ok(0)
		} else {
			Err(DispatchError::Other("Unauthorized"))
		}
	}

	fn can_remove(&self, who: &T::AttesterId, attestation: &AttestationDetails<T>) -> Result<Weight, DispatchError> {
		if attestation.authorization_id.as_ref().map_or(false, |auth| auth == who) {
			Ok(0)
		} else {
			Err(DispatchError::Other("Unauthorized"))
		}
	}

	fn authorization_id(&self) -> T::AuthorizationId {
		self.0.clone()
	}
}

/// Mocks that are only used internally
#[cfg(test)]
pub(crate) mod runtime {
	use ctype::CtypeCreatorOf;
	use frame_support::{parameter_types, weights::constants::RocksDbWeight};
	use sp_core::{ed25519, sr25519, Pair};
	use sp_keystore::{testing::KeyStore, KeystoreExt};
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
		MultiSigner,
	};
	use std::sync::Arc;

	use kilt_support::mock::{mock_origin, SubjectId};
	use runtime_common::constants::{attestation::ATTESTATION_DEPOSIT, MILLI_KILT};

	use super::*;
	use crate::Pallet;

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
	type Block = frame_system::mocking::MockBlock<Test>;

	type TestCtypeHash = runtime_common::Hash;
	type TestClaimHash = runtime_common::Hash;
	type TestBalance = runtime_common::Balance;

	frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			Attestation: crate::{Pallet, Call, Storage, Event<T>},
			Ctype: ctype::{Pallet, Call, Storage, Event<T>},
			Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
			MockOrigin: mock_origin::{Pallet, Origin<T>},
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
		type Hash = runtime_common::Hash;
		type Hashing = BlakeTwo256;
		type AccountId = <<runtime_common::Signature as Verify>::Signer as IdentifyAccount>::AccountId;
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
		pub const ExistentialDeposit: TestBalance = MILLI_KILT;
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
		type CtypeCreatorId = SubjectId;
		type EnsureOrigin = mock_origin::EnsureDoubleOrigin<runtime_common::AccountId, Self::CtypeCreatorId>;
		type OriginSuccess = mock_origin::DoubleOrigin<runtime_common::AccountId, Self::CtypeCreatorId>;
		type Event = ();
		type WeightInfo = ();

		type Currency = Balances;
		type Fee = Fee;
		type FeeCollector = ();
	}

	impl mock_origin::Config for Test {
		type Origin = Origin;
		type AccountId = runtime_common::AccountId;
		type SubjectId = SubjectId;
	}

	parameter_types! {
		pub const MaxDelegatedAttestations: u32 = 1000;
		pub const Deposit: TestBalance = ATTESTATION_DEPOSIT;
	}

	impl Config for Test {
		type EnsureOrigin = mock_origin::EnsureDoubleOrigin<runtime_common::AccountId, AttesterOf<Self>>;
		type OriginSuccess = mock_origin::DoubleOrigin<runtime_common::AccountId, AttesterOf<Self>>;
		type Event = ();
		type WeightInfo = ();

		type Currency = Balances;
		type Deposit = Deposit;
		type MaxDelegatedAttestations = MaxDelegatedAttestations;
		type AttesterId = SubjectId;
		type AuthorizationId = SubjectId;
		type AccessControl = MockAccessControl<Self>;
	}

	pub(crate) const ACCOUNT_00: runtime_common::AccountId = runtime_common::AccountId::new([1u8; 32]);
	pub(crate) const ACCOUNT_01: runtime_common::AccountId = runtime_common::AccountId::new([2u8; 32]);

	pub(crate) const ALICE_SEED: [u8; 32] = [1u8; 32];
	pub(crate) const BOB_SEED: [u8; 32] = [2u8; 32];

	pub const CLAIM_HASH_SEED_01: u64 = 1u64;
	pub const CLAIM_HASH_SEED_02: u64 = 2u64;

	pub fn ed25519_did_from_seed(seed: &[u8; 32]) -> SubjectId {
		MultiSigner::from(ed25519::Pair::from_seed(seed).public())
			.into_account()
			.into()
	}

	pub fn sr25519_did_from_seed(seed: &[u8; 32]) -> SubjectId {
		MultiSigner::from(sr25519::Pair::from_seed(seed).public())
			.into_account()
			.into()
	}

	pub fn claim_hash_from_seed(seed: u64) -> TestClaimHash {
		TestClaimHash::from_low_u64_be(seed)
	}

	#[derive(Clone, Default)]
	pub struct ExtBuilder {
		/// initial ctypes & owners
		ctypes: Vec<(TestCtypeHash, CtypeCreatorOf<Test>)>,
		/// endowed accounts with balances
		balances: Vec<(AccountIdOf<Test>, BalanceOf<Test>)>,
		attestations: Vec<(TestClaimHash, AttestationDetails<Test>)>,
	}

	impl ExtBuilder {
		pub fn with_ctypes(mut self, ctypes: Vec<(TestCtypeHash, CtypeCreatorOf<Test>)>) -> Self {
			self.ctypes = ctypes;
			self
		}

		pub fn with_balances(mut self, balances: Vec<(AccountIdOf<Test>, BalanceOf<Test>)>) -> Self {
			self.balances = balances;
			self
		}

		pub fn with_attestations(mut self, attestations: Vec<(TestClaimHash, AttestationDetails<Test>)>) -> Self {
			self.attestations = attestations;
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
				for ctype in self.ctypes {
					ctype::Ctypes::<Test>::insert(ctype.0, ctype.1.clone());
				}

				for (claim_hash, details) in self.attestations {
					Pallet::<Test>::reserve_deposit(details.deposit.owner.clone(), details.deposit.amount)
						.expect("Should have balance");

					crate::Attestations::<Test>::insert(&claim_hash, details.clone());
					if let Some(delegation_id) = details.authorization_id.as_ref() {
						crate::DelegatedAttestations::<Test>::try_mutate(delegation_id, |attestations| {
							let attestations = attestations.get_or_insert_with(Default::default);
							attestations.try_push(claim_hash)
						})
						.expect("Couldn't initialise delegated attestation");
					}
				}
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
}
