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

use sp_std::sync::Arc;
use xcm::v4::{Junction, Junctions, Location, Parent, ParentThen};
use xcm_executor::traits::OnResponse;

use crate::{
	switch::UnconfirmedSwitchInfo,
	xcm::{
		query::mock::{ExtBuilder, MockRuntime, ACCOUNT_0},
		test_utils::get_switch_pair_info_for_remote_location_with_pool_usable_balance,
	},
	Pallet, SwitchPairStatus,
};

#[test]
fn origin_checks_with_running_pair() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		100,
		SwitchPairStatus::Running,
	);

	// Same location as configured works.
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
			assert!(Pallet::<MockRuntime>::expecting_response(
				&location,
				0,
				Some(&Junctions::Here.into_location())
			))
		});
	// Parent location of configured works.
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
			assert!(Pallet::<MockRuntime>::expecting_response(
				&Parent.into(),
				0,
				Some(&Junctions::Here.into_location())
			))
		});
	// Descendent location of configured does not work.
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
			assert!(!Pallet::<MockRuntime>::expecting_response(
				&location
					.pushed_with_interior(Junction::AccountId32 {
						network: None,
						id: ACCOUNT_0.into()
					})
					.unwrap(),
				0,
				Some(&Junctions::Here.into_location())
			))
		});
	// Different location does not work.
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
			assert!(!Pallet::<MockRuntime>::expecting_response(
				&ParentThen(Junctions::X1(Arc::new([Junction::Parachain(1_001)]))).into(),
				0,
				Some(&Junctions::Here.into_location())
			))
		});
}

#[test]
fn origin_checks_with_paused_pair() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		100,
		SwitchPairStatus::Paused,
	);

	// Same location as configured works.
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
			assert!(Pallet::<MockRuntime>::expecting_response(
				&location,
				0,
				Some(&Junctions::Here.into_location())
			))
		});
	// Parent location of configured works.
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
			assert!(Pallet::<MockRuntime>::expecting_response(
				&Parent.into(),
				0,
				Some(&Junctions::Here.into_location())
			))
		});
	// Descendent location of configured does not work.
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
			assert!(!Pallet::<MockRuntime>::expecting_response(
				&location
					.pushed_with_interior(Junction::AccountId32 {
						network: None,
						id: ACCOUNT_0.into()
					})
					.unwrap(),
				0,
				Some(&Junctions::Here.into_location())
			))
		});
	// Different location does not work.
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
			assert!(!Pallet::<MockRuntime>::expecting_response(
				&ParentThen(Junctions::X1(Arc::new([Junction::Parachain(1_001)]))).into(),
				0,
				Some(&Junctions::Here.into_location())
			))
		});
}

#[test]
fn querier_checks_with_running_pair() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		100,
		SwitchPairStatus::Running,
	);

	// Same querier works.
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
			assert!(Pallet::<MockRuntime>::expecting_response(
				&location,
				0,
				Some(&Junctions::Here.into_location())
			))
		});
	// None querier does not work.
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
			assert!(!Pallet::<MockRuntime>::expecting_response(&location, 0, None))
		});
	// Parent querier does not work.
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
			assert!(!Pallet::<MockRuntime>::expecting_response(
				&location,
				0,
				Some(&Parent.into())
			))
		});
	// Nested querier does not work.
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
			assert!(!Pallet::<MockRuntime>::expecting_response(
				&location,
				0,
				Some(
					&Junctions::Here
						.pushed_with(Junction::AccountId32 {
							network: None,
							id: ACCOUNT_0.into()
						})
						.unwrap()
						.into_location()
				),
			))
		});
	// Different querier does not work.
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
			assert!(!Pallet::<MockRuntime>::expecting_response(
				&location,
				0,
				Some(&ParentThen(Junctions::X1(Arc::new([Junction::Parachain(1_001)]))).into()),
			))
		});
}

#[test]
fn querier_checks_with_paused_pair() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		100,
		SwitchPairStatus::Paused,
	);

	// Same querier works.
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
			assert!(Pallet::<MockRuntime>::expecting_response(
				&location,
				0,
				Some(&Junctions::Here.into_location())
			))
		});
	// None querier does not work.
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
			assert!(!Pallet::<MockRuntime>::expecting_response(&location, 0, None))
		});
	// Parent querier does not work.
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
			assert!(!Pallet::<MockRuntime>::expecting_response(
				&location,
				0,
				Some(&Parent.into())
			))
		});
	// Nested querier does not work.
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
			assert!(!Pallet::<MockRuntime>::expecting_response(
				&location,
				0,
				Some(
					&Junctions::Here
						.pushed_with(Junction::AccountId32 {
							network: None,
							id: ACCOUNT_0.into()
						})
						.unwrap()
						.into_location()
				),
			))
		});
	// Different querier does not work.
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
			assert!(!Pallet::<MockRuntime>::expecting_response(
				&location,
				0,
				Some(&ParentThen(Junctions::X1(Arc::new([Junction::Parachain(1_001)]))).into()),
			))
		});
}

#[test]
fn switch_pair_not_set() {
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
			assert!(!Pallet::<MockRuntime>::expecting_response(
				&Junctions::Here.into_location(),
				0,
				Some(&Junctions::Here.into_location())
			))
		});
}

#[test]
fn query_id_not_found() {
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
			assert!(!Pallet::<MockRuntime>::expecting_response(
				&location,
				1,
				Some(&Junctions::Here.into_location())
			))
		});
}
