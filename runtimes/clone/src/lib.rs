// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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

use frame_support::{
	construct_runtime, parameter_types,
	traits::Contains,
	weights::{constants::RocksDbWeight, ConstantMultiplier, Weight},
	PalletId,
};
use frame_system::EnsureRoot;
use pallet_did_lookup::linkable_account::LinkableAccountId;
use sp_api::impl_runtime_apis;
use sp_core::OpaqueMetadata;
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, OpaqueKeys},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult,
};
use sp_std::prelude::*;
use sp_version::RuntimeVersion;
use xcm::opaque::latest::BodyId;
use xcm_executor::XcmExecutor;

use runtime_common::{
	assets::PublicCredentialsFilter,
	authorization::AuthorizationId,
	constants::{self, HOURS, MILLI_KILT},
	errors::PublicCredentialsApiError,
	fees::{ToAuthor, WeightToFee},
	AccountId, AuthorityId, Balance, BlockHashCount, BlockLength, BlockNumber, BlockWeights, DidIdentifier, FeeSplit,
	Hash, Header, Index, Signature, SlowAdjustingFeeUpdate,
};

use crate::xcm_config::{XcmConfig, XcmOriginToTransactDispatchOrigin};

#[cfg(feature = "std")]
use sp_version::NativeVersion;

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

mod weights;
mod xcm_config;

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("KILT"),
	impl_name: create_runtime_str!("KILT"),
	authoring_version: 0,
	spec_version: 10902,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 0,
	state_version: 0,
};

/// The version information used to identify this runtime when compiled
/// natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const SS58Prefix: u8 = 38;
}

pub struct CloneCallFilter;
impl Contains<Call> for CloneCallFilter {
	fn contains(call: &Call) -> bool {
		matches!(call, Call::ParachainSystem(..) | Call::Sudo(..) | Call::Timestamp(..))
	}
}

impl frame_system::Config for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in
	/// dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = runtime_common::Header;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest
	/// pruned first).
	type BlockHashCount = BlockHashCount;
	/// Runtime version.
	type Version = Version;
	/// Converts a module to an index of this module in the runtime.
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = RocksDbWeight;
	type BaseCallFilter = CloneCallFilter;
	type SystemWeightInfo = weights::frame_system::WeightInfo<Runtime>;
	type BlockWeights = BlockWeights;
	type BlockLength = BlockLength;
	type SS58Prefix = SS58Prefix;
	/// The set code logic
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Runtime>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = constants::SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = weights::pallet_timestamp::WeightInfo<Runtime>;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 10 * MILLI_KILT;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = weights::pallet_balances::WeightInfo<Runtime>;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
}

impl pallet_transaction_payment::Config for Runtime {
	type Event = Event;
	type OnChargeTransaction =
		pallet_transaction_payment::CurrencyAdapter<Balances, FeeSplit<Runtime, (), ToAuthor<Runtime>>>;
	type OperationalFeeMultiplier = constants::fee::OperationalFeeMultiplier;
	type WeightToFee = WeightToFee<Runtime>;
	type LengthToFee = ConstantMultiplier<Balance, constants::fee::TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = constants::MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = constants::MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type OnSystemEvent = cumulus_pallet_solo_to_para::Pallet<Runtime>;
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type OutboundXcmpMessageSource = XcmpQueue;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

impl cumulus_pallet_solo_to_para::Config for Runtime {
	type Event = Event;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = PolkadotXcm;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = cumulus_pallet_xcmp_queue::weights::SubstrateWeight<Self>;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
	pub const MaxAuthorities: u32 = constants::staking::MAX_CANDIDATES;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuthorityId;
	//TODO: handle disabled validators
	type DisabledValidators = ();
	type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
	pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = ();
}

parameter_types! {
	pub const Period: u64 = 6 * HOURS;
	pub const Offset: u64 = 0;
}

impl pallet_session::Config for Runtime {
	type Event = Event;
	type ValidatorId = AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionManager = CollatorSelection;
	type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = weights::pallet_session::WeightInfo<Runtime>;
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = weights::pallet_utility::WeightInfo<Runtime>;
}

parameter_types! {
	pub const PotId: PalletId = PalletId(*b"PotStake");
	pub const MaxCandidates: u32 = 1000;
	pub const MinCandidates: u32 = 5;
	pub const SessionLength: BlockNumber = 6 * HOURS;
	pub const MaxInvulnerables: u32 = 100;
	pub const ExecutiveBody: BodyId = BodyId::Executive;
}

impl pallet_collator_selection::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type UpdateOrigin = EnsureRoot<AccountId>;
	type PotId = PotId;
	type MaxCandidates = MaxCandidates;
	type MinCandidates = MinCandidates;
	type MaxInvulnerables = MaxInvulnerables;
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = Period;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = ();
}

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

