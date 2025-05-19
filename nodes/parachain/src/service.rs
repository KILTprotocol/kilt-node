// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

// rpc
use jsonrpsee::RpcModule;

use cumulus_client_cli::CollatorOptions;
use cumulus_client_collator::service::CollatorService;
use cumulus_client_consensus_common::ParachainBlockImport as TParachainBlockImport;
use cumulus_client_consensus_proposer::Proposer;
use cumulus_client_service::{
	build_relay_chain_interface, prepare_node_config, start_relay_chain_tasks, CollatorSybilResistance,
	DARecoveryProfile, StartRelayChainTasksParams,
};
use cumulus_primitives_core::{
	relay_chain::{CollatorPair, ValidationCode},
	ParaId,
};
use cumulus_relay_chain_interface::{OverseerHandle, RelayChainInterface};
use sc_consensus::ImportQueue;
use sc_executor::WasmExecutor;
use sc_network::NetworkBlock;
use sc_network_sync::SyncingService;
use sc_service::{Configuration, TFullBackend, TFullClient, TaskManager};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use sp_api::ConstructRuntimeApi;
use sp_io::SubstrateHostFunctions;
use sp_keystore::KeystorePtr;
use sp_runtime::traits::BlakeTwo256;
use std::{sync::Arc, time::Duration};
use substrate_prometheus_endpoint::Registry;

use runtime_common::{AccountId, AuthorityId, Balance, BlockNumber, Hash, Nonce};

pub const AUTHORING_DURATION: u64 = 1500;
pub const TASK_MANAGER_IDENTIFIER: &str = "aura";

type Header = sp_runtime::generic::Header<BlockNumber, sp_runtime::traits::BlakeTwo256>;

pub(crate) type Block = sp_runtime::generic::Block<Header, sp_runtime::OpaqueExtrinsic>;

#[cfg(not(feature = "runtime-benchmarks"))]
type HostFunctions = (
	sp_io::SubstrateHostFunctions,
	cumulus_client_service::storage_proof_size::HostFunctions,
);

#[cfg(feature = "runtime-benchmarks")]
type HostFunctions = (
	SubstrateHostFunctions,
	cumulus_client_service::storage_proof_size::HostFunctions,
	frame_benchmarking::benchmarking::HostFunctions,
);

type ParachainExecutor = WasmExecutor<HostFunctions>;

type ParachainClient<RuntimeApi> = TFullClient<Block, RuntimeApi, ParachainExecutor>;

type ParachainBackend = TFullBackend<Block>;

type ParachainBlockImport<RuntimeApi> =
	TParachainBlockImport<Block, Arc<ParachainClient<RuntimeApi>>, ParachainBackend>;

pub(crate) type TransactionPool<Block, RuntimeApi> =
	sc_transaction_pool::FullPool<Block, TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>>;

type PartialComponents<Block, RuntimeApi, Telemetry, TelemetryWorkerHandle> = sc_service::PartialComponents<
	TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>,
	TFullBackend<Block>,
	(),
	sc_consensus::DefaultImportQueue<Block>,
	TransactionPool<Block, RuntimeApi>,
	(
		ParachainBlockImport<RuntimeApi>,
		Option<Telemetry>,
		Option<TelemetryWorkerHandle>,
	),
>;

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the
/// builder in order to be able to perform chain operations.
pub(crate) fn new_partial<RuntimeApi, BIQ>(
	config: &Configuration,
	build_import_queue: BIQ,
) -> Result<PartialComponents<Block, RuntimeApi, Telemetry, TelemetryWorkerHandle>, sc_service::Error>
where
	RuntimeApi:
		ConstructRuntimeApi<Block, TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::Metadata<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_api::ApiExt<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_block_builder::BlockBuilder<Block>,
	sc_client_api::StateBackendFor<TFullBackend<Block>, Block>: sp_state_machine::Backend<BlakeTwo256>,
	BIQ: FnOnce(
		Arc<TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>>,
		ParachainBlockImport<RuntimeApi>,
		&Configuration,
		Option<TelemetryHandle>,
		&TaskManager,
	) -> Result<sc_consensus::DefaultImportQueue<Block>, sc_service::Error>,
{
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	#[allow(deprecated)]
	let executor = ParachainExecutor::new(
		config.executor.wasm_method,
		config.executor.default_heap_pages,
		config.executor.max_runtime_instances,
		None,
		config.executor.runtime_cache_size,
	);

	let (client, backend, keystore_container, task_manager) = sc_service::new_full_parts::<Block, RuntimeApi, _>(
		config,
		telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
		executor,
	)?;
	let client = Arc::new(client);

	let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		Arc::clone(&client),
	);

	let block_import = ParachainBlockImport::<RuntimeApi>::new(Arc::clone(&client), Arc::clone(&backend));

	let import_queue = build_import_queue(
		Arc::clone(&client),
		block_import.clone(),
		config,
		telemetry.as_ref().map(|telemetry| telemetry.handle()),
		&task_manager,
	)?;

	Ok(PartialComponents {
		backend,
		client,
		import_queue,
		keystore_container,
		task_manager,
		transaction_pool,
		select_chain: (),
		other: (block_import, telemetry, telemetry_worker_handle),
	})
}

