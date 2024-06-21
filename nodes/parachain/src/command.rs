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
use log::info;
use parity_scale_codec::Encode;
use runtime_common::Block;
use sc_cli::SubstrateCli;
use sc_executor::NativeExecutionDispatch;
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::traits::{AccountIdConversion, Block as BlockT};

use crate::{
	chain_spec::{self, ParachainRuntime},
	cli::{Cli, RelayChainCli, Subcommand},
	service::{new_partial, PeregrineRuntimeExecutor, SpiritnetRuntimeExecutor},
};

// Returns the provided (`--chain`, <selected_runtime>) given only a reference
// to the global `Cli` object.

fn get_selected_chainspec(params: &sc_cli::SharedParams) -> Result<(String, ParachainRuntime), sc_cli::Error> {
	let chain_id = params.chain_id(params.is_dev());
	let runtime = chain_id.parse::<ParachainRuntime>().map_err(sc_cli::Error::Input)?;
	Ok((chain_id, runtime))
}

macro_rules! construct_async_run {
	(|$components:ident, $cli:ident, $cmd:ident, $config:ident| $( $code:tt )* ) => {{
		let (_, runtime) = get_selected_chainspec(&$cli.run.base.shared_params)?;
		let runner = $cli.create_runner($cmd)?;

		match runtime {
			ParachainRuntime::Spiritnet(_) => {
				runner.async_run(|$config| {
					let $components = new_partial::<spiritnet_runtime::RuntimeApi, SpiritnetRuntimeExecutor, _>(
						&$config,
						crate::service::build_import_queue::<SpiritnetRuntimeExecutor, spiritnet_runtime::RuntimeApi>,
					)?;
					let task_manager = $components.task_manager;
					{ $( $code )* }.map(|v| (v, task_manager))
				})
			},
			ParachainRuntime::Peregrine(_) => {
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
			let (chain_spec_id, runtime) = get_selected_chainspec(&cmd.shared_params)?;
			let spec = cli.load_spec(chain_spec_id.as_str())?;

			println!("Dispatching task for spec id: {chain_spec_id}.");
			println!("The following runtime was chosen based on the spec id: {runtime}.");

			let runner = cli.create_runner(cmd)?;

			match runtime {
				ParachainRuntime::Spiritnet(_) => runner.sync_run(|config| {
					let partials = new_partial::<spiritnet_runtime::RuntimeApi, SpiritnetRuntimeExecutor, _>(
						&config,
						crate::service::build_import_queue,
					)?;
					cmd.run::<Block>(&*spec, &*partials.client)
				}),
				ParachainRuntime::Peregrine(_) => runner.sync_run(|config| {
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
				let (chain_spec_id, _) = get_selected_chainspec(&cmd.shared_params)?;
				let spec = cli.load_spec(chain_spec_id.as_str())?;

				cmd.run(&*spec)
			})
		}
		Some(Subcommand::Benchmark(cmd)) => {

			let shared_params = match cmd {
				BenchmarkCmd::Block(c) => &c.shared_params,
				BenchmarkCmd::Pallet(c) => &c.shared_params,
				BenchmarkCmd::Extrinsic(c) => &c.shared_params,
				BenchmarkCmd::Machine(c) => &c.shared_params,
				BenchmarkCmd::Overhead(c) => &c.shared_params,
				BenchmarkCmd::Storage(c) => &c.shared_params,
			};

			let (_, runtime) = get_selected_chainspec(shared_params)?;

			let runner = cli.create_runner(cmd)?;

			match (cmd, runtime) {
				(BenchmarkCmd::Pallet(cmd), ParachainRuntime::Spiritnet(_)) => {
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
				(BenchmarkCmd::Pallet(cmd), ParachainRuntime::Peregrine(_)) => {
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
				(BenchmarkCmd::Block(cmd), ParachainRuntime::Spiritnet(_)) => runner.sync_run(|config| {
					let partials = new_partial::<spiritnet_runtime::RuntimeApi, SpiritnetRuntimeExecutor, _>(
						&config,
						crate::service::build_import_queue,
					)?;
					cmd.run(partials.client)
				}),
				(BenchmarkCmd::Block(cmd), ParachainRuntime::Peregrine(_)) => runner.sync_run(|config| {
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
				(BenchmarkCmd::Storage(cmd), ParachainRuntime::Spiritnet(_)) => runner.sync_run(|config| {
					let partials = new_partial::<spiritnet_runtime::RuntimeApi, SpiritnetRuntimeExecutor, _>(
						&config,
						crate::service::build_import_queue,
					)?;

					let db = partials.backend.expose_db();
					let storage = partials.backend.expose_storage();

					cmd.run(config, partials.client.clone(), db, storage)
				}),
				#[cfg(feature = "runtime-benchmarks")]
				(BenchmarkCmd::Storage(cmd), ParachainRuntime::Peregrine(_)) => runner.sync_run(|config| {
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
				(_, ParachainRuntime::Spiritnet(_)) | (_, ParachainRuntime::Peregrine(_)) => {
					Err("Benchmarking sub-command unsupported".into())
				}
			}
		}
		Some(Subcommand::TryRuntime) => Err("The `try-runtime` subcommand has been migrated to a standalone CLI (https://github.com/paritytech/try-runtime-cli). It is no longer being maintained here.".into()),
		None => {
			let runner = cli.create_runner(&cli.run.normalize())?;
			let collator_options = cli.run.collator_options();

			runner.run_node_until_exit(|config| async move {
				let hwbench = (!cli.no_hardware_benchmarks)
					.then_some(config.database.path().map(|database_path| {
						let _ = std::fs::create_dir_all(database_path);
						sc_sysinfo::gather_hwbench(Some(database_path))
					}))
					.flatten();

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

				let (_, runtime) = get_selected_chainspec(&cli.run.base.shared_params)?;

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

				match runtime {
					ParachainRuntime::Peregrine(_) => {
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
					ParachainRuntime::Spiritnet(_) => {
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
