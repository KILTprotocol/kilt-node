// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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
	chain_spec,
	cli::{Cli, RelayChainCli, Subcommand, DEFAULT_PARA_ID},
	service::{new_partial, MashRuntimeExecutor, SpiritRuntimeExecutor},
};
use codec::Encode;
use cumulus_client_service::genesis::generate_genesis_block;
use cumulus_primitives_core::ParaId;
use kilt_parachain_runtime::Block;
use log::info;
use polkadot_parachain::primitives::AccountIdConversion;
use sc_cli::{
	ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams, NetworkParams, Result,
	RuntimeVersion, SharedParams, SubstrateCli,
};
use sc_service::config::{BasePath, PrometheusConfig};
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::traits::Block as BlockT;
use std::{io::Write, net::SocketAddr};

fn load_spec(id: &str, runtime: &str, para_id: ParaId) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
	match id {
		"staging" => Ok(Box::new(chain_spec::mashnet::make_staging_spec(para_id)?)),
		"rococo" => Ok(Box::new(chain_spec::mashnet::load_rococo_spec()?)),
		"dev" => Ok(Box::new(chain_spec::mashnet::make_dev_spec(para_id)?)),
		"spiritnet-dev" => Ok(Box::new(chain_spec::spiritnet::get_chain_spec(para_id)?)),
		"spiritnet" => Ok(Box::new(chain_spec::spiritnet::load_spiritnet_spec()?)),
		"" => match runtime {
			"spiritnet" => Ok(Box::new(chain_spec::spiritnet::get_chain_spec(para_id)?)),
			"mashnet" => Ok(Box::new(chain_spec::mashnet::make_dev_spec(para_id)?)),
			_ => Err("Unknown runtime".to_owned()),
		},
		path => match runtime {
			"spiritnet" => Ok(Box::new(chain_spec::spiritnet::ChainSpec::from_json_file(path.into())?)),
			"mashnet" => Ok(Box::new(chain_spec::mashnet::ChainSpec::from_json_file(path.into())?)),
			_ => Err("Unknown runtime".to_owned()),
		},
	}
}

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"KILT collator".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		format!(
			"KILT collator\n\nThe command-line arguments provided first will be \
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
		load_spec(
			id,
			&self.runtime,
			self.run
				.parachain_id
				.unwrap_or_else(|| DEFAULT_PARA_ID.parse().expect("Default parachain id is a valid u32"))
				.into(),
		)
	}

	fn native_runtime_version(spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		if spec.id().starts_with("spiritnet") {
			&spiritnet_runtime::VERSION
		} else {
			&kilt_parachain_runtime::VERSION
		}
	}
}

impl SubstrateCli for RelayChainCli {
	fn impl_name() -> String {
		"KILT collator".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		"KILT collator\n\nThe command-line arguments provided first will be \
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

macro_rules! construct_async_run {
	(|$components:ident, $cli:ident, $cmd:ident, $config:ident| $( $code:tt )* ) => {{
		let runner = $cli.create_runner($cmd)?;
		match $cli.runtime.as_str() {
			"spiritnet" => {
					runner.async_run(|$config| {
						let $components = new_partial::<spiritnet_runtime::RuntimeApi, SpiritRuntimeExecutor, _>(
							&$config,
							crate::service::build_import_queue::<SpiritRuntimeExecutor, spiritnet_runtime::RuntimeApi>,
						)?;
						let task_manager = $components.task_manager;
						{ $( $code )* }.map(|v| (v, task_manager))
					})
				},
			"mashnet" => {
					runner.async_run(|$config| {
						let $components = new_partial::<kilt_parachain_runtime::RuntimeApi, MashRuntimeExecutor, _>(
							&$config,
							crate::service::build_import_queue::<MashRuntimeExecutor, kilt_parachain_runtime::RuntimeApi>,
						)?;
						let task_manager = $components.task_manager;
						{ $( $code )* }.map(|v| (v, task_manager))
					})
				}
			_ => panic!("unkown runtime"),
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
					SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, config.task_executor.clone())
						.map_err(|err| format!("Relay chain argument error: {}", err))?;

				cmd.run(config, polkadot_config)
			})
		}
		Some(Subcommand::Revert(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| Ok(cmd.run(components.client, components.backend)))
		}
		Some(Subcommand::Benchmark(cmd)) => {
			if cfg!(feature = "runtime-benchmarks") {
				let runner = cli.create_runner(cmd)?;
				match cli.runtime.as_str() {
					"mashnet" => runner.sync_run(|config| cmd.run::<Block, MashRuntimeExecutor>(config)),
					"spiritnet" => runner.sync_run(|config| cmd.run::<Block, SpiritRuntimeExecutor>(config)),
					_ => Err("Unknown runtime".into()),
				}
			} else {
				Err("Benchmarking wasn't enabled when building the node. \
				You can enable it with `--features runtime-benchmarks`."
					.into())
			}
		}
		Some(Subcommand::ExportGenesisState(params)) => {
			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			let block: Block = generate_genesis_block(&load_spec(
				&params.chain.clone().unwrap_or_default(),
				&params.runtime,
				params.parachain_id.into(),
			)?)?;
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
		None => {
			let runner = cli.create_runner(&cli.run.normalize())?;

			runner.run_node_until_exit(|config| async move {
				// TODO
				let key = sp_core::Pair::generate().0;

				let para_id = chain_spec::Extensions::try_get(&*config.chain_spec).map(|e| e.para_id);

				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()]
						.iter()
						.chain(cli.relaychain_args.iter()),
				);

				let id = ParaId::from(cli.run.parachain_id.or(para_id).unwrap_or(100));

				let parachain_account = AccountIdConversion::<polkadot_primitives::v0::AccountId>::into_account(&id);

				let block: Block = generate_genesis_block(&config.chain_spec).map_err(|e| format!("{:?}", e))?;
				let genesis_state = format!("0x{:?}", HexDisplay::from(&block.header().encode()));

				let task_executor = config.task_executor.clone();
				let polkadot_config = SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, task_executor)
					.map_err(|err| format!("Relay chain argument error: {}", err))?;

				info!("Parachain id: {:?}", id);
				info!("Parachain Account: {}", parachain_account);
				info!("Parachain genesis state: {}", genesis_state);
				info!(
					"Is collating: {}",
					if config.role.is_authority() { "yes" } else { "no" }
				);

				match cli.runtime.as_str() {
					"mashnet" => crate::service::start_node::<MashRuntimeExecutor, kilt_parachain_runtime::RuntimeApi>(
						config,
						key,
						polkadot_config,
						id,
					)
					.await
					.map(|r| r.0)
					.map_err(Into::into),
					"spiritnet" => crate::service::start_node::<SpiritRuntimeExecutor, spiritnet_runtime::RuntimeApi>(
						config,
						key,
						polkadot_config,
						id,
					)
					.await
					.map(|r| r.0)
					.map_err(Into::into),
					_ => Err("Unknown runtime".into()),
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

	fn prometheus_config(&self, default_listen_port: u16) -> Result<Option<PrometheusConfig>> {
		self.base.base.prometheus_config(default_listen_port)
	}

	fn init<C: SubstrateCli>(&self) -> Result<()> {
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

	fn telemetry_external_transport(&self) -> Result<Option<sc_service::config::ExtTransport>> {
		self.base.base.telemetry_external_transport()
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
