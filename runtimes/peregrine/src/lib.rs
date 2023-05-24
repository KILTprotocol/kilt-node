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
pub use parachain_staking::InflationInfo;
pub use public_credentials;

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

#[cfg(test)]
mod tests;

pub mod config;
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
	spec_name: create_runtime_str!("mashnet-node"),
	impl_name: create_runtime_str!("mashnet-node"),
	authoring_version: 4,
	spec_version: 11100,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 6,
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
	pub const ExistentialDeposit: u128 = EXISTENTIAL_DEPOSIT;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = constants::multisig::DepositBase;
	type DepositFactor = constants::multisig::DepositFactor;
	type MaxSignatories = constants::multisig::MaxSignitors;
	type WeightInfo = weights::pallet_multisig::WeightInfo<Runtime>;
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = Index;
	type Currency = pallet_balances::Pallet<Runtime>;
	type Deposit = constants::IndicesDeposit;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_indices::WeightInfo<Runtime>;
}

impl pallet_vesting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	// disable vested transfers by setting min amount to max balance
	type MinVestedTransfer = constants::MinVestedTransfer;
	type WeightInfo = weights::pallet_vesting::WeightInfo<Runtime>;
	type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
	const MAX_VESTING_SCHEDULES: u32 = constants::MAX_VESTING_SCHEDULES;
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = weights::pallet_utility::WeightInfo<Runtime>;
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
		Sudo: pallet_sudo = 8,

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

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	frame_benchmarking::define_benchmarks!(
		[frame_system, SystemBench::<Runtime>]
		[pallet_timestamp, Timestamp]
		[pallet_indices, Indices]
		[pallet_balances, Balances]
		[pallet_session, SessionBench::<Runtime>]
		[parachain_staking, ParachainStaking]
		[pallet_democracy, Democracy]
		[pallet_collective, Council]
		[pallet_collective, TechnicalCommittee]
		[pallet_membership, TechnicalMembership]
		[pallet_treasury, Treasury]
		[pallet_utility, Utility]
		[pallet_vesting, Vesting]
		[pallet_scheduler, Scheduler]
		[pallet_proxy, Proxy]
		[pallet_preimage, Preimage]
		[pallet_tips, Tips]
		[pallet_multisig, Multisig]
		[ctype, Ctype]
		[attestation, Attestation]
		[delegation, Delegation]
		[did, Did]
		[pallet_inflation, Inflation]
		[pallet_did_lookup, DidLookup]
		[pallet_web3_names, Web3Names]
		[public_credentials, PublicCredentials]
		[pallet_xcm, PolkadotXcm]
		[frame_benchmarking::baseline, Baseline::<Runtime>]

	);
}

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
			frame_system::Pallet::<Runtime>::account_nonce(account)
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

		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
	for Runtime
	{
		fn query_call_info(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
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

	impl kilt_runtime_api_did::Did<
		Block,
		DidIdentifier,
		AccountId,
		LinkableAccountId,
		Balance,
		Hash,
		BlockNumber
	> for Runtime {
		fn query_by_web3_name(name: Vec<u8>) -> Option<kilt_runtime_api_did::RawDidLinkedInfo<
				DidIdentifier,
				AccountId,
				LinkableAccountId,
				Balance,
				Hash,
				BlockNumber
			>
		> {
			let name: pallet_web3_names::web3_name::AsciiWeb3Name<Runtime> = name.try_into().ok()?;
			pallet_web3_names::Owner::<Runtime>::get(&name)
				.and_then(|owner_info| {
					did::Did::<Runtime>::get(&owner_info.owner).map(|details| (owner_info, details))
				})
				.map(|(owner_info, details)| {
					let accounts = pallet_did_lookup::ConnectedAccounts::<Runtime>::iter_key_prefix(
						&owner_info.owner,
					).collect();
					let service_endpoints = did::ServiceEndpoints::<Runtime>::iter_prefix(&owner_info.owner).map(|e| From::from(e.1)).collect();

					kilt_runtime_api_did::RawDidLinkedInfo{
						identifier: owner_info.owner,
						w3n: Some(name.into()),
						accounts,
						service_endpoints,
						details: details.into(),
					}
			})
		}

		fn query_by_account(account: LinkableAccountId) -> Option<
			kilt_runtime_api_did::RawDidLinkedInfo<
				DidIdentifier,
				AccountId,
				LinkableAccountId,
				Balance,
				Hash,
				BlockNumber
			>
		> {
			pallet_did_lookup::ConnectedDids::<Runtime>::get(account)
				.and_then(|owner_info| {
					did::Did::<Runtime>::get(&owner_info.did).map(|details| (owner_info, details))
				})
				.map(|(connection_record, details)| {
					let w3n = pallet_web3_names::Names::<Runtime>::get(&connection_record.did).map(Into::into);
					let accounts = pallet_did_lookup::ConnectedAccounts::<Runtime>::iter_key_prefix(&connection_record.did).collect();
					let service_endpoints = did::ServiceEndpoints::<Runtime>::iter_prefix(&connection_record.did).map(|e| From::from(e.1)).collect();

					kilt_runtime_api_did::RawDidLinkedInfo {
						identifier: connection_record.did,
						w3n,
						accounts,
						service_endpoints,
						details: details.into(),
					}
				})
		}

		fn query(did: DidIdentifier) -> Option<
			kilt_runtime_api_did::RawDidLinkedInfo<
				DidIdentifier,
				AccountId,
				LinkableAccountId,
				Balance,
				Hash,
				BlockNumber
			>
		> {
			let details = did::Did::<Runtime>::get(&did)?;
			let w3n = pallet_web3_names::Names::<Runtime>::get(&did).map(Into::into);
			let accounts = pallet_did_lookup::ConnectedAccounts::<Runtime>::iter_key_prefix(&did).collect();
			let service_endpoints = did::ServiceEndpoints::<Runtime>::iter_prefix(&did).map(|e| From::from(e.1)).collect();

			Some(kilt_runtime_api_did::RawDidLinkedInfo {
				identifier: did,
				w3n,
				accounts,
				service_endpoints,
				details: details.into(),
			})
		}
	}

	impl kilt_runtime_api_public_credentials::PublicCredentials<Block, Vec<u8>, Hash, public_credentials::CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>>, PublicCredentialsFilter<Hash, AccountId>, PublicCredentialsApiError> for Runtime {
		fn get_by_id(credential_id: Hash) -> Option<public_credentials::CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>>> {
			let subject = public_credentials::CredentialSubjects::<Runtime>::get(credential_id)?;
			public_credentials::Credentials::<Runtime>::get(subject, credential_id)
		}

		fn get_by_subject(subject: Vec<u8>, filter: Option<PublicCredentialsFilter<Hash, AccountId>>) -> Result<Vec<(Hash, public_credentials::CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>>)>, PublicCredentialsApiError> {
			let asset_did = AssetDid::try_from(subject).map_err(|_| PublicCredentialsApiError::InvalidSubjectId)?;
			let credentials_prefix = public_credentials::Credentials::<Runtime>::iter_prefix(asset_did);
			if let Some(filter) = filter {
				Ok(credentials_prefix.filter(|(_, entry)| filter.should_include(entry)).collect())
			} else {
				Ok(credentials_prefix.collect())
			}
		}
	}

	impl kilt_runtime_api_staking::Staking<Block, AccountId, Balance> for Runtime {
		fn get_unclaimed_staking_rewards(account: &AccountId) -> Balance {
			ParachainStaking::get_unclaimed_staking_rewards(account)
		}

		fn get_staking_rates() -> kilt_runtime_api_staking::StakingRates {
			ParachainStaking::get_staking_rates()
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;

			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			use frame_benchmarking::baseline::Pallet as Baseline;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();
			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, TrackedStorageKey};
			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			use frame_benchmarking::baseline::Pallet as Baseline;

			impl frame_system_benchmarking::Config for Runtime {}
			impl cumulus_pallet_session_benchmarking::Config for Runtime {}
			impl frame_benchmarking::baseline::Config for Runtime {}

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

			add_benchmarks!(params, batches);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: UpgradeCheckSelect) -> (Weight, Weight) {
			log::info!("try-runtime::on_runtime_upgrade peregrine.");
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, BlockWeights::get().max_block)
		}

		fn execute_block(block: Block, state_root_check: bool, sig_check: bool, select: frame_try_runtime::TryStateSelect) -> Weight {
			log::info!(
				target: "runtime::peregrine", "try-runtime: executing block #{} ({:?}) / root checks: {:?} / sig check: {:?} / sanity-checks: {:?}",
				block.header.number,
				block.header.hash(),
				state_root_check,
				sig_check,
				select,
			);
			Executive::try_execute_block(block, state_root_check, sig_check, select).expect("try_execute_block failed")
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
