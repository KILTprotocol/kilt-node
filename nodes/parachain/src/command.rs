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

use cumulus_client_cli::generate_genesis_block;
use cumulus_primitives_core::ParaId;
use frame_benchmarking_cli::{BenchmarkCmd, SUBSTRATE_REFERENCE_HARDWARE};
use log::{info, warn};
use parity_scale_codec::Encode;
use runtime_common::Block;
use sc_cli::{CliConfiguration, SubstrateCli};
use sc_executor::NativeExecutionDispatch;
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::traits::{AccountIdConversion, Block as BlockT, Zero};

use crate::{
	chain_spec::{self, ChainRuntime},
	cli::{Cli, RelayChainCli, Subcommand},
	service::{new_partial, PeregrineRuntimeExecutor, SpiritnetRuntimeExecutor},
};

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

	fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
		chain_spec::load_spec(id)
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

	fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
		polkadot_cli::Cli::from_iter([RelayChainCli::executable_name()].iter()).load_spec(id)
	}
}

macro_rules! construct_async_run {
	(|$components:ident, $cli:ident, $cmd:ident, $config:ident| $( $code:tt )* ) => {{
		let chain_spec_id = $cmd.chain_id($cmd.is_dev()?)?;
		let runtime = chain_spec_id.parse::<ChainRuntime>()?;
		let runner = $cli.create_runner($cmd)?;

		match runtime {
			ChainRuntime::Spiritnet => {
				runner.async_run(|$config| {
					let $components = new_partial::<spiritnet_runtime::RuntimeApi, SpiritnetRuntimeExecutor, _>(
						&$config,
						crate::service::build_import_queue::<SpiritnetRuntimeExecutor, spiritnet_runtime::RuntimeApi>,
					)?;
					let task_manager = $components.task_manager;
					{ $( $code )* }.map(|v| (v, task_manager))
				})
			},
			ChainRuntime::Peregrine => {
				runner.async_run(|$config| {
					let $components = new_partial::<peregrine_runtime::RuntimeApi, PeregrineRuntimeExecutor, _>(
						&$config,
						crate::service::build_import_queue::<PeregrineRuntimeExecutor, peregrine_runtime::RuntimeApi>,
					)?;
					let task_manager = $components.task_manager;
					{ $( $code )* }.map(|v| (v, task_manager))
				})
			}
		}
	}}
}

