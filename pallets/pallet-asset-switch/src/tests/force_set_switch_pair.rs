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

use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use sp_runtime::{
	traits::{One, Zero},
	AccountId32, DispatchError,
};

use crate::{
	mock::{get_asset_hub_location, get_remote_erc20_asset_id, ExtBuilder, MockRuntime, System, XCM_ASSET_FEE},
	switch::{SwitchPairStatus, UnconfirmedSwitchInfo},
	Event, NewSwitchPairInfoOf, Pallet, SwitchPair, SwitchPairInfoOf,
};

#[test]
fn successful() {
	let pool_account_address =
		Pallet::<MockRuntime>::pool_account_id_for_remote_asset(&get_remote_erc20_asset_id().into()).unwrap();
	// Case where all issuance is circulating supply requires the same balance (+ED)
	// for the pool account
	ExtBuilder::default()
		.with_balances(vec![(pool_account_address.clone(), u64::MAX, 0, 0)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(Pallet::<MockRuntime>::force_set_switch_pair(
				RawOrigin::Root.into(),
				u64::MAX as u128,
				Box::new(get_remote_erc20_asset_id().into()),
				// Need to leave 1 on this chain for ED, so `MAX - 1` can at most be exchanged back (and transferred
				// out from the pool account).
				(u64::MAX - 1) as u128,
				Box::new(get_asset_hub_location().into()),
				0,
				Box::new(XCM_ASSET_FEE.into()),
			));

			let switch_pair = SwitchPair::<MockRuntime>::get();
			let expected_switch_pair =
				SwitchPairInfoOf::<MockRuntime>::from_input_unchecked(NewSwitchPairInfoOf::<MockRuntime> {
					pool_account: pool_account_address.clone(),
					remote_asset_circulating_supply: (u64::MAX - 1) as u128,
					remote_asset_ed: 0,
					remote_asset_id: get_remote_erc20_asset_id().into(),
					remote_asset_total_supply: u64::MAX as u128,
					remote_reserve_location: get_asset_hub_location().into(),
					remote_xcm_fee: XCM_ASSET_FEE.into(),
					status: SwitchPairStatus::Paused,
				});
			assert_eq!(switch_pair, Some(expected_switch_pair));
			// Unit balance since we had to leave ED on this chain and no min balance
			// requirement (ED) is set for the remote asset.
			assert!(switch_pair.unwrap().reducible_remote_balance().is_one());
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::SwitchPairCreated {
					pool_account: pool_account_address.clone(),
					remote_asset_circulating_supply: (u64::MAX - 1) as u128,
					remote_asset_ed: 0,
					remote_asset_total_supply: u64::MAX as u128,
					remote_asset_id: get_remote_erc20_asset_id().into(),
					remote_reserve_location: get_asset_hub_location().into(),
					remote_xcm_fee: Box::new(XCM_ASSET_FEE.into())
				}
				.into()));
		});
	// Case where all issuance is locked and controlled by our sovereign account.
	ExtBuilder::default()
		.with_balances(vec![(pool_account_address.clone(), 1, 0, 0)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(Pallet::<MockRuntime>::force_set_switch_pair(
				RawOrigin::Root.into(),
				u64::MAX as u128,
				Box::new(get_remote_erc20_asset_id().into()),
				0,
				Box::new(get_asset_hub_location().into()),
				0,
				Box::new(XCM_ASSET_FEE.into()),
			));

			let switch_pair = SwitchPair::<MockRuntime>::get();
			let expected_switch_pair =
				SwitchPairInfoOf::<MockRuntime>::from_input_unchecked(NewSwitchPairInfoOf::<MockRuntime> {
					pool_account: pool_account_address.clone(),
					remote_asset_circulating_supply: 0,
					remote_asset_ed: 0,
					remote_asset_id: get_remote_erc20_asset_id().into(),
					remote_asset_total_supply: u64::MAX as u128,
					remote_reserve_location: get_asset_hub_location().into(),
					remote_xcm_fee: XCM_ASSET_FEE.into(),
					status: SwitchPairStatus::Paused,
				});
			assert_eq!(switch_pair, Some(expected_switch_pair));
			// Max balance since all circulating supply is controlled by us.
			assert_eq!(switch_pair.unwrap().reducible_remote_balance(), u64::MAX as u128);
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::SwitchPairCreated {
					pool_account: pool_account_address.clone(),
					remote_asset_circulating_supply: 0,
					remote_asset_ed: 0,
					remote_asset_total_supply: u64::MAX as u128,
					remote_asset_id: get_remote_erc20_asset_id().into(),
					remote_reserve_location: get_asset_hub_location().into(),
					remote_xcm_fee: Box::new(XCM_ASSET_FEE.into())
				}
				.into()));
		});
	// Case where all issuance is circulating supply and there's a min balance >=
	// `0` on the remote chain requires the same balance (+ED) for the pool account,
	// and the remote balance is calculated accordingly.
	ExtBuilder::default()
		.with_balances(vec![(pool_account_address.clone(), u64::MAX, 0, 0)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(Pallet::<MockRuntime>::force_set_switch_pair(
				RawOrigin::Root.into(),
				u64::MAX as u128,
				Box::new(get_remote_erc20_asset_id().into()),
				(u64::MAX - 1) as u128,
				Box::new(get_asset_hub_location().into()),
				1,
				Box::new(XCM_ASSET_FEE.into()),
			));

			let switch_pair = SwitchPair::<MockRuntime>::get();
			let expected_switch_pair =
				SwitchPairInfoOf::<MockRuntime>::from_input_unchecked(NewSwitchPairInfoOf::<MockRuntime> {
					pool_account: pool_account_address.clone(),
					remote_asset_circulating_supply: (u64::MAX - 1) as u128,
					remote_asset_ed: 1,
					remote_asset_id: get_remote_erc20_asset_id().into(),
					remote_asset_total_supply: u64::MAX as u128,
					remote_reserve_location: get_asset_hub_location().into(),
					remote_xcm_fee: XCM_ASSET_FEE.into(),
					status: SwitchPairStatus::Paused,
				});
			assert_eq!(switch_pair, Some(expected_switch_pair));
			// Zero balance since we everything but the required remote asset ED is
			// circulating.
			assert!(switch_pair.unwrap().reducible_remote_balance().is_zero());
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::SwitchPairCreated {
					pool_account: pool_account_address.clone(),
					remote_asset_circulating_supply: (u64::MAX - 1) as u128,
					remote_asset_ed: 1,
					remote_asset_total_supply: u64::MAX as u128,
					remote_asset_id: get_remote_erc20_asset_id().into(),
					remote_reserve_location: get_asset_hub_location().into(),
					remote_xcm_fee: Box::new(XCM_ASSET_FEE.into())
				}
				.into()));
		});
	// Case where all issuance is locked and controlled by our sovereign account,
	// but there's a min balance >= `0` on the remote chain.
	ExtBuilder::default()
		.with_balances(vec![(pool_account_address.clone(), 1, 0, 0)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(Pallet::<MockRuntime>::force_set_switch_pair(
				RawOrigin::Root.into(),
				u64::MAX as u128,
				Box::new(get_remote_erc20_asset_id().into()),
				0,
				Box::new(get_asset_hub_location().into()),
				1,
				Box::new(XCM_ASSET_FEE.into()),
			));

			let switch_pair = SwitchPair::<MockRuntime>::get();
			let expected_switch_pair =
				SwitchPairInfoOf::<MockRuntime>::from_input_unchecked(NewSwitchPairInfoOf::<MockRuntime> {
					pool_account: pool_account_address.clone(),
					remote_asset_circulating_supply: 0,
					remote_asset_ed: 1,
					remote_asset_id: get_remote_erc20_asset_id().into(),
					remote_asset_total_supply: u64::MAX as u128,
					remote_reserve_location: get_asset_hub_location().into(),
					remote_xcm_fee: XCM_ASSET_FEE.into(),
					status: SwitchPairStatus::Paused,
				});
			assert_eq!(switch_pair, Some(expected_switch_pair));
			// We cannot go below `1` on the remote chain, so of all the locked assets we
			// control, we can only exchange all but one.
			assert_eq!(switch_pair.unwrap().reducible_remote_balance(), (u64::MAX - 1) as u128);
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::SwitchPairCreated {
					pool_account: pool_account_address.clone(),
					remote_asset_circulating_supply: 0,
					remote_asset_ed: 1,
					remote_asset_total_supply: u64::MAX as u128,
					remote_asset_id: get_remote_erc20_asset_id().into(),
					remote_reserve_location: get_asset_hub_location().into(),
					remote_xcm_fee: Box::new(XCM_ASSET_FEE.into())
				}
				.into()));
		});
}

