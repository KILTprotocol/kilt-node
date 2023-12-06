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

use std::path::PathBuf;

use cumulus_client_cli::{ExportGenesisStateCommand, ExportGenesisWasmCommand, PurgeChainCmd};
use polkadot_cli::RunCmd;
use sc_cli::{BuildSpecCmd, CheckBlockCmd, ExportBlocksCmd, ExportStateCmd, ImportBlocksCmd, RevertCmd};
use sc_service::Configuration;

use crate::chain_spec::Extensions;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
	BuildSpec(BuildSpecCmd),

	CheckBlock(CheckBlockCmd),

	ExportBlocks(ExportBlocksCmd),

	ExportState(ExportStateCmd),

	ImportBlocks(ImportBlocksCmd),

	Revert(RevertCmd),

	PurgeChain(PurgeChainCmd),

	ExportGenesisState(ExportGenesisStateCommand),

	ExportGenesisWasm(ExportGenesisWasmCommand),
	#[command(subcommand)]
	Benchmark(frame_benchmarking_cli::BenchmarkCmd),
}

#[derive(Debug, clap::Parser)]
#[command(
	propagate_version = true,
	args_conflicts_with_subcommands = true,
	subcommand_negates_reqs = true
)]
pub struct Cli {
	#[command(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[command(flatten)]
	pub run: cumulus_client_cli::RunCmd,

	#[arg(long)]
	pub no_hardware_benchmarks: bool,

	#[arg(raw = true)]
	pub relay_chain_args: Vec<String>,
}

#[derive(Debug)]
pub struct RelayChainCli {
	pub base: RunCmd,

	pub chain_id: Option<String>,

	pub base_path: Option<PathBuf>,
}

impl RelayChainCli {
	pub fn new<'a>(para_config: &Configuration, relay_chain_args: impl Iterator<Item = &'a String>) -> Self {
		let extension = Extensions::try_get(&*para_config.chain_spec);
		let chain_id = extension.map(|e| e.relay_chain.clone());
		let base_path = Some(para_config.base_path.path().join("polkadot"));
		Self {
			base_path,
			chain_id,
			base: clap::Parser::parse_from(relay_chain_args),
		}
	}
}
