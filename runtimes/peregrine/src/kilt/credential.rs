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

use delegation::DelegationAc;
use did::{DidRawOrigin, EnsureDidOrigin};
use frame_system::EnsureRoot;
use runtime_common::{
	assets::AssetDid,
	authorization::{AuthorizationId, PalletAuthorize},
	constants, AccountId, DidIdentifier, Hash, SendDustAndFeesToTreasury,
};
use sp_runtime::traits::BlakeTwo256;

use crate::{weights, Balances, Migration, Runtime, RuntimeEvent, RuntimeHoldReason};

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
