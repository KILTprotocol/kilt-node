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

// If you feel like getting in touch with us, you can do so at <hello@kilt.org>

use frame_support::{
	assert_storage_noop,
	traits::{
		fungible::Inspect,
		tokens::{Fortitude, Preservation},
	},
};
use sp_runtime::{traits::Zero, AccountId32};
use xcm::v4::{Asset, Junction, Junctions, Location, Response, Weight, XcmContext};
use xcm_executor::traits::OnResponse;

use crate::{
	switch::UnconfirmedSwitchInfo,
	xcm::{
		query::mock::{Balances, ExtBuilder, MockRuntime, System, ACCOUNT_0},
		test_utils::get_switch_pair_info_for_remote_location_with_pool_usable_balance,
	},
	Event, Pallet, PendingSwitchConfirmations, SwitchPair, SwitchPairInfo, SwitchPairStatus,
};

#[test]
fn successful_storage_clean_up_on_transfer_successful_and_running_pair() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		100,
		SwitchPairStatus::Running,
	);

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.with_pending_switches(vec![(
			0,
			UnconfirmedSwitchInfo {
				from: ACCOUNT_0,
				to: Junctions::Here.into_location().into_versioned(),
				amount: 10,
			},
		)])
		.build_and_execute_with_sanity_tests(|| {
			Pallet::<MockRuntime>::on_response(
				&location,
				0,
				Some(&Junctions::Here.into_location()),
				// No assets
				Response::Assets(vec![].into()),
				Weight::zero(),
				&XcmContext::with_message_id([0; 32]),
			);
			let pool_balance_after = <Balances as Inspect<AccountId32>>::reducible_balance(
				&new_switch_pair_info.pool_account,
				Preservation::Preserve,
				Fortitude::Polite,
			);
			let switch_info_before = SwitchPairInfo::from_input_unchecked(new_switch_pair_info.clone());
			let switch_info_after = SwitchPair::<MockRuntime>::get().unwrap();

			assert_eq!(pool_balance_after, 100);
			assert!(<Balances as Inspect<AccountId32>>::balance(&ACCOUNT_0).is_zero());
			assert_eq!(switch_info_before, switch_info_after);
			// The pending switch is removed from storage.
			assert!(!PendingSwitchConfirmations::<MockRuntime>::contains_key(0));
			// The right event is generated.
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::LocalToRemoteSwitchFinalized {
					from: ACCOUNT_0,
					to: Junctions::Here.into_location().into_versioned(),
					amount: 10
				}
				.into()));
		});

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.with_pending_switches(vec![(
			0,
			UnconfirmedSwitchInfo {
				from: ACCOUNT_0,
				to: Junctions::Here.into_location().into_versioned(),
				amount: 10,
			},
		)])
		.build_and_execute_with_sanity_tests(|| {
			Pallet::<MockRuntime>::on_response(
				&location,
				0,
				Some(&Junctions::Here.into_location()),
				// Other assets
				Response::Assets(
					Asset {
						id: Junctions::Here.into(),
						fun: 10.into(),
					}
					.into(),
				),
				Weight::zero(),
				&XcmContext::with_message_id([0; 32]),
			);
			let pool_balance_after = <Balances as Inspect<AccountId32>>::reducible_balance(
				&new_switch_pair_info.pool_account,
				Preservation::Preserve,
				Fortitude::Polite,
			);
			let switch_info_before = SwitchPairInfo::from_input_unchecked(new_switch_pair_info.clone());
			let switch_info_after = SwitchPair::<MockRuntime>::get().unwrap();

			assert_eq!(pool_balance_after, 100);
			assert!(<Balances as Inspect<AccountId32>>::balance(&ACCOUNT_0).is_zero());
			assert_eq!(switch_info_before, switch_info_after);
			// The pending switch is removed from storage.
			assert!(!PendingSwitchConfirmations::<MockRuntime>::contains_key(0));
			// The right event is generated.
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::LocalToRemoteSwitchFinalized {
					from: ACCOUNT_0,
					to: Junctions::Here.into_location().into_versioned(),
					amount: 10
				}
				.into()));
		});
}