/// Start a node with the given parachain `Configuration` and relay chain
/// `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the
/// runtime api.
#[allow(clippy::too_many_arguments)]
#[sc_tracing::logging::prefix_logs_with("Parachain")]
async fn start_node_impl<RuntimeApi, RB, BIQ>(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	id: ParaId,
	_rpc_ext_builder: RB,
	build_import_queue: BIQ,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(
	TaskManager,
	Arc<TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>>,
)>
where
	RuntimeApi:
		ConstructRuntimeApi<Block, TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::Metadata<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_api::ApiExt<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ cumulus_primitives_core::CollectCollationInfo<Block>
		+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
		+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
		+ sp_consensus_aura::AuraApi<Block, AuthorityId>
		+ cumulus_primitives_aura::AuraUnincludedSegmentApi<Block>
		+ pallet_ismp_runtime_api::IsmpRuntimeApi<Block, sp_core::H256>
		+ ismp_parachain_runtime_api::IsmpParachainApi<Block>,
	sc_client_api::StateBackendFor<TFullBackend<Block>, Block>: sp_state_machine::Backend<BlakeTwo256>,
	RB: FnOnce(
			Arc<TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>>,
		) -> Result<RpcModule<()>, sc_service::Error>
		+ Send
		+ 'static,
	BIQ: FnOnce(
		Arc<TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>>,
		ParachainBlockImport<RuntimeApi>,
		&Configuration,
		Option<TelemetryHandle>,
		&TaskManager,
	) -> Result<sc_consensus::DefaultImportQueue<Block>, sc_service::Error>,
{
	let parachain_config = prepare_node_config(parachain_config);

	let params = new_partial::<RuntimeApi, BIQ>(&parachain_config, build_import_queue)?;
	let (block_import, mut telemetry, telemetry_worker_handle) = params.other;

	let client = Arc::clone(&params.client);
	let backend = Arc::clone(&params.backend);
	let mut task_manager = params.task_manager;

	let (relay_chain_interface, collator_key) = build_relay_chain_interface(
		polkadot_config,
		&parachain_config,
		telemetry_worker_handle,
		&mut task_manager,
		collator_options.clone(),
		hwbench.clone(),
	)
	.await
	.map_err(|e| sc_service::Error::Application(Box::new(e) as Box<_>))?;

	let validator = parachain_config.role.is_authority();
	let prometheus_registry = parachain_config.prometheus_registry().cloned();
	let transaction_pool = Arc::clone(&params.transaction_pool);
	let import_queue_service = params.import_queue.service();
	let net_config = sc_network::config::FullNetworkConfiguration::<_, _, sc_network::NetworkWorker<Block, Hash>>::new(
		&parachain_config.network,
		prometheus_registry.clone(),
	);
	let (network, system_rpc_tx, tx_handler_controller, start_network, sync_service) =
		cumulus_client_service::build_network(cumulus_client_service::BuildNetworkParams {
			parachain_config: &parachain_config,
			client: Arc::clone(&client),
			transaction_pool: Arc::clone(&transaction_pool),
			para_id: id,
			net_config,
			spawn_handle: task_manager.spawn_handle(),
			relay_chain_interface: Arc::clone(&relay_chain_interface),
			import_queue: params.import_queue,
			sybil_resistance_level: CollatorSybilResistance::Resistant, // because of Aura
		})
		.await?;

	let rpc_builder = {
		let client = Arc::clone(&client);
		let transaction_pool = Arc::clone(&transaction_pool);
		let backend = Arc::clone(&backend);

		Box::new(move |_| {
			let deps = crate::rpc::FullDeps {
				client: Arc::clone(&client),
				pool: Arc::clone(&transaction_pool),
				backend: Arc::clone(&backend),
			};

			crate::rpc::create_full(deps).map_err(Into::into)
		})
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		rpc_builder,
		sync_service: Arc::clone(&sync_service),
		client: Arc::clone(&client),
		transaction_pool: Arc::clone(&transaction_pool),
		task_manager: &mut task_manager,
		config: parachain_config,
		keystore: params.keystore_container.keystore(),
		backend: Arc::clone(&backend),
		network,
		system_rpc_tx,
		tx_handler_controller,
		telemetry: telemetry.as_mut(),
	})?;

	if let Some(hwbench) = hwbench {
		sc_sysinfo::print_hwbench(&hwbench);

		if let Some(ref mut telemetry) = telemetry {
			let telemetry_handle = telemetry.handle();
			task_manager.spawn_handle().spawn(
				"telemetry_hwbench",
				None,
				sc_sysinfo::initialize_hwbench_telemetry(telemetry_handle, hwbench),
			);
		}
	}

	let announce_block = {
		let sync = Arc::clone(&sync_service);
		Arc::new(move |hash, data| sync.announce_block(hash, data))
	};

	let relay_chain_slot_duration = Duration::from_secs(6);

	let overseer_handle = relay_chain_interface
		.overseer_handle()
		.map_err(|e| sc_service::Error::Application(Box::new(e)))?;

	start_relay_chain_tasks(StartRelayChainTasksParams {
		client: Arc::clone(&client),
		#[allow(clippy::clone_on_ref_ptr)]
		announce_block: announce_block.clone(),
		para_id: id,
		relay_chain_interface: Arc::clone(&relay_chain_interface),
		task_manager: &mut task_manager,
		da_recovery_profile: if validator {
			DARecoveryProfile::Collator
		} else {
			DARecoveryProfile::FullNode
		},
		import_queue: import_queue_service,
		relay_chain_slot_duration,
		recovery_handle: Box::new(overseer_handle.clone()),
		sync_service: Arc::clone(&sync_service),
	})?;

	if validator {
		start_consensus::<RuntimeApi>(
			Arc::clone(&client),
			Arc::clone(&backend),
			block_import,
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|t| t.handle()),
			&task_manager,
			Arc::clone(&relay_chain_interface),
			transaction_pool,
			sync_service,
			params.keystore_container.keystore(),
			relay_chain_slot_duration,
			id,
			collator_key.expect("Collator key not provided"),
			overseer_handle,
			announce_block,
		)?;
	}

	start_network.start_network();

	Ok((task_manager, client))
}