#[test]
fn successful_overwrites_existing_pool() {
	let pool_account_address =
		Pallet::<MockRuntime>::pool_account_id_for_remote_asset(&get_remote_erc20_asset_id().into()).unwrap();
	ExtBuilder::default()
		.with_balances(vec![(pool_account_address.clone(), 1, 0, 0)])
		.with_switch_pair_info(NewSwitchPairInfoOf::<MockRuntime> {
			pool_account: [0u8; 32].into(),
			remote_asset_circulating_supply: 0,
			remote_asset_ed: 0,
			remote_asset_id: get_remote_erc20_asset_id().into(),
			remote_asset_total_supply: 1_000,
			remote_reserve_location: get_asset_hub_location().into(),
			remote_xcm_fee: XCM_ASSET_FEE.into(),
			status: Default::default(),
		})
		.build()
		// We ignore try-runtime tests here since we are testing the breaking of invariants.
		.execute_with(|| {
			assert_ok!(Pallet::<MockRuntime>::force_set_switch_pair(
				RawOrigin::Root.into(),
				100_000,
				Box::new(get_remote_erc20_asset_id().into()),
				50_000,
				Box::new(get_asset_hub_location().into()),
				0,
				Box::new(XCM_ASSET_FEE.into()),
			));

			let switch_pair = SwitchPair::<MockRuntime>::get();
			let expected_switch_pair =
				SwitchPairInfoOf::<MockRuntime>::from_input_unchecked(NewSwitchPairInfoOf::<MockRuntime> {
					pool_account: pool_account_address.clone(),
					remote_asset_circulating_supply: 50_000,
					remote_asset_ed: 0,
					remote_asset_id: get_remote_erc20_asset_id().into(),
					remote_asset_total_supply: 100_000,
					remote_reserve_location: get_asset_hub_location().into(),
					remote_xcm_fee: XCM_ASSET_FEE.into(),
					status: SwitchPairStatus::Paused,
				});
			assert_eq!(switch_pair, Some(expected_switch_pair));
			assert_eq!(switch_pair.unwrap().reducible_remote_balance(), 50_000);
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::SwitchPairCreated {
					pool_account: pool_account_address.clone(),
					remote_asset_circulating_supply: 50_000,
					remote_asset_ed: 0,
					remote_asset_total_supply: 100_000,
					remote_asset_id: get_remote_erc20_asset_id().into(),
					remote_reserve_location: get_asset_hub_location().into(),
					remote_xcm_fee: Box::new(XCM_ASSET_FEE.into())
				}
				.into()));
		});
}

