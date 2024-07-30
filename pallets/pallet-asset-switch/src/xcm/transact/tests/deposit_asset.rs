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

use frame_support::{assert_noop, assert_ok, traits::fungible::Inspect as InspectFungible};
use sp_runtime::AccountId32;
use xcm::{
	v4::{Asset, AssetId, AssetInstance, Error, Fungibility, Junction, Junctions, Location},
	IntoVersion,
};
use xcm_executor::traits::TransactAsset;

use crate::{
	xcm::{
		test_utils::get_switch_pair_info_for_remote_location_with_pool_usable_balance,
		transact::mock::{
			Balances, ExtBuilder, FailingAccountIdConverter, MockRuntime, SuccessfulAccountIdConverter, System,
			SUCCESSFUL_ACCOUNT_ID,
		},
		SwitchPairRemoteAssetTransactor,
	},
	Event, SwitchPairStatus,
};

#[test]
fn successful_with_stored_remote_asset_id_latest() {
	let location = xcm::latest::Location {
		parents: 1,
		interior: xcm::latest::Junctions::X1([xcm::latest::Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info =
		{
			// Pool account balance = ED (2) + 2
			let mut new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<
				MockRuntime,
			>(&location, 2, SwitchPairStatus::Running);
			// Set remote asset to the latest XCM version.
			new_switch_pair_info.remote_asset_id = new_switch_pair_info.remote_asset_id.into_latest().unwrap();
			new_switch_pair_info
		};
	// Ignored by the mock converter logic
	let who = Location::here();

	// Works if all balance is free
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let asset_to_deposit = Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				fun: Fungibility::Fungible(2),
			};
			assert_ok!(SwitchPairRemoteAssetTransactor::<
				SuccessfulAccountIdConverter,
				MockRuntime,
				_,
			>::deposit_asset(&asset_to_deposit, &who, None));
			// Reduced by 2.
			assert_eq!(
				<Balances as InspectFungible<AccountId32>>::balance(&new_switch_pair_info.pool_account),
				2
			);
			// Destination account created and increased by 2.
			assert_eq!(
				<Balances as InspectFungible<AccountId32>>::balance(&new_switch_pair_info.pool_account),
				2
			);
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::RemoteToLocalSwitchExecuted {
					amount: 2,
					to: SUCCESSFUL_ACCOUNT_ID
				}
				.into()));
		});
	// Works if some balance is frozen, since freezes count towards ED as well.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		// We freeze 2 units for the pool account
		.with_additional_balance_entries(vec![(new_switch_pair_info.clone().pool_account, 0, 0, 2)])
		.build_and_execute_with_sanity_tests(|| {
			let asset_to_deposit = Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				fun: Fungibility::Fungible(2),
			};
			assert_ok!(SwitchPairRemoteAssetTransactor::<
				SuccessfulAccountIdConverter,
				MockRuntime,
				_,
			>::deposit_asset(&asset_to_deposit, &who, None));
			// Reduced by 2.
			assert_eq!(
				<Balances as InspectFungible<AccountId32>>::balance(&new_switch_pair_info.pool_account),
				2
			);
			// Destination account created and increased by 2.
			assert_eq!(
				<Balances as InspectFungible<AccountId32>>::balance(&new_switch_pair_info.pool_account),
				2
			);
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::RemoteToLocalSwitchExecuted {
					amount: 2,
					to: SUCCESSFUL_ACCOUNT_ID
				}
				.into()));
		});
}

#[test]
fn successful_with_stored_remote_asset_id_v4() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info =
		{
			let mut new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<
				MockRuntime,
			>(&location, 2, SwitchPairStatus::Running);
			// Set remote asset to the XCM version 3.
			new_switch_pair_info.remote_asset_id = new_switch_pair_info.remote_asset_id.into_version(3).unwrap();
			new_switch_pair_info
		};
	// Ignored by the mock converter logic
	let who = Location::here();

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let asset_to_deposit = Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				fun: Fungibility::Fungible(2),
			};
			assert_ok!(SwitchPairRemoteAssetTransactor::<
				SuccessfulAccountIdConverter,
				MockRuntime,
				_,
			>::deposit_asset(&asset_to_deposit, &who, None));
			// Reduced by 2.
			assert_eq!(
				<Balances as InspectFungible<AccountId32>>::balance(&new_switch_pair_info.pool_account),
				2
			);
			// Destination account created and increased by 2.
			assert_eq!(
				<Balances as InspectFungible<AccountId32>>::balance(&new_switch_pair_info.pool_account),
				2
			);
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::RemoteToLocalSwitchExecuted {
					amount: 2,
					to: SUCCESSFUL_ACCOUNT_ID
				}
				.into()));
		});
	// Works if some balance is frozen, since freezes count towards ED as well.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		// We freeze 2 units for the pool account
		.with_additional_balance_entries(vec![(new_switch_pair_info.clone().pool_account, 0, 0, 2)])
		.build_and_execute_with_sanity_tests(|| {
			let asset_to_deposit = Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				fun: Fungibility::Fungible(2),
			};
			assert_ok!(SwitchPairRemoteAssetTransactor::<
				SuccessfulAccountIdConverter,
				MockRuntime,
				_,
			>::deposit_asset(&asset_to_deposit, &who, None));
			// Reduced by 2.
			assert_eq!(
				<Balances as InspectFungible<AccountId32>>::balance(&new_switch_pair_info.pool_account),
				2
			);
			// Destination account created and increased by 2.
			assert_eq!(
				<Balances as InspectFungible<AccountId32>>::balance(&new_switch_pair_info.pool_account),
				2
			);
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::RemoteToLocalSwitchExecuted {
					amount: 2,
					to: SUCCESSFUL_ACCOUNT_ID
				}
				.into()));
		});
}