/// Parse command line arguments into service configuration.
pub(crate) fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
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
		Some(Subcommand::Revert(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.backend, None))
			})
		}
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| {
				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()]
						.iter()
						.chain(cli.relay_chain_args.iter()),
				);

				let polkadot_config =
					SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, config.tokio_handle.clone())
						.map_err(|err| format!("Relay chain argument error: {}", err))?;

				cmd.run(config, polkadot_config)
			})
		}
		Some(Subcommand::ExportGenesisState(cmd)) => {
			let chain_spec_id = cmd.chain_id(cmd.is_dev()?)?;
			let runtime = chain_spec_id.parse::<ChainRuntime>()?;
			let spec = cli.load_spec(chain_spec_id.as_str())?;
			let runner = cli.create_runner(cmd)?;

			match runtime {
				ChainRuntime::Spiritnet => runner.sync_run(|config| {
					let partials = new_partial::<spiritnet_runtime::RuntimeApi, SpiritnetRuntimeExecutor, _>(
						&config,
						crate::service::build_import_queue,
					)?;
					cmd.run::<Block>(&*spec, &*partials.client)
				}),
				ChainRuntime::Peregrine => runner.sync_run(|config| {
					let partials = new_partial::<peregrine_runtime::RuntimeApi, PeregrineRuntimeExecutor, _>(
						&config,
						crate::service::build_import_queue,
					)?;
					cmd.run::<Block>(&*spec, &*partials.client)
				}),
			}
		}
		Some(Subcommand::ExportGenesisWasm(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|_config| {
				let chain_spec_id = cmd.chain_id(cmd.is_dev()?)?;
				let spec = cli.load_spec(chain_spec_id.as_str())?;
				cmd.run(&*spec)
			})
		}
		Some(Subcommand::Benchmark(cmd)) => {
			let chain_spec_id = cmd.chain_id(cmd.is_dev()?)?;
			let runtime = chain_spec_id.parse::<ChainRuntime>()?;
			let runner = cli.create_runner(cmd)?;

			match (cmd, runtime) {
				(BenchmarkCmd::Pallet(cmd), ChainRuntime::Spiritnet) => {
					if cfg!(feature = "runtime-benchmarks") {
						runner.sync_run(|config| {
							cmd.run::<Block, <SpiritnetRuntimeExecutor as NativeExecutionDispatch>::ExtendHostFunctions>(config)
						})
					} else {
						Err("Benchmarking wasn't enabled when building the node. \
							You can enable it with `--features runtime-benchmarks`."
							.into())
					}
				}
				(BenchmarkCmd::Pallet(cmd), ChainRuntime::Peregrine) => {
					if cfg!(feature = "runtime-benchmarks") {
						runner.sync_run(|config| {
							cmd.run::<Block, <PeregrineRuntimeExecutor as NativeExecutionDispatch>::ExtendHostFunctions>(config)
						})
					} else {
						Err("Benchmarking wasn't enabled when building the node. \
							You can enable it with `--features runtime-benchmarks`."
							.into())
					}
				}
				(BenchmarkCmd::Block(cmd), ChainRuntime::Spiritnet) => runner.sync_run(|config| {
					let partials = new_partial::<spiritnet_runtime::RuntimeApi, SpiritnetRuntimeExecutor, _>(
						&config,
						crate::service::build_import_queue,
					)?;
					cmd.run(partials.client)
				}),
				(BenchmarkCmd::Block(cmd), ChainRuntime::Peregrine) => runner.sync_run(|config| {
					let partials = new_partial::<peregrine_runtime::RuntimeApi, PeregrineRuntimeExecutor, _>(
						&config,
						crate::service::build_import_queue,
					)?;
					cmd.run(partials.client)
				}),
				#[cfg(not(feature = "runtime-benchmarks"))]
				(BenchmarkCmd::Storage(_), _) => Err(sc_cli::Error::Input(
					"Compile with --features=runtime-benchmarks \
						to enable storage benchmarks."
						.into(),
				)),
				#[cfg(feature = "runtime-benchmarks")]
				(BenchmarkCmd::Storage(cmd), ChainRuntime::Spiritnet) => runner.sync_run(|config| {
					let partials = new_partial::<spiritnet_runtime::RuntimeApi, SpiritnetRuntimeExecutor, _>(
						&config,
						crate::service::build_import_queue,
					)?;

					let db = partials.backend.expose_db();
					let storage = partials.backend.expose_storage();

					cmd.run(config, partials.client.clone(), db, storage)
				}),
				#[cfg(feature = "runtime-benchmarks")]
				(BenchmarkCmd::Storage(cmd), ChainRuntime::Peregrine) => runner.sync_run(|config| {
					let partials = new_partial::<peregrine_runtime::RuntimeApi, PeregrineRuntimeExecutor, _>(
						&config,
						crate::service::build_import_queue,
					)?;

					let db = partials.backend.expose_db();
					let storage = partials.backend.expose_storage();

					cmd.run(config, partials.client.clone(), db, storage)
				}),
				(BenchmarkCmd::Overhead(_), _) => Err("Unsupported benchmarking command".into()),
				(BenchmarkCmd::Machine(cmd), _) => {
					runner.sync_run(|config| cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone()))
				}
				// NOTE: this allows the Client to leniently implement
				// new benchmark commands without requiring a companion MR.
				(_, ChainRuntime::Spiritnet) | (_, ChainRuntime::Peregrine) => {
					Err("Benchmarking sub-command unsupported".into())
				}
			}
		}
		#[cfg(feature = "try-runtime")]
		Some(Subcommand::TryRuntime(cmd)) => {
			use runtime_common::constants::MILLISECS_PER_BLOCK;
			use sc_executor::sp_wasm_interface::ExtendedHostFunctions;
			use try_runtime_cli::block_building_info::timestamp_with_aura_info;

			let runner = cli.create_runner(cmd)?;
			let registry = &runner.config().prometheus_config.as_ref().map(|cfg| &cfg.registry);
			let task_manager = polkadot_service::TaskManager::new(runner.config().tokio_handle.clone(), *registry)
				.map_err(|e| format!("Error: {:?}", e))?;
			let info_provider = timestamp_with_aura_info(MILLISECS_PER_BLOCK);

			let chain_spec_id = cmd.chain_id(cmd.is_dev()?)?;
			let runtime = chain_spec_id.parse::<ChainRuntime>()?;

			match runtime {
				ChainRuntime::Peregrine => runner.async_run(|_| {
					Ok((
						cmd.run::<Block, ExtendedHostFunctions<
							sp_io::SubstrateHostFunctions,
							<PeregrineRuntimeExecutor as NativeExecutionDispatch>::ExtendHostFunctions,
						>, _>(Some(info_provider)),
						task_manager,
					))
				}),
				ChainRuntime::Spiritnet => runner.async_run(|_| {
					Ok((
						cmd.run::<Block, ExtendedHostFunctions<
							sp_io::SubstrateHostFunctions,
							<SpiritnetRuntimeExecutor as NativeExecutionDispatch>::ExtendHostFunctions,
						>, _>(Some(info_provider)),
						task_manager,
					))
				}),
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
				let hwbench = (!cli.no_hardware_benchmarks).then_some(
					config.database.path().map(|database_path| {
						let _ = std::fs::create_dir_all(database_path);
						sc_sysinfo::gather_hwbench(Some(database_path))
					})).flatten();


				let para_id = chain_spec::Extensions::try_get(&*config.chain_spec)
					.map(|e| e.para_id)
					.ok_or("Could not find parachain ID in chain-spec.")?;

				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()]
						.iter()
						.chain(cli.relay_chain_args.iter()),
				);

				let id = ParaId::from(para_id);

				let parachain_account =
					AccountIdConversion::<polkadot_primitives::AccountId>::into_account_truncating(&id);

				let chain_spec_id = config.chain_spec.id();
				let runtime = chain_spec_id.parse::<ChainRuntime>()?;

				let state_version = runtime.native_version().state_version();
				let block: Block =
					generate_genesis_block(&*config.chain_spec, state_version).map_err(|e| format!("{:?}", e))?;
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

				if !collator_options.relay_chain_rpc_urls.len().is_zero() && !cli.relay_chain_args.len().is_zero() {
					warn!("Detected relay chain node arguments together with --relay-chain-rpc-urls. This command starts a minimal Polkadot node that only uses a network-related subset of all relay chain CLI options.");
				}

				match runtime {
					ChainRuntime::Peregrine => {
						crate::service::start_node::<PeregrineRuntimeExecutor, peregrine_runtime::RuntimeApi>(
							config,
							polkadot_config,
							collator_options,
							id,
							hwbench,
						)
						.await
						.map(|r| r.0)
						.map_err(Into::into)
					},
					ChainRuntime::Spiritnet => {
						crate::service::start_node::<SpiritnetRuntimeExecutor, spiritnet_runtime::RuntimeApi>(
							config,
							polkadot_config,
							collator_options,
							id,
							hwbench,
						)
						.await
						.map(|r| r.0)
						.map_err(Into::into)
					},
				}
			})
		}
	}
}
