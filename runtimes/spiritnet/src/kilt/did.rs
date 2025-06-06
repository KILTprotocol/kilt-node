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

use did::{
	DeriveDidCallAuthorizationVerificationKeyRelationship, DeriveDidCallKeyRelationshipResult, DidRawOrigin,
	DidVerificationKeyRelationship, EnsureDidOrigin, RelationshipDeriveError,
};
use frame_system::EnsureRoot;
use runtime_common::{constants, AccountId, DidIdentifier, SendDustAndFeesToTreasury};
use sp_core::ConstBool;

use crate::{
	weights::{self},
	Balances, Migration, Runtime, RuntimeCall, RuntimeEvent, RuntimeHoldReason, RuntimeOrigin,
};

impl DeriveDidCallAuthorizationVerificationKeyRelationship for RuntimeCall {
	fn derive_verification_key_relationship(&self) -> DeriveDidCallKeyRelationshipResult {
		/// ensure that all calls have the same VerificationKeyRelationship
		fn single_key_relationship(calls: &[RuntimeCall]) -> DeriveDidCallKeyRelationshipResult {
			let init = calls
				.first()
				.ok_or(RelationshipDeriveError::InvalidCallParameter)?
				.derive_verification_key_relationship()?;
			calls
				.iter()
				.skip(1)
				.map(RuntimeCall::derive_verification_key_relationship)
				.try_fold(init, |acc, next| {
					if next.is_err() {
						next
					} else if Ok(acc) == next {
						Ok(acc)
					} else {
						Err(RelationshipDeriveError::InvalidCallParameter)
					}
				})
		}
		match self {
			RuntimeCall::Attestation { .. } => Ok(DidVerificationKeyRelationship::AssertionMethod),
			RuntimeCall::Ctype { .. } => Ok(DidVerificationKeyRelationship::AssertionMethod),
			RuntimeCall::Delegation { .. } => Ok(DidVerificationKeyRelationship::CapabilityDelegation),
			RuntimeCall::DipProvider { .. } => Ok(DidVerificationKeyRelationship::Authentication),
			// DID creation is not allowed through the DID proxy.
			RuntimeCall::Did(did::Call::create { .. }) => Err(RelationshipDeriveError::NotCallableByDid),
			RuntimeCall::Did { .. } => Ok(DidVerificationKeyRelationship::Authentication),
			RuntimeCall::Web3Names { .. } => Ok(DidVerificationKeyRelationship::Authentication),
			RuntimeCall::PublicCredentials { .. } => Ok(DidVerificationKeyRelationship::AssertionMethod),
			RuntimeCall::DidLookup { .. } => Ok(DidVerificationKeyRelationship::Authentication),
			RuntimeCall::Utility(pallet_utility::Call::batch { calls }) => single_key_relationship(&calls[..]),
			RuntimeCall::Utility(pallet_utility::Call::batch_all { calls }) => single_key_relationship(&calls[..]),
			RuntimeCall::Utility(pallet_utility::Call::force_batch { calls }) => single_key_relationship(&calls[..]),
			#[cfg(not(feature = "runtime-benchmarks"))]
			_ => Err(RelationshipDeriveError::NotCallableByDid),
			// By default, returns the authentication key
			#[cfg(feature = "runtime-benchmarks")]
			_ => Ok(DidVerificationKeyRelationship::Authentication),
		}
	}

	// Always return a System::remark() extrinsic call
	#[cfg(feature = "runtime-benchmarks")]
	fn get_call_for_did_call_benchmark() -> Self {
		RuntimeCall::System(frame_system::Call::remark { remark: sp_std::vec![] })
	}
}

pub struct DidLifecycleHooks;

impl did::traits::DidLifecycleHooks<Runtime> for DidLifecycleHooks {
	type DeletionHook = EnsureNoNamesAndNoLinkedAccountsOnDidDeletion;
}

pub struct EnsureNoNamesAndNoLinkedAccountsOnDidDeletion;

impl did::traits::DidDeletionHook<Runtime> for EnsureNoNamesAndNoLinkedAccountsOnDidDeletion {
	fn can_delete(did: &did::DidIdentifierOf<Runtime>) -> bool {
		// 1. Check if there's a linked Web3name
		if pallet_web3_names::Names::<Runtime>::contains_key(did) {
			return false;
		}

		// 2. Check if there's a Web3name linked account
		if pallet_did_lookup::ConnectedAccounts::<Runtime>::iter_key_prefix(did)
			.next()
			.is_some()
		{
			return false;
		}

		true
	}
}