#[allow(clippy::type_complexity)]
/// Build the import queue for THE runtime.
pub(crate) fn build_import_queue<API>(
	client: Arc<TFullClient<Block, API, WasmExecutor<HostFunctions>>>,
	block_import: ParachainBlockImport<API>,
	config: &Configuration,
	telemetry: Option<TelemetryHandle>,
	task_manager: &TaskManager,
) -> Result<sc_consensus::DefaultImportQueue<Block>, sc_service::Error>
where
	API: ConstructRuntimeApi<Block, TFullClient<Block, API, WasmExecutor<HostFunctions>>> + Send + Sync + 'static,
	API::RuntimeApi: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::Metadata<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_api::ApiExt<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
		+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
		+ sp_consensus_aura::AuraApi<Block, AuthorityId>
		+ cumulus_primitives_core::CollectCollationInfo<Block>,
	sc_client_api::StateBackendFor<TFullBackend<Block>, Block>: sp_state_machine::Backend<BlakeTwo256>,
{
	let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;

	cumulus_client_consensus_aura::import_queue::<sp_consensus_aura::sr25519::AuthorityPair, _, _, _, _, _>(
		cumulus_client_consensus_aura::ImportQueueParams {
			block_import,
			client,
			create_inherent_data_providers: move |_, _| async move {
				let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

				let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
					*timestamp,
					slot_duration,
				);

				Ok((slot, timestamp))
			},
			registry: config.prometheus_registry(),
			spawner: &task_manager.spawn_essential_handle(),
			telemetry,
		},
	)
	.map_err(Into::into)
}

