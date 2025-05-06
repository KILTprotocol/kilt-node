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

use frame_benchmarking::{
	define_benchmarks, whitelisted_caller, BenchmarkBatch, BenchmarkConfig, BenchmarkError, BenchmarkList, Benchmarking,
};
use frame_support::{
	pallet_prelude::ValueQuery,
	storage::generator::StorageValue as _,
	storage_alias,
	traits::{Currency, StorageInfo, StorageInfoTrait},
};
use frame_system::{pallet_prelude::BlockNumberFor, EventRecord, Phase};
use runtime_common::{
	benchmarks::xcm::{NativeAsset, ParachainDeliveryHelper, ParachainLocation, RandomParaId},
	constants::EXISTENTIAL_DEPOSIT,
	Hash,
};
use sp_core::storage::TrackedStorageKey;
use sp_runtime::RuntimeString;
use sp_std::{boxed::Box, vec, vec::Vec};
use xcm::v4::{Asset, Assets, Fungibility, Location};

use crate::{
	xcm::XcmConfig, AllPalletsWithSystem, AssetSwitchPool1, Attestation, Balances, BondedCurrencies, BondedFungibles,
	Collators, Council, Ctype, Delegation, Democracy, DepositStorage, Did, DidLookup, DipProvider, Fungibles, Indices,
	MessageQueue, Migration, Multisig, ParachainStaking, ParachainSystem, Preimage, Proxy, PublicCredentials, Runtime,
	RuntimeEvent, Scheduler, Sudo, System, TechnicalCommittee, TechnicalMembership, Timestamp, Tips, TipsMembership,
	Treasury, Utility, Vesting, Web3Names,
};

pub(crate) mod asset_switch;
pub(crate) mod bonded_coins;
pub(crate) mod governance;
pub(crate) mod web3_names;

/// Workaround for a bug in the benchmarking code around instances.
/// Upstream fix PR: https://github.com/paritytech/polkadot-sdk/pull/6435
#[allow(unused_imports)]
use pallet_assets as pallet_bonded_assets;
#[allow(unused_imports)]
use pallet_collective as pallet_technical_committee_collective;
#[allow(unused_imports)]
use pallet_membership as pallet_technical_membership;
#[allow(unused_imports)]
use pallet_membership as pallet_collators;

define_benchmarks!(
	[frame_system, frame_system_benchmarking::Pallet::<Runtime>]
	[pallet_timestamp, Timestamp]
	[pallet_indices, Indices]
	[pallet_balances, Balances]
	[pallet_session, cumulus_pallet_session_benchmarking::Pallet::<Runtime>]
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
	[public_credentials, PublicCredentials]
	[pallet_xcm, pallet_xcm::benchmarking::Pallet::<Runtime>]
	[pallet_migration, Migration]
	[pallet_dip_provider, DipProvider]
	[pallet_deposit_storage, DepositStorage]
	[pallet_asset_switch, AssetSwitchPool1]
	[pallet_message_queue, MessageQueue]
	[pallet_bonded_coins, BondedCurrencies]
	[cumulus_pallet_parachain_system, ParachainSystem]
	[frame_benchmarking::baseline, frame_benchmarking::baseline::Pallet::<Runtime>]
	// pallet_collective instances
	[pallet_collective, Council]
	[pallet_technical_committee_collective, TechnicalCommittee]
	// pallet_membership instances
	[pallet_membership, TipsMembership]
	[pallet_technical_membership, TechnicalMembership]
	[pallet_collators, Collators]
	// pallet_did_lookup instances
	[pallet_did_lookup, DidLookup]
	// pallet_web3_names instances
	[pallet_web3_names, Web3Names]
	// pallet assets instances
	[pallet_assets, Fungibles]
	[pallet_bonded_assets, BondedFungibles]
);

impl pallet_xcm::benchmarking::Config for Runtime {
	type DeliveryHelper = ParachainDeliveryHelper<ParachainSystem, XcmConfig>;

