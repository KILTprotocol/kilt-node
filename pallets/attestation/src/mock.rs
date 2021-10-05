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

use crate::{AccountIdOf, AttestationDetails, AttesterOf, BalanceOf, ClaimHashOf, Config};
use ctype::CtypeHashOf;
use delegation::DelegationNodeIdOf;
use frame_support::traits::Get;
use kilt_support::deposit::Deposit;
use sp_core::H256;

#[cfg(test)]
pub use crate::mock::runtime::*;

pub struct AttestationCreationDetails<T: Config> {
	pub claim_hash: ClaimHashOf<T>,
	pub ctype_hash: CtypeHashOf<T>,
	pub delegation_id: Option<DelegationNodeIdOf<T>>,
}

pub fn generate_base_attestation_creation_details<T: Config>(
	claim_hash: ClaimHashOf<T>,
	attestation: AttestationDetails<T>,
) -> AttestationCreationDetails<T> {
	AttestationCreationDetails {
		claim_hash,
		ctype_hash: attestation.ctype_hash,
		delegation_id: attestation.delegation_id,
	}
}

pub struct AttestationRevocationDetails<T: Config> {
	pub claim_hash: ClaimHashOf<T>,
	pub max_parent_checks: u32,
}

pub fn generate_base_attestation_revocation_details<T: Config>(
	claim_hash: ClaimHashOf<T>,
) -> AttestationRevocationDetails<T> {
	AttestationRevocationDetails {
		claim_hash,
		max_parent_checks: 0u32,
	}
}

pub fn generate_base_attestation<T>(attester: AttesterOf<T>, payer: AccountIdOf<T>) -> AttestationDetails<T>
where
	T: Config,
	T::Hash: From<H256>,
{
	AttestationDetails {
		attester,
		delegation_id: None,
		ctype_hash: ctype::mock::get_ctype_hash::<T>(true),
		revoked: false,
		deposit: Deposit::<AccountIdOf<T>, BalanceOf<T>> {
			owner: payer,
			amount: <T as Config>::Deposit::get(),
		},
	}
}

/// Mocks that are only used internally
#[cfg(test)]
pub(crate) mod runtime {
	use std::sync::Arc;

	use crate::Pallet;

	use super::*;

