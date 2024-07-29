// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

use frame_support::{
	construct_runtime,
	sp_runtime::{
		testing::H256,
		traits::{BlakeTwo256, IdentityLookup},
		AccountId32,
	},
	traits::{ConstU16, ConstU32, ConstU64, Contains, Currency, Everything},
};
use frame_system::{mocking::MockBlock, EnsureSigned};

use crate::{traits::IdentityProofVerifier, Config, DipOrigin, EnsureDipOrigin, IdentityEntries, RuntimeCallOf};

// This mock is used both for benchmarks and unit tests.
// For benchmarks, the `system::remark` call must be allowed to be dispatched,
// while for the unit tests we use the `pallet_did_lookup` as an example pallet
// consuming the generated DIP origin.
construct_runtime!(
	pub struct TestRuntime {
		System: frame_system,
		Balances: pallet_balances,
		DidLookup: pallet_did_lookup,
		DipConsumer: crate,
	}
);

impl frame_system::Config for TestRuntime {
	type AccountData = pallet_balances::AccountData<u64>;
	type AccountId = AccountId32;
	type BaseCallFilter = Everything;
	type Block = MockBlock<TestRuntime>;
	type BlockHashCount = ConstU64<256>;
	type BlockLength = ();
	type BlockWeights = ();
	type DbWeight = ();
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaxConsumers = ConstU32<16>;
	type Nonce = u64;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeTask = ();
	type SS58Prefix = ConstU16<1>;
	type SystemWeightInfo = ();
	type Version = ();
}

impl pallet_balances::Config for TestRuntime {
	type AccountStore = System;
	type Balance = u64;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type FreezeIdentifier = [u8; 8];
	type MaxFreezes = ConstU32<10>;
	type MaxLocks = ConstU32<10>;
	type MaxReserves = ConstU32<10>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type WeightInfo = ();
}

impl pallet_did_lookup::Config for TestRuntime {
	type BalanceMigrationManager = ();
	type Currency = Balances;
	type Deposit = ConstU64<1>;
	type DidIdentifier = AccountId32;
	type EnsureOrigin = EnsureDipOrigin<AccountId32, AccountId32, ()>;
	type OriginSuccess = DipOrigin<AccountId32, AccountId32, ()>;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type WeightInfo = ();
}

pub struct OnlySystemRemarksWithoutEventsAndDidLookupCalls;

impl Contains<RuntimeCall> for OnlySystemRemarksWithoutEventsAndDidLookupCalls {
	fn contains(t: &RuntimeCall) -> bool {
		matches!(
			t,
			// Required by the benchmarking logic
			RuntimeCall::System(frame_system::Call::remark { .. }) |
			// Used in these tests
			RuntimeCall::DidLookup { .. }
		)
	}
}

// Returns success if `Proof` is `true`, and bumps the identity details by one,
// or instantiates them to `Default` if they're `None`.
pub struct BooleanProofVerifier;
impl IdentityProofVerifier<TestRuntime> for BooleanProofVerifier {
	type Error = u16;
	type Proof = bool;
	type VerificationResult = ();

	fn verify_proof_for_call_against_details(
		_call: &RuntimeCallOf<TestRuntime>,
		_subject: &<TestRuntime as Config>::Identifier,
		_submitter: &<TestRuntime as frame_system::Config>::AccountId,
		identity_details: &mut Option<<TestRuntime as Config>::LocalIdentityInfo>,
		proof: Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		if proof {
			*identity_details = identity_details.map(|d| Some(d + 1)).unwrap_or(Some(u128::default()));
			Ok(())
		} else {
			Err(1)
		}
	}
}

#[cfg(feature = "runtime-benchmarks")]
impl kilt_support::traits::GetWorstCase for BooleanProofVerifier {
	type Output = crate::benchmarking::WorstCaseOf<TestRuntime>;

	fn worst_case(_context: ()) -> Self::Output {
		crate::benchmarking::WorstCaseOf {
			call: frame_system::Call::remark {
				remark: b"Hello!".to_vec(),
			}
			.into(),
			proof: true,
			subject: AccountId32::new([100; 32]),
			submitter: AccountId32::new([200; 32]),
		}
	}
}

impl crate::Config for TestRuntime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type ProofVerifier = BooleanProofVerifier;
	type LocalIdentityInfo = u128;
	type Identifier = AccountId32;
	type DispatchOriginCheck = EnsureSigned<Self::Identifier>;
	type DipCallOriginFilter = OnlySystemRemarksWithoutEventsAndDidLookupCalls;
	type WeightInfo = ();
}

pub(crate) const SUBMITTER: AccountId32 = AccountId32::new([100u8; 32]);
pub(crate) const SUBJECT: AccountId32 = AccountId32::new([200u8; 32]);

#[derive(Default)]
pub(crate) struct ExtBuilder(Vec<(AccountId32, u64)>, Vec<(AccountId32, u128)>);

impl ExtBuilder {
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId32, u64)>) -> Self {
		self.0 = balances;
		self
	}
	pub(crate) fn with_identity_details(mut self, details: Vec<(AccountId32, u128)>) -> Self {
		self.1 = details;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut ext = sp_io::TestExternalities::default();

		ext.execute_with(|| {
			for (account_id, balance) in self.0 {
				Balances::make_free_balance_be(&account_id, balance);
			}
			for (subject, details) in self.1 {
				IdentityEntries::<TestRuntime>::insert(subject, details)
			}
		});

		ext
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub(crate) fn build_with_keystore(self) -> sp_io::TestExternalities {
		let mut ext = self.build();
		let keystore = sp_keystore::testing::MemoryKeystore::new();
		ext.register_extension(sp_keystore::KeystoreExt(sp_std::sync::Arc::new(keystore)));
		ext
	}
}