	fn reachable_dest() -> Option<Location> {
		ParachainSystem::open_outbound_hrmp_channel_for_benchmarks_or_tests(RandomParaId::get());
		Some(ParachainLocation::get())
	}

	fn reserve_transferable_asset_and_dest() -> Option<(Asset, Location)> {
		ParachainSystem::open_outbound_hrmp_channel_for_benchmarks_or_tests(RandomParaId::get());
		Some((NativeAsset::get(), ParachainLocation::get()))
	}

	fn set_up_complex_asset_transfer() -> Option<(Assets, u32, Location, Box<dyn FnOnce()>)> {
		let (transferable_asset, dest) = Self::reserve_transferable_asset_and_dest().unwrap();

		let fee_amount = EXISTENTIAL_DEPOSIT;
		let fee_asset: Asset = (Location::here(), fee_amount).into();

		// Make account free to pay the fee
		let who = whitelisted_caller();
		let balance = fee_amount + EXISTENTIAL_DEPOSIT * 1000;
		let _ = <Balances as Currency<_>>::make_free_balance_be(&who, balance);

		// verify initial balance
		assert_eq!(Balances::free_balance(&who), balance);

		let assets: Assets = vec![fee_asset.clone(), transferable_asset.clone()].into();
		let fee_index = if assets.get(0).unwrap().eq(&fee_asset) { 0 } else { 1 };

		let verify = Box::new(move || {
			let Fungibility::Fungible(transferable_amount) = transferable_asset.fun else {
				return;
			};
			assert_eq!(Balances::free_balance(&who), balance - fee_amount - transferable_amount);
		});

		Some((assets, fee_index, dest, verify))
	}

	fn get_asset() -> Asset {
		NativeAsset::get()
	}
}

impl frame_system_benchmarking::Config for Runtime {
	fn setup_set_code_requirements(code: &sp_std::vec::Vec<u8>) -> Result<(), BenchmarkError> {
		ParachainSystem::initialize_for_set_code_benchmark(code.len() as u32);
		Ok(())
	}

	fn verify_set_code() {
		System::assert_last_event(cumulus_pallet_parachain_system::Event::<Runtime>::ValidationFunctionStored.into());
	}
}

impl cumulus_pallet_session_benchmarking::Config for Runtime {}
impl frame_benchmarking::baseline::Config for Runtime {}

pub(crate) fn benchmark_metadata(extra: bool) -> (Vec<BenchmarkList>, Vec<StorageInfo>) {
	let mut list = Vec::<BenchmarkList>::new();
	list_benchmarks!(list, extra);

	let storage_info = AllPalletsWithSystem::storage_info();
	(list, storage_info)
}

pub(crate) fn dispatch_benchmark(config: BenchmarkConfig) -> Result<Vec<BenchmarkBatch>, RuntimeString> {
	#[storage_alias]
	type Number = StorageValue<System, BlockNumberFor<Runtime>, ValueQuery>;
	#[storage_alias]
	type ExecutionPhase = StorageValue<System, Phase>;
	#[storage_alias]
	type EventCount = StorageValue<System, u32, ValueQuery>;
	#[storage_alias]
	type Events = StorageValue<System, Vec<Box<EventRecord<RuntimeEvent, Hash>>>, ValueQuery>;

	let whitelist: Vec<TrackedStorageKey> = vec![
		// Block Number
		Number::storage_value_final_key().to_vec().into(),
		// Total Issuance
		pallet_balances::TotalIssuance::<Runtime>::storage_value_final_key()
			.to_vec()
			.into(),
		// Execution Phase
		ExecutionPhase::storage_value_final_key().to_vec().into(),
		// Event Count
		EventCount::storage_value_final_key().to_vec().into(),
		// System Events
		Events::storage_value_final_key().to_vec().into(),
	];

	let mut batches = Vec::<BenchmarkBatch>::new();
	let params = (&config, &whitelist);

	add_benchmarks!(params, batches);

	if batches.is_empty() {
		return Err("Benchmark not found for this pallet.".into());
	}
	Ok(batches)
}
