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
		ExtBuilder, MockRuntime, NewSwapPairInfo, System, ASSET_HUB_LOCATION, REMOTE_ERC20_ASSET_ID, XCM_ASSET_FEE,
	},
	swap::SwapPairStatus,
	tests::assert_total_supply_invariant,
	Error, Event, Pallet, SwapPair, SwapPairInfoOf,
};

#[test]
fn successful() {
	let pool_account_address =
		Pallet::<MockRuntime>::pool_account_id_for_remote_asset(&REMOTE_ERC20_ASSET_ID.into()).unwrap();
	ExtBuilder::default()
		.with_balances(vec![(pool_account_address.clone(), 1_000, 0, 0)])
		.build()
		.execute_with(|| {
			assert_ok!(Pallet::<MockRuntime>::set_swap_pair(
				RawOrigin::Root.into(),
				Box::new(ASSET_HUB_LOCATION.into()),
				Box::new(REMOTE_ERC20_ASSET_ID.into()),
				Box::new(XCM_ASSET_FEE.into()),
				u64::MAX as u128,
				1_000,
			));

			let swap_pair = SwapPair::<MockRuntime>::get();
			let expected_swap_pair = SwapPairInfoOf::<MockRuntime> {
				pool_account: pool_account_address.clone(),
				// Must match total supply - circulating supply
				remote_asset_balance: u64::MAX as u128 - 1_000,
				remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
				remote_fee: XCM_ASSET_FEE.into(),
				remote_reserve_location: ASSET_HUB_LOCATION.into(),
				status: SwapPairStatus::Paused,
			};
			assert_eq!(swap_pair, Some(expected_swap_pair.clone()));
			assert_total_supply_invariant(u64::MAX, expected_swap_pair.remote_asset_balance, &pool_account_address);
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::SwapPairCreated {
					circulating_supply: 1_000,
					pool_account: pool_account_address.clone(),
					remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
					remote_asset_reserve_location: ASSET_HUB_LOCATION.into(),
					remote_xcm_fee: Box::new(XCM_ASSET_FEE.into()),
					total_issuance: u64::MAX as u128,
				}
				.into()));
		});
	// Case where all issuance is circulating supply requires the same balance for
	// the pool account
	ExtBuilder::default()
		.with_balances(vec![(pool_account_address.clone(), u64::MAX, 0, 0)])
		.build()
		.execute_with(|| {
			assert_ok!(Pallet::<MockRuntime>::set_swap_pair(
				RawOrigin::Root.into(),
				Box::new(ASSET_HUB_LOCATION.into()),
				Box::new(REMOTE_ERC20_ASSET_ID.into()),
				Box::new(XCM_ASSET_FEE.into()),
				u64::MAX as u128,
				u64::MAX as u128,
			));

			let swap_pair = SwapPair::<MockRuntime>::get();
			let expected_swap_pair = SwapPairInfoOf::<MockRuntime> {
				pool_account: pool_account_address.clone(),
				// No balance on remote since all circulating supply is unlocked.
				remote_asset_balance: 0,
				remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
				remote_fee: XCM_ASSET_FEE.into(),
				remote_reserve_location: ASSET_HUB_LOCATION.into(),
				status: SwapPairStatus::Paused,
			};
			assert_eq!(swap_pair, Some(expected_swap_pair.clone()));
			assert_total_supply_invariant(u64::MAX, expected_swap_pair.remote_asset_balance, &pool_account_address);
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::SwapPairCreated {
					circulating_supply: u64::MAX as u128,
					pool_account: pool_account_address.clone(),
					remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
					remote_asset_reserve_location: ASSET_HUB_LOCATION.into(),
					remote_xcm_fee: Box::new(XCM_ASSET_FEE.into()),
					total_issuance: u64::MAX as u128,
				}
				.into()));
		});
	// Case where all issuance is locked and controlled by our sovereign account.
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Pallet::<MockRuntime>::set_swap_pair(
			RawOrigin::Root.into(),
			Box::new(ASSET_HUB_LOCATION.into()),
			Box::new(REMOTE_ERC20_ASSET_ID.into()),
			Box::new(XCM_ASSET_FEE.into()),
			u64::MAX as u128,
			0,
		));

		let swap_pair = SwapPair::<MockRuntime>::get();
		let expected_swap_pair = SwapPairInfoOf::<MockRuntime> {
			pool_account: pool_account_address.clone(),
			// Max balance since all circulating supply is controlled by us.
			remote_asset_balance: u64::MAX as u128,
			remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
			remote_fee: XCM_ASSET_FEE.into(),
			remote_reserve_location: ASSET_HUB_LOCATION.into(),
			status: SwapPairStatus::Paused,
		};
		assert_eq!(swap_pair, Some(expected_swap_pair.clone()));
		assert_total_supply_invariant(u64::MAX, expected_swap_pair.remote_asset_balance, &pool_account_address);
		assert!(System::events().into_iter().map(|e| e.event).any(|e| e
			== Event::<MockRuntime>::SwapPairCreated {
				circulating_supply: 0,
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
fn fails_on_invalid_origin() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Pallet::<MockRuntime>::set_swap_pair(
				RawOrigin::None.into(),
				Box::new(ASSET_HUB_LOCATION.into()),
				Box::new(REMOTE_ERC20_ASSET_ID.into()),
				Box::new(XCM_ASSET_FEE.into()),
				100_000,
				1_000,
			),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn fails_on_pool_existing() {
	ExtBuilder::default()
		.with_swap_pair_info(NewSwapPairInfo {
			circulating_supply: 0,
			pool_account: [0u8; 32].into(),
			remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
			remote_fee: XCM_ASSET_FEE.into(),
			remote_reserve_location: ASSET_HUB_LOCATION.into(),
			status: Default::default(),
			total_issuance: 1_000,
		})
		.build()
		.execute_with(|| {
			assert_noop!(
				Pallet::<MockRuntime>::set_swap_pair(
					RawOrigin::Root.into(),
					Box::new(ASSET_HUB_LOCATION.into()),
					Box::new(REMOTE_ERC20_ASSET_ID.into()),
					Box::new(XCM_ASSET_FEE.into()),
					100_000,
					1_000,
				),
				Error::<MockRuntime>::SwapPairAlreadyExisting
			);
		});
}

#[test]
fn fails_on_invalid_supply_values() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Pallet::<MockRuntime>::set_swap_pair(
				RawOrigin::Root.into(),
				Box::new(ASSET_HUB_LOCATION.into()),
				Box::new(REMOTE_ERC20_ASSET_ID.into()),
				Box::new(XCM_ASSET_FEE.into()),
				// Total supply less than locked supply
				1_000,
				1_001,
			),
			Error::<MockRuntime>::InvalidInput
		);
	});
}

