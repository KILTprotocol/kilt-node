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

use delegation::DelegationAc;
use did::{
	DeriveDidCallAuthorizationVerificationKeyRelationship, DeriveDidCallKeyRelationshipResult, DidRawOrigin,
	DidVerificationKeyRelationship, EnsureDidOrigin, RelationshipDeriveError,
};
use frame_support::parameter_types;
use frame_system::{pallet_prelude::BlockNumberFor, EnsureRoot, EnsureSigned};
use pallet_asset_switch::xcm::{AccountId32ToAccountId32JunctionConverter, MatchesSwitchPairXcmFeeFungibleAsset};
use runtime_common::{
	asset_switch::hooks::RestrictSwitchDestinationToSelf,
	assets::AssetDid,
	authorization::{AuthorizationId, PalletAuthorize},
	AccountId, Balance, DidIdentifier, Hash, SendDustAndFeesToTreasury, Web3Name,
};
use sp_core::ConstBool;
use sp_runtime::traits::BlakeTwo256;
use xcm_builder::{FungiblesAdapter, NoChecking};

use crate::{
	constants, weights,
	xcm::{LocationToAccountIdConverter, UniversalLocation, XcmRouter},
	Balances, Fungibles, Migration, PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent, RuntimeFreezeReason,
	RuntimeHoldReason, RuntimeOrigin,
};

impl attestation::Config for Runtime {
	type EnsureOrigin = EnsureDidOrigin<DidIdentifier, AccountId>;
	type OriginSuccess = DidRawOrigin<AccountId, DidIdentifier>;
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::attestation::WeightInfo<Runtime>;

	type Currency = Balances;
	type Deposit = constants::attestation::AttestationDeposit;
	type MaxDelegatedAttestations = constants::attestation::MaxDelegatedAttestations;
	type AttesterId = DidIdentifier;
	type AuthorizationId = AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>;
	type AccessControl = PalletAuthorize<DelegationAc<Runtime>>;
	type BalanceMigrationManager = Migration;
}

impl delegation::Config for Runtime {
	type DelegationEntityId = DidIdentifier;
	type DelegationNodeId = Hash;

	type EnsureOrigin = EnsureDidOrigin<DidIdentifier, AccountId>;
	type OriginSuccess = DidRawOrigin<AccountId, DidIdentifier>;

	#[cfg(not(feature = "runtime-benchmarks"))]
	type DelegationSignatureVerification = did::DidSignatureVerify<Runtime>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type Signature = did::DidSignature;

	#[cfg(feature = "runtime-benchmarks")]
	type Signature = runtime_common::benchmarks::DummySignature;
	#[cfg(feature = "runtime-benchmarks")]
	type DelegationSignatureVerification =
		kilt_support::signature::AlwaysVerify<AccountId, sp_std::vec::Vec<u8>, Self::Signature>;

	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxSignatureByteLength = constants::delegation::MaxSignatureByteLength;
	type MaxParentChecks = constants::delegation::MaxParentChecks;
	type MaxRevocations = constants::delegation::MaxRevocations;
	type MaxRemovals = constants::delegation::MaxRemovals;
	type MaxChildren = constants::delegation::MaxChildren;
	type WeightInfo = weights::delegation::WeightInfo<Runtime>;
	type Currency = Balances;
	type Deposit = constants::delegation::DelegationDeposit;
	type BalanceMigrationManager = Migration;
}

impl ctype::Config for Runtime {
	type CtypeCreatorId = AccountId;
	type Currency = Balances;
	type Fee = constants::CtypeFee;
	type FeeCollector = SendDustAndFeesToTreasury<Runtime>;

	type EnsureOrigin = EnsureDidOrigin<DidIdentifier, AccountId>;
	type OriginSuccess = DidRawOrigin<AccountId, DidIdentifier>;
	type OverarchingOrigin = EnsureRoot<AccountId>;

	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::ctype::WeightInfo<Runtime>;
}

