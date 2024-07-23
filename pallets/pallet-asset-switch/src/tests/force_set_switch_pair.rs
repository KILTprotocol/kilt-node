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

use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use sp_runtime::DispatchError;

use crate::{
	mock::{
		ExtBuilder, MockRuntime, NewSwitchPairInfo, System, ASSET_HUB_LOCATION, REMOTE_ERC20_ASSET_ID, XCM_ASSET_FEE,
	},
	switch::SwitchPairStatus,
	tests::assert_supply_invariant,
	Error, Event, Pallet, SwitchPair, SwitchPairInfoOf,
};

#[test]
fn successful() {
	let pool_account_address =
		Pallet::<MockRuntime>::pool_account_id_for_remote_asset(&REMOTE_ERC20_ASSET_ID.into()).unwrap();
	// Case where all issuance is circulating supply requires the same balance (+ED)
	// for the pool account
	ExtBuilder::default()
		.with_balances(vec![(pool_account_address.clone(), u64::MAX, 0, 0)])
		.build()
		.execute_with(|| {
			assert_ok!(Pallet::<MockRuntime>::force_set_switch_pair(
				RawOrigin::Root.into(),
				Box::new(ASSET_HUB_LOCATION.into()),
				Box::new(REMOTE_ERC20_ASSET_ID.into()),
				Box::new(XCM_ASSET_FEE.into()),
				u64::MAX as u128,
				// Need to leave 1 on this chain for ED, so `MAX - 1` can at most be exchanged back (and transferred
				// out from the pool account).
				(u64::MAX - 1) as u128,
				0
			));

			let switch_pair = SwitchPair::<MockRuntime>::get();
			let expected_switch_pair = SwitchPairInfoOf::<MockRuntime> {
				pool_account: pool_account_address.clone(),
				// Unit balance since we had to leave ED on this chain and no min balance requirement (ED) is set for the remote asset.
				remote_asset_balance: 1,
				remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
				remote_fee: XCM_ASSET_FEE.into(),
				remote_reserve_location: ASSET_HUB_LOCATION.into(),
				status: SwitchPairStatus::Paused,
			};
			assert_eq!(switch_pair, Some(expected_switch_pair.clone()));
			assert_supply_invariant(
				u64::MAX,
				u64::MAX - 1,
				expected_switch_pair.remote_asset_balance,
				&pool_account_address,
			);
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::SwitchPairCreated {
					circulating_supply: (u64::MAX - 1) as u128,
					remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
					pool_account: pool_account_address.clone(),
					remote_asset_reserve_location: ASSET_HUB_LOCATION.into(),
					remote_xcm_fee: Box::new(XCM_ASSET_FEE.into()),
					total_issuance: u64::MAX as u128,
					min_remote_balance: 0
				}
				.into()));
		});
	// Case where all issuance is locked and controlled by our sovereign account.
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Pallet::<MockRuntime>::force_set_switch_pair(
			RawOrigin::Root.into(),
			Box::new(ASSET_HUB_LOCATION.into()),
			Box::new(REMOTE_ERC20_ASSET_ID.into()),
			Box::new(XCM_ASSET_FEE.into()),
			u64::MAX as u128,
			0,
			0,
		));

		let switch_pair = SwitchPair::<MockRuntime>::get();
		let expected_switch_pair = SwitchPairInfoOf::<MockRuntime> {
			pool_account: pool_account_address.clone(),
			// Max balance since all circulating supply is controlled by us.
			remote_asset_balance: u64::MAX as u128,
			remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
			remote_fee: XCM_ASSET_FEE.into(),
			remote_reserve_location: ASSET_HUB_LOCATION.into(),
			status: SwitchPairStatus::Paused,
		};
		assert_eq!(switch_pair, Some(expected_switch_pair.clone()));
		assert_supply_invariant(
			u64::MAX,
			0u128,
			expected_switch_pair.remote_asset_balance,
			&pool_account_address,
		);
		assert!(System::events().into_iter().map(|e| e.event).any(|e| e
			== Event::<MockRuntime>::SwitchPairCreated {
				circulating_supply: 0,
				min_remote_balance: 0,
				remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
				pool_account: pool_account_address.clone(),
				remote_asset_reserve_location: ASSET_HUB_LOCATION.into(),
				remote_xcm_fee: Box::new(XCM_ASSET_FEE.into()),
				total_issuance: u64::MAX as u128
			}
			.into()));
	});
	// Case where all issuance is circulating supply and there's a min balance >=
	// `0` on the remote chain requires the same balance (+ED) for the pool account,
	// and the remote balance is calculated accordingly.
	ExtBuilder::default()
		.with_balances(vec![(pool_account_address.clone(), u64::MAX, 0, 0)])
		.build()
		.execute_with(|| {
			assert_ok!(Pallet::<MockRuntime>::force_set_switch_pair(
				RawOrigin::Root.into(),
				Box::new(ASSET_HUB_LOCATION.into()),
				Box::new(REMOTE_ERC20_ASSET_ID.into()),
				Box::new(XCM_ASSET_FEE.into()),
				u64::MAX as u128,
				// Need to leave 1 on this chain for ED, so `MAX - 1` can at most be exchanged back (and transferred
				// out from the pool account).
				(u64::MAX - 1) as u128,
				// The `1` remaining is used to cover our ED for the remote asset on the remote location.
				1,
			));

			let switch_pair = SwitchPair::<MockRuntime>::get();
			let expected_switch_pair = SwitchPairInfoOf::<MockRuntime> {
				pool_account: pool_account_address.clone(),
				// Zero balance since we everything but the required remote asset ED is circulating.
				remote_asset_balance: 0,
				remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
				remote_fee: XCM_ASSET_FEE.into(),
				remote_reserve_location: ASSET_HUB_LOCATION.into(),
				status: SwitchPairStatus::Paused,
			};
			assert_eq!(switch_pair, Some(expected_switch_pair.clone()));
			assert_supply_invariant(
				u64::MAX,
				u64::MAX - 1,
				// We re-add the min balance requirement to check for invariants.
				expected_switch_pair.remote_asset_balance + 1,
				&pool_account_address,
			);
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::SwitchPairCreated {
					circulating_supply: (u64::MAX - 1) as u128,
					min_remote_balance: 1,
					pool_account: pool_account_address.clone(),
					remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
					remote_asset_reserve_location: ASSET_HUB_LOCATION.into(),
					remote_xcm_fee: Box::new(XCM_ASSET_FEE.into()),
					total_issuance: u64::MAX as u128,
				}
				.into()));
		});
	// Case where all issuance is locked and controlled by our sovereign account,
	// but there's a min balance >= `0` on the remote chain.
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Pallet::<MockRuntime>::force_set_switch_pair(
			RawOrigin::Root.into(),
			Box::new(ASSET_HUB_LOCATION.into()),
			Box::new(REMOTE_ERC20_ASSET_ID.into()),
			Box::new(XCM_ASSET_FEE.into()),
			u64::MAX as u128,
			0,
			1,
		));

		let switch_pair = SwitchPair::<MockRuntime>::get();
		let expected_switch_pair = SwitchPairInfoOf::<MockRuntime> {
			pool_account: pool_account_address.clone(),
			// We cannot go below `1` on the remote chain, so of all the locked assets we control, we can only exchange
			// all but one.
			remote_asset_balance: (u64::MAX - 1) as u128,
			remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
			remote_fee: XCM_ASSET_FEE.into(),
			remote_reserve_location: ASSET_HUB_LOCATION.into(),
			status: SwitchPairStatus::Paused,
		};
		assert_eq!(switch_pair, Some(expected_switch_pair.clone()));
		assert_supply_invariant(
			u64::MAX,
			0u128,
			// We re-add the min balance requirement to check for invariants.
			expected_switch_pair.remote_asset_balance + 1,
			&pool_account_address,
		);
		assert!(System::events().into_iter().map(|e| e.event).any(|e| e
			== Event::<MockRuntime>::SwitchPairCreated {
				circulating_supply: 0,
				min_remote_balance: 1,
				pool_account: pool_account_address.clone(),
				remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
				remote_asset_reserve_location: ASSET_HUB_LOCATION.into(),
				remote_xcm_fee: Box::new(XCM_ASSET_FEE.into()),
				total_issuance: u64::MAX as u128,
			}
			.into()));
	});
}

