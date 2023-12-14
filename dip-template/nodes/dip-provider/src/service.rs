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

use std::{error::Error, sync::Arc, time::Duration};

use cumulus_client_cli::CollatorOptions;
use cumulus_client_consensus_aura::{
	import_queue, slot_duration, AuraConsensus, BuildAuraConsensusParams, ImportQueueParams, SlotProportion,
};
use cumulus_client_consensus_common::{ParachainBlockImport as TParachainBlockImport, ParachainConsensus};
use cumulus_client_service::{
	build_network, build_relay_chain_interface, prepare_node_config, start_collator, start_full_node,
	BuildNetworkParams, StartCollatorParams, StartFullNodeParams,
};
use cumulus_primitives_core::ParaId;
use cumulus_primitives_parachain_inherent::ParachainInherentData;
use cumulus_relay_chain_interface::RelayChainInterface;
use dip_provider_runtime_template::{api, native_version, NodeBlock as Block, RuntimeApi};
use frame_benchmarking::benchmarking::HostFunctions;
use frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE;
use sc_basic_authorship::ProposerFactory;
use sc_client_api::Backend;
use sc_consensus::{DefaultImportQueue, ImportQueue};
use sc_executor::NativeElseWasmExecutor;
use sc_network::NetworkBlock;
use sc_network_sync::SyncingService;
use sc_service::{
	new_full_parts, spawn_tasks, Configuration, PartialComponents, SpawnTasksParams, TFullBackend, TFullClient,
	TaskManager,
};
use sc_sysinfo::{initialize_hwbench_telemetry, print_hwbench, HwBench};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use sc_transaction_pool::{BasicPool, FullPool};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_keystore::KeystorePtr;
use substrate_prometheus_endpoint::Registry;

use crate::rpc::{create_full, FullDeps};

pub struct ParachainNativeExecutor;

impl sc_executor::NativeExecutionDispatch for ParachainNativeExecutor {
	type ExtendHostFunctions = HostFunctions;

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		native_version()
	}
}

type ParachainExecutor = NativeElseWasmExecutor<ParachainNativeExecutor>;
type ParachainClient = TFullClient<Block, RuntimeApi, ParachainExecutor>;
type ParachainBackend = TFullBackend<Block>;
type ParachainBlockImport = TParachainBlockImport<Block, Arc<ParachainClient>, ParachainBackend>;

#[allow(clippy::type_complexity)]
pub fn new_partial(
	config: &Configuration,
) -> Result<
	PartialComponents<
		ParachainClient,
		ParachainBackend,
		(),
		DefaultImportQueue<Block, ParachainClient>,
		FullPool<Block, ParachainClient>,
		(ParachainBlockImport, Option<Telemetry>, Option<TelemetryWorkerHandle>),
	>,
	sc_service::Error,
> {
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

	let executor = sc_service::new_native_or_wasm_executor(config);

	let (client, backend, keystore_container, task_manager) = new_full_parts::<Block, RuntimeApi, _>(
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

	let transaction_pool = BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);

	let block_import = ParachainBlockImport::new(client.clone(), backend.clone());

	let import_queue = build_import_queue(
		client.clone(),
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

#[sc_tracing::logging::prefix_logs_with("Parachain")]
async fn start_node_impl(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	para_id: ParaId,
	hwbench: Option<HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<ParachainClient>)> {
	let parachain_config = prepare_node_config(parachain_config);

	let params = new_partial(&parachain_config)?;
	let (block_import, mut telemetry, telemetry_worker_handle) = params.other;

	let client = params.client.clone();
	let backend = params.backend.clone();
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

	let force_authoring = parachain_config.force_authoring;
	let validator = parachain_config.role.is_authority();
	let prometheus_registry = parachain_config.prometheus_registry().cloned();
	let transaction_pool = params.transaction_pool.clone();
	let import_queue_service = params.import_queue.service();
	let net_config = sc_network::config::FullNetworkConfiguration::new(&parachain_config.network);

	let (network, system_rpc_tx, tx_handler_controller, start_network, sync_service) =
		build_network(BuildNetworkParams {
			parachain_config: &parachain_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			para_id,
			net_config,
			spawn_handle: task_manager.spawn_handle(),
			relay_chain_interface: relay_chain_interface.clone(),
			import_queue: params.import_queue,
		})
		.await?;

	if parachain_config.offchain_worker.enabled {
		use futures::FutureExt;

		task_manager.spawn_handle().spawn(
			"offchain-workers-runner",
			"offchain-work",
			sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
				runtime_api_provider: client.clone(),
				keystore: Some(params.keystore_container.keystore()),
				offchain_db: backend.offchain_storage(),
				transaction_pool: Some(OffchainTransactionPoolFactory::new(transaction_pool.clone())),
				network_provider: network.clone(),
				is_validator: parachain_config.role.is_authority(),
				enable_http_requests: false,
				custom_extensions: move |_| vec![],
			})
			.run(client.clone(), task_manager.spawn_handle())
			.boxed(),
		);
	}

	let rpc_builder = {
		let client = client.clone();
		let transaction_pool = transaction_pool.clone();

		Box::new(move |deny_unsafe, _| {
			let deps = FullDeps {
				client: client.clone(),
				pool: transaction_pool.clone(),
				deny_unsafe,
			};

			create_full(deps).map_err(Into::into)
		})
	};

	spawn_tasks(SpawnTasksParams {
		rpc_builder,
		sync_service: sync_service.clone(),
		client: client.clone(),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		config: parachain_config,
		keystore: params.keystore_container.keystore(),
		backend,
		network: network.clone(),
		system_rpc_tx,
		tx_handler_controller,
		telemetry: telemetry.as_mut(),
	})?;

	if let Some(hwbench) = hwbench {
		print_hwbench(&hwbench);
		if !SUBSTRATE_REFERENCE_HARDWARE.check_hardware(&hwbench) && validator {
			log::warn!("⚠️  The hardware does not meet the minimal requirements for role 'Authority'.");
		}

		if let Some(ref mut telemetry) = telemetry {
			let telemetry_handle = telemetry.handle();
			task_manager.spawn_handle().spawn(
				"telemetry_hwbench",
				None,
				initialize_hwbench_telemetry(telemetry_handle, hwbench),
			);
		}
	}

	let announce_block = {
		let sync = sync_service.clone();
		Arc::new(move |hash, data| sync.announce_block(hash, data))
	};

	let relay_chain_slot_duration = Duration::from_secs(6);
	let overseer_handle = relay_chain_interface
		.overseer_handle()
		.map_err(|e| sc_service::Error::Application(Box::new(e)))?;

	if validator {
		let parachain_consensus = build_consensus(
			client.clone(),
			block_import,
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|t| t.handle()),
			&task_manager,
			relay_chain_interface.clone(),
			transaction_pool,
			sync_service.clone(),
			params.keystore_container.keystore(),
			force_authoring,
			para_id,
		)?;

		let spawner = task_manager.spawn_handle();
		let params = StartCollatorParams {
			para_id,
			block_status: client.clone(),
			announce_block,
			client: client.clone(),
			task_manager: &mut task_manager,
			relay_chain_interface,
			sync_service: sync_service.clone(),
			spawner,
			parachain_consensus,
			import_queue: import_queue_service,
			collator_key: collator_key.expect("Command line arguments do not allow this. qed"),
			relay_chain_slot_duration,
			recovery_handle: Box::new(overseer_handle),
		};

		start_collator(params).await?;
	} else {
		let params = StartFullNodeParams {
			client: client.clone(),
			announce_block,
			task_manager: &mut task_manager,
			para_id,
			relay_chain_interface,
			sync_service,
			relay_chain_slot_duration,
			import_queue: import_queue_service,
			recovery_handle: Box::new(overseer_handle),
		};

		start_full_node(params)?;
	}

	start_network.start_network();

	Ok((task_manager, client))
}