#[test]
fn successful_storage_clean_up_on_transfer_successful_and_paused_pair() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		100,
		SwitchPairStatus::Paused,
	);

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.with_pending_switches(vec![(
			0,
			UnconfirmedSwitchInfo {
				from: ACCOUNT_0,
				to: Junctions::Here.into_location().into_versioned(),
				amount: 10,
			},
		)])
		.build_and_execute_with_sanity_tests(|| {
			Pallet::<MockRuntime>::on_response(
				&location,
				0,
				Some(&Junctions::Here.into_location()),
				// No assets
				Response::Assets(vec![].into()),
				Weight::zero(),
				&XcmContext::with_message_id([0; 32]),
			);
			let pool_balance_after = <Balances as Inspect<AccountId32>>::reducible_balance(
				&new_switch_pair_info.pool_account,
				Preservation::Preserve,
				Fortitude::Polite,
			);
			let switch_info_before = SwitchPairInfo::from_input_unchecked(new_switch_pair_info.clone());
			let switch_info_after = SwitchPair::<MockRuntime>::get().unwrap();

			assert_eq!(pool_balance_after, 100);
			assert!(<Balances as Inspect<AccountId32>>::balance(&ACCOUNT_0).is_zero());
			assert_eq!(switch_info_before, switch_info_after);
			// The pending switch is removed from storage.
			assert!(!PendingSwitchConfirmations::<MockRuntime>::contains_key(0));
			// The right event is generated.
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::LocalToRemoteSwitchFinalized {
					from: ACCOUNT_0,
					to: Junctions::Here.into_location().into_versioned(),
					amount: 10
				}
				.into()));
		});

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.with_pending_switches(vec![(
			0,
			UnconfirmedSwitchInfo {
				from: ACCOUNT_0,
				to: Junctions::Here.into_location().into_versioned(),
				amount: 10,
			},
		)])
		.build_and_execute_with_sanity_tests(|| {
			Pallet::<MockRuntime>::on_response(
				&location,
				0,
				Some(&Junctions::Here.into_location()),
				// Other assets
				Response::Assets(
					Asset {
						id: Junctions::Here.into(),
						fun: 10.into(),
					}
					.into(),
				),
				Weight::zero(),
				&XcmContext::with_message_id([0; 32]),
			);
			let pool_balance_after = <Balances as Inspect<AccountId32>>::reducible_balance(
				&new_switch_pair_info.pool_account,
				Preservation::Preserve,
				Fortitude::Polite,
			);
			let switch_info_before = SwitchPairInfo::from_input_unchecked(new_switch_pair_info.clone());
			let switch_info_after = SwitchPair::<MockRuntime>::get().unwrap();

			assert_eq!(pool_balance_after, 100);
			assert!(<Balances as Inspect<AccountId32>>::balance(&ACCOUNT_0).is_zero());
			assert_eq!(switch_info_before, switch_info_after);
			// The pending switch is removed from storage.
			assert!(!PendingSwitchConfirmations::<MockRuntime>::contains_key(0));
			// The right event is generated.
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::LocalToRemoteSwitchFinalized {
					from: ACCOUNT_0,
					to: Junctions::Here.into_location().into_versioned(),
					amount: 10
				}
				.into()));
		});
}