#[test]
fn successful_with_stored_remote_asset_id_v3() {
	let location = xcm::v3::MultiLocation {
		parents: 1,
		interior: xcm::v3::Junctions::X1(xcm::v3::Junction::Parachain(1_000)),
	};
	let new_switch_pair_info =
		{
			let mut new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<
				MockRuntime,
			>(&location.try_into().unwrap(), 2, SwitchPairStatus::Running);
			// Set remote asset to the XCM version 3.
			new_switch_pair_info.remote_asset_id = new_switch_pair_info.remote_asset_id.into_version(3).unwrap();
			new_switch_pair_info
		};
	// Ignored by the mock converter logic
	let who = Location::here();

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let asset_to_deposit = Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				fun: Fungibility::Fungible(2),
			};
			assert_ok!(SwitchPairRemoteAssetTransactor::<
				SuccessfulAccountIdConverter,
				MockRuntime,
				_,
			>::deposit_asset(&asset_to_deposit, &who, None));
			// Reduced by 2.
			assert_eq!(
				<Balances as InspectFungible<AccountId32>>::balance(&new_switch_pair_info.pool_account),
				2
			);
			// Destination account created and increased by 2.
			assert_eq!(
				<Balances as InspectFungible<AccountId32>>::balance(&new_switch_pair_info.pool_account),
				2
			);
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::RemoteToLocalSwitchExecuted {
					amount: 2,
					to: SUCCESSFUL_ACCOUNT_ID
				}
				.into()));
		});
	// Works if some balance is frozen, since freezes count towards ED as well.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		// We freeze 2 units for the pool account
		.with_additional_balance_entries(vec![(new_switch_pair_info.clone().pool_account, 0, 0, 2)])
		.build_and_execute_with_sanity_tests(|| {
			let asset_to_deposit = Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				fun: Fungibility::Fungible(2),
			};
			assert_ok!(SwitchPairRemoteAssetTransactor::<
				SuccessfulAccountIdConverter,
				MockRuntime,
				_,
			>::deposit_asset(&asset_to_deposit, &who, None));
			// Reduced by 2.
			assert_eq!(
				<Balances as InspectFungible<AccountId32>>::balance(&new_switch_pair_info.pool_account),
				2
			);
			// Destination account created and increased by 2.
			assert_eq!(
				<Balances as InspectFungible<AccountId32>>::balance(&new_switch_pair_info.pool_account),
				2
			);
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::RemoteToLocalSwitchExecuted {
					amount: 2,
					to: SUCCESSFUL_ACCOUNT_ID
				}
				.into()));
		});
}

#[test]
fn skips_on_switch_pair_not_set() {
	let who = Location::here();

	ExtBuilder::default().build_and_execute_with_sanity_tests(|| {
		let asset_to_deposit = Asset {
			id: AssetId(Junctions::Here.into()),
			fun: Fungibility::Fungible(2),
		};
		assert_noop!(
			SwitchPairRemoteAssetTransactor::<SuccessfulAccountIdConverter, MockRuntime, _>::deposit_asset(
				&asset_to_deposit,
				&who,
				None
			),
			Error::AssetNotFound
		);
	});
}

#[test]
fn skips_on_different_input_asset_id() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		2,
		SwitchPairStatus::Running,
	);
	let who = Location::here();

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build_and_execute_with_sanity_tests(|| {
			let asset_to_deposit = Asset {
				// Different than what's stored.
				id: AssetId(Location::parent()),
				fun: Fungibility::Fungible(2),
			};
			assert_noop!(
				SwitchPairRemoteAssetTransactor::<SuccessfulAccountIdConverter, MockRuntime, _>::deposit_asset(
					&asset_to_deposit,
					&who,
					None
				),
				Error::AssetNotFound
			);
		});
}