construct_runtime! {
	pub enum Runtime where
		Block = Block,
		NodeBlock = runtime_common::Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system = 0,
		// RandomnessCollectiveFlip: pallet_randomness_collective_flip = 1,

		Timestamp: pallet_timestamp = 2,
		// Indices: pallet_indices::{Pallet, Call, Storage, Event<T>} = 5,
		Balances: pallet_balances = 6,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 7,
		Sudo: pallet_sudo = 8,

		// Consensus support.
		// The following order MUST NOT be changed: Authorship -> Staking -> Session -> Aura -> AuraExt
		Authorship: pallet_authorship::{Pallet, Call, Storage} = 20,
		// ParachainStaking: parachain_staking = 21,
		CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 22,
		Session: pallet_session = 23,
		Aura: pallet_aura = 24,
		AuraExt: cumulus_pallet_aura_ext = 25,
		SoloToPara: cumulus_pallet_solo_to_para::{Pallet, Call, Storage, Event} = 26,

		// Governance stuff
		// Democracy: pallet_democracy = 30,
		// Council: pallet_collective::<Instance1> = 31,
		// TechnicalCommittee: pallet_collective::<Instance2> = 32,
		// parachain council election = 33,
		// TechnicalMembership: pallet_membership::<Instance1> = 34,
		// Treasury: pallet_treasury = 35,

		// Utility module.
		Utility: pallet_utility = 40,

		// Vesting. Usable initially, but removed once all vesting is finished.
		// Vesting: pallet_vesting = 41,

		// System scheduler.
		// Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 42,

		// Proxy pallet.
		// Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 43,

		// Preimage registrar
		// Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 44,

		// Tips module to reward contributions to the ecosystem with small amount of KILTs.
		// TipsMembership: pallet_membership::<Instance2> = 45,
		// Tips: pallet_tips::{Pallet, Call, Storage, Event<T>} = 46,

		// KILT Pallets. Start indices 60 to leave room
		// DELETED: KiltLaunch: kilt_launch = 60,
		// Ctype: ctype = 61,
		// Attestation: attestation = 62,
		// Delegation: delegation = 63,
		// Did: did = 64,
		// DELETED: CrowdloanContributors = 65,
		// Inflation: pallet_inflation = 66,
		// DidLookup: pallet_did_lookup = 67,
		// Web3Names: pallet_web3_names = 68,

		// Parachains pallets. Start indices at 80 to leave room.

		// Many other things but also: Send and receive DMP and XCMP messages
		ParachainSystem: cumulus_pallet_parachain_system = 80,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 81,
		// Wrap and unwrap XCMP messages to send and receive them. Queue them for later processing.
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 82,
		// Build XCM scripts
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin, Config} = 83,
		// Does nothing cool. Provides an origin...
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 84,
		// Queue and DMP messages and pass them on to be executed
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
	cumulus_pallet_solo_to_para::CheckSudo<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	// Executes pallet hooks in the order of definition in construct_runtime
	AllPalletsWithSystem,
	(),