	use delegation::{DelegationHierarchyDetails, DelegationNode, DelegatorIdOf};
	use frame_support::{ensure, parameter_types, weights::constants::RocksDbWeight};
	use frame_system::EnsureSigned;
	use kilt_primitives::constants::{attestation::ATTESTATION_DEPOSIT, delegation::DELEGATION_DEPOSIT, MILLI_KILT};
	use sp_core::{ed25519, sr25519, Pair};
	use sp_keystore::{testing::KeyStore, KeystoreExt};
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
		MultiSignature, MultiSigner,
	};

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
	type Block = frame_system::mocking::MockBlock<Test>;

	type TestCtypeOwner = kilt_primitives::AccountId;
	type TestCtypeHash = kilt_primitives::Hash;
	type TestDelegationNodeId = kilt_primitives::Hash;
	type TestDelegatorId = kilt_primitives::AccountId;
	type TestClaimHash = kilt_primitives::Hash;
	type TestAttester = TestDelegatorId;
	type TestBalance = kilt_primitives::Balance;

	frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			Attestation: crate::{Pallet, Call, Storage, Event<T>},
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

	impl ctype::Config for Test {
		type CtypeCreatorId = TestCtypeOwner;
		type EnsureOrigin = EnsureSigned<TestCtypeOwner>;
		type OriginSuccess = TestCtypeOwner;
		type Event = ();
		type WeightInfo = ();
	}

	parameter_types! {
		pub const MaxSignatureByteLength: u16 = 64;
		pub const MaxParentChecks: u32 = 5;
		pub const MaxRevocations: u32 = 5;
		pub const MaxRemovals: u32 = 5;
		#[derive(Clone)]
		pub const MaxChildren: u32 = 1000;
		pub const DelegationDeposit: TestBalance = DELEGATION_DEPOSIT;
	}

	impl delegation::Config for Test {
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
		type WeightInfo = ();

		type Currency = Balances;
		type Deposit = DelegationDeposit;
	}

	parameter_types! {
		pub const MaxDelegatedAttestations: u32 = 1000;
		pub const Deposit: TestBalance = ATTESTATION_DEPOSIT;
	}

	impl Config for Test {
		type EnsureOrigin = EnsureSigned<TestAttester>;
		type OriginSuccess = TestAttester;
		type Event = ();
		type WeightInfo = ();

		type Currency = Balances;
		type Deposit = Deposit;
		type MaxDelegatedAttestations = MaxDelegatedAttestations;
	}

	pub struct DelegateSignatureVerifier;
	impl delegation::VerifyDelegateSignature for DelegateSignatureVerifier {
		type DelegateId = TestDelegatorId;
		type Payload = Vec<u8>;
		type Signature = MultiSignature;

		// No need to retrieve delegate details as it is simply an AccountId.
		fn verify(
			delegate: &Self::DelegateId,
			payload: &Self::Payload,
			signature: &Self::Signature,
		) -> delegation::SignatureVerificationResult {
			ensure!(
				signature.verify(&payload[..], delegate),
				delegation::SignatureVerificationError::SignatureInvalid
			);

			Ok(())
		}
	}

	const ALICE_SEED: [u8; 32] = [0u8; 32];
	const BOB_SEED: [u8; 32] = [1u8; 32];

	const DEFAULT_CLAIM_HASH_SEED: u64 = 1u64;
	const ALTERNATIVE_CLAIM_HASH_SEED: u64 = 2u64;

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

	pub fn get_claim_hash(default: bool) -> TestClaimHash {
		if default {
			TestClaimHash::from_low_u64_be(DEFAULT_CLAIM_HASH_SEED)
		} else {
			TestClaimHash::from_low_u64_be(ALTERNATIVE_CLAIM_HASH_SEED)
		}
	}

	#[derive(Clone, Default)]
	pub struct ExtBuilder {
		delegation_hierarchies: Vec<(
			TestDelegationNodeId,
			DelegationHierarchyDetails<Test>,
			DelegatorIdOf<Test>,
		)>,
		delegations: Vec<(TestDelegationNodeId, DelegationNode<Test>)>,

		/// initial ctypes & owners
		ctypes: Vec<(TestCtypeHash, AccountIdOf<Test>)>,
		/// endowed accounts with balances
		balances: Vec<(AccountIdOf<Test>, BalanceOf<Test>)>,
		attestations: Vec<(TestClaimHash, AttestationDetails<Test>)>,
	}

	impl ExtBuilder {
		pub fn with_delegation_hierarchies(
			mut self,
			delegation_hierarchies: Vec<(
				TestDelegationNodeId,
				DelegationHierarchyDetails<Test>,
				DelegatorIdOf<Test>,
			)>,
		) -> Self {
			self.delegation_hierarchies = delegation_hierarchies;
			self
		}

		pub fn with_delegations(mut self, delegations: Vec<(TestDelegationNodeId, DelegationNode<Test>)>) -> Self {
			self.delegations = delegations;
			self
		}

		pub fn with_ctypes(mut self, ctypes: Vec<(TestCtypeHash, TestCtypeOwner)>) -> Self {
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

				delegation::mock::initialize_pallet(self.delegations, self.delegation_hierarchies);

				for (claim_hash, details) in self.attestations {
					Pallet::<Test>::reserve_deposit(details.deposit.owner.clone(), details.deposit.amount)
						.expect("Should have balance");

					crate::Attestations::<Test>::insert(&claim_hash, details.clone());
					if let Some(delegation_id) = details.delegation_id.as_ref() {
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
