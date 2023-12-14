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
	weights, AccountId, Balances, DidIdentifier, Runtime, RuntimeEvent, RuntimeHoldReason,
};

const MAX_LINKED_ACCOUNTS: u32 = 20;

pub mod runtime_api {
	use super::*;

	/// Parameters for a DIP proof request.
	#[derive(Encode, Decode, TypeInfo)]
	pub struct DipProofRequest {
		/// The subject identifier for which to generate the DIP proof.
		pub(crate) identifier: DidIdentifier,
		/// The DIP version.
		pub(crate) version: IdentityCommitmentVersion,
		/// The DID key IDs of the subject's DID Document to reveal in the DIP
		/// proof.
		pub(crate) keys: Vec<KeyIdOf<Runtime>>,
		/// The list of accounts linked to the subject's DID to reveal in the
		/// DIP proof.
		pub(crate) accounts: Vec<LinkableAccountId>,
		/// A flag indicating whether the web3name claimed by the DID subject
		/// should revealed in the DIP proof.
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

	/// The namespace to use in the [`pallet_deposit_storage::Pallet`] to store
	/// all deposits related to DIP commitments.
	pub struct DipProviderDepositNamespace;

	impl Get<DepositNamespaces> for DipProviderDepositNamespace {
		fn get() -> DepositNamespaces {
			DepositNamespaces::DipProvider
		}
	}

	/// The amount of tokens locked for each identity commitment.
	pub const DEPOSIT_AMOUNT: Balance = 2 * UNIT;

	/// The additional logic to execute whenever a deposit is removed by its
	/// owner directly via the [`pallet_deposit_storage::Pallet`] pallet.
	pub type DepositCollectorHooks = FixedDepositCollectorViaDepositsPallet<
		DipProviderDepositNamespace,
		ConstU128<DEPOSIT_AMOUNT>,
		(DidIdentifier, IdentityCommitmentVersion),
	>;

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

	/// The logic to execute whenever an identity commitment is generated and
	/// stored in the [`pallet_dip_provider::Pallet`] pallet.
	///
	/// Upon storing and removing identity commitments, this hook will reserve
	/// or release deposits from the [`pallet_deposit_storage::Pallet`] pallet.
	pub struct DepositHooks;

	impl DepositStorageHooks<Runtime> for DepositHooks {
		type Error = CommitmentDepositRemovalHookError;

		fn on_deposit_reclaimed(
			_namespace: &<Runtime as pallet_deposit_storage::Config>::Namespace,
			key: &DepositKeyOf<Runtime>,
			deposit: DepositEntryOf<Runtime>,
		) -> Result<(), Self::Error> {
			let (identifier, commitment_version) = <(DidIdentifier, IdentityCommitmentVersion)>::decode(&mut &key[..])
				.map_err(|_| CommitmentDepositRemovalHookError::DecodeKey)?;
			pallet_dip_provider::Pallet::<Runtime>::delete_identity_commitment_storage_entry(
				&identifier,
				// Deposit owner is the only one authorized to remove the deposit.
				&deposit.deposit.owner,
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

	#[cfg(feature = "runtime-benchmarks")]
	pub struct PalletDepositStorageBenchmarkHooks;

	#[cfg(feature = "runtime-benchmarks")]
	impl pallet_deposit_storage::traits::BenchmarkHooks<Runtime> for PalletDepositStorageBenchmarkHooks {
		fn pre_reclaim_deposit() -> (
			<Runtime as frame_system::Config>::AccountId,
			<Runtime as pallet_deposit_storage::Config>::Namespace,
			sp_runtime::BoundedVec<u8, <Runtime as pallet_deposit_storage::Config>::MaxKeyLength>,
		) {
			let submitter = AccountId::from([100u8; 32]);
			let namespace = DepositNamespaces::DipProvider;
			let did_identifier = DidIdentifier::from([200u8; 32]);
			let commitment_version = 0u16;
			let key: DepositKeyOf<Runtime> = (did_identifier.clone(), 0)
				.encode()
				.try_into()
				.expect("Should not fail to create a key for a DIP commitment.");

			pallet_dip_provider::IdentityCommitments::<Runtime>::insert(
				&did_identifier,
				commitment_version,
				<Runtime as frame_system::Config>::Hash::default(),
			);

			assert!(
				pallet_dip_provider::IdentityCommitments::<Runtime>::get(did_identifier, commitment_version).is_some()
			);

			(submitter, namespace, key)
		}

		fn post_reclaim_deposit() {
			let did_identifier = DidIdentifier::from([200u8; 32]);
			let commitment_version = 0u16;
			assert!(
				pallet_dip_provider::IdentityCommitments::<Runtime>::get(did_identifier, commitment_version).is_none()
			);
		}
	}
}

impl pallet_deposit_storage::Config for Runtime {
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHooks = deposit::PalletDepositStorageBenchmarkHooks;
	// Any signed origin can submit the tx, which will go through only if the
	// deposit payer matches the signed origin.
	type CheckOrigin = EnsureSigned<AccountId>;
	// The balances pallet is used to reserve/unreserve tokens.
	type Currency = Balances;
	type DepositHooks = DepositHooks;
	type MaxKeyLength = ConstU32<256>;
	type Namespace = DepositNamespaces;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type WeightInfo = weights::pallet_deposit_storage::WeightInfo<Runtime>;
}

impl pallet_dip_provider::Config for Runtime {
	// Only DID origins can submit the commitment identity tx, which will go through
	// only if the DID in the origin matches the identifier specified in the tx.
	type CommitOriginCheck = EnsureDidOrigin<DidIdentifier, AccountId>;
	type CommitOrigin = DidRawOrigin<DidIdentifier, AccountId>;
	type Identifier = DidIdentifier;
	// The identity commitment is defined as the Merkle root of the linked identity
	// info, as specified by the [`LinkedDidInfoProvider`].
	type IdentityCommitmentGenerator = DidMerkleRootGenerator<Runtime>;
	// Identity info is defined as the collection of DID keys, linked accounts, and
	// the optional web3name of a given DID subject.
	type IdentityProvider = LinkedDidInfoProvider<MAX_LINKED_ACCOUNTS>;
	type ProviderHooks = deposit::DepositCollectorHooks;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_dip_provider::WeightInfo<Runtime>;
}