#[test]
fn successful_overwrites_existing_pool() {
	let pool_account_address =
		Pallet::<MockRuntime>::pool_account_id_for_remote_asset(&REMOTE_ERC20_ASSET_ID.into()).unwrap();
	ExtBuilder::default()
		.with_switch_pair_info(NewSwitchPairInfo {
			circulating_supply: 0,
			min_remote_balance: 0,
			pool_account: [0u8; 32].into(),
			remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
			remote_fee: XCM_ASSET_FEE.into(),
			remote_reserve_location: ASSET_HUB_LOCATION.into(),
			status: Default::default(),
			total_issuance: 1_000,
		})
		.build()
		.execute_with(|| {
			assert_ok!(Pallet::<MockRuntime>::force_set_switch_pair(
				RawOrigin::Root.into(),
				Box::new(ASSET_HUB_LOCATION.into()),
				Box::new(REMOTE_ERC20_ASSET_ID.into()),
				Box::new(XCM_ASSET_FEE.into()),
				100_000,
				50_000,
				0,
			));

			let switch_pair = SwitchPair::<MockRuntime>::get();
			let expected_switch_pair = SwitchPairInfoOf::<MockRuntime> {
				pool_account: pool_account_address.clone(),
				// Remote asset balance updates with the new value.
				remote_asset_balance: 50_000,
				remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
				remote_fee: XCM_ASSET_FEE.into(),
				remote_reserve_location: ASSET_HUB_LOCATION.into(),
				status: SwitchPairStatus::Paused,
			};
			assert_eq!(switch_pair, Some(expected_switch_pair));
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::SwitchPairCreated {
					circulating_supply: 50_000,
					min_remote_balance: 0,
					remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
					remote_asset_reserve_location: ASSET_HUB_LOCATION.into(),
					remote_xcm_fee: Box::new(XCM_ASSET_FEE.into()),
					pool_account: pool_account_address.clone(),
					total_issuance: 100_000
				}
				.into()));
		});
}

