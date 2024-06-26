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

use frame_support::{
	assert_noop, assert_ok,
	traits::fungible::{Inspect, InspectFreeze, InspectHold},
};
use frame_system::RawOrigin;
use sp_runtime::{
	traits::{One, TryConvert, Zero},
	AccountId32, DispatchError,
};
use xcm::v3::{Fungibility, MultiAsset};

use crate::{
	mock::{
		Balances, ExtBuilder, MockFungibleAssetTransactor, MockRuntime, NewSwapPairInfo, System, ASSET_HUB_LOCATION,
		FREEZE_REASON, HOLD_REASON, REMOTE_ERC20_ASSET_ID, XCM_ASSET_FEE,
	},
	swap::SwapPairStatus,
	tests::assert_total_supply_invariant,
	xcm::convert::AccountId32ToAccountId32JunctionConverter,
	Error, Event, Pallet, SwapPair,
};

#[test]
fn successful() {
	let user = AccountId32::from([0; 32]);
	let pool_account = AccountId32::from([1; 32]);
	// It works with entire balance unfrozen and un-held.
	ExtBuilder::default()
		.with_balances(vec![(user.clone(), 100_000, 0, 0)])
		.with_fungibles(vec![(user.clone(), XCM_ASSET_FEE)])
		.with_swap_pair_info(NewSwapPairInfo {
			circulating_supply: 0,
			pool_account: pool_account.clone(),
			remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
			remote_fee: XCM_ASSET_FEE.into(),
			remote_reserve_location: ASSET_HUB_LOCATION.into(),
			status: SwapPairStatus::Running,
			total_issuance: 100_000,
		})
		.build()
		.execute_with(|| {
			let total_currency_issuance_before = <Balances as Inspect<AccountId32>>::total_issuance();
			assert_ok!(Pallet::<MockRuntime>::swap(
				RawOrigin::Signed(user.clone()).into(),
				99_999,
				Box::new(ASSET_HUB_LOCATION.into())
			));
			let total_currency_issuance_after = <Balances as Inspect<AccountId32>>::total_issuance();
			// Total issuance of currency has not changed
			assert_eq!(total_currency_issuance_after, total_currency_issuance_before);
			// User's currency balance is reduced by swap amount
			assert!(<Balances as Inspect<AccountId32>>::total_balance(&user).is_one());
			// User's frozen balance has remained unchanged.
			assert!(<Balances as InspectFreeze<AccountId32>>::balance_frozen(&FREEZE_REASON, &user).is_zero());
			// User's held balance has remained unchanged.
			assert!(<Balances as InspectHold<AccountId32>>::balance_on_hold(&HOLD_REASON, &user).is_zero());
			// Pool's currency balance is increased by swap amount
			assert_eq!(<Balances as Inspect<AccountId32>>::total_balance(&pool_account), 99_999);
			// Pool's remote balance is decreased by swap amount
			assert!(SwapPair::<MockRuntime>::get().unwrap().remote_asset_balance.is_one());
			// User's fungible balance is reduced by XCM fee
			assert!(MockFungibleAssetTransactor::get_balance_for(
				&AccountId32ToAccountId32JunctionConverter::try_convert(user.clone())
					.unwrap()
					.into()
			)
			.is_zero());
			// Pool's fungible balance is not changed (we're testing that fees are burnt and
			// not transferred).
			assert!(MockFungibleAssetTransactor::get_balance_for(
				&AccountId32ToAccountId32JunctionConverter::try_convert(pool_account.clone())
					.unwrap()
					.into()
			)
			.is_zero());
			// Invariant is held
			assert_total_supply_invariant(100_000u128, 1u128, &pool_account);
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::LocalToRemoteSwapExecuted {
					amount: 99_999,
					from: user.clone(),
					to: ASSET_HUB_LOCATION.into()
				}
				.into()));
		});
	// It works with balance partially frozen.
	ExtBuilder::default()
		.with_balances(vec![(user.clone(), 100_000, 1, 0)])
		.with_fungibles(vec![(user.clone(), XCM_ASSET_FEE)])
		.with_swap_pair_info(NewSwapPairInfo {
			circulating_supply: 0,
			pool_account: pool_account.clone(),
			remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
			remote_fee: XCM_ASSET_FEE.into(),
			remote_reserve_location: ASSET_HUB_LOCATION.into(),
			status: SwapPairStatus::Running,
			total_issuance: 100_000,
		})
		.build()
		.execute_with(|| {
			let total_currency_issuance_before = <Balances as Inspect<AccountId32>>::total_issuance();
			assert_ok!(Pallet::<MockRuntime>::swap(
				RawOrigin::Signed(user.clone()).into(),
				99_999,
				Box::new(ASSET_HUB_LOCATION.into())
			));
			let total_currency_issuance_after = <Balances as Inspect<AccountId32>>::total_issuance();
			// Total issuance of currency has not changed
			assert_eq!(total_currency_issuance_after, total_currency_issuance_before);
			// User's currency balance is reduced by swap amount
			assert!(<Balances as Inspect<AccountId32>>::total_balance(&user).is_one());
			// User's frozen balance has remained unchanged.
			assert!(<Balances as InspectFreeze<AccountId32>>::balance_frozen(&FREEZE_REASON, &user).is_one());
			// User's held balance has remained unchanged.
			assert!(<Balances as InspectHold<AccountId32>>::balance_on_hold(&HOLD_REASON, &user).is_zero());
			// Pool's currency balance is increased by swap amount
			assert_eq!(<Balances as Inspect<AccountId32>>::total_balance(&pool_account), 99_999);
			// Pool's remote balance is decreased by swap amount
			assert!(SwapPair::<MockRuntime>::get().unwrap().remote_asset_balance.is_one());
			// User's fungible balance is reduced by XCM fee
			assert!(MockFungibleAssetTransactor::get_balance_for(
				&AccountId32ToAccountId32JunctionConverter::try_convert(user.clone())
					.unwrap()
					.into()
			)
			.is_zero());
			// Pool's fungible balance is not changed (we're testing that fees are burnt and
			// not transferred).
			assert!(MockFungibleAssetTransactor::get_balance_for(
				&AccountId32ToAccountId32JunctionConverter::try_convert(pool_account.clone())
					.unwrap()
					.into()
			)
			.is_zero());
			// Invariant is held
			assert_total_supply_invariant(100_000u128, 1u128, &pool_account);
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::LocalToRemoteSwapExecuted {
					amount: 99_999,
					from: user.clone(),
					to: ASSET_HUB_LOCATION.into()
				}
				.into()));
		});
	// It works with balance partially held.
	ExtBuilder::default()
		// Free balance not allowed to go to zero.
		.with_balances(vec![(user.clone(), 100_001, 0, 1)])
		.with_fungibles(vec![(user.clone(), XCM_ASSET_FEE)])
		.with_swap_pair_info(NewSwapPairInfo {
			circulating_supply: 0,
			pool_account: pool_account.clone(),
			remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
			remote_fee: XCM_ASSET_FEE.into(),
			remote_reserve_location: ASSET_HUB_LOCATION.into(),
			status: SwapPairStatus::Running,
			total_issuance: 100_000,
		})
		.build()
		.execute_with(|| {
			let total_currency_issuance_before = <Balances as Inspect<AccountId32>>::total_issuance();
			assert_ok!(Pallet::<MockRuntime>::swap(
				RawOrigin::Signed(user.clone()).into(),
				99_999,
				Box::new(ASSET_HUB_LOCATION.into())
			));
			let total_currency_issuance_after = <Balances as Inspect<AccountId32>>::total_issuance();
			// Total issuance of currency has not changed
			assert_eq!(total_currency_issuance_after, total_currency_issuance_before);
			// User's currency balance is reduced by swap amount
			assert_eq!(<Balances as Inspect<AccountId32>>::total_balance(&user), 2);
			// User's frozen balance has remained unchanged.
			assert!(<Balances as InspectFreeze<AccountId32>>::balance_frozen(&FREEZE_REASON, &user).is_zero());
			// User's held balance has remained unchanged.
			assert!(<Balances as InspectHold<AccountId32>>::balance_on_hold(&HOLD_REASON, &user).is_one());
			// Pool's currency balance is increased by swap amount
			assert_eq!(<Balances as Inspect<AccountId32>>::total_balance(&pool_account), 99_999);
			// Pool's remote balance is decreased by swap amount
			assert!(SwapPair::<MockRuntime>::get().unwrap().remote_asset_balance.is_one());
			// User's fungible balance is reduced by XCM fee
			assert!(MockFungibleAssetTransactor::get_balance_for(
				&AccountId32ToAccountId32JunctionConverter::try_convert(user.clone())
					.unwrap()
					.into()
			)
			.is_zero());
			// Pool's fungible balance is not changed (we're testing that fees are burnt and
			// not transferred).
			assert!(MockFungibleAssetTransactor::get_balance_for(
				&AccountId32ToAccountId32JunctionConverter::try_convert(pool_account.clone())
					.unwrap()
					.into()
			)
			.is_zero());
			// Invariant is held
			assert_total_supply_invariant(100_000u128, 1u128, &pool_account);
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::LocalToRemoteSwapExecuted {
					amount: 99_999,
					from: user.clone(),
					to: ASSET_HUB_LOCATION.into()
				}
				.into()));
		});
}

