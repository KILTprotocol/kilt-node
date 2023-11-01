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
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_provider::{
	traits::{IdentityProvider, NoopHooks},
	IdentityCommitmentVersion,
};
use parity_scale_codec::{Decode, Encode};
use runtime_common::dip::{
	did::LinkedDidInfoProviderOf,
	merkle::{DidMerkleProofError, DidMerkleRootGenerator},
};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

use crate::{AccountId, DidIdentifier, Hash, Runtime, RuntimeEvent};

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
		IdentityNotFound,
		IdentityProviderError(<LinkedDidInfoProviderOf<Runtime> as IdentityProvider<DidIdentifier>>::Error),
		MerkleProofError(DidMerkleProofError),
	}
}

pub mod deposit {
	use super::*;
	use crate::{Balance, Balances, RuntimeHoldReason};

	use frame_support::traits::{fungible::Inspect, tokens::fungible::MutateHold};

	use kilt_support::{traits::StorageDepositCollector, Deposit};
	use pallet_dip_provider::{traits::ProviderHooks, HoldReason};
	use sp_runtime::DispatchError;

	pub enum CommitmentDepositCollectorError {
		Internal,
	}

	impl From<CommitmentDepositCollectorError> for u16 {
		fn from(value: CommitmentDepositCollectorError) -> Self {
			match value {
				CommitmentDepositCollectorError::Internal => u16::MAX,
			}
		}
	}

	impl From<CommitmentDepositCollectorError> for DispatchError {
		fn from(value: CommitmentDepositCollectorError) -> Self {
			match value {
				CommitmentDepositCollectorError::Internal => {
					DispatchError::Other("CommitmentDepositCollectorError::Internal")
				}
			}
		}
	}

	pub const DEPOSIT: Balance = 100_000;

	// TODO: Store deposits somewhere, so that they can be freed up even after the
	// deposit amount changes.
	pub struct CommitmentDepositCollector;

	impl StorageDepositCollector<AccountId, (AccountId, IdentityCommitmentVersion), RuntimeHoldReason>
		for CommitmentDepositCollector
	{
		type Currency = Balances;
		type Reason = HoldReason;

		fn reason() -> Self::Reason {
			Self::Reason::Deposit
		}

		fn deposit(
			key: &(AccountId, IdentityCommitmentVersion),
		) -> Result<Deposit<AccountId, <Self::Currency as Inspect<AccountId>>::Balance>, DispatchError> {
			log::error!("CommitmentDepositCollector::deposit(key) called, when it should not have, since it returns a dummy value.");
			Err(CommitmentDepositCollectorError::Internal.into())
		}

		fn deposit_amount(
			key: &(AccountId, IdentityCommitmentVersion),
		) -> <Self::Currency as Inspect<AccountId>>::Balance {
			DEPOSIT
		}

		fn get_hashed_key(key: &(AccountId, IdentityCommitmentVersion)) -> Result<Vec<u8>, DispatchError> {
			log::error!("CommitmentDepositCollector::get_hashed_key(key) called, when it should not have, since it returns a dummy value.");
			Err(CommitmentDepositCollectorError::Internal.into())
		}

		fn store_deposit(
			key: &(AccountId, IdentityCommitmentVersion),
			deposit: Deposit<AccountId, <Self::Currency as Inspect<AccountId>>::Balance>,
		) -> Result<(), DispatchError> {
			log::error!("CommitmentDepositCollector::get_hashed_key(key) called, when it should not have, since it returns a dummy value.");
			Err(CommitmentDepositCollectorError::Internal.into())
		}
	}

	impl ProviderHooks for CommitmentDepositCollector {
		type Error = CommitmentDepositCollectorError;
		type Identifier = DidIdentifier;
		type IdentityCommitment = Hash;
		type Submitter = AccountId;
		type Success = ();

		fn on_identity_committed(
			identifier: &Self::Identifier,
			submitter: &Self::Submitter,
			commitment: &Self::IdentityCommitment,
			version: IdentityCommitmentVersion,
		) -> Result<Self::Success, Self::Error> {
			let _deposit = Self::create_deposit(submitter, DEPOSIT);
			// TODO: Store deposit somewhere, perhaps inside the provider pallet, via some
			// metadata tricks?
			Ok(())
		}

		fn on_commitment_removed(
			identifier: &Self::Identifier,
			submitter: &Self::Submitter,
			commitment: &Self::IdentityCommitment,
			version: IdentityCommitmentVersion,
		) -> Result<Self::Success, Self::Error> {
			let deposit = Self::deposit((submitter, version))?;
			Self::free_deposit(deposit)
		}
	}
}

impl pallet_dip_provider::Config for Runtime {
	type CommitOriginCheck = EnsureDidOrigin<DidIdentifier, AccountId>;
	type CommitOrigin = DidRawOrigin<DidIdentifier, AccountId>;
	type Identifier = DidIdentifier;
	type IdentityCommitment = Hash;
	type IdentityCommitmentGenerator = DidMerkleRootGenerator<Runtime>;
	type IdentityCommitmentGeneratorError = DidMerkleProofError;
	type IdentityProvider = LinkedDidInfoProviderOf<Runtime>;
	type IdentityProviderError = <LinkedDidInfoProviderOf<Runtime> as IdentityProvider<DidIdentifier>>::Error;
	// TODO: Change to deposit collector
	type ProviderHooks = NoopHooks<Self::Identifier, Self::IdentityCommitment, Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
}
