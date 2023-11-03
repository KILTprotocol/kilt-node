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
use pallet_dip_provider::{traits::IdentityProvider, IdentityCommitmentVersion};
use parity_scale_codec::{Decode, Encode};
use runtime_common::dip::{
	did::LinkedDidInfoProviderOf,
	merkle::{DidMerkleProofError, DidMerkleRootGenerator},
};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

use crate::{AccountId, Balances, DidIdentifier, Hash, Runtime, RuntimeEvent, RuntimeHoldReason};

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

	use crate::{Balance, UNIT};

	use frame_support::traits::Get;
	use pallet_deposit_storage::{FixedDepositCollectorViaDepositsPallet, MAX_NAMESPACE_LENGTH};
	use sp_core::{ConstU128, ConstU32};
	use sp_runtime::BoundedVec;

	pub const NAMESPACE: [u8; 11] = *b"DipProvider";

	pub struct Namespace;

	impl Get<BoundedVec<u8, ConstU32<MAX_NAMESPACE_LENGTH>>> for Namespace {
		fn get() -> BoundedVec<u8, ConstU32<MAX_NAMESPACE_LENGTH>> {
			debug_assert!(NAMESPACE.len() <= MAX_NAMESPACE_LENGTH as usize, "Namespace is longer than the maximum namespace length configured in the pallet_deposit_storage pallet.");
			NAMESPACE
				.to_vec()
				.try_into()
				.expect("Namespace should never fail to be converted to a BoundedVec.")
		}
	}

	pub const DEPOSIT_AMOUNT: Balance = 2 * UNIT;

	pub type DepositCollectorHooks =
		FixedDepositCollectorViaDepositsPallet<Runtime, Namespace, ConstU128<DEPOSIT_AMOUNT>>;
}

impl pallet_deposit_storage::Config for Runtime {
	type CheckOrigin = EnsureSigned<AccountId>;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
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
	type ProviderHooks = deposit::DepositCollectorHooks;
	type RuntimeEvent = RuntimeEvent;
}
