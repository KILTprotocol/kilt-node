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

use frame_system::RawOrigin;
use runtime_common::AccountId;
use xcm::v4::{Asset, AssetId, Fungibility, Junction, Junctions, Location, ParentThen};

frame_support::parameter_types! {
	pub const MaxBalance: crate::Balance = crate::Balance::max_value();
}

use pallet_asset_switch::PartialBenchmarkInfo;
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

use crate::{kilt::DotNamesDeployment, Fungibles, ParachainSystem, Runtime};

frame_benchmarking::define_benchmarks!(
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

// Required since the pallet `AssetTransactor` will try to deduct the XCM fee
// from the user's balance, and the asset must exist.
pub struct CreateFungibleForAssetSwitchPool1;

impl pallet_asset_switch::BenchmarkHelper for CreateFungibleForAssetSwitchPool1 {
	fn setup() -> Option<PartialBenchmarkInfo> {
		const DESTINATION_PARA_ID: u32 = 1_000;

		let asset_location: Location = Junctions::Here.into();
		Fungibles::create(
			RawOrigin::Root.into(),
			asset_location.clone(),
			AccountId::from([0; 32]).into(),
			1u32.into(),
		)
		.unwrap();
		let beneficiary = Junctions::X1(
			[Junction::AccountId32 {
				network: None,
				id: [0; 32],
			}]
			.into(),
		)
		.into();
		let destination = Location::from(ParentThen(Junctions::X1(
			[Junction::Parachain(DESTINATION_PARA_ID)].into(),
		)))
		.into();
		let remote_xcm_fee = Asset {
			id: AssetId(asset_location),
			fun: Fungibility::Fungible(1_000),
		}
		.into();

		ParachainSystem::open_outbound_hrmp_channel_for_benchmarks_or_tests(DESTINATION_PARA_ID.into());

		Some(PartialBenchmarkInfo {
			beneficiary: Some(beneficiary),
			destination: Some(destination),
			remote_asset_id: None,
			remote_xcm_fee: Some(remote_xcm_fee),
		})
	}
}

pub struct Web3NamesBenchmarkHelper;

impl pallet_web3_names::BenchmarkHelper for Web3NamesBenchmarkHelper {
	fn generate_name_input_with_length(length: usize) -> Vec<u8> {
		let input = vec![b'a'; length];

		debug_assert!(<Runtime as pallet_web3_names::Config<()>>::Web3Name::try_from(input.clone()).is_ok());
		input
	}
}

pub struct DotNamesBenchmarkHelper;

impl pallet_web3_names::BenchmarkHelper for DotNamesBenchmarkHelper {
	// Returns the name `11[...]111.dot` with as many `1`s as the provided length -
	// 4, to account for the ".dot" suffix.
	fn generate_name_input_with_length(length: usize) -> Vec<u8> {
		let suffix_length = runtime_common::constants::dot_names::DOT_NAME_SUFFIX.len();
		let remaining_name_length = length
			.checked_sub(suffix_length)
			.expect("Provided length should cover at least the length of the suffix.");
		let input = vec![b'1'; remaining_name_length]
			.into_iter()
			.chain(runtime_common::constants::dot_names::DOT_NAME_SUFFIX.bytes())
			.collect::<Vec<_>>();

		debug_assert!(
			<Runtime as pallet_web3_names::Config<DotNamesDeployment>>::Web3Name::try_from(input.clone()).is_ok()
		);
		input
	}
}