#[test]
fn success_with_pending_switches() {
	let pool_account_address =
		Pallet::<MockRuntime>::pool_account_id_for_remote_asset(&get_remote_erc20_asset_id().into()).unwrap();
	ExtBuilder::default()
		.with_balances(vec![(pool_account_address.clone(), u64::MAX, 0, 0)])
		.with_pending_switches(vec![(
			0,
			UnconfirmedSwitchInfo {
				amount: 10,
				from: AccountId32::new([100; 32]),
				to: get_asset_hub_location().into_versioned(),
			},
		)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(Pallet::<MockRuntime>::force_set_switch_pair(
				RawOrigin::Root.into(),
				u64::MAX as u128,
				Box::new(get_remote_erc20_asset_id().into()),
				// Need to leave 1 on this chain for ED, so `MAX - 1` can at most be exchanged back (and transferred
				// out from the pool account).
				(u64::MAX - 1) as u128,
				Box::new(get_asset_hub_location().into()),
				0,
				Box::new(XCM_ASSET_FEE.into()),
			));

			let switch_pair = SwitchPair::<MockRuntime>::get();
			let expected_switch_pair =
				SwitchPairInfoOf::<MockRuntime>::from_input_unchecked(NewSwitchPairInfoOf::<MockRuntime> {
					pool_account: pool_account_address.clone(),
					remote_asset_circulating_supply: (u64::MAX - 1) as u128,
					remote_asset_ed: 0,
					remote_asset_id: get_remote_erc20_asset_id().into(),
					remote_asset_total_supply: u64::MAX as u128,
					remote_reserve_location: get_asset_hub_location().into(),
					remote_xcm_fee: XCM_ASSET_FEE.into(),
					status: SwitchPairStatus::Paused,
				});
			assert_eq!(switch_pair, Some(expected_switch_pair));
			// Unit balance since we had to leave ED on this chain and no min balance
			// requirement (ED) is set for the remote asset.
			assert!(switch_pair.unwrap().reducible_remote_balance().is_one());
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::SwitchPairCreated {
					pool_account: pool_account_address.clone(),
					remote_asset_circulating_supply: (u64::MAX - 1) as u128,
					remote_asset_ed: 0,
					remote_asset_total_supply: u64::MAX as u128,
					remote_asset_id: get_remote_erc20_asset_id().into(),
					remote_reserve_location: get_asset_hub_location().into(),
					remote_xcm_fee: Box::new(XCM_ASSET_FEE.into())
				}
				.into()));
		});
}

