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

use crate::{DidIdentifier, Runtime};
use frame_support::traits::Get;
use pallet_deposit_storage::{
	traits::DepositStorageHooks, DepositEntryOf, DepositKeyOf, FixedDepositCollectorViaDepositsPallet,
};
use pallet_dip_provider::IdentityCommitmentVersion;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use runtime_common::{constants::dip_provider::COMMITMENT_DEPOSIT, AccountId};
use scale_info::TypeInfo;
use sp_core::{ConstU128, RuntimeDebug};

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum DepositNamespace {
	DipProvider,
}

/// The namespace to use in the [`pallet_deposit_storage::Pallet`] to store
/// all deposits related to DIP commitments.
pub struct DipProviderDepositNamespace;

impl Get<DepositNamespace> for DipProviderDepositNamespace {
	fn get() -> DepositNamespace {
		DepositNamespace::DipProvider
	}
}

/// The various different keys that can be stored in the storage-tracking
/// pallet.
/// Although the namespace is used to distinguish between keys, it is useful to
/// group all keys under the same enum to calculate the maximum length that a
/// key can take.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum DepositKey {
	DipProvider {
		identifier: DidIdentifier,
		version: IdentityCommitmentVersion,
	},
}

impl From<(DidIdentifier, AccountId, IdentityCommitmentVersion)> for DepositKey {
	fn from((identifier, _, version): (DidIdentifier, AccountId, IdentityCommitmentVersion)) -> Self {
		Self::DipProvider { identifier, version }
	}
}

/// The additional logic to execute whenever a deposit is removed by its
/// owner directly via the [`pallet_deposit_storage::Pallet`] pallet.
pub type DepositCollectorHooks =
	FixedDepositCollectorViaDepositsPallet<DipProviderDepositNamespace, ConstU128<COMMITMENT_DEPOSIT>, DepositKey>;

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
		_deposit: DepositEntryOf<Runtime>,
	) -> Result<(), Self::Error> {
		let DepositKey::DipProvider { identifier, version } =
			DepositKey::decode(&mut &key[..]).map_err(|_| CommitmentDepositRemovalHookError::DecodeKey)?;
		// No hook must be called otherwise it would try to delete the deposit again,
		// leading to a circular call graph with leads to failure as soon as the deposit
		// is trying to be deleted again.
		pallet_dip_provider::Pallet::<Runtime>::delete_identity_commitment_storage_entry_without_hook(
			&identifier,
			version,
		)
		.map_err(|_| {
			log::error!(
				"Should not fail to remove commitment for identifier {:#?} and version {version}",
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
		let submitter = runtime_common::AccountId::from([100u8; 32]);
		let namespace = DepositNamespace::DipProvider;
		let did_identifier = DidIdentifier::from([200u8; 32]);
		let commitment_version = 0u16;
		let key: DepositKeyOf<Runtime> =
			DepositKey::from((did_identifier.clone(), submitter.clone(), commitment_version))
				.encode()
				.try_into()
				.expect("Should not fail to create a key for a DIP commitment.");

		pallet_dip_provider::IdentityCommitments::<Runtime>::insert(
			&did_identifier,
			commitment_version,
			<Runtime as frame_system::Config>::Hash::default(),
		);

		assert!(pallet_dip_provider::IdentityCommitments::<Runtime>::get(did_identifier, commitment_version).is_some());

		(submitter, namespace, key)
	}

	fn post_reclaim_deposit() {
		let did_identifier = DidIdentifier::from([200u8; 32]);
		let commitment_version = 0u16;
		assert!(pallet_dip_provider::IdentityCommitments::<Runtime>::get(did_identifier, commitment_version).is_none());
	}
}
