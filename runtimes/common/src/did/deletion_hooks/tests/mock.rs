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

use did::{did_details::DidVerificationKey, traits::deletion::RequireBoth, DidVerificationKeyRelationship};
use frame_support::{construct_runtime, parameter_types};
use frame_system::{mocking::MockBlock, EnsureRoot, EnsureSigned, RawOrigin};
use kilt_support::test_utils::MockCurrency;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::{ConstBool, ConstU32, ConstU64, H256};
use sp_io::TestExternalities;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};

use crate::{EnsureNoLinkedAccountDeletionHook, EnsureNoLinkedWeb3NameDeletionHook};

construct_runtime!(
	pub enum TestRuntime
	{
		System: frame_system,
		Did: did,
		Web3Names: pallet_web3_names,
		LinkedAccounts: pallet_did_lookup,
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

impl did::DeriveDidCallAuthorizationVerificationKeyRelationship for RuntimeCall {
	fn derive_verification_key_relationship(&self) -> did::DeriveDidCallKeyRelationshipResult {
		Ok(DidVerificationKeyRelationship::Authentication)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn get_call_for_did_call_benchmark() -> Self {
		Self::System(frame_system::Call::remark {
			remark: b"test".to_vec(),
		})
	}
}

pub(super) const DID: AccountId32 = AccountId32::new([100u8; 32]);

pub struct DidLifecycleHooks;

impl did::traits::DidLifecycleHooks<TestRuntime> for DidLifecycleHooks {
	type DeletionHook =
		RequireBoth<EnsureNoLinkedWeb3NameDeletionHook<1, 1, ()>, EnsureNoLinkedAccountDeletionHook<1, 1, ()>>;
}

impl did::Config for TestRuntime {
	type BalanceMigrationManager = ();
	type BaseDeposit = ConstU64<0>;
	type Currency = MockCurrency<u64, RuntimeHoldReason>;
	type DidIdentifier = AccountId32;
	type DidLifecycleHooks = DidLifecycleHooks;
	type EnsureOrigin = EnsureSigned<AccountId32>;
	type Fee = ConstU64<0>;
	type FeeCollector = ();
	type KeyDeposit = ConstU64<0>;
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
	type ServiceEndpointDeposit = ConstU64<0>;
	type WeightInfo = ();
}

impl pallet_did_lookup::Config for TestRuntime {
	type AssociateOrigin = Self::EnsureOrigin;
	type BalanceMigrationManager = ();
	type Currency = MockCurrency<u64, RuntimeHoldReason>;
	type Deposit = ConstU64<0>;
	type DidIdentifier = AccountId32;
	type EnsureOrigin = EnsureSigned<AccountId32>;
	type OriginSuccess = AccountId32;
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeEvent = ();
	type UniqueLinkingEnabled = ConstBool<false>;
	type WeightInfo = ();
}

pub type Web3Name = crate::Web3Name<1, 2>;
impl pallet_web3_names::Config for TestRuntime {
	type BalanceMigrationManager = ();
	type BanOrigin = EnsureRoot<AccountId32>;
	type ClaimOrigin = Self::OwnerOrigin;
	type Currency = MockCurrency<u64, RuntimeHoldReason>;
	type Deposit = ConstU64<0>;
	type OriginSuccess = AccountId32;
	type MaxNameLength = ConstU32<1>;
	type MinNameLength = ConstU32<1>;
	type OwnerOrigin = EnsureSigned<AccountId32>;
	type RuntimeEvent = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type Web3Name = Web3Name;
	type Web3NameOwner = AccountId32;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

#[derive(Default)]
pub(super) struct ExtBuilder(
	Vec<(AccountId32, Option<Web3Name>, bool)>,
	Vec<(AccountId32, Option<Web3Name>, bool)>,
);

impl ExtBuilder {
	pub(super) fn with_dids(mut self, did_links: Vec<(AccountId32, Option<Web3Name>, bool)>) -> Self {
		self.0 = did_links;
		self
	}

	pub(super) fn with_dangling_dids(mut self, dangling_dids: Vec<(AccountId32, Option<Web3Name>, bool)>) -> Self {
		self.1 = dangling_dids;
		self
	}

	pub(super) fn build(self) -> TestExternalities {
		let _ = env_logger::try_init();
		let mut ext = TestExternalities::default();

		ext.execute_with(|| {
			for (did, maybe_web3_name, should_link_account) in self.0 {
				// Store DID.
				Did::create_from_account(
					RawOrigin::Signed(did.clone()).into(),
					DidVerificationKey::Account(did.clone()),
				)
				.expect("Failed to create DID.");

				// If specified, link web3name.
				if let Some(web3_name) = maybe_web3_name {
					Web3Names::claim(
						RawOrigin::Signed(did.clone()).into(),
						Vec::<u8>::from(web3_name.clone()).try_into().unwrap(),
					)
					.expect("Failed to link web3name.");
				}

				// If specified, link account.
				if should_link_account {
					LinkedAccounts::associate_sender(RawOrigin::Signed(did.clone()).into())
						.expect("Failed to link account.");
				}
			}

			for (did, maybe_web3_name, should_link_account) in self.1 {
				// Cannot write the same DID as both linked and dangling.
				assert!(!did::Did::<TestRuntime>::contains_key(&did));
				if maybe_web3_name.is_none() && !should_link_account {
					panic!("One of web3name or linked account must be set.");
				}
				if let Some(web3_name) = maybe_web3_name {
					Web3Names::claim(
						RawOrigin::Signed(did.clone()).into(),
						Vec::<u8>::from(web3_name.clone()).try_into().unwrap(),
					)
					.expect("Failed to set dangling web3name.");
				}
				if should_link_account {
					LinkedAccounts::associate_sender(RawOrigin::Signed(did.clone()).into())
						.expect("Failed to set dangling account.");
				}
			}
		});

		ext
	}
}
