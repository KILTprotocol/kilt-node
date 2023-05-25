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

//! The KILT runtime. This can be compiled with `#[no_std]`, ready for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU32, EitherOfDiverse, Everything, InstanceFilter, PrivilegeCmp},
	weights::{ConstantMultiplier, Weight},
};
use frame_system::{EnsureRoot, EnsureSigned};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};

#[cfg(feature = "try-runtime")]
use frame_try_runtime::UpgradeCheckSelect;

use sp_api::impl_runtime_apis;
use sp_core::OpaqueMetadata;
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto, OpaqueKeys},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, Perbill, Permill, RuntimeDebug,
};
use sp_std::{cmp::Ordering, prelude::*};
use sp_version::RuntimeVersion;
use xcm_executor::XcmExecutor;

use delegation::DelegationAc;
use kilt_support::traits::ItemFilter;
use pallet_did_lookup::linkable_account::LinkableAccountId;

use runtime_common::{
	assets::{AssetDid, PublicCredentialsFilter},
	authorization::{AuthorizationId, PalletAuthorize},
	constants::{self, UnvestedFundsAllowedWithdrawReasons, EXISTENTIAL_DEPOSIT, KILT},
	errors::PublicCredentialsApiError,
	fees::{ToAuthor, WeightToFee},
	pallet_id, AccountId, AuthorityId, Balance, BlockHashCount, BlockLength, BlockNumber, BlockWeights, DidIdentifier,
	FeeSplit, Hash, Header, Index, Signature, SlowAdjustingFeeUpdate,
};

use crate::xcm_config::{XcmConfig, XcmOriginToTransactDispatchOrigin};

#[cfg(feature = "std")]
use sp_version::NativeVersion;
#[cfg(feature = "runtime-benchmarks")]
use {kilt_support::signature::AlwaysVerify, runtime_common::benchmarks::DummySignature};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

pub use crate::api::{RuntimeApi, VERSION};
pub use parachain_staking::InflationInfo;
pub use public_credentials;

#[cfg(test)]
mod tests;

pub mod api;
pub mod config;
mod weights;
mod xcm_config;

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

/// The version information used to identify this runtime when compiled
/// natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

construct_runtime! {
	pub enum Runtime where
		Block = Block,
		NodeBlock = runtime_common::Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system = 0,
		// DELETED: RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip = 1,

		Timestamp: pallet_timestamp = 2,
		Indices: pallet_indices::{Pallet, Call, Storage, Event<T>} = 5,
		Balances: pallet_balances = 6,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 7,

		// Consensus support.
		// The following order MUST NOT be changed: Aura -> Session -> Staking -> Authorship -> AuraExt
		// Dependencies: AuraExt on Aura, Authorship and Session on ParachainStaking
		Aura: pallet_aura = 23,
		Session: pallet_session = 22,
		ParachainStaking: parachain_staking = 21,
		Authorship: pallet_authorship::{Pallet, Storage} = 20,
		AuraExt: cumulus_pallet_aura_ext = 24,

		Democracy: pallet_democracy = 30,
		Council: pallet_collective::<Instance1> = 31,
		TechnicalCommittee: pallet_collective::<Instance2> = 32,
		// reserved: parachain council election = 33,
		TechnicalMembership: pallet_membership::<Instance1> = 34,
		Treasury: pallet_treasury = 35,
		// DELETED: RelayMigration: pallet_relay_migration::{Pallet, Call, Storage, Event<T>} = 36,
		// DELETED: DynFilter: pallet_dyn_filter = 37,

		//  A stateless pallet with helper extrinsics (batch extrinsics, send from different origins, ...)
		Utility: pallet_utility = 40,

		// Vesting. Usable initially, but removed once all vesting is finished.
		Vesting: pallet_vesting = 41,

		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 42,

		// Allowing accounts to give permission to other accounts to dispatch types of calls from their signed origin
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 43,

		// Preimage pallet allows the storage of large bytes blob
		Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 44,

		// Tips module to reward contributions to the ecosystem with small amount of KILTs.
		TipsMembership: pallet_membership::<Instance2> = 45,
		Tips: pallet_tips::{Pallet, Call, Storage, Event<T>} = 46,

		Multisig: pallet_multisig = 47,

		// KILT Pallets. Start indices 60 to leave room
		// DELETED: KiltLaunch: kilt_launch = 60,
		Ctype: ctype = 61,
		Attestation: attestation = 62,
		Delegation: delegation = 63,
		Did: did = 64,
		// DELETED: CrowdloanContributors = 65,
		Inflation: pallet_inflation = 66,
		DidLookup: pallet_did_lookup = 67,
		Web3Names: pallet_web3_names = 68,
		PublicCredentials: public_credentials = 69,

		// Parachains pallets. Start indices at 80 to leave room.

		// Among others: Send and receive DMP and XCMP messages.
		ParachainSystem: cumulus_pallet_parachain_system = 80,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 81,
		// Wrap and unwrap XCMP messages to send and receive them. Queue them for later processing.
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 82,
		// Build XCM scripts.
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin, Config} = 83,
		// Does nothing cool, just provides an origin.
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 84,
		// Queue and pass DMP messages on to be executed.
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 85,
	}
}

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	// Executes pallet hooks in the order of definition in construct_runtime
	AllPalletsWithSystem,
	(
		pallet_did_lookup::migrations::CleanupMigration<Runtime>,
		runtime_common::migrations::RemoveInsecureRandomnessPallet<Runtime>,
	),
>;

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data = cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
			relay_chain_slot,
			sp_std::time::Duration::from_secs(6),
		)
		.create_inherent_data()
		.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(block)
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
}
