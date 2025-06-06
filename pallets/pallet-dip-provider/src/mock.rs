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

use frame_support::{
	construct_runtime,
	sp_runtime::{
		testing::H256,
		traits::{BlakeTwo256, IdentityLookup},
		AccountId32,
	},
	traits::{ConstU16, ConstU32, ConstU64, Everything},
};

use frame_system::mocking::MockBlock;
use kilt_support::mock::mock_origin::{self as mock_origin, DoubleOrigin, EnsureDoubleOrigin};

use crate::{
	traits::{IdentityCommitmentGenerator, IdentityProvider},
	DefaultIdentityCommitmentGenerator, DefaultIdentityProvider, IdentityCommitmentOf, IdentityCommitmentVersion,
};

construct_runtime!(
	pub struct TestRuntime {
		System: frame_system,
		DipProvider: crate,
		MockOrigin: mock_origin,
	}
);

impl frame_system::Config for TestRuntime {
	type AccountData = ();
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
	type MultiBlockMigrator = ();
	type SingleBlockMigrations = ();
	type PostInherents = ();
	type PostTransactions = ();
	type PreInherents = ();
}

impl crate::Config for TestRuntime {
	type CommitOrigin = DoubleOrigin<Self::AccountId, Self::Identifier>;
	type CommitOriginCheck = EnsureDoubleOrigin<Self::AccountId, Self::Identifier>;
	type Identifier = AccountId32;
	type IdentityCommitmentGenerator = DefaultIdentityCommitmentGenerator<u32>;
	type IdentityProvider = DefaultIdentityProvider<u32>;
	type ProviderHooks = ();
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

impl mock_origin::Config for TestRuntime {
	type AccountId = <Self as frame_system::Config>::AccountId;
	type RuntimeOrigin = RuntimeOrigin;
	type SubjectId = <Self as crate::Config>::Identifier;
}

pub(crate) const ACCOUNT_ID: AccountId32 = AccountId32::new([100u8; 32]);
pub(crate) const DID: AccountId32 = AccountId32::new([200u8; 32]);

pub(crate) fn get_expected_commitment_for(
	subject: &<TestRuntime as crate::Config>::Identifier,
	version: IdentityCommitmentVersion,
) -> IdentityCommitmentOf<TestRuntime> {
	let expected_identity_details =
		<<TestRuntime as crate::Config>::IdentityProvider as IdentityProvider<TestRuntime>>::retrieve(subject)
			.expect("Should not fail to generate identity details for the provided DID.");
	<<TestRuntime as crate::Config>::IdentityCommitmentGenerator as IdentityCommitmentGenerator<TestRuntime>>::generate_commitment(
				subject,
				&expected_identity_details,
				version,
			)
			.expect("Should not fail to generate identity commitment for the provided DID.")
}

#[derive(Default)]
pub(crate) struct ExtBuilder(
	Vec<(
		AccountId32,
		IdentityCommitmentVersion,
		IdentityCommitmentOf<TestRuntime>,
	)>,
);

impl ExtBuilder {
	pub(crate) fn with_commitments(
		mut self,
		commitments: Vec<(
			AccountId32,
			IdentityCommitmentVersion,
			IdentityCommitmentOf<TestRuntime>,
		)>,
	) -> Self {
		self.0 = commitments;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut ext = sp_io::TestExternalities::default();

		ext.execute_with(|| {
			for (subject, commitment_version, commitment) in self.0 {
				crate::pallet::IdentityCommitments::<TestRuntime>::insert(subject, commitment_version, commitment);
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
