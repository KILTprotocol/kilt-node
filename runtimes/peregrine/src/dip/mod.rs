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

use did::{DidRawOrigin, EnsureDidOrigin};
use frame_system::EnsureSigned;
use runtime_common::{
	constants::{deposit_storage::MAX_DEPOSIT_PALLET_KEY_LENGTH, dip_provider::MAX_LINKED_ACCOUNTS},
	dip::{did::LinkedDidInfoProvider, merkle::DidMerkleRootGenerator},
	AccountId, DidIdentifier,
};
use sp_core::ConstU32;

use crate::{
	dip::deposit::{DepositCollectorHooks, DepositHooks, DepositNamespace},
	Balances, Runtime, RuntimeEvent, RuntimeHoldReason,
};

pub(crate) mod deposit;
pub(crate) mod runtime_api;

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
	type ProviderHooks = DepositCollectorHooks;
	type RuntimeEvent = RuntimeEvent;
	// TODO: Change after benchmarks
	type WeightInfo = ();
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
	type MaxKeyLength = ConstU32<MAX_DEPOSIT_PALLET_KEY_LENGTH>;
	type Namespace = DepositNamespace;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	// TODO: Change after benchmarks
	type WeightInfo = ();
}
