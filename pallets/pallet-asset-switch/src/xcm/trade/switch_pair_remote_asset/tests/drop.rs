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

use frame_support::{assert_storage_noop, traits::fungible::Inspect as InspectFungible};
use sp_core::Get;
use sp_runtime::{
	traits::{One, Zero},
	AccountId32,
};
use xcm::v3::{Junction, Junctions, MultiLocation};
use xcm_executor::traits::WeightTrader;

use crate::xcm::{
	trade::{
		switch_pair_remote_asset::mock::{
			get_switch_pair_info_for_remote_location, Balances, ExtBuilder, MockRuntime, ToDestinationAccount,
		},
		test_utils::SumTimeAndProofValues,
	},
	UsingComponentsForSwitchPairRemoteAsset,
};

#[test]
fn happy_path() {
	let location = MultiLocation {
		parents: 1,
		interior: Junctions::X1(Junction::Parachain(1_000)),
	};
	// ED + 1
	let new_switch_pair_info = get_switch_pair_info_for_remote_location(&location, 1);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let weigher = {
				let mut weigher = UsingComponentsForSwitchPairRemoteAsset::<
					MockRuntime,
					_,
					SumTimeAndProofValues,
					ToDestinationAccount,
				>::new();
				weigher.remaining_fungible_balance = 1;
				weigher
			};
			assert!(<Balances as InspectFungible<AccountId32>>::balance(&ToDestinationAccount::get()).is_zero());
			drop(weigher);
			assert!(<Balances as InspectFungible<AccountId32>>::balance(&ToDestinationAccount::get()).is_one());
			assert!(<Balances as InspectFungible<AccountId32>>::balance(&new_switch_pair_info.pool_account).is_one());
		});
}

#[test]
fn no_switch_pair() {
	ExtBuilder::default().build_and_execute_with_sanity_tests(|| {
		let weigher = {
			let mut weigher = UsingComponentsForSwitchPairRemoteAsset::<
				MockRuntime,
				_,
				SumTimeAndProofValues,
				ToDestinationAccount,
			>::new();
			weigher.remaining_fungible_balance = 1;
			weigher
		};
		assert_storage_noop!(drop(weigher));
	});
}

#[test]
fn zero_remaining_balance() {
	let location = MultiLocation {
		parents: 1,
		interior: Junctions::X1(Junction::Parachain(1_000)),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location(&location, 0);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build_and_execute_with_sanity_tests(|| {
			let weigher = {
				let mut weigher = UsingComponentsForSwitchPairRemoteAsset::<
					MockRuntime,
					_,
					SumTimeAndProofValues,
					ToDestinationAccount,
				>::new();
				weigher.remaining_fungible_balance = u128::zero();
				weigher
			};
			assert_storage_noop!(drop(weigher));
		});
}

#[test]
#[should_panic(expected = "Transferring from pool account to fee destination failed.")]
fn fail_to_transfer_from_pool_account() {
	// Same setup as the happy path, minus the balance set for the pool.
	let location = MultiLocation {
		parents: 1,
		interior: Junctions::X1(Junction::Parachain(1_000)),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location(&location, 0);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let weigher = {
				let mut weigher = UsingComponentsForSwitchPairRemoteAsset::<
					MockRuntime,
					_,
					SumTimeAndProofValues,
					ToDestinationAccount,
				>::new();
				weigher.remaining_fungible_balance = 1;
				weigher
			};
			assert!(<Balances as InspectFungible<AccountId32>>::balance(&ToDestinationAccount::get()).is_zero());
			drop(weigher);
			assert!(<Balances as InspectFungible<AccountId32>>::balance(&ToDestinationAccount::get()).is_one());
			assert!(<Balances as InspectFungible<AccountId32>>::balance(&new_switch_pair_info.pool_account).is_zero());
		});
}