#[test]
fn fails_on_not_enough_funds_on_pool_balance() {
	let pool_account_address =
		Pallet::<MockRuntime>::pool_account_id_for_remote_asset(&REMOTE_ERC20_ASSET_ID.into()).unwrap();
	// Does not work if not enough free balance is available
	ExtBuilder::default()
		.with_balances(vec![(pool_account_address.clone(), u64::MAX - 1, 0, 0)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Pallet::<MockRuntime>::set_swap_pair(
					RawOrigin::Root.into(),
					Box::new(ASSET_HUB_LOCATION.into()),
					Box::new(REMOTE_ERC20_ASSET_ID.into()),
					Box::new(XCM_ASSET_FEE.into()),
					u64::MAX as u128,
					u64::MAX as u128,
				),
				Error::<MockRuntime>::PoolInitialLiquidityRequirement
			);
		});
	// Does not work if balance is frozen.
	ExtBuilder::default()
		.with_balances(vec![(pool_account_address.clone(), u64::MAX, 1, 0)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Pallet::<MockRuntime>::set_swap_pair(
					RawOrigin::Root.into(),
					Box::new(ASSET_HUB_LOCATION.into()),
					Box::new(REMOTE_ERC20_ASSET_ID.into()),
					Box::new(XCM_ASSET_FEE.into()),
					u64::MAX as u128,
					u64::MAX as u128,
				),
				Error::<MockRuntime>::PoolInitialLiquidityRequirement
			);
		});
	// Does not work if balance is held.
	ExtBuilder::default()
		.with_balances(vec![(pool_account_address, u64::MAX, 0, 1)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Pallet::<MockRuntime>::set_swap_pair(
					RawOrigin::Root.into(),
					Box::new(ASSET_HUB_LOCATION.into()),
					Box::new(REMOTE_ERC20_ASSET_ID.into()),
					Box::new(XCM_ASSET_FEE.into()),
					u64::MAX as u128,
					u64::MAX as u128,
				),
				Error::<MockRuntime>::PoolInitialLiquidityRequirement
			);
		});
}