impl DeriveDidCallAuthorizationVerificationKeyRelationship for RuntimeCall {
	fn derive_verification_key_relationship(&self) -> DeriveDidCallKeyRelationshipResult {
		/// ensure that all calls have the same VerificationKeyRelationship
		fn single_key_relationship(calls: &[RuntimeCall]) -> DeriveDidCallKeyRelationshipResult {
			let init = calls
				.get(0)
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
			RuntimeCall::DotNames { .. } => Ok(DidVerificationKeyRelationship::Authentication),
			RuntimeCall::PublicCredentials { .. } => Ok(DidVerificationKeyRelationship::AssertionMethod),
			RuntimeCall::DidLookup { .. } => Ok(DidVerificationKeyRelationship::Authentication),
			RuntimeCall::UniqueLinking { .. } => Ok(DidVerificationKeyRelationship::Authentication),
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
}

impl pallet_did_lookup::Config for Runtime {
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeEvent = RuntimeEvent;

	type DidIdentifier = DidIdentifier;

	type Currency = Balances;
	type Deposit = constants::did_lookup::DidLookupDeposit;

	type EnsureOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
	type OriginSuccess = did::DidRawOrigin<AccountId, DidIdentifier>;

	type WeightInfo = weights::pallet_did_lookup::WeightInfo<Runtime>;
	type BalanceMigrationManager = Migration;
	// Do not change the below flag to `true` without also deploying a runtime
	// migration which removes any links that point to the same DID!
	type UniqueLinkingEnabled = ConstBool<false>;
}

pub(crate) type UniqueLinkingDeployment = pallet_did_lookup::Instance2;
impl pallet_did_lookup::Config<UniqueLinkingDeployment> for Runtime {
	type BalanceMigrationManager = ();
	type Currency = Balances;
	type Deposit = constants::did_lookup::DidLookupDeposit;
	type DidIdentifier = DidIdentifier;
	type EnsureOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
	type OriginSuccess = did::DidRawOrigin<AccountId, DidIdentifier>;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type UniqueLinkingEnabled = ConstBool<true>;
	type WeightInfo = weights::pallet_unique_linking::WeightInfo<Runtime>;
}

impl pallet_web3_names::Config for Runtime {
	type RuntimeHoldReason = RuntimeHoldReason;
	type BanOrigin = EnsureRoot<AccountId>;
	type OwnerOrigin = EnsureDidOrigin<DidIdentifier, AccountId>;
	type OriginSuccess = DidRawOrigin<AccountId, DidIdentifier>;
	type Currency = Balances;
	type Deposit = constants::web3_names::Web3NameDeposit;
	type RuntimeEvent = RuntimeEvent;
	type MaxNameLength = constants::web3_names::MaxNameLength;
	type MinNameLength = constants::web3_names::MinNameLength;
	type Web3Name = Web3Name<{ Self::MinNameLength::get() }, { Self::MaxNameLength::get() }>;
	type Web3NameOwner = DidIdentifier;
	type WeightInfo = weights::pallet_web3_names::WeightInfo<Runtime>;
	type BalanceMigrationManager = Migration;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = crate::benchmarks::web3_names::Web3NamesBenchmarkHelper;
}

pub type DotName = runtime_common::DotName<{ constants::dot_names::MIN_LENGTH }, { constants::dot_names::MAX_LENGTH }>;

pub(crate) type DotNamesDeployment = pallet_web3_names::Instance2;
impl pallet_web3_names::Config<DotNamesDeployment> for Runtime {
	type BalanceMigrationManager = ();
	type BanOrigin = EnsureRoot<AccountId>;
	type Currency = Balances;
	type Deposit = constants::dot_names::Web3NameDeposit;
	type MaxNameLength = constants::dot_names::MaxNameLength;
	type MinNameLength = constants::dot_names::MinNameLength;
	type OriginSuccess = did::DidRawOrigin<AccountId, DidIdentifier>;
	type OwnerOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type Web3Name = DotName;
	type Web3NameOwner = DidIdentifier;
	type WeightInfo = weights::pallet_dot_names::WeightInfo<Runtime>;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = crate::benchmarks::web3_names::DotNamesBenchmarkHelper;
}

impl parachain_staking::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type CurrencyBalance = Balance;
	type FreezeIdentifier = RuntimeFreezeReason;
	type MinBlocksPerRound = constants::staking::MinBlocksPerRound;
	type DefaultBlocksPerRound = constants::staking::DefaultBlocksPerRound;
	type StakeDuration = constants::staking::StakeDuration;
	type ExitQueueDelay = constants::staking::ExitQueueDelay;
	type MinCollators = constants::staking::MinCollators;
	type MinRequiredCollators = constants::staking::MinRequiredCollators;
	type MaxDelegationsPerRound = constants::staking::MaxDelegationsPerRound;
	type MaxDelegatorsPerCollator = constants::staking::MaxDelegatorsPerCollator;
	type MinCollatorStake = constants::staking::MinCollatorStake;
	type MinCollatorCandidateStake = constants::staking::MinCollatorStake;
	type MaxTopCandidates = constants::staking::MaxCollatorCandidates;
	type MinDelegatorStake = constants::staking::MinDelegatorStake;
	type MaxUnstakeRequests = constants::staking::MaxUnstakeRequests;
	type NetworkRewardRate = constants::staking::NetworkRewardRate;
	type NetworkRewardStart = constants::staking::NetworkRewardStart;
	type NetworkRewardBeneficiary = SendDustAndFeesToTreasury<Runtime>;
	type WeightInfo = weights::parachain_staking::WeightInfo<Runtime>;

	const BLOCKS_PER_YEAR: BlockNumberFor<Self> = constants::BLOCKS_PER_YEAR;
}

impl pallet_inflation::Config for Runtime {
	type Currency = Balances;
	type InitialPeriodLength = constants::treasury::InitialPeriodLength;
	type InitialPeriodReward = constants::treasury::InitialPeriodReward;
	type Beneficiary = SendDustAndFeesToTreasury<Runtime>;
	type WeightInfo = weights::pallet_inflation::WeightInfo<Runtime>;
}

impl public_credentials::Config for Runtime {
	type RuntimeHoldReason = RuntimeHoldReason;
	type AccessControl = PalletAuthorize<DelegationAc<Runtime>>;
	type AttesterId = DidIdentifier;
	type AuthorizationId = AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>;
	type CredentialId = Hash;
	type CredentialHash = BlakeTwo256;
	type Currency = Balances;
	type Deposit = constants::public_credentials::Deposit;
	type EnsureOrigin = EnsureDidOrigin<DidIdentifier, AccountId>;
	type MaxEncodedClaimsLength = constants::public_credentials::MaxEncodedClaimsLength;
	type MaxSubjectIdLength = constants::public_credentials::MaxSubjectIdLength;
	type OriginSuccess = DidRawOrigin<AccountId, DidIdentifier>;
	type RuntimeEvent = RuntimeEvent;
	type SubjectId = AssetDid;
	type WeightInfo = weights::public_credentials::WeightInfo<Runtime>;
	type BalanceMigrationManager = Migration;
}

parameter_types! {
	pub CheckingAccount: AccountId = PolkadotXcm::check_account();
}

pub(crate) type KiltToEKiltSwitchPallet = pallet_asset_switch::Instance1;
impl pallet_asset_switch::Config<KiltToEKiltSwitchPallet> for Runtime {
	type AccountIdConverter = AccountId32ToAccountId32JunctionConverter;
	type AssetTransactor = FungiblesAdapter<
		Fungibles,
		MatchesSwitchPairXcmFeeFungibleAsset<Runtime, KiltToEKiltSwitchPallet>,
		LocationToAccountIdConverter,
		AccountId,
		NoChecking,
		CheckingAccount,
	>;
	type FeeOrigin = EnsureRoot<AccountId>;
	type LocalCurrency = Balances;
	type PauseOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type SubmitterOrigin = EnsureSigned<AccountId>;
	type SwitchHooks = RestrictSwitchDestinationToSelf;
	type SwitchOrigin = EnsureRoot<AccountId>;
	type UniversalLocation = UniversalLocation;
	type WeightInfo = weights::pallet_asset_switch::WeightInfo<Runtime>;
	type XcmRouter = XcmRouter;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = crate::benchmarks::asset_switch::CreateFungibleForAssetSwitchPool1;
}

impl pallet_migration::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MaxMigrationsPerPallet = constants::pallet_migration::MaxMigrationsPerPallet;
	type WeightInfo = weights::pallet_migration::WeightInfo<Runtime>;
}
