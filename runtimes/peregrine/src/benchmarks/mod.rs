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

use frame_benchmarking::define_benchmarks;

pub(crate) mod asset_switch;
pub(crate) mod web3_names;

/// Workaround for a bug in the benchmarking code around instances.
/// Upstream fix PR: https://github.com/paritytech/polkadot-sdk/pull/6435
#[allow(unused_imports)]
use pallet_collective as pallet_technical_committee_collective;
#[allow(unused_imports)]
use pallet_did_lookup as pallet_unique_linking;
#[allow(unused_imports)]
use pallet_membership as pallet_technical_membership;
#[allow(unused_imports)]
use pallet_web3_names as pallet_dot_names;

define_benchmarks!(
	[frame_system, SystemBench::<Runtime>]
	[pallet_timestamp, Timestamp]
	[pallet_indices, Indices]
	[pallet_balances, Balances]
	[pallet_session, SessionBench::<Runtime>]
	[parachain_staking, ParachainStaking]
	[pallet_democracy, Democracy]
	[pallet_treasury, Treasury]
	[pallet_sudo, Sudo]
	[pallet_utility, Utility]
	[pallet_vesting, Vesting]
	[pallet_scheduler, Scheduler]
	[pallet_proxy, Proxy]
	[pallet_preimage, Preimage]
	[pallet_tips, Tips]
	[pallet_multisig, Multisig]
	[ctype, Ctype]
	[attestation, Attestation]
	[delegation, Delegation]
	[did, Did]
	[pallet_inflation, Inflation]
	[public_credentials, PublicCredentials]
	[pallet_xcm, PalletXcmExtrinsicsBenchmark::<Runtime>]
	[pallet_migration, Migration]
	[pallet_dip_provider, DipProvider]
	[pallet_deposit_storage, DepositStorage]
	[pallet_asset_switch, AssetSwitchPool1]
	[pallet_assets, Fungibles]
	[pallet_message_queue, MessageQueue]
	[cumulus_pallet_parachain_system, ParachainSystem]
	[frame_benchmarking::baseline, Baseline::<Runtime>]
	// pallet_collective instances
	[pallet_collective, Council]
	[pallet_technical_committee_collective, TechnicalCommittee]
	// pallet_membership instances
	[pallet_membership, TipsMembership]
	[pallet_technical_membership, TechnicalMembership]
	// pallet_did_lookup instances
	[pallet_did_lookup, DidLookup]
	[pallet_unique_linking, UniqueLinking]
	// pallet_web3_names instances
	[pallet_web3_names, Web3Names]
	[pallet_dot_names, DotNames]
);