#[test]
fn fails_on_invalid_origin() {
	ExtBuilder::default().build_and_execute_with_sanity_tests(|| {
		assert_noop!(
			Pallet::<MockRuntime>::force_set_switch_pair(
				RawOrigin::None.into(),
				100_000,
				Box::new(get_remote_erc20_asset_id().into()),
				1_000,
				Box::new(get_asset_hub_location().into()),
				0,
				Box::new(XCM_ASSET_FEE.into()),
			),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn fails_on_invalid_supply_values() {
	ExtBuilder::default().build_and_execute_with_sanity_tests(|| {
		assert_noop!(
			Pallet::<MockRuntime>::force_set_switch_pair(
				RawOrigin::None.into(),
				// Total supply less than locked supply
				1_000,
				Box::new(get_remote_erc20_asset_id().into()),
				1_001,
				Box::new(get_asset_hub_location().into()),
				0,
				Box::new(XCM_ASSET_FEE.into()),
			),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn successful_on_not_enough_funds_on_pool_balance() {
	let pool_account_address =
		Pallet::<MockRuntime>::pool_account_id_for_remote_asset(&get_remote_erc20_asset_id().into()).unwrap();
	// It works if not enough free balance is available
	ExtBuilder::default()
		.with_balances(vec![(pool_account_address.clone(), u64::MAX - 1, 0, 0)])
		.build()
		// We ignore try-runtime tests here since we are testing the breaking of invariants.
		.execute_with(|| {
			assert_ok!(Pallet::<MockRuntime>::force_set_switch_pair(
				RawOrigin::Root.into(),
				u64::MAX as u128,
				Box::new(get_remote_erc20_asset_id().into()),
				u64::MAX as u128,
				Box::new(get_asset_hub_location().into()),
				0,
				Box::new(XCM_ASSET_FEE.into()),
			),);
			let switch_pair = SwitchPair::<MockRuntime>::get();
			let expected_switch_pair =
				SwitchPairInfoOf::<MockRuntime>::from_input_unchecked(NewSwitchPairInfoOf::<MockRuntime> {
					pool_account: pool_account_address.clone(),
					remote_asset_circulating_supply: u64::MAX as u128,
					remote_asset_ed: 0,
					remote_asset_id: get_remote_erc20_asset_id().into(),
					remote_asset_total_supply: u64::MAX as u128,
					remote_reserve_location: get_asset_hub_location().into(),
					remote_xcm_fee: XCM_ASSET_FEE.into(),
					status: SwitchPairStatus::Paused,
				});
			assert_eq!(switch_pair, Some(expected_switch_pair));
			assert!(switch_pair.unwrap().reducible_remote_balance().is_zero());
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::SwitchPairCreated {
					pool_account: pool_account_address.clone(),
					remote_asset_circulating_supply: u64::MAX as u128,
					remote_asset_ed: 0,
					remote_asset_total_supply: u64::MAX as u128,
					remote_asset_id: get_remote_erc20_asset_id().into(),
					remote_reserve_location: get_asset_hub_location().into(),
					remote_xcm_fee: Box::new(XCM_ASSET_FEE.into())
				}
				.into()));
		});
	// It works if balance is frozen.
	ExtBuilder::default()
		.with_balances(vec![(pool_account_address.clone(), u64::MAX, 1, 0)])
		.build()
		// We ignore try-runtime tests here since we are testing the breaking of invariants.
		.execute_with(|| {
			assert_ok!(Pallet::<MockRuntime>::force_set_switch_pair(
				RawOrigin::Root.into(),
				u64::MAX as u128,
				Box::new(get_remote_erc20_asset_id().into()),
				u64::MAX as u128,
				Box::new(get_asset_hub_location().into()),
				0,
				Box::new(XCM_ASSET_FEE.into()),
			),);
			let switch_pair = SwitchPair::<MockRuntime>::get();
			let expected_switch_pair =
				SwitchPairInfoOf::<MockRuntime>::from_input_unchecked(NewSwitchPairInfoOf::<MockRuntime> {
					pool_account: pool_account_address.clone(),
					remote_asset_circulating_supply: u64::MAX as u128,
					remote_asset_ed: 0,
					remote_asset_id: get_remote_erc20_asset_id().into(),
					remote_asset_total_supply: u64::MAX as u128,
					remote_reserve_location: get_asset_hub_location().into(),
					remote_xcm_fee: XCM_ASSET_FEE.into(),
					status: SwitchPairStatus::Paused,
				});
			assert_eq!(switch_pair, Some(expected_switch_pair));
			assert!(switch_pair.unwrap().reducible_remote_balance().is_zero());
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::SwitchPairCreated {
					pool_account: pool_account_address.clone(),
					remote_asset_circulating_supply: u64::MAX as u128,
					remote_asset_ed: 0,
					remote_asset_total_supply: u64::MAX as u128,
					remote_asset_id: get_remote_erc20_asset_id().into(),
					remote_reserve_location: get_asset_hub_location().into(),
					remote_xcm_fee: Box::new(XCM_ASSET_FEE.into())
				}
				.into()));
		});
	// It works if balance is held.
	ExtBuilder::default()
		.with_balances(vec![(pool_account_address.clone(), u64::MAX, 0, 1)])
		.build()
		// We ignore try-runtime tests here since we are testing the breaking of invariants.
		.execute_with(|| {
			assert_ok!(Pallet::<MockRuntime>::force_set_switch_pair(
				RawOrigin::Root.into(),
				u64::MAX as u128,
				Box::new(get_remote_erc20_asset_id().into()),
				u64::MAX as u128,
				Box::new(get_asset_hub_location().into()),
				0,
				Box::new(XCM_ASSET_FEE.into()),
			),);
			let switch_pair = SwitchPair::<MockRuntime>::get();
			let expected_switch_pair =
				SwitchPairInfoOf::<MockRuntime>::from_input_unchecked(NewSwitchPairInfoOf::<MockRuntime> {
					pool_account: pool_account_address.clone(),
					remote_asset_circulating_supply: u64::MAX as u128,
					remote_asset_ed: 0,
					remote_asset_id: get_remote_erc20_asset_id().into(),
					remote_asset_total_supply: u64::MAX as u128,
					remote_reserve_location: get_asset_hub_location().into(),
					remote_xcm_fee: XCM_ASSET_FEE.into(),
					status: SwitchPairStatus::Paused,
				});
			assert_eq!(switch_pair, Some(expected_switch_pair));
			assert!(switch_pair.unwrap().reducible_remote_balance().is_zero());
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::SwitchPairCreated {
					pool_account: pool_account_address.clone(),
					remote_asset_circulating_supply: u64::MAX as u128,
					remote_asset_ed: 0,
					remote_asset_total_supply: u64::MAX as u128,
					remote_asset_id: get_remote_erc20_asset_id().into(),
					remote_reserve_location: get_asset_hub_location().into(),
					remote_xcm_fee: Box::new(XCM_ASSET_FEE.into())
				}
				.into()));
		});
}
