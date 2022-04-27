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

use crate::{
	chain_spec::{self},
	cli::{Cli, RelayChainCli, Subcommand},
	service::{new_partial, PeregrineRuntimeExecutor, SpiritnetRuntimeExecutor},
};
use codec::Encode;
use cumulus_client_service::genesis::generate_genesis_block;
use cumulus_primitives_core::ParaId;
use frame_benchmarking_cli::BenchmarkCmd;
use log::info;
use polkadot_parachain::primitives::AccountIdConversion;
#[cfg(feature = "try-runtime")]
use polkadot_service::TaskManager;
use runtime_common::Block;
use sc_cli::{
	ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams, NetworkParams, Result,
	RuntimeVersion, SharedParams, SubstrateCli,
};
use sc_service::config::{BasePath, PrometheusConfig};
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::traits::Block as BlockT;
use std::{io::Write, net::SocketAddr};

trait IdentifyChain {
	fn is_peregrine(&self) -> bool;
	fn is_spiritnet(&self) -> bool;
}

impl IdentifyChain for dyn sc_service::ChainSpec {
	fn is_peregrine(&self) -> bool {
		self.id().contains("peregrine") || self.id().eq("kilt_parachain_testnet")
	}
	fn is_spiritnet(&self) -> bool {
		self.id().contains("spiritnet")
			|| self.id().eq("kilt_westend")
			|| self.id().eq("kilt_rococo_2")
			|| self.id().eq("kilt")
	}
}

impl<T: sc_service::ChainSpec + 'static> IdentifyChain for T {
	fn is_peregrine(&self) -> bool {
		<dyn sc_service::ChainSpec>::is_peregrine(self)
	}
	fn is_spiritnet(&self) -> bool {
		<dyn sc_service::ChainSpec>::is_spiritnet(self)
	}
}

fn load_spec(id: &str, runtime: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
	match (id, runtime) {
		("dev", _) => Ok(Box::new(chain_spec::peregrine::make_dev_spec()?)),
		("peregrine-new", _) => Ok(Box::new(chain_spec::peregrine::make_new_spec()?)),
		("spiritnet-dev", _) => Ok(Box::new(chain_spec::spiritnet::get_chain_spec_dev()?)),
		("wilt-new", _) => Ok(Box::new(chain_spec::spiritnet::get_chain_spec_wilt()?)),
		("rilt-new", _) => Ok(Box::new(chain_spec::spiritnet::get_chain_spec_rilt()?)),
		("rilt", _) => Ok(Box::new(chain_spec::spiritnet::load_rilt_spec()?)),
		("spiritnet", _) => Ok(Box::new(chain_spec::spiritnet::load_spiritnet_spec()?)),
		("", "spiritnet") => Ok(Box::new(chain_spec::spiritnet::get_chain_spec_dev()?)),
		("", "peregrine") => Ok(Box::new(chain_spec::peregrine::make_dev_spec()?)),
		(path, "spiritnet") => Ok(Box::new(chain_spec::spiritnet::ChainSpec::from_json_file(path.into())?)),
		(path, "peregrine") => Ok(Box::new(chain_spec::peregrine::ChainSpec::from_json_file(path.into())?)),
		_ => Err("Unknown runtime".to_owned()),
	}
}

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"KILT".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		format!(
			"KILT\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relaychain node.\n\n\
		{} [parachain-args] -- [relaychain-args]",
			Self::executable_name()
		)
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/kiltprotocol/kilt-parachain/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2017
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		load_spec(id, &self.runtime)
	}

	fn native_runtime_version(spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		if spec.id().starts_with("spiritnet") {
			&spiritnet_runtime::VERSION
		} else {
			&peregrine_runtime::VERSION
		}
	}
}

impl SubstrateCli for RelayChainCli {
	fn impl_name() -> String {
		"KILT".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		"KILT\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relaychain node.\n\n\
		kilt-parachain [parachain-args] -- [relaychain-args]"
			.into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/kiltprotocol/kilt-parachain/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2017
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		polkadot_cli::Cli::from_iter([RelayChainCli::executable_name()].iter()).load_spec(id)
	}

	fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		polkadot_cli::Cli::native_runtime_version(chain_spec)
	}
}

#[allow(clippy::borrowed_box)]
fn extract_genesis_wasm(chain_spec: &Box<dyn sc_service::ChainSpec>) -> Result<Vec<u8>> {
	let mut storage = chain_spec.build_storage()?;

	storage
		.top
		.remove(sp_core::storage::well_known_keys::CODE)
		.ok_or_else(|| "Could not find wasm file in genesis state!".into())
}

