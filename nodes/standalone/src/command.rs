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
	chain_spec,
	cli::{Cli, Subcommand},
	service,
};
use frame_benchmarking_cli::BenchmarkCmd;
use mashnet_node_runtime::opaque::Block;
use sc_cli::{ChainSpec, RuntimeVersion, SubstrateCli};
use sc_service::PartialComponents;

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"KILT Node".to_string()
	}

	fn impl_version() -> String {
		env!("CARGO_PKG_VERSION").to_string()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").to_string()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").to_string()
	}

	fn support_url() -> String {
		"https://github.com/KILTprotocol/mashnet-node/issues/new".to_string()
	}

	fn copyright_start_year() -> i32 {
		2019
	}

	fn executable_name() -> String {
		env!("CARGO_PKG_NAME").to_string()
	}

	fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
		chain_spec::load_spec(id)
	}

	fn native_runtime_version(_: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		&mashnet_node_runtime::VERSION
	}
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		}
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents {
					client,
					task_manager,
					import_queue,
					..
				} = service::new_partial(&config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents {
					client, task_manager, ..
				} = service::new_partial(&config)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		}
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents {
					client, task_manager, ..
				} = service::new_partial(&config)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		}
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents {
					client,
					task_manager,
					import_queue,
					..
				} = service::new_partial(&config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.database))
		}
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents {
					client,
					task_manager,
					backend,
					..
				} = service::new_partial(&config)?;
				let aux_revert = Box::new(move |client, _, blocks| {
					sc_finality_grandpa::revert(client, blocks)?;
					Ok(())
				});
				Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
			})
		}
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| {
				let PartialComponents { client, backend, .. } = service::new_partial(&config)?;

				// This switch needs to be in the client, since the client decides
				// which sub-commands it wants to support.
				match cmd {
					BenchmarkCmd::Pallet(cmd) => {
						if !cfg!(feature = "runtime-benchmarks") {
							return Err("Runtime benchmarking wasn't enabled when building the node. \
							You can enable it with `--features runtime-benchmarks`."
								.into());
						}

						cmd.run::<Block, service::ExecutorDispatch>(config)
					}
					BenchmarkCmd::Block(cmd) => cmd.run(client),
					BenchmarkCmd::Storage(cmd) => {
						let db = backend.expose_db();
						let storage = backend.expose_storage();

						cmd.run(config, client, db, storage)
					}
					BenchmarkCmd::Overhead(_) => Err("Unsupported benchmarking command".into()),
				}
			})
		}
		#[cfg(feature = "try-runtime")]
		Some(Subcommand::TryRuntime(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				// we don't need any of the components of new_partial, just a runtime, or a task
				// manager to do `async_run`.
				let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
				let task_manager = sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
					.map_err(|e| sc_cli::Error::Service(sc_service::Error::Prometheus(e)))?;

				Ok((cmd.run::<Block, service::ExecutorDispatch>(config), task_manager))
			})
		}
		#[cfg(not(feature = "try-runtime"))]
		Some(Subcommand::TryRuntime) => Err("TryRuntime wasn't enabled when building the node. \
				You can enable it with `--features try-runtime`."
			.into()),
		None => {
			let runner = cli.create_runner(&cli.run)?;
			runner
				.run_node_until_exit(|config| async move { service::new_full(config).map_err(sc_cli::Error::Service) })
		}
	}
}