#[test]
fn fails_on_invalid_origin() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Pallet::<MockRuntime>::swap(RawOrigin::Root.into(), 1, Box::new(ASSET_HUB_LOCATION.into())),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn fails_on_non_existing_pool() {
	let user = AccountId32::from([0; 32]);
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Pallet::<MockRuntime>::swap(RawOrigin::Signed(user).into(), 1, Box::new(ASSET_HUB_LOCATION.into())),
			Error::<MockRuntime>::SwapPairNotFound
		);
	});
}

#[test]
fn fails_on_pool_not_running() {
	let user = AccountId32::from([0; 32]);
	let pool_account = AccountId32::from([1; 32]);
	ExtBuilder::default()
		.with_swap_pair_info(NewSwapPairInfo {
			circulating_supply: 0,
			pool_account,
			remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
			remote_fee: XCM_ASSET_FEE.into(),
			remote_reserve_location: ASSET_HUB_LOCATION.into(),
			status: SwapPairStatus::Paused,
			total_issuance: 100_000,
		})
		.build()
		.execute_with(|| {
			assert_noop!(
				Pallet::<MockRuntime>::swap(RawOrigin::Signed(user).into(), 1, Box::new(ASSET_HUB_LOCATION.into())),
				Error::<MockRuntime>::SwapPairNotEnabled
			);
		});
}

