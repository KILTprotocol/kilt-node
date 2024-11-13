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

//! The KILT runtime. This can be compiled with `#[no_std]`, ready for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
// Triggered by `impl_runtime_apis` macro
#![allow(clippy::empty_structs_with_brackets)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

// External crates
use cfg_if::cfg_if;

// Polkadot-sdk crates
use cumulus_pallet_parachain_system::RelayNumberMonotonicallyIncreases;
use frame_support::{
	construct_runtime,
	genesis_builder_helper::{build_config, create_default_config},
	parameter_types,
	traits::{
		fungible::HoldConsideration,
		tokens::{PayFromAccount, UnityAssetBalanceConversion},
		AsEnsureOriginWithArg, ChangeMembers, ConstU32, EitherOfDiverse, EnqueueWithOrigin, Everything, InstanceFilter,
		LinearStoragePrice, PrivilegeCmp,
	},
	weights::{ConstantMultiplier, Weight},
};
use frame_system::{pallet_prelude::BlockNumberFor, EnsureRoot, EnsureSigned};
use pallet_asset_switch::xcm::{AccountId32ToAccountId32JunctionConverter, MatchesSwitchPairXcmFeeFungibleAsset};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_api::impl_runtime_apis;
use sp_core::{ConstBool, ConstU128, ConstU64, ConstU8, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto, OpaqueKeys},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, Perbill, Permill, RuntimeDebug,
};
use sp_std::{cmp::Ordering, prelude::*};
use sp_version::RuntimeVersion;
use xcm::{v4::Location, VersionedAssetId, VersionedLocation, VersionedXcm};
use xcm_builder::{FungiblesAdapter, NoChecking};

// Internal crates
use delegation::DelegationAc;
use kilt_support::traits::ItemFilter;
use pallet_did_lookup::{linkable_account::LinkableAccountId, ConnectionRecord};
use pallet_web3_names::web3_name::Web3NameOwnership;
pub use parachain_staking::InflationInfo;
pub use public_credentials;
use runtime_common::{
	asset_switch::{runtime_api::Error as AssetSwitchApiError, EnsureRootAsTreasury},
	assets::{AssetDid, PublicCredentialsFilter},
	authorization::{AuthorizationId, PalletAuthorize},
	constants::{
		self, UnvestedFundsAllowedWithdrawReasons, BLOCK_PROCESSING_VELOCITY, EXISTENTIAL_DEPOSIT, KILT,
		RELAY_CHAIN_SLOT_DURATION_MILLIS, SLOT_DURATION, UNINCLUDED_SEGMENT_CAPACITY,
	},
	dip::merkle::{CompleteMerkleProof, DidMerkleProofOf, DidMerkleRootGenerator},
	errors::PublicCredentialsApiError,
	fees::{ToAuthorCredit, WeightToFee},
	pallet_id,
	xcm_config::RelayOrigin,
	AccountId, AuthorityId, Balance, BlockHashCount, BlockLength, BlockNumber, BlockWeights, DidIdentifier, FeeSplit,
	Hash, Header, Nonce, SendDustAndFeesToTreasury, Signature, SlowAdjustingFeeUpdate,
};
use unique_linking_runtime_api::{AddressResult, NameResult};

use crate::xcm_config::{LocationToAccountIdConverter, UniversalLocation, XcmRouter};

mod dip;
mod governance;
mod kilt;
mod parachain;
mod runtime_apis;
mod system;
mod weights;
mod xcm_config;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;

#[cfg(test)]
mod tests;

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("mashnet-node"),
	impl_name: create_runtime_str!("mashnet-node"),
	authoring_version: 4,
	spec_version: 11500,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 8,
	state_version: 0,
};

/// The version information used to identify this runtime when compiled
/// natively.
#[cfg(feature = "std")]
pub fn native_version() -> sp_version::NativeVersion {
	sp_version::NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

construct_runtime! {
	pub enum Runtime
	{
		System: frame_system = 0,
		// DELETED: RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip = 1,

		Timestamp: pallet_timestamp = 2,
		Indices: pallet_indices exclude_parts { Config } = 5,
		Balances: pallet_balances = 6,
		TransactionPayment: pallet_transaction_payment exclude_parts { Config } = 7,
		Sudo: pallet_sudo = 8,
		// Configuration: pallet_configuration = 9,

		// Consensus support.
		// The following order MUST NOT be changed: Aura -> Session -> Staking -> Authorship -> AuraExt
		// Dependencies: AuraExt on Aura, Authorship and Session on ParachainStaking
		Aura: pallet_aura = 23,
		Session: pallet_session = 22,
		ParachainStaking: parachain_staking = 21,
		Authorship: pallet_authorship = 20,
		AuraExt: cumulus_pallet_aura_ext = 24,

		Democracy: pallet_democracy = 30,
		Council: pallet_collective::<Instance1> = 31,
		TechnicalCommittee: pallet_collective::<Instance2> = 32,
		// reserved: parachain council election = 33,
		TechnicalMembership: pallet_membership::<Instance1> = 34,
		Treasury: pallet_treasury = 35,
		// DELETED: RelayMigration: pallet_relay_migration = 36,
		// DELETED: DynFilter: pallet_dyn_filter = 37,

		// A stateless pallet with helper extrinsics (batch extrinsics, send from different origins, ...)
		Utility: pallet_utility = 40,

		// Vesting. Usable initially, but removed once all vesting is finished.
		Vesting: pallet_vesting = 41,

		Scheduler: pallet_scheduler = 42,

		// Allowing accounts to give permission to other accounts to dispatch types of calls from their signed origin
		Proxy: pallet_proxy = 43,

		// Preimage pallet allows the storage of large bytes blob
		Preimage: pallet_preimage = 44,

		// Tips module to reward contributions to the ecosystem with small amount of KILTs.
		TipsMembership: pallet_membership::<Instance2> = 45,
		Tips: pallet_tips = 46,

		Multisig: pallet_multisig = 47,

		AssetSwitchPool1: pallet_asset_switch::<Instance1> = 48,
		Fungibles: pallet_assets = 49,

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
		Migration: pallet_migration = 70,
		DipProvider: pallet_dip_provider = 71,
		DepositStorage: pallet_deposit_storage = 72,
		DotNames: pallet_web3_names::<Instance2> = 73,
		UniqueLinking: pallet_did_lookup::<Instance2> = 74,

		// Parachains pallets. Start indices at 80 to leave room.

		// Among others: Send and receive DMP and XCMP messages.
		ParachainSystem: cumulus_pallet_parachain_system = 80,
		ParachainInfo: parachain_info = 81,
		// Wrap and unwrap XCMP messages to send and receive them. Queue them for later processing.
		XcmpQueue: cumulus_pallet_xcmp_queue = 82,
		// Build XCM scripts.
		PolkadotXcm: pallet_xcm = 83,
		// Does nothing cool, just provides an origin.
		CumulusXcm: cumulus_pallet_xcm exclude_parts { Call } = 84,
		// DmpQueue: cumulus_pallet_dmp_queue = 85,
		// Queue and pass DMP messages on to be executed.
		MessageQueue: pallet_message_queue = 86,
	}
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = constants::MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = constants::MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
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
	frame_support::migrations::RemovePallet<DmpPalletName, <Runtime as frame_system::Config>::DbWeight>,
>;

parameter_types! {
	pub const DmpPalletName: &'static str = "DmpQueue";
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}