#[test]
fn successful_revert_on_transfer_revert_and_running_pair() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		100,
		SwitchPairStatus::Running,
	);

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.with_pending_switches(vec![(
			0,
			UnconfirmedSwitchInfo {
				from: ACCOUNT_0,
				to: Junctions::Here.into_location().into_versioned(),
				amount: 10,
			},
		)])
		.build_and_execute_with_sanity_tests(|| {
			Pallet::<MockRuntime>::on_response(
				&location,
				0,
				Some(&Junctions::Here.into_location()),
				Response::Assets(
					// We put only the recognized asset in here.
					Asset {
						id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
						fun: 10.into(),
					}
					.into(),
				),
				Weight::zero(),
				&XcmContext::with_message_id([0; 32]),
			);
			let pool_balance_after = <Balances as Inspect<AccountId32>>::reducible_balance(
				&new_switch_pair_info.pool_account,
				Preservation::Preserve,
				Fortitude::Polite,
			);
			let switch_info_before = SwitchPairInfo::from_input_unchecked(new_switch_pair_info.clone());
			let switch_info_after = SwitchPair::<MockRuntime>::get().unwrap();

			assert_eq!(pool_balance_after, 90);
			assert_eq!(<Balances as Inspect<AccountId32>>::balance(&ACCOUNT_0), 10);
			assert_eq!(
				switch_info_after.reducible_remote_balance() - switch_info_before.reducible_remote_balance(),
				10
			);
			assert_eq!(
				switch_info_before.remote_asset_circulating_supply - switch_info_after.remote_asset_circulating_supply,
				10
			);
			// The pending switch is removed from storage.
			assert!(!PendingSwitchConfirmations::<MockRuntime>::contains_key(0));
			// The revert event is generated
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::LocalToRemoteSwitchReverted {
					from: ACCOUNT_0,
					to: Junctions::Here.into_location().into_versioned(),
					amount: 10
				}
				.into()));
		});

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.with_pending_switches(vec![(
			0,
			UnconfirmedSwitchInfo {
				from: ACCOUNT_0,
				to: Junctions::Here.into_location().into_versioned(),
				amount: 10,
			},
		)])
		.build_and_execute_with_sanity_tests(|| {
			Pallet::<MockRuntime>::on_response(
				&location,
				0,
				Some(&Junctions::Here.into_location()),
				Response::Assets(
					// We put also a different asset in here, which will be ignored.
					vec![
						Asset {
							id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
							fun: 10.into(),
						},
						Asset {
							id: Junctions::Here.into(),
							fun: 10.into(),
						},
					]
					.into(),
				),
				Weight::zero(),
				&XcmContext::with_message_id([0; 32]),
			);
			let pool_balance_after = <Balances as Inspect<AccountId32>>::reducible_balance(
				&new_switch_pair_info.pool_account,
				Preservation::Preserve,
				Fortitude::Polite,
			);
			let switch_info_before = SwitchPairInfo::from_input_unchecked(new_switch_pair_info.clone());
			let switch_info_after = SwitchPair::<MockRuntime>::get().unwrap();

			assert_eq!(pool_balance_after, 90);
			assert_eq!(<Balances as Inspect<AccountId32>>::balance(&ACCOUNT_0), 10);
			assert_eq!(
				switch_info_after.reducible_remote_balance() - switch_info_before.reducible_remote_balance(),
				10
			);
			assert_eq!(
				switch_info_before.remote_asset_circulating_supply - switch_info_after.remote_asset_circulating_supply,
				10
			);
			// The pending switch is removed from storage.
			assert!(!PendingSwitchConfirmations::<MockRuntime>::contains_key(0));
			// The revert event is generated
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::LocalToRemoteSwitchReverted {
					from: ACCOUNT_0,
					to: Junctions::Here.into_location().into_versioned(),
					amount: 10
				}
				.into()));
		});
}