impl did::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeOrigin = RuntimeOrigin;
	type Currency = Balances;
	type DidIdentifier = DidIdentifier;
	type KeyDeposit = constants::did::KeyDeposit;
	type ServiceEndpointDeposit = constants::did::ServiceEndpointDeposit;
	type BaseDeposit = constants::did::DidBaseDeposit;
	type Fee = constants::did::DidFee;
	type FeeCollector = SendDustAndFeesToTreasury<Runtime>;

	#[cfg(not(feature = "runtime-benchmarks"))]
	type EnsureOrigin = EnsureDidOrigin<DidIdentifier, AccountId>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type OriginSuccess = DidRawOrigin<AccountId, DidIdentifier>;

	#[cfg(feature = "runtime-benchmarks")]
	type EnsureOrigin = frame_system::EnsureSigned<DidIdentifier>;
	#[cfg(feature = "runtime-benchmarks")]
	type OriginSuccess = DidIdentifier;

	type MaxNewKeyAgreementKeys = constants::did::MaxNewKeyAgreementKeys;
	type MaxTotalKeyAgreementKeys = constants::did::MaxTotalKeyAgreementKeys;
	type MaxPublicKeysPerDid = constants::did::MaxPublicKeysPerDid;
	type MaxBlocksTxValidity = constants::did::MaxBlocksTxValidity;
	type MaxNumberOfServicesPerDid = constants::did::MaxNumberOfServicesPerDid;
	type MaxServiceIdLength = constants::did::MaxServiceIdLength;
	type MaxServiceTypeLength = constants::did::MaxServiceTypeLength;
	type MaxServiceUrlLength = constants::did::MaxServiceUrlLength;
	type MaxNumberOfTypesPerService = constants::did::MaxNumberOfTypesPerService;
	type MaxNumberOfUrlsPerService = constants::did::MaxNumberOfUrlsPerService;
	type WeightInfo = weights::did::WeightInfo<Runtime>;
	type BalanceMigrationManager = Migration;
	type DidLifecycleHooks = DidLifecycleHooks;
}

impl pallet_did_lookup::Config for Runtime {
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeEvent = RuntimeEvent;

	type DidIdentifier = DidIdentifier;

	type Currency = Balances;
	type Deposit = constants::did_lookup::DidLookupDeposit;

	type EnsureOrigin = EnsureDidOrigin<DidIdentifier, AccountId>;
	type AssociateOrigin = Self::EnsureOrigin;
	type OriginSuccess = DidRawOrigin<AccountId, DidIdentifier>;

	type WeightInfo = weights::pallet_did_lookup::WeightInfo<Runtime>;
	type BalanceMigrationManager = Migration;
	// Do not change the below flag to `true` without also deploying a runtime
	// migration which removes any links that point to the same DID!
	type UniqueLinkingEnabled = ConstBool<false>;
}

pub type Web3Name =
	runtime_common::Web3Name<{ constants::web3_names::MIN_LENGTH }, { constants::web3_names::MAX_LENGTH }>;
impl pallet_web3_names::Config for Runtime {
	type RuntimeHoldReason = RuntimeHoldReason;
	type BanOrigin = EnsureRoot<AccountId>;
	type ClaimOrigin = Self::OwnerOrigin;
	type OwnerOrigin = EnsureDidOrigin<DidIdentifier, AccountId>;
	type OriginSuccess = DidRawOrigin<AccountId, DidIdentifier>;
	type Currency = Balances;
	type Deposit = constants::web3_names::Web3NameDeposit;
	type RuntimeEvent = RuntimeEvent;
	type MaxNameLength = constants::web3_names::MaxNameLength;
	type MinNameLength = constants::web3_names::MinNameLength;
	type Web3Name = Web3Name;
	type Web3NameOwner = DidIdentifier;
	type WeightInfo = weights::pallet_web3_names::WeightInfo<Runtime>;
	type BalanceMigrationManager = Migration;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = crate::benchmarks::web3_names::Web3NamesBenchmarkHelper;
}