/// Start a parachain node.
pub(crate) async fn start_node<API>(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	id: ParaId,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<TFullClient<Block, API, WasmExecutor<HostFunctions>>>)>
where
	API: ConstructRuntimeApi<Block, TFullClient<Block, API, WasmExecutor<HostFunctions>>> + Send + Sync + 'static,
	API::RuntimeApi: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::Metadata<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_api::ApiExt<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
		+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
		+ sp_consensus_aura::AuraApi<Block, AuthorityId>
		+ cumulus_primitives_core::CollectCollationInfo<Block>
		+ cumulus_primitives_aura::AuraUnincludedSegmentApi<Block>
		+ pallet_ismp_runtime_api::IsmpRuntimeApi<Block, sp_core::H256>
		+ ismp_parachain_runtime_api::IsmpParachainApi<Block>,
	sc_client_api::StateBackendFor<TFullBackend<Block>, Block>: sp_state_machine::Backend<BlakeTwo256>,
{
	start_node_impl::<API, _, _>(
		parachain_config,
		polkadot_config,
		collator_options,
		id,
		|_| Ok(RpcModule::new(())),
		build_import_queue::<API>,
		hwbench,
	)
	.await
}

#[allow(clippy::too_many_arguments)]
fn start_consensus<RuntimeApi>(
	client: Arc<ParachainClient<RuntimeApi>>,
	backend: Arc<ParachainBackend>,
	block_import: ParachainBlockImport<RuntimeApi>,
	prometheus_registry: Option<&Registry>,
	telemetry: Option<TelemetryHandle>,
	task_manager: &TaskManager,
	relay_chain_interface: Arc<dyn RelayChainInterface>,
	transaction_pool: Arc<sc_transaction_pool::FullPool<Block, ParachainClient<RuntimeApi>>>,
	_sync_oracle: Arc<SyncingService<Block>>,
	keystore: KeystorePtr,
	relay_chain_slot_duration: Duration,
	para_id: ParaId,
	collator_key: CollatorPair,
	overseer_handle: OverseerHandle,
	announce_block: Arc<dyn Fn(Hash, Option<Vec<u8>>) + Send + Sync>,
) -> Result<(), sc_service::Error>
where
	RuntimeApi:
		ConstructRuntimeApi<Block, TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::Metadata<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_api::ApiExt<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ cumulus_primitives_core::CollectCollationInfo<Block>
		+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
		+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
		+ sp_consensus_aura::AuraApi<Block, AuthorityId>
		+ cumulus_primitives_aura::AuraUnincludedSegmentApi<Block>
		+ ismp_parachain_runtime_api::IsmpParachainApi<Block>
		+ pallet_ismp_runtime_api::IsmpRuntimeApi<Block, sp_core::H256>,
{
	use cumulus_client_consensus_aura::collators::lookahead::{self as aura, Params as AuraParams};

	let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
		task_manager.spawn_handle(),
		Arc::clone(&client),
		transaction_pool,
		prometheus_registry,
		telemetry,
	);

	let proposer = Proposer::new(proposer_factory);

	let collator_service = CollatorService::new(
		Arc::clone(&client),
		Arc::new(task_manager.spawn_handle()),
		announce_block,
		Arc::clone(&client),
	);

	let (client_clone, relay_chain_interface_clone) = (Arc::clone(&client), Arc::clone(&relay_chain_interface));

	let params = AuraParams {
		create_inherent_data_providers: move |parent, ()| {
			let client = Arc::clone(&client_clone);
			let relay_chain_interface = Arc::clone(&relay_chain_interface_clone);
			async move {
				let inherent =
					ismp_parachain_inherent::ConsensusInherentProvider::create(parent, client, relay_chain_interface)
						.await?;

				Ok(inherent)
			}
		},
		block_import,
		para_client: Arc::clone(&client),
		para_backend: backend,
		relay_client: relay_chain_interface,
		code_hash_provider: move |block_hash| client.code_at(block_hash).ok().map(|c| ValidationCode::from(c).hash()),
		keystore,
		collator_key,
		para_id,
		overseer_handle,
		relay_chain_slot_duration,
		proposer,
		collator_service,
		authoring_duration: Duration::from_millis(AUTHORING_DURATION),
		reinitialize: false,
	};

	let fut = aura::run::<Block, sp_consensus_aura::sr25519::AuthorityPair, _, _, _, _, _, _, _, _>(params);
	task_manager
		.spawn_essential_handle()
		.spawn(TASK_MANAGER_IDENTIFIER, None, fut);

	Ok(())
}
