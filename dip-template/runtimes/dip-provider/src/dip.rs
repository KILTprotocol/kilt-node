// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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

use did::{DidRawOrigin, EnsureDidOrigin, KeyIdOf};
use frame_system::EnsureSigned;
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_provider::IdentityCommitmentVersion;
use parity_scale_codec::{Decode, Encode};
use runtime_common::dip::{
	did::{LinkedDidInfoProvider, LinkedDidInfoProviderError},
	merkle::{DidMerkleProofError, DidMerkleRootGenerator},
};
use scale_info::TypeInfo;
use sp_core::ConstU32;
use sp_std::vec::Vec;

use crate::{
	deposit::{DepositHooks, DepositNamespaces},
	AccountId, Balances, DidIdentifier, Runtime, RuntimeEvent, RuntimeHoldReason,
};

pub mod runtime_api {
	use super::*;

	#[derive(Encode, Decode, TypeInfo)]
	pub struct DipProofRequest {
		pub(crate) identifier: DidIdentifier,
		pub(crate) version: IdentityCommitmentVersion,
		pub(crate) keys: Vec<KeyIdOf<Runtime>>,
		pub(crate) accounts: Vec<LinkableAccountId>,
		pub(crate) should_include_web3_name: bool,
	}

	#[derive(Encode, Decode, TypeInfo)]
	pub enum DipProofError {
		IdentityProvider(LinkedDidInfoProviderError),
		MerkleProof(DidMerkleProofError),
	}
}

pub mod deposit {
	use super::*;

	use crate::{Balance, UNIT};

	use frame_support::traits::Get;
	use pallet_deposit_storage::{
		traits::DepositStorageHooks, DepositEntryOf, DepositKeyOf, FixedDepositCollectorViaDepositsPallet,
	};
	use parity_scale_codec::MaxEncodedLen;
	use sp_core::{ConstU128, RuntimeDebug};

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
	pub enum DepositNamespaces {
		DipProvider,
	}

	pub struct DipProviderDepositNamespace;

	impl Get<DepositNamespaces> for DipProviderDepositNamespace {
		fn get() -> DepositNamespaces {
			DepositNamespaces::DipProvider
		}
	}

	pub const DEPOSIT_AMOUNT: Balance = 2 * UNIT;

	pub type DepositCollectorHooks =
		FixedDepositCollectorViaDepositsPallet<DipProviderDepositNamespace, ConstU128<DEPOSIT_AMOUNT>>;

	pub enum CommitmentDepositRemovalHookError {
		DecodeKey,
		Internal,
	}

	impl From<CommitmentDepositRemovalHookError> for u16 {
		fn from(value: CommitmentDepositRemovalHookError) -> Self {
			match value {
				CommitmentDepositRemovalHookError::DecodeKey => 0,
				CommitmentDepositRemovalHookError::Internal => u16::MAX,
			}
		}
	}

	pub struct DepositHooks;

	impl DepositStorageHooks<Runtime> for DepositHooks {
		type Error = CommitmentDepositRemovalHookError;

		fn on_deposit_reclaimed(
			_namespace: &<Runtime as pallet_deposit_storage::Config>::Namespace,
			key: &DepositKeyOf<Runtime>,
			_deposit: DepositEntryOf<Runtime>,
		) -> Result<(), Self::Error> {
			let (identifier, commitment_version) = <(DidIdentifier, IdentityCommitmentVersion)>::decode(&mut &key[..])
				.map_err(|_| CommitmentDepositRemovalHookError::DecodeKey)?;
			pallet_dip_provider::Pallet::<Runtime>::delete_identity_commitment_storage_entry(
				&identifier,
				commitment_version,
			)
			.map_err(|_| {
				log::error!(
					"Should not fail to remove commitment for identifier {:#?} and version {commitment_version}",
					identifier
				);
				CommitmentDepositRemovalHookError::Internal
			})?;
			Ok(())
		}
	}
}

impl pallet_deposit_storage::Config for Runtime {
	type CheckOrigin = EnsureSigned<AccountId>;
	type Currency = Balances;
	type DepositHooks = DepositHooks;
	type MaxKeyLength = ConstU32<256>;
	type Namespace = DepositNamespaces;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
}

impl pallet_dip_provider::Config for Runtime {
	type CommitOriginCheck = EnsureDidOrigin<DidIdentifier, AccountId>;
	type CommitOrigin = DidRawOrigin<DidIdentifier, AccountId>;
	type Identifier = DidIdentifier;
	type IdentityCommitmentGenerator = DidMerkleRootGenerator<Runtime>;
	type IdentityProvider = LinkedDidInfoProvider;
	type ProviderHooks = deposit::DepositCollectorHooks;
	type RuntimeEvent = RuntimeEvent;
}