#[test]
fn fails_on_not_enough_user_local_balance() {
	let user = AccountId32::from([0; 32]);
	let pool_account = AccountId32::from([1; 32]);
	// Fails if user has not enough balance.
	ExtBuilder::default()
		.with_swap_pair_info(NewSwapPairInfo {
			circulating_supply: 0,
			pool_account: pool_account.clone(),
			remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
			remote_fee: XCM_ASSET_FEE.into(),
			remote_reserve_location: ASSET_HUB_LOCATION.into(),
			status: SwapPairStatus::Running,
			total_issuance: 100_000,
		})
		.build()
		.execute_with(|| {
			assert_noop!(
				Pallet::<MockRuntime>::swap(
					RawOrigin::Signed(user.clone()).into(),
					100_000,
					Box::new(ASSET_HUB_LOCATION.into())
				),
				Error::<MockRuntime>::UserSwapBalance
			);
		});
	// Fails if user has frozen balance.
	ExtBuilder::default()
		.with_balances(vec![(user.clone(), 100_000, 1, 0)])
		.with_swap_pair_info(NewSwapPairInfo {
			circulating_supply: 0,
			pool_account: pool_account.clone(),
			remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
			remote_fee: XCM_ASSET_FEE.into(),
			remote_reserve_location: ASSET_HUB_LOCATION.into(),
			status: SwapPairStatus::Running,
			total_issuance: 100_000,
		})
		.build()
		.execute_with(|| {
			assert_noop!(
				Pallet::<MockRuntime>::swap(
					RawOrigin::Signed(user.clone()).into(),
					100_000,
					Box::new(ASSET_HUB_LOCATION.into())
				),
				Error::<MockRuntime>::UserSwapBalance
			);
		});
	// Fails if user has held balance.
	ExtBuilder::default()
		.with_balances(vec![(user.clone(), 100_000, 0, 1)])
		.with_swap_pair_info(NewSwapPairInfo {
			circulating_supply: 0,
			pool_account,
			remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
			remote_fee: XCM_ASSET_FEE.into(),
			remote_reserve_location: ASSET_HUB_LOCATION.into(),
			status: SwapPairStatus::Running,
			total_issuance: 100_000,
		})
		.build()
		.execute_with(|| {
			assert_noop!(
				Pallet::<MockRuntime>::swap(
					RawOrigin::Signed(user).into(),
					100_000,
					Box::new(ASSET_HUB_LOCATION.into())
				),
				Error::<MockRuntime>::UserSwapBalance
			);
		});
}

#[test]
fn fails_on_not_enough_remote_balance() {
	let user = AccountId32::from([0; 32]);
	let pool_account = AccountId32::from([1; 32]);
	ExtBuilder::default()
		.with_balances(vec![(user.clone(), 100_000, 0, 1)])
		.with_swap_pair_info(NewSwapPairInfo {
			circulating_supply: 0,
			pool_account,
			remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
			remote_fee: XCM_ASSET_FEE.into(),
			remote_reserve_location: ASSET_HUB_LOCATION.into(),
			status: SwapPairStatus::Running,
			total_issuance: 50_000,
		})
		.build()
		.execute_with(|| {
			assert_noop!(
				Pallet::<MockRuntime>::swap(
					RawOrigin::Signed(user.clone()).into(),
					50_001,
					Box::new(ASSET_HUB_LOCATION.into())
				),
				Error::<MockRuntime>::Liquidity
			);
		});
}

#[test]
fn fails_on_not_enough_user_xcm_balance() {
	let user = AccountId32::from([0; 32]);
	let pool_account = AccountId32::from([1; 32]);
	ExtBuilder::default()
		.with_balances(vec![(user.clone(), 100_000, 0, 1)])
		.with_fungibles(vec![(
			user.clone(),
			MultiAsset {
				// 1 unit less than required
				fun: Fungibility::Fungible(999),
				..XCM_ASSET_FEE
			},
		)])
		.with_swap_pair_info(NewSwapPairInfo {
			circulating_supply: 0,
			pool_account,
			remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
			remote_fee: XCM_ASSET_FEE.into(),
			remote_reserve_location: ASSET_HUB_LOCATION.into(),
			status: SwapPairStatus::Running,
			total_issuance: 100_000,
		})
		.build()
		.execute_with(|| {
			assert_noop!(
				Pallet::<MockRuntime>::swap(
					RawOrigin::Signed(user.clone()).into(),
					50_001,
					Box::new(ASSET_HUB_LOCATION.into())
				),
				Error::<MockRuntime>::UserXcmBalance
			);
		});
}