#[test]
fn successful_revert_on_transfer_revert_and_paused_pair() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		100,
		SwitchPairStatus::Paused,
	);

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.with_pending_switches(vec![(
			0,
			UnconfirmedSwitchInfo {
				from: ACCOUNT_0,
				to: Junctions::Here.into_location().into_versioned(),
				amount: 10,
			},
		)])
		.build_and_execute_with_sanity_tests(|| {
			Pallet::<MockRuntime>::on_response(
				&location,
				0,
				Some(&Junctions::Here.into_location()),
				Response::Assets(
					// We put only the recognized asset in here.
					Asset {
						id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
						fun: 10.into(),
					}
					.into(),
				),
				Weight::zero(),
				&XcmContext::with_message_id([0; 32]),
			);
			let pool_balance_after = <Balances as Inspect<AccountId32>>::reducible_balance(
				&new_switch_pair_info.pool_account,
				Preservation::Preserve,
				Fortitude::Polite,
			);
			let switch_info_before = SwitchPairInfo::from_input_unchecked(new_switch_pair_info.clone());
			let switch_info_after = SwitchPair::<MockRuntime>::get().unwrap();

			assert_eq!(pool_balance_after, 90);
			assert_eq!(<Balances as Inspect<AccountId32>>::balance(&ACCOUNT_0), 10);
			assert_eq!(
				switch_info_after.reducible_remote_balance() - switch_info_before.reducible_remote_balance(),
				10
			);
			assert_eq!(
				switch_info_before.remote_asset_circulating_supply - switch_info_after.remote_asset_circulating_supply,
				10
			);
			// The pending switch is removed from storage.
			assert!(!PendingSwitchConfirmations::<MockRuntime>::contains_key(0));
			// The revert event is generated
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::LocalToRemoteSwitchReverted {
					from: ACCOUNT_0,
					to: Junctions::Here.into_location().into_versioned(),
					amount: 10
				}
				.into()));
		});

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.with_pending_switches(vec![(
			0,
			UnconfirmedSwitchInfo {
				from: ACCOUNT_0,
				to: Junctions::Here.into_location().into_versioned(),
				amount: 10,
			},
		)])
		.build_and_execute_with_sanity_tests(|| {
			Pallet::<MockRuntime>::on_response(
				&location,
				0,
				Some(&Junctions::Here.into_location()),
				Response::Assets(
					// We put also a different asset in here, which will be ignored.
					vec![
						Asset {
							id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
							fun: 10.into(),
						},
						Asset {
							id: Junctions::Here.into(),
							fun: 10.into(),
						},
					]
					.into(),
				),
				Weight::zero(),
				&XcmContext::with_message_id([0; 32]),
			);
			let pool_balance_after = <Balances as Inspect<AccountId32>>::reducible_balance(
				&new_switch_pair_info.pool_account,
				Preservation::Preserve,
				Fortitude::Polite,
			);
			let switch_info_before = SwitchPairInfo::from_input_unchecked(new_switch_pair_info.clone());
			let switch_info_after = SwitchPair::<MockRuntime>::get().unwrap();

			assert_eq!(pool_balance_after, 90);
			assert_eq!(<Balances as Inspect<AccountId32>>::balance(&ACCOUNT_0), 10);
			assert_eq!(
				switch_info_after.reducible_remote_balance() - switch_info_before.reducible_remote_balance(),
				10
			);
			assert_eq!(
				switch_info_before.remote_asset_circulating_supply - switch_info_after.remote_asset_circulating_supply,
				10
			);
			// The pending switch is removed from storage.
			assert!(!PendingSwitchConfirmations::<MockRuntime>::contains_key(0));
			// The revert event is generated
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::LocalToRemoteSwitchReverted {
					from: ACCOUNT_0,
					to: Junctions::Here.into_location().into_versioned(),
					amount: 10
				}
				.into()));
		});
}

#[test]
fn fail_on_invalid_origin() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		100,
		SwitchPairStatus::Paused,
	);

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.with_pending_switches(vec![(
			0,
			UnconfirmedSwitchInfo {
				from: ACCOUNT_0,
				to: Junctions::Here.into_location().into_versioned(),
				amount: 10,
			},
		)])
		.build_and_execute_with_sanity_tests(|| {
			assert_storage_noop!(Pallet::<MockRuntime>::on_response(
				&Junction::Parachain(1001).into(),
				0,
				Some(&Junctions::Here.into_location()),
				Response::Assets(
					Asset {
						id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
						fun: 10.into(),
					}
					.into(),
				),
				Weight::zero(),
				&XcmContext::with_message_id([0; 32]),
			));
		});
}