// TODO: Make use of the macro in Benchmark cmd
macro_rules! construct_async_run {
	(|$components:ident, $cli:ident, $cmd:ident, $config:ident| $( $code:tt )* ) => {{
		let runner = $cli.create_runner($cmd)?;
		match $cli.runtime.as_str() {
			"spiritnet" => {
					runner.async_run(|$config| {
						let $components = new_partial::<spiritnet_runtime::RuntimeApi, SpiritnetRuntimeExecutor, _>(
							&$config,
							crate::service::build_import_queue::<SpiritnetRuntimeExecutor, spiritnet_runtime::RuntimeApi>,
						)?;
						let task_manager = $components.task_manager;
						{ $( $code )* }.map(|v| (v, task_manager))
					})
				},
			"peregrine" => {
					runner.async_run(|$config| {
						let $components = new_partial::<peregrine_runtime::RuntimeApi, PeregrineRuntimeExecutor, _>(
							&$config,
							crate::service::build_import_queue::<PeregrineRuntimeExecutor, peregrine_runtime::RuntimeApi>,
						)?;
						let task_manager = $components.task_manager;
						{ $( $code )* }.map(|v| (v, task_manager))
					})
				}
			_ => panic!("unknown runtime"),
		}
	}}
}

/// The runtime argument is not always on the root level of the cli struct. Make
/// sure that it is.
fn fix_runtime_arg(cli: &mut Cli) {
	let sub_command_runtime = match &cli.subcommand {
		Some(Subcommand::BuildSpec(cmd)) => Some(&cmd.runtime),
		Some(Subcommand::ExportGenesisState(cmd)) => Some(&cmd.runtime),
		_ => None,
	};
	cli.runtime = sub_command_runtime.unwrap_or(&cli.runtime).clone();
}

