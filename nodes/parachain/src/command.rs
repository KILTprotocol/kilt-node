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
use cumulus_primitives_core::ParaId;
use frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE;
use log::info;
use sc_cli::SubstrateCli;
use sp_runtime::traits::AccountIdConversion;
use std::iter::once;

use crate::{
	chain_spec::{self, ParachainRuntime},
	cli::{Cli, RelayChainCli, Subcommand},
	service::new_partial,
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
					let $components = new_partial::<spiritnet_runtime::RuntimeApi, _>(
						&$config,
						crate::service::build_import_queue::<spiritnet_runtime::RuntimeApi>,
					)?;
					let task_manager = $components.task_manager;
					{ $( $code )* }.map(|v| (v, task_manager))
				})
			},
			ParachainRuntime::Peregrine(_) => {
				runner.async_run(|$config| {
					let $components = new_partial::<peregrine_runtime::RuntimeApi, _>(
						&$config,
						crate::service::build_import_queue::<peregrine_runtime::RuntimeApi>,
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
	let mut cli = Cli::from_args();

	// all full nodes should store request/responses, otherwise they'd basically be
	// useless without it. https://docs.hyperbridge.network/developers/polkadot/pallet-ismp#offchain-indexing
	cli.run.base.offchain_worker_params.indexing_enabled = true;

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
					once(&RelayChainCli::executable_name())
						.chain(cli.relay_chain_args.iter()),
				);

				let polkadot_config =
					SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, config.tokio_handle.clone())
						.map_err(|err| format!("Relay chain argument error: {}", err))?;

				cmd.run(config, polkadot_config)
			})
		}
		Some(Subcommand::ExportGenesisHead(cmd)) => {
			let (_, runtime) = get_selected_chainspec(&cmd.shared_params)?;

			let runner = cli.create_runner(cmd)?;

			match runtime {
				ParachainRuntime::Spiritnet(_) => runner.sync_run(|config| {
					let partials = new_partial::<spiritnet_runtime::RuntimeApi, _>(
						&config,
						crate::service::build_import_queue,
					)?;

					cmd.run(partials.client)
				}),
				ParachainRuntime::Peregrine(_) => runner.sync_run(|config| {
					let partials = new_partial::<peregrine_runtime::RuntimeApi, _>(
						&config,
						crate::service::build_import_queue,
					)?;
					cmd.run(partials.client)
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
		Some(Subcommand::Benchmark(_)) => Err("The `benchmark` subcommand has been migrated to a standalone CLI (https://crates.io/crates/frame-omni-bencher). It is no longer being maintained here.".into()), 
		Some(Subcommand::TryRuntime) => Err("The `try-runtime` subcommand has been migrated to a standalone CLI (https://github.com/paritytech/try-runtime-cli). It is no longer being maintained here.".into()),
		None => {
			let runner = cli.create_runner(&cli.run.normalize())?;
			let collator_options = cli.run.collator_options();

			runner.run_node_until_exit(|config| async move {
				let hwbench = (!cli.no_hardware_benchmarks)
					.then_some(config.database.path().map(|database_path| {
						let _ = std::fs::create_dir_all(database_path);
						sc_sysinfo::gather_hwbench(Some(database_path) , &SUBSTRATE_REFERENCE_HARDWARE)
					}))
					.flatten();

				let para_id = chain_spec::Extensions::try_get(&*config.chain_spec)
					.map(|e| e.para_id)
					.ok_or("Could not find parachain ID in chain-spec.")?;

				let polkadot_cli = RelayChainCli::new(
					&config,
					once(&RelayChainCli::executable_name())
						.chain(cli.relay_chain_args.iter()),
				);

				let id = ParaId::from(para_id);

				let parachain_account =
					AccountIdConversion::<polkadot_primitives::AccountId>::into_account_truncating(&id);

				let (_, runtime) = get_selected_chainspec(&cli.run.base.shared_params)?;

				let tokio_handle = config.tokio_handle.clone();
				let polkadot_config = SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, tokio_handle)
					.map_err(|err| format!("Relay chain argument error: {}", err))?;

				info!("Parachain id: {:?}", id);
				info!("Parachain Account: {}", parachain_account);
				info!(
					"Is collating: {}",
					if config.role.is_authority() { "yes" } else { "no" }
				);

				match runtime {
					ParachainRuntime::Peregrine(_) => {
						crate::service::start_node::<peregrine_runtime::RuntimeApi>(
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
						crate::service::start_node::<spiritnet_runtime::RuntimeApi>(
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