#[test]
fn skips_on_non_fungible_input_asset() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		2,
		SwitchPairStatus::Running,
	);
	let who = Location::here();

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let asset_to_deposit = Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				// Different than what's stored.
				fun: Fungibility::NonFungible(AssetInstance::Index(1)),
			};
			assert_noop!(
				SwitchPairRemoteAssetTransactor::<SuccessfulAccountIdConverter, MockRuntime, _>::deposit_asset(
					&asset_to_deposit,
					&who,
					None
				),
				Error::AssetNotFound
			);
		});
}

#[test]
fn fails_on_switch_pair_not_enabled() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		2,
		SwitchPairStatus::Paused,
	);
	let who = Location::here();

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let asset_to_deposit = Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				fun: Fungibility::Fungible(2),
			};
			assert_noop!(
				SwitchPairRemoteAssetTransactor::<SuccessfulAccountIdConverter, MockRuntime, _>::deposit_asset(
					&asset_to_deposit,
					&who,
					None
				),
				Error::FailedToTransactAsset("switch pair is not running.")
			);
		});
}

#[test]
fn fails_on_failed_account_id_conversion() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		2,
		SwitchPairStatus::Running,
	);
	let who = Location::here();

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let asset_to_deposit = Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				fun: Fungibility::Fungible(2),
			};
			assert_noop!(
				SwitchPairRemoteAssetTransactor::<FailingAccountIdConverter, MockRuntime, _>::deposit_asset(
					&asset_to_deposit,
					&who,
					None
				),
				Error::FailedToTransactAsset("Failed to convert beneficiary to valid account."),
			);
		});
}

#[test]
fn fails_on_not_enough_funds_in_pool() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		2,
		SwitchPairStatus::Running,
	);
	let who = Location::here();

	// Fails if reducible balance less than requested amount.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let asset_to_deposit = Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				// Amount bigger than the reducible balance of the pool (which is `2`).
				fun: Fungibility::Fungible(3),
			};
			assert_noop!(
				SwitchPairRemoteAssetTransactor::<SuccessfulAccountIdConverter, MockRuntime, _>::deposit_asset(
					&asset_to_deposit,
					&who,
					None
				),
				Error::FailedToTransactAsset("Failed to transfer assets from pool account to specified account.")
			);
		});
	// Fails if balance - holds less than requested amount.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.with_additional_balance_entries(vec![(new_switch_pair_info.clone().pool_account, 0, 1, 0)])
		.build_and_execute_with_sanity_tests(|| {
			let asset_to_deposit = Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				// Amount bigger than the reducible balance of the pool (which is `1`, 4 - 2 (ED) - 1 (hold)).
				fun: Fungibility::Fungible(2),
			};
			assert_noop!(
				SwitchPairRemoteAssetTransactor::<SuccessfulAccountIdConverter, MockRuntime, _>::deposit_asset(
					&asset_to_deposit,
					&who,
					None
				),
				Error::FailedToTransactAsset("Failed to transfer assets from pool account to specified account.")
			);
		});
	// Fails if freezes are higher than the requested amount.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		// Freezes do not reduce the reducible balance if they are less than ED.
		.with_additional_balance_entries(vec![(new_switch_pair_info.clone().pool_account, 0, 0, 3)])
		.build_and_execute_with_sanity_tests(|| {
			let asset_to_deposit = Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				// Amount bigger than the reducible balance of the pool (which is `1`, 4 - 2 (ED) - 1 (freeze beyond
				// ED)).
				fun: Fungibility::Fungible(2),
			};
			assert_noop!(
				SwitchPairRemoteAssetTransactor::<SuccessfulAccountIdConverter, MockRuntime, _>::deposit_asset(
					&asset_to_deposit,
					&who,
					None
				),
				Error::FailedToTransactAsset("Failed to transfer assets from pool account to specified account.")
			);
		});
}

#[test]
fn fails_on_amount_below_ed() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		2,
		SwitchPairStatus::Running,
	);
	let who = Location::here();

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let asset_to_deposit = Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				// ED is 2, anything that would not result in the account having at least ED will fail.
				fun: Fungibility::Fungible(1),
			};
			assert_noop!(
				SwitchPairRemoteAssetTransactor::<SuccessfulAccountIdConverter, MockRuntime, _>::deposit_asset(
					&asset_to_deposit,
					&who,
					None
				),
				Error::FailedToTransactAsset("Failed to transfer assets from pool account to specified account."),
			);
		});
}