fn build_import_queue(
	client: Arc<ParachainClient>,
	block_import: ParachainBlockImport,
	config: &Configuration,
	telemetry: Option<TelemetryHandle>,
	task_manager: &TaskManager,
) -> Result<DefaultImportQueue<Block, ParachainClient>, sc_service::Error> {
	let slot_duration = slot_duration(&*client)?;

	import_queue::<sp_consensus_aura::sr25519::AuthorityPair, _, _, _, _, _>(ImportQueueParams {
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
	})
	.map_err(Into::into)
}

#[allow(clippy::too_many_arguments)]
fn build_consensus(
	client: Arc<ParachainClient>,
	block_import: ParachainBlockImport,
	prometheus_registry: Option<&Registry>,
	telemetry: Option<TelemetryHandle>,
	task_manager: &TaskManager,
	relay_chain_interface: Arc<dyn RelayChainInterface>,
	transaction_pool: Arc<FullPool<Block, ParachainClient>>,
	sync_oracle: Arc<SyncingService<Block>>,
	keystore: KeystorePtr,
	force_authoring: bool,
	para_id: ParaId,
) -> Result<Box<dyn ParachainConsensus<Block>>, sc_service::Error> {
	let slot_duration = slot_duration(&*client)?;

	let proposer_factory = ProposerFactory::with_proof_recording(
		task_manager.spawn_handle(),
		client.clone(),
		transaction_pool,
		prometheus_registry,
		telemetry.clone(),
	);

	let params = BuildAuraConsensusParams {
		proposer_factory,
		create_inherent_data_providers: move |_, (relay_parent, validation_data)| {
			let relay_chain_interface = relay_chain_interface.clone();
			async move {
				let parachain_inherent =
					ParachainInherentData::create_at(relay_parent, &relay_chain_interface, &validation_data, para_id)
						.await;
				let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

				let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
					*timestamp,
					slot_duration,
				);

				let parachain_inherent = parachain_inherent
					.ok_or_else(|| Box::<dyn Error + Send + Sync>::from("Failed to create parachain inherent"))?;
				Ok((slot, timestamp, parachain_inherent))
			}
		},
		block_import,
		para_client: client,
		backoff_authoring_blocks: Option::<()>::None,
		sync_oracle,
		keystore,
		force_authoring,
		slot_duration,
		block_proposal_slot_portion: SlotProportion::new(1f32 / 24f32),
		max_block_proposal_slot_portion: Some(SlotProportion::new(1f32 / 16f32)),
		telemetry,
	};

	Ok(AuraConsensus::build::<
		sp_consensus_aura::sr25519::AuthorityPair,
		_,
		_,
		_,
		_,
		_,
		_,
	>(params))
}

pub async fn start_parachain_node(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	para_id: ParaId,
	hwbench: Option<HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<ParachainClient>)> {
	start_node_impl(parachain_config, polkadot_config, collator_options, para_id, hwbench).await
}