#[test]
fn fail_on_query_id_not_found() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		100,
		SwitchPairStatus::Paused,
	);

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.with_pending_switches(vec![(
			0,
			UnconfirmedSwitchInfo {
				from: ACCOUNT_0,
				to: Junctions::Here.into_location().into_versioned(),
				amount: 10,
			},
		)])
		.build_and_execute_with_sanity_tests(|| {
			assert_storage_noop!(Pallet::<MockRuntime>::on_response(
				&location,
				1,
				Some(&Junctions::Here.into_location()),
				Response::Assets(
					Asset {
						id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
						fun: 10.into(),
					}
					.into(),
				),
				Weight::zero(),
				&XcmContext::with_message_id([0; 32]),
			));
		});
}

#[test]
fn fail_on_invalid_querier() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		100,
		SwitchPairStatus::Paused,
	);

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.with_pending_switches(vec![(
			0,
			UnconfirmedSwitchInfo {
				from: ACCOUNT_0,
				to: Junctions::Here.into_location().into_versioned(),
				amount: 10,
			},
		)])
		.build_and_execute_with_sanity_tests(|| {
			assert_storage_noop!(Pallet::<MockRuntime>::on_response(
				&location,
				0,
				None,
				Response::Assets(
					Asset {
						id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
						fun: 10.into(),
					}
					.into(),
				),
				Weight::zero(),
				&XcmContext::with_message_id([0; 32]),
			));
		});
}

#[test]
fn fail_on_invalid_response_type() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		100,
		SwitchPairStatus::Paused,
	);

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.with_pending_switches(vec![(
			0,
			UnconfirmedSwitchInfo {
				from: ACCOUNT_0,
				to: Junctions::Here.into_location().into_versioned(),
				amount: 10,
			},
		)])
		.build_and_execute_with_sanity_tests(|| {
			assert_storage_noop!(Pallet::<MockRuntime>::on_response(
				&location,
				0,
				Some(&Junctions::Here.into_location()),
				Response::ExecutionResult(None),
				Weight::zero(),
				&XcmContext::with_message_id([0; 32]),
			));
		});
}

#[test]
fn fail_on_invalid_assets() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		100,
		SwitchPairStatus::Paused,
	);

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.with_pending_switches(vec![(
			0,
			UnconfirmedSwitchInfo {
				from: ACCOUNT_0,
				to: Junctions::Here.into_location().into_versioned(),
				amount: 10,
			},
		)])
		.build_and_execute_with_sanity_tests(|| {
			assert_storage_noop!(Pallet::<MockRuntime>::on_response(
				&location,
				0,
				Some(&Junctions::Here.into_location()),
				// Very same asset ID, but with one less in terms of amount, should still fail.
				Response::Assets(
					Asset {
						id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
						fun: 9.into(),
					}
					.into(),
				),
				Weight::zero(),
				&XcmContext::with_message_id([0; 32]),
			));
		});

	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.with_pending_switches(vec![(
			0,
			UnconfirmedSwitchInfo {
				from: ACCOUNT_0,
				to: Junctions::Here.into_location().into_versioned(),
				amount: 10,
			},
		)])
		.build_and_execute_with_sanity_tests(|| {
			assert_storage_noop!(Pallet::<MockRuntime>::on_response(
				&location,
				0,
				Some(&Junctions::Here.into_location()),
				// Same for one more.
				Response::Assets(
					Asset {
						id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
						fun: 11.into(),
					}
					.into(),
				),
				Weight::zero(),
				&XcmContext::with_message_id([0; 32]),
			));
		});
}

#[test]
fn fail_on_switch_pair_not_present() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};

	ExtBuilder::default()
		.with_pending_switches(vec![(
			0,
			UnconfirmedSwitchInfo {
				from: ACCOUNT_0,
				to: Junctions::Here.into_location().into_versioned(),
				amount: 10,
			},
		)])
		.build_and_execute_with_sanity_tests(|| {
			assert_storage_noop!(Pallet::<MockRuntime>::on_response(
				&location,
				0,
				Some(&Junctions::Here.into_location()),
				Response::Assets(vec![].into()),
				Weight::zero(),
				&XcmContext::with_message_id([0; 32]),
			));
		});
}