>;

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			frame_system::Pallet::<Runtime>::account_nonce(&account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}

		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(
			extrinsic: <Block as BlockT>::Extrinsic,
		) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(block: Block, data: sp_inherents::InherentData) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}

		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuthorityId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuthorityId> {
			Aura::authorities().into_inner()
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	// Solely required from parachain client
	impl kilt_runtime_api_did::Did<
		Block,
		AccountId,
		AccountId,
		LinkableAccountId,
		Balance,
		Hash,
		BlockNumber
	> for Runtime {
		fn query_by_web3_name(_: Vec<u8>) -> Option<kilt_runtime_api_did::RawDidLinkedInfo<
				AccountId,
				AccountId,
				LinkableAccountId,
				Balance,
				Hash,
				BlockNumber
			>
		> {
			None
		}

		fn query_by_account(_: LinkableAccountId) -> Option<
			kilt_runtime_api_did::RawDidLinkedInfo<
				AccountId,
				AccountId,
				LinkableAccountId,
				Balance,
				Hash,
				BlockNumber
			>
		> {
			None
		}

		fn query(_: AccountId) -> Option<
			kilt_runtime_api_did::RawDidLinkedInfo<
				AccountId,
				AccountId,
				LinkableAccountId,
				Balance,
				Hash,
				BlockNumber
			>
		> {
			None
		}
	}

	impl kilt_runtime_api_public_credentials::PublicCredentials<Block, Vec<u8>, Hash, public_credentials::CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<Hash>>, PublicCredentialsFilter<Hash, AccountId>, PublicCredentialsApiError> for Runtime {
		fn get_by_id(_credential_id: Hash) -> Option<public_credentials::CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<Hash>>> {
			None
		}

		fn get_by_subject(_subject: Vec<u8>, _filter: Option<PublicCredentialsFilter<Hash, AccountId>>) -> Result<Vec<(Hash, public_credentials::CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<Hash>>)>, PublicCredentialsApiError> {
			Ok(vec![])
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{list_benchmark, baseline, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			use baseline::Pallet as BaselineBench;

			let mut list = Vec::<BenchmarkList>::new();

			// Substrate
			list_benchmark!(list, extra, frame_benchmarking, BaselineBench::<Runtime>);
			list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
			list_benchmark!(list, extra, pallet_session, SessionBench::<Runtime>);
			list_benchmark!(list, extra, pallet_balances, Balances);
			// list_benchmark!(list, extra, pallet_collective, Council);
			// list_benchmark!(list, extra, pallet_democracy, Democracy);
			// list_benchmark!(list, extra, pallet_indices, Indices);
			// list_benchmark!(list, extra, pallet_membership, TechnicalMembership);
			// list_benchmark!(list, extra, pallet_preimage, Preimage);
			// list_benchmark!(list, extra, pallet_scheduler, Scheduler);
			list_benchmark!(list, extra, pallet_timestamp, Timestamp);
			// list_benchmark!(list, extra, pallet_tips, Tips);
			// list_benchmark!(list, extra, pallet_treasury, Treasury);
			list_benchmark!(list, extra, pallet_utility, Utility);
			// list_benchmark!(list, extra, pallet_vesting, Vesting);
			// list_benchmark!(list, extra, pallet_proxy, Proxy);

			// KILT
			// list_benchmark!(list, extra, attestation, Attestation);
			// list_benchmark!(list, extra, ctype, Ctype);
			// list_benchmark!(list, extra, delegation, Delegation);
			// list_benchmark!(list, extra, did, Did);
			// list_benchmark!(list, extra, pallet_did_lookup, DidLookup);
			// list_benchmark!(list, extra, pallet_inflation, Inflation);
			// list_benchmark!(list, extra, parachain_staking, ParachainStaking);
			// list_benchmark!(list, extra, pallet_web3_names, Web3Names);

			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};
			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			use baseline::Pallet as BaselineBench;

			impl frame_system_benchmarking::Config for Runtime {}
			impl cumulus_pallet_session_benchmarking::Config for Runtime {}
			impl baseline::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac")
					.to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80")
					.to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a")
					.to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850")
					.to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7")
					.to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			// Substrate
			add_benchmark!(params, batches, frame_benchmarking, BaselineBench::<Runtime>);
			add_benchmark!(params, batches, pallet_balances, Balances);
			// add_benchmark!(params, batches, pallet_collective, Council);
			// add_benchmark!(params, batches, pallet_democracy, Democracy);
			// add_benchmark!(params, batches, pallet_indices, Indices);
			// add_benchmark!(params, batches, pallet_membership, TechnicalMembership);
			// add_benchmark!(params, batches, pallet_preimage, Preimage);
			// add_benchmark!(params, batches, pallet_scheduler, Scheduler);
			add_benchmark!(params, batches, pallet_session, SessionBench::<Runtime>);
			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);
			// add_benchmark!(params, batches, pallet_tips, Tips);
			// add_benchmark!(params, batches, pallet_treasury, Treasury);
			add_benchmark!(params, batches, pallet_utility, Utility);
			// add_benchmark!(params, batches, pallet_vesting, Vesting);
			// add_benchmark!(params, batches, pallet_proxy, Proxy);

			// KILT
			// add_benchmark!(params, batches, attestation, Attestation);
			// add_benchmark!(params, batches, ctype, Ctype);
			// add_benchmark!(params, batches, delegation, Delegation);
			// add_benchmark!(params, batches, did, Did);
			// add_benchmark!(params, batches, pallet_did_lookup, DidLookup);
			// add_benchmark!(params, batches, pallet_inflation, Inflation);
			// add_benchmark!(params, batches, parachain_staking, ParachainStaking);
			// add_benchmark!(params, batches, pallet_web3_names, Web3Names);

			// No benchmarks for these pallets
			// add_benchmark!(params, batches, cumulus_pallet_parachain_system, ParachainSystem);
			// add_benchmark!(params, batches, parachain_info, ParachainInfo);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade() -> (Weight, Weight) {
			log::info!("try-runtime::on_runtime_upgrade clone.");
			let weight = Executive::try_runtime_upgrade().unwrap();
			(weight, BlockWeights::get().max_block)
		}

		fn execute_block(block: Block, state_root_check: bool, select: frame_try_runtime::TryStateSelect) -> Weight {
			log::info!(
				target: "runtime::clone", "try-runtime: executing block #{} ({:?}) / root checks: {:?} / sanity-checks: {:?}",
				block.header.number,
				block.header.hash(),
				state_root_check,
				select,
			);
			Executive::try_execute_block(block, state_root_check, select).expect("try_execute_block failed")
		}
	}
}

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
