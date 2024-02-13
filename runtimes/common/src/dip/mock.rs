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

use did::DeriveDidCallAuthorizationVerificationKeyRelationship;
use frame_support::{construct_runtime, traits::Everything};
use frame_system::{mocking::MockBlock, EnsureRoot, EnsureSigned};
use pallet_web3_names::web3_name::AsciiWeb3Name;
use sp_core::{ConstU128, ConstU16, ConstU32, ConstU64};
use sp_runtime::traits::IdentityLookup;

use crate::{
	constants::{
		did::{
			MaxNewKeyAgreementKeys, MaxNumberOfServicesPerDid, MaxNumberOfTypesPerService, MaxNumberOfUrlsPerService,
			MaxPublicKeysPerDid, MaxServiceIdLength, MaxServiceTypeLength, MaxServiceUrlLength,
			MaxTotalKeyAgreementKeys,
		},
		dip_provider::MAX_LINKED_ACCOUNTS,
		web3_names::{MaxNameLength, MinNameLength},
		KILT,
	},
	dip::{did::LinkedDidInfoProvider, merkle::DidMerkleRootGenerator},
	AccountId, Balance, BlockHashCount, BlockLength, BlockWeights, DidIdentifier, Hash, Hasher, Nonce,
};

construct_runtime!(
	pub struct TestRuntime {
		System: frame_system,
		Balances: pallet_balances,
		Did: did,
		Web3Names: pallet_web3_names,
		DidLookup: pallet_did_lookup,
		DipProvider: pallet_dip_provider,
	}
);

impl frame_system::Config for TestRuntime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type BaseCallFilter = Everything;
	type Block = MockBlock<TestRuntime>;
	type BlockHashCount = BlockHashCount;
	type BlockLength = BlockLength;
	type BlockWeights = BlockWeights;
	type DbWeight = ();
	type Hash = Hash;
	type Hashing = Hasher;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaxConsumers = ConstU32<16>;
	type Nonce = Nonce;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SS58Prefix = ConstU16<1>;
	type SystemWeightInfo = ();
	type Version = ();
}

impl pallet_balances::Config for TestRuntime {
	type AccountStore = System;
	type Balance = Balance;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<KILT>;
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxFreezes = ConstU32<50>;
	type MaxHolds = ConstU32<50>;
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type WeightInfo = ();
}

impl DeriveDidCallAuthorizationVerificationKeyRelationship for RuntimeCall {
	fn derive_verification_key_relationship(&self) -> did::DeriveDidCallKeyRelationshipResult {
		Ok(did::DidVerificationKeyRelationship::Authentication)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn get_call_for_did_call_benchmark() -> Self {
		RuntimeCall::System(frame_system::Call::remark { remark: sp_std::vec![] })
	}
}

impl did::Config for TestRuntime {
	type BalanceMigrationManager = ();
	type BaseDeposit = ConstU128<KILT>;
	type Currency = Balances;
	type DidIdentifier = DidIdentifier;
	type EnsureOrigin = EnsureSigned<AccountId>;
	type Fee = ConstU128<KILT>;
	type FeeCollector = ();
	type KeyDeposit = ConstU128<KILT>;
	type MaxBlocksTxValidity = ConstU64<10>;
	type MaxNewKeyAgreementKeys = MaxNewKeyAgreementKeys;
	type MaxNumberOfServicesPerDid = MaxNumberOfServicesPerDid;
	type MaxNumberOfTypesPerService = MaxNumberOfTypesPerService;
	type MaxNumberOfUrlsPerService = MaxNumberOfUrlsPerService;
	type MaxPublicKeysPerDid = MaxPublicKeysPerDid;
	type MaxServiceIdLength = MaxServiceIdLength;
	type MaxServiceTypeLength = MaxServiceTypeLength;
	type MaxServiceUrlLength = MaxServiceUrlLength;
	type MaxTotalKeyAgreementKeys = MaxTotalKeyAgreementKeys;
	type OriginSuccess = AccountId;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeOrigin = RuntimeOrigin;
	type ServiceEndpointDeposit = ConstU128<KILT>;
	type WeightInfo = ();
}

impl pallet_web3_names::Config for TestRuntime {
	type BalanceMigrationManager = ();
	type BanOrigin = EnsureRoot<AccountId>;
	type Currency = Balances;
	type Deposit = ConstU128<KILT>;
	type MaxNameLength = MaxNameLength;
	type MinNameLength = MinNameLength;
	type OriginSuccess = AccountId;
	type OwnerOrigin = EnsureSigned<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type Web3Name = AsciiWeb3Name<Self>;
	type Web3NameOwner = DidIdentifier;
	type WeightInfo = ();
}

impl pallet_did_lookup::Config for TestRuntime {
	type BalanceMigrationManager = ();
	type Currency = Balances;
	type Deposit = ConstU128<KILT>;
	type DidIdentifier = DidIdentifier;
	type EnsureOrigin = EnsureSigned<AccountId>;
	type OriginSuccess = AccountId;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type WeightInfo = ();
}

impl pallet_dip_provider::Config for TestRuntime {
	type CommitOrigin = AccountId;
	type CommitOriginCheck = EnsureSigned<AccountId>;
	type Identifier = DidIdentifier;
	type IdentityCommitmentGenerator = DidMerkleRootGenerator<Self>;
	type IdentityProvider = LinkedDidInfoProvider<MAX_LINKED_ACCOUNTS>;
	type ProviderHooks = ();
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}
