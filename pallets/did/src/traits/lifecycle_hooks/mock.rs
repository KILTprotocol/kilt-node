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

use frame_support::{construct_runtime, parameter_types};
use frame_system::{mocking::MockBlock, EnsureSigned};
use kilt_support::mock::MockCurrency;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::{ConstU32, ConstU64, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};

use crate::{
	Config, DeriveDidCallAuthorizationVerificationKeyRelationship, DeriveDidCallKeyRelationshipResult,
	DidVerificationKeyRelationship,
};

construct_runtime!(
	pub enum TestRuntime
	{
		System: frame_system,
		Did: crate,
	}
);

impl frame_system::Config for TestRuntime {
	type AccountData = ();
	type AccountId = AccountId32;
	type BaseCallFilter = ();
	type Block = MockBlock<TestRuntime>;
	type BlockHashCount = ConstU64<1>;
	type BlockLength = ();
	type BlockWeights = ();
	type DbWeight = ();
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaxConsumers = ConstU32<1>;
	type Nonce = u64;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type PalletInfo = PalletInfo;
	type RuntimeEvent = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type RuntimeTask = ();
	type SS58Prefix = ();
	type SystemWeightInfo = ();
	type Version = ();
}

parameter_types! {
	#[derive(TypeInfo, Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub const MaxNewKeyAgreementKeys: u32 = 1;
	#[derive(TypeInfo, Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub const MaxTotalKeyAgreementKeys: u32 = 1;
}

impl DeriveDidCallAuthorizationVerificationKeyRelationship for RuntimeCall {
	fn derive_verification_key_relationship(&self) -> DeriveDidCallKeyRelationshipResult {
		Ok(DidVerificationKeyRelationship::Authentication)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn get_call_for_did_call_benchmark() -> Self {
		Self::System(frame_system::Call::remark {
			remark: b"test".to_vec(),
		})
	}
}

impl Config for TestRuntime {
	type BalanceMigrationManager = ();
	type BaseDeposit = ConstU64<1>;
	type Currency = MockCurrency<u64, RuntimeHoldReason>;
	type DidIdentifier = AccountId32;
	type DidLifecycleHooks = ();
	type EnsureOrigin = EnsureSigned<AccountId32>;
	type Fee = ConstU64<1>;
	type FeeCollector = ();
	type KeyDeposit = ConstU64<1>;
	type MaxBlocksTxValidity = ConstU64<1>;
	type MaxNewKeyAgreementKeys = MaxNewKeyAgreementKeys;
	type MaxNumberOfServicesPerDid = ConstU32<1>;
	type MaxNumberOfTypesPerService = ConstU32<1>;
	type MaxNumberOfUrlsPerService = ConstU32<1>;
	type MaxPublicKeysPerDid = ConstU32<1>;
	type MaxServiceIdLength = ConstU32<1>;
	type MaxServiceTypeLength = ConstU32<1>;
	type MaxServiceUrlLength = ConstU32<1>;
	type MaxTotalKeyAgreementKeys = MaxTotalKeyAgreementKeys;
	type OriginSuccess = AccountId32;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeOrigin = RuntimeOrigin;
	type ServiceEndpointDeposit = ConstU64<1>;
	type WeightInfo = ();
}
