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

use clap::Parser;
use std::{ops::Deref, path::PathBuf};

pub(crate) const DEFAULT_RUNTIME: &str = "peregrine";

/// Sub-commands supported by the collator.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
	/// Build a chain specification.
	BuildSpec(BuildSpecCmd),

	/// Validate blocks.
	CheckBlock(sc_cli::CheckBlockCmd),

	/// Export blocks.
	ExportBlocks(sc_cli::ExportBlocksCmd),

	/// Export the state of a given block into a chain spec.
	ExportState(sc_cli::ExportStateCmd),

	/// Import blocks.
	ImportBlocks(sc_cli::ImportBlocksCmd),

	/// Revert the chain to a previous state.
	Revert(sc_cli::RevertCmd),

	/// Remove the whole chain.
	PurgeChain(cumulus_client_cli::PurgeChainCmd),

	/// Export the genesis state of the parachain.
	ExportGenesisState(cumulus_client_cli::ExportGenesisStateCommand),

	/// Export the genesis wasm of the parachain.
	ExportGenesisWasm(cumulus_client_cli::ExportGenesisWasmCommand),

	/// Sub-commands concerned with benchmarking.
	/// The pallet benchmarking moved to the `pallet` sub-command.
	#[command(subcommand)]
	Benchmark(frame_benchmarking_cli::BenchmarkCmd),

	/// Try some command against runtime state.
	#[cfg(feature = "try-runtime")]
	TryRuntime(try_runtime_cli::TryRuntimeCmd),

	/// Try some command against runtime state. Note: `try-runtime` feature must
	/// be enabled.
	#[cfg(not(feature = "try-runtime"))]
	TryRuntime,
}

/// Command for building the genesis state of the parachain
#[derive(Debug, Parser)]
pub(crate) struct BuildSpecCmd {
	#[command(flatten)]
	pub(crate) inner_args: sc_cli::BuildSpecCmd,

	/// The name of the runtime which should get executed.
	#[arg(long, default_value = DEFAULT_RUNTIME)]
	pub(crate) runtime: String,
}

impl Deref for BuildSpecCmd {
	type Target = sc_cli::BuildSpecCmd;

	fn deref(&self) -> &Self::Target {
		&self.inner_args
	}
}

#[derive(Debug, clap::Parser)]
#[command(
	propagate_version = true,
	args_conflicts_with_subcommands = true,
	subcommand_negates_reqs = true
)]
pub(crate) struct Cli {
	#[command(subcommand)]
	pub(crate) subcommand: Option<Subcommand>,

	#[command(flatten)]
	pub(crate) run: cumulus_client_cli::RunCmd,

	// Disable automatic hardware benchmarks.
	///
	/// By default these benchmarks are automatically ran at startup and measure
	/// the CPU speed, the memory bandwidth and the disk speed.
	///
	/// The results are then printed out in the logs, and also sent as part of
	/// telemetry, if telemetry is enabled.
	#[arg(long)]
	pub no_hardware_benchmarks: bool,

	/// The name of the runtime which should get executed.
	#[arg(long, default_value = DEFAULT_RUNTIME)]
	pub(crate) runtime: String,

	/// Relaychain arguments
	#[arg(raw = true)]
	pub(crate) relay_chain_args: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct RelayChainCli {
	/// The actual relay chain cli object.
	pub(crate) base: polkadot_cli::RunCmd,

	/// Optional chain id that should be passed to the relay chain.
	pub(crate) chain_id: Option<String>,

	/// The base path that should be used by the relay chain.
	pub(crate) base_path: Option<PathBuf>,
}

impl RelayChainCli {
	/// Parse the relay chain CLI parameters using the para chain
	/// `Configuration`.
	pub(crate) fn new<'a>(
		para_config: &sc_service::Configuration,
		relay_chain_args: impl Iterator<Item = &'a String>,
	) -> Self {
		let extension = crate::chain_spec::Extensions::try_get(&*para_config.chain_spec);
		let chain_id = extension.map(|e| e.relay_chain.clone());
		let base_path = Some(para_config.base_path.path().join("polkadot"));
		Self {
			base_path,
			chain_id,
			base: clap::Parser::parse_from(relay_chain_args),
		}
	}
}