#[test]
fn fails_on_invalid_origin() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Pallet::<MockRuntime>::force_set_switch_pair(
				RawOrigin::None.into(),
				Box::new(ASSET_HUB_LOCATION.into()),
				Box::new(REMOTE_ERC20_ASSET_ID.into()),
				Box::new(XCM_ASSET_FEE.into()),
				100_000,
				1_000,
				0,
			),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn fails_on_invalid_supply_values() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Pallet::<MockRuntime>::force_set_switch_pair(
				RawOrigin::Root.into(),
				Box::new(ASSET_HUB_LOCATION.into()),
				Box::new(REMOTE_ERC20_ASSET_ID.into()),
				Box::new(XCM_ASSET_FEE.into()),
				// Total supply less than locked supply
				1_000,
				1_001,
				0,
			),
			Error::<MockRuntime>::InvalidInput
		);
	});
}

#[test]
fn successful_on_not_enough_funds_on_pool_balance() {
	let pool_account_address =
		Pallet::<MockRuntime>::pool_account_id_for_remote_asset(&REMOTE_ERC20_ASSET_ID.into()).unwrap();
	// It works if not enough free balance is available
	ExtBuilder::default()
		.with_balances(vec![(pool_account_address.clone(), u64::MAX - 1, 0, 0)])
		.build()
		.execute_with(|| {
			assert_ok!(Pallet::<MockRuntime>::force_set_switch_pair(
				RawOrigin::Root.into(),
				Box::new(ASSET_HUB_LOCATION.into()),
				Box::new(REMOTE_ERC20_ASSET_ID.into()),
				Box::new(XCM_ASSET_FEE.into()),
				u64::MAX as u128,
				u64::MAX as u128,
				0,
			),);
			let switch_pair = SwitchPair::<MockRuntime>::get();
			let expected_switch_pair = SwitchPairInfoOf::<MockRuntime> {
				pool_account: pool_account_address.clone(),
				remote_asset_balance: 0,
				remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
				remote_fee: XCM_ASSET_FEE.into(),
				remote_reserve_location: ASSET_HUB_LOCATION.into(),
				status: SwitchPairStatus::Paused,
			};
			assert_eq!(switch_pair, Some(expected_switch_pair));
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::SwitchPairCreated {
					min_remote_balance: 0,
					circulating_supply: u64::MAX as u128,
					pool_account: pool_account_address.clone(),
					remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
					remote_asset_reserve_location: ASSET_HUB_LOCATION.into(),
					remote_xcm_fee: Box::new(XCM_ASSET_FEE.into()),
					total_issuance: u64::MAX as u128
				}
				.into()));
		});
	// It works if balance is frozen.
	ExtBuilder::default()
		.with_balances(vec![(pool_account_address.clone(), u64::MAX, 1, 0)])
		.build()
		.execute_with(|| {
			assert_ok!(Pallet::<MockRuntime>::force_set_switch_pair(
				RawOrigin::Root.into(),
				Box::new(ASSET_HUB_LOCATION.into()),
				Box::new(REMOTE_ERC20_ASSET_ID.into()),
				Box::new(XCM_ASSET_FEE.into()),
				u64::MAX as u128,
				u64::MAX as u128,
				0,
			));
			let switch_pair = SwitchPair::<MockRuntime>::get();
			let expected_switch_pair = SwitchPairInfoOf::<MockRuntime> {
				pool_account: pool_account_address.clone(),
				remote_asset_balance: 0,
				remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
				remote_fee: XCM_ASSET_FEE.into(),
				remote_reserve_location: ASSET_HUB_LOCATION.into(),
				status: SwitchPairStatus::Paused,
			};
			assert_eq!(switch_pair, Some(expected_switch_pair));
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::SwitchPairCreated {
					circulating_supply: u64::MAX as u128,
					min_remote_balance: 0,
					pool_account: pool_account_address.clone(),
					remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
					remote_asset_reserve_location: ASSET_HUB_LOCATION.into(),
					remote_xcm_fee: Box::new(XCM_ASSET_FEE.into()),
					total_issuance: u64::MAX as u128,
				}
				.into()));
		});
	// It works if balance is held.
	ExtBuilder::default()
		.with_balances(vec![(pool_account_address.clone(), u64::MAX, 0, 1)])
		.build()
		.execute_with(|| {
			assert_ok!(Pallet::<MockRuntime>::force_set_switch_pair(
				RawOrigin::Root.into(),
				Box::new(ASSET_HUB_LOCATION.into()),
				Box::new(REMOTE_ERC20_ASSET_ID.into()),
				Box::new(XCM_ASSET_FEE.into()),
				u64::MAX as u128,
				u64::MAX as u128,
				0,
			),);
			let switch_pair = SwitchPair::<MockRuntime>::get();
			let expected_switch_pair = SwitchPairInfoOf::<MockRuntime> {
				pool_account: pool_account_address.clone(),
				remote_asset_balance: 0,
				remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
				remote_fee: XCM_ASSET_FEE.into(),
				remote_reserve_location: ASSET_HUB_LOCATION.into(),
				status: SwitchPairStatus::Paused,
			};
			assert_eq!(switch_pair, Some(expected_switch_pair));
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::SwitchPairCreated {
					circulating_supply: u64::MAX as u128,
					min_remote_balance: 0,
					pool_account: pool_account_address.clone(),
					remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
					remote_asset_reserve_location: ASSET_HUB_LOCATION.into(),
					remote_xcm_fee: Box::new(XCM_ASSET_FEE.into()),
					total_issuance: u64::MAX as u128,
				}
				.into()));
		});
}