/// Parse command line arguments into service configuration.
pub fn run() -> Result<()> {
	let mut cli = Cli::from_args();
	fix_runtime_arg(&mut cli);
	let cli = cli;

	match &cli.subcommand {
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(&cmd.inner_args)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		}
		Some(Subcommand::CheckBlock(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		}
		Some(Subcommand::ExportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| Ok(cmd.run(components.client, config.database)))
		}
		Some(Subcommand::ExportState(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| Ok(cmd.run(components.client, config.chain_spec)))
		}
		Some(Subcommand::ImportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		}
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| {
				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()]
						.iter()
						.chain(cli.relaychain_args.iter()),
				);

				let polkadot_config =
					SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, config.tokio_handle.clone())
						.map_err(|err| format!("Relay chain argument error: {}", err))?;

				cmd.run(config, polkadot_config)
			})
		}
		Some(Subcommand::Revert(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| Ok(cmd.run(
				components.client,
				components.backend,
				None
			)))
		}
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			// Switch on the concrete benchmark sub-command
			match (cmd, cli.runtime.as_str()) {
				(BenchmarkCmd::Pallet(cmd), runtime) => {
					if cfg!(feature = "runtime-benchmarks") {
						match runtime {
							"spiritnet" => runner.sync_run(|config| cmd.run::<Block, SpiritnetRuntimeExecutor>(config)),
							"peregrine" => runner.sync_run(|config| cmd.run::<Block, PeregrineRuntimeExecutor>(config)),
							_ => Err("Unknown parachain runtime".into()),
						}
					} else {
						Err("Benchmarking wasn't enabled when building the node. \
							You can enable it with `--features runtime-benchmarks`."
							.into())
					}
				}
				(BenchmarkCmd::Block(cmd), "spiritnet") => runner.sync_run(|config| {
					let partials = new_partial::<spiritnet_runtime::RuntimeApi, SpiritnetRuntimeExecutor, _>(
						&config,
						crate::service::build_import_queue,
					)?;
					cmd.run(partials.client)
				}),
				(BenchmarkCmd::Block(cmd), "peregrine") => runner.sync_run(|config| {
					let partials = new_partial::<peregrine_runtime::RuntimeApi, PeregrineRuntimeExecutor, _>(
						&config,
						crate::service::build_import_queue,
					)?;
					cmd.run(partials.client)
				}),
				(BenchmarkCmd::Storage(cmd), "spiritnet") => runner.sync_run(|config| {
					let partials = new_partial::<spiritnet_runtime::RuntimeApi, SpiritnetRuntimeExecutor, _>(
						&config,
						crate::service::build_import_queue,
					)?;

					let db = partials.backend.expose_db();
					let storage = partials.backend.expose_storage();

					cmd.run(config, partials.client.clone(), db, storage)
				}),
				(BenchmarkCmd::Storage(cmd), "peregrine") => runner.sync_run(|config| {
					let partials = new_partial::<peregrine_runtime::RuntimeApi, PeregrineRuntimeExecutor, _>(
						&config,
						crate::service::build_import_queue,
					)?;

					let db = partials.backend.expose_db();
					let storage = partials.backend.expose_storage();

					cmd.run(config, partials.client.clone(), db, storage)
				}),
				(BenchmarkCmd::Overhead(_), _) => Err("Unsupported benchmarking command".into()),
				(_, _) => Err("Unknown parachain runtime".into()),
			}
		}
		Some(Subcommand::ExportGenesisState(params)) => {
			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			let spec = load_spec(&params.chain.clone().unwrap_or_default(), &params.runtime)?;
			let state_version = Cli::native_runtime_version(&spec).state_version();
			let block: Block = generate_genesis_block(&spec, state_version)?;
			let raw_header = block.header().encode();
			let output_buf = if params.raw {
				raw_header
			} else {
				format!("0x{:?}", HexDisplay::from(&block.header().encode())).into_bytes()
			};

			if let Some(output) = &params.output {
				std::fs::write(output, output_buf)?;
			} else {
				std::io::stdout().write_all(&output_buf)?;
			}

			Ok(())
		}
		Some(Subcommand::ExportGenesisWasm(params)) => {
			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			let raw_wasm_blob = extract_genesis_wasm(&cli.load_spec(&params.chain.clone().unwrap_or_default())?)?;
			let output_buf = if params.raw {
				raw_wasm_blob
			} else {
				format!("0x{:?}", HexDisplay::from(&raw_wasm_blob)).into_bytes()
			};

			if let Some(output) = &params.output {
				std::fs::write(output, output_buf)?;
			} else {
				std::io::stdout().write_all(&output_buf)?;
			}

			Ok(())
		}
		#[cfg(feature = "try-runtime")]
		Some(Subcommand::TryRuntime(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let registry = &runner.config().prometheus_config.as_ref().map(|cfg| &cfg.registry);
			let task_manager = TaskManager::new(runner.config().tokio_handle.clone(), *registry)
				.map_err(|e| format!("Error: {:?}", e))?;

			if runner.config().chain_spec.is_peregrine() {
				runner.async_run(|config| Ok((cmd.run::<Block, PeregrineRuntimeExecutor>(config), task_manager)))
			} else if runner.config().chain_spec.is_spiritnet() {
				runner.async_run(|config| Ok((cmd.run::<Block, SpiritnetRuntimeExecutor>(config), task_manager)))
			} else {
				Err("Chain doesn't support try-runtime".into())
			}
		}
		#[cfg(not(feature = "try-runtime"))]
		Some(Subcommand::TryRuntime) => Err("TryRuntime wasn't enabled when building the node. \
				You can enable it with `--features try-runtime`."
			.into()),
		None => {
			let runner = cli.create_runner(&cli.run.normalize())?;
			let collator_options = cli.run.collator_options();

			runner.run_node_until_exit(|config| async move {
				let para_id = chain_spec::Extensions::try_get(&*config.chain_spec)
					.map(|e| e.para_id)
					.ok_or("Could not find parachain ID in chain-spec.")?;

				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()]
						.iter()
						.chain(cli.relaychain_args.iter()),
				);

				let id = ParaId::from(para_id);

				let parachain_account = AccountIdConversion::<polkadot_primitives::v2::AccountId>::into_account(&id);

				let state_version = RelayChainCli::native_runtime_version(&config.chain_spec).state_version();
				let block: Block =
					generate_genesis_block(&config.chain_spec, state_version).map_err(|e| format!("{:?}", e))?;
				let genesis_state = format!("0x{:?}", HexDisplay::from(&block.header().encode()));

				let tokio_handle = config.tokio_handle.clone();
				let polkadot_config = SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, tokio_handle)
					.map_err(|err| format!("Relay chain argument error: {}", err))?;

				info!("Parachain id: {:?}", id);
				info!("Parachain Account: {}", parachain_account);
				info!("Parachain genesis state: {}", genesis_state);
				info!(
					"Is collating: {}",
					if config.role.is_authority() { "yes" } else { "no" }
				);

				if config.chain_spec.is_peregrine() {
					crate::service::start_node::<PeregrineRuntimeExecutor, peregrine_runtime::RuntimeApi>(
						config,
						polkadot_config,
						collator_options,
						id,
					)
					.await
					.map(|r| r.0)
					.map_err(Into::into)
				} else if config.chain_spec.is_spiritnet() {
					crate::service::start_node::<SpiritnetRuntimeExecutor, spiritnet_runtime::RuntimeApi>(
						config,
						polkadot_config,
						collator_options,
						id,
					)
					.await
					.map(|r| r.0)
					.map_err(Into::into)
				} else {
					Err("Unknown runtime".into())
				}
			})
		}
	}
}

impl DefaultConfigurationValues for RelayChainCli {
	fn p2p_listen_port() -> u16 {
		30334
	}

	fn rpc_ws_listen_port() -> u16 {
		9945
	}

	fn rpc_http_listen_port() -> u16 {
		9934
	}

	fn prometheus_listen_port() -> u16 {
		9616
	}
}

impl CliConfiguration<Self> for RelayChainCli {
	fn shared_params(&self) -> &SharedParams {
		self.base.base.shared_params()
	}

	fn import_params(&self) -> Option<&ImportParams> {
		self.base.base.import_params()
	}

	fn network_params(&self) -> Option<&NetworkParams> {
		self.base.base.network_params()
	}

	fn keystore_params(&self) -> Option<&KeystoreParams> {
		self.base.base.keystore_params()
	}

	fn base_path(&self) -> Result<Option<BasePath>> {
		Ok(self
			.shared_params()
			.base_path()
			.or_else(|| self.base_path.clone().map(Into::into)))
	}

	fn rpc_http(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_http(default_listen_port)
	}

	fn rpc_ipc(&self) -> Result<Option<String>> {
		self.base.base.rpc_ipc()
	}

	fn rpc_ws(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_ws(default_listen_port)
	}

	fn prometheus_config(
		&self,
		default_listen_port: u16,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<PrometheusConfig>> {
		self.base.base.prometheus_config(default_listen_port, chain_spec)
	}

	fn init<F>(
		&self,
		_support_url: &String,
		_impl_version: &String,
		_logger_hook: F,
		_config: &sc_service::Configuration,
	) -> Result<()>
	where
		F: FnOnce(&mut sc_cli::LoggerBuilder, &sc_service::Configuration),
	{
		unreachable!("PolkadotCli is never initialized; qed");
	}

	fn chain_id(&self, is_dev: bool) -> Result<String> {
		let chain_id = self.base.base.chain_id(is_dev)?;

		Ok(if chain_id.is_empty() {
			self.chain_id.clone().unwrap_or_default()
		} else {
			chain_id
		})
	}

	fn role(&self, is_dev: bool) -> Result<sc_service::Role> {
		self.base.base.role(is_dev)
	}

	fn transaction_pool(&self) -> Result<sc_service::config::TransactionPoolOptions> {
		self.base.base.transaction_pool()
	}

	fn state_cache_child_ratio(&self) -> Result<Option<usize>> {
		self.base.base.state_cache_child_ratio()
	}

	fn rpc_methods(&self) -> Result<sc_service::config::RpcMethods> {
		self.base.base.rpc_methods()
	}

	fn rpc_ws_max_connections(&self) -> Result<Option<usize>> {
		self.base.base.rpc_ws_max_connections()
	}

	fn rpc_cors(&self, is_dev: bool) -> Result<Option<Vec<String>>> {
		self.base.base.rpc_cors(is_dev)
	}

	fn default_heap_pages(&self) -> Result<Option<u64>> {
		self.base.base.default_heap_pages()
	}

	fn force_authoring(&self) -> Result<bool> {
		self.base.base.force_authoring()
	}

	fn disable_grandpa(&self) -> Result<bool> {
		self.base.base.disable_grandpa()
	}

	fn max_runtime_instances(&self) -> Result<Option<usize>> {
		self.base.base.max_runtime_instances()
	}

	fn announce_block(&self) -> Result<bool> {
		self.base.base.announce_block()
	}

	fn telemetry_endpoints(&self, chain_spec: &Box<dyn ChainSpec>) -> Result<Option<sc_telemetry::TelemetryEndpoints>> {
		self.base.base.telemetry_endpoints(chain_spec)
	}
}
