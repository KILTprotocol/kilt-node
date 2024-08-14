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
use xcm::v4::{Asset, Fungibility};

use crate::{
	mock::{
		get_asset_hub_location, get_remote_erc20_asset_id, Balances, ExtBuilder, MockFungibleAssetTransactor,
		MockRuntime, System, FREEZE_REASON, HOLD_REASON, XCM_ASSET_FEE,
	},
	switch::SwitchPairStatus,
	xcm::convert::AccountId32ToAccountId32JunctionConverter,
	Error, Event, NewSwitchPairInfoOf, Pallet, SwitchPair,
};

#[test]
fn successful() {
	let user = AccountId32::from([0; 32]);
	let pool_account = AccountId32::from([1; 32]);
	// It works with entire balance unfrozen and un-held.
	ExtBuilder::default()
		.with_balances(vec![(user.clone(), 100_000, 0, 0), (pool_account.clone(), 1, 0, 0)])
		.with_fungibles(vec![(user.clone(), XCM_ASSET_FEE)])
		.with_switch_pair_info(NewSwitchPairInfoOf::<MockRuntime> {
			pool_account: pool_account.clone(),
			remote_asset_circulating_supply: 0,
			remote_asset_ed: 0,
			remote_asset_id: get_remote_erc20_asset_id().into(),
			remote_asset_total_supply: 100_000,
			remote_reserve_location: get_asset_hub_location().into(),
			remote_xcm_fee: XCM_ASSET_FEE.into(),
			status: SwitchPairStatus::Running,
		})
		.build_and_execute_with_sanity_tests(|| {
			let total_currency_issuance_before = <Balances as Inspect<AccountId32>>::total_issuance();
			assert_ok!(Pallet::<MockRuntime>::switch(
				RawOrigin::Signed(user.clone()).into(),
				// Cannot switch ED (1 in the mock), so we need to exclude that.
				99_999,
				Box::new(get_asset_hub_location().into())
			));
			let total_currency_issuance_after = <Balances as Inspect<AccountId32>>::total_issuance();
			// Total issuance of currency has not changed
			assert_eq!(total_currency_issuance_after, total_currency_issuance_before);
			// User's currency balance is reduced by switch amount
			assert!(<Balances as Inspect<AccountId32>>::total_balance(&user).is_one());
			// User's frozen balance has remained unchanged.
			assert!(<Balances as InspectFreeze<AccountId32>>::balance_frozen(&FREEZE_REASON, &user).is_zero());
			// User's held balance has remained unchanged.
			assert!(<Balances as InspectHold<AccountId32>>::balance_on_hold(&HOLD_REASON, &user).is_zero());
			// Pool's currency balance (previously only ED) is increased by switch amount
			assert_eq!(
				<Balances as Inspect<AccountId32>>::total_balance(&pool_account),
				100_000
			);
			// Pool's remote balance is decreased by switch amount
			assert!(SwitchPair::<MockRuntime>::get()
				.unwrap()
				.reducible_remote_balance()
				.is_one());
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
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::LocalToRemoteSwitchExecuted {
					amount: 99_999,
					from: user.clone(),
					to: get_asset_hub_location().into()
				}
				.into()));
		});
	// It works with balance partially frozen.
	ExtBuilder::default()
		.with_balances(vec![(user.clone(), 100_000, 1, 0), (pool_account.clone(), 1, 0, 0)])
		.with_fungibles(vec![(user.clone(), XCM_ASSET_FEE)])
		.with_switch_pair_info(NewSwitchPairInfoOf::<MockRuntime> {
			pool_account: pool_account.clone(),
			remote_asset_circulating_supply: 0,
			remote_asset_ed: 0,
			remote_asset_id: get_remote_erc20_asset_id().into(),
			remote_asset_total_supply: 100_000,
			remote_reserve_location: get_asset_hub_location().into(),
			remote_xcm_fee: XCM_ASSET_FEE.into(),
			status: SwitchPairStatus::Running,
		})
		.build_and_execute_with_sanity_tests(|| {
			let total_currency_issuance_before = <Balances as Inspect<AccountId32>>::total_issuance();
			assert_ok!(Pallet::<MockRuntime>::switch(
				RawOrigin::Signed(user.clone()).into(),
				99_999,
				Box::new(get_asset_hub_location().into())
			));
			let total_currency_issuance_after = <Balances as Inspect<AccountId32>>::total_issuance();
			// Total issuance of currency has not changed
			assert_eq!(total_currency_issuance_after, total_currency_issuance_before);
			// User's currency balance is reduced by switch amount
			assert!(<Balances as Inspect<AccountId32>>::total_balance(&user).is_one());
			// User's frozen balance has remained unchanged.
			assert!(<Balances as InspectFreeze<AccountId32>>::balance_frozen(&FREEZE_REASON, &user).is_one());
			// User's held balance has remained unchanged.
			assert!(<Balances as InspectHold<AccountId32>>::balance_on_hold(&HOLD_REASON, &user).is_zero());
			// Pool's currency balance (previously only ED) is increased by switch amount
			assert_eq!(
				<Balances as Inspect<AccountId32>>::total_balance(&pool_account),
				100_000
			);
			// Pool's remote balance is decreased by switch amount
			assert!(SwitchPair::<MockRuntime>::get()
				.unwrap()
				.reducible_remote_balance()
				.is_one());
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
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::LocalToRemoteSwitchExecuted {
					amount: 99_999,
					from: user.clone(),
					to: get_asset_hub_location().into()
				}
				.into()));
		});
	// It works with balance partially held.
	ExtBuilder::default()
		// Free balance not allowed to go to zero.
		.with_balances(vec![(user.clone(), 100_001, 0, 1), (pool_account.clone(), 1, 0, 0)])
		.with_fungibles(vec![(user.clone(), XCM_ASSET_FEE)])
		.with_switch_pair_info(NewSwitchPairInfoOf::<MockRuntime> {
			pool_account: pool_account.clone(),
			remote_asset_circulating_supply: 0,
			remote_asset_ed: 0,
			remote_asset_id: get_remote_erc20_asset_id().into(),
			remote_asset_total_supply: 100_000,
			remote_reserve_location: get_asset_hub_location().into(),
			remote_xcm_fee: XCM_ASSET_FEE.into(),
			status: SwitchPairStatus::Running,
		})
		.build_and_execute_with_sanity_tests(|| {
			let total_currency_issuance_before = <Balances as Inspect<AccountId32>>::total_issuance();
			assert_ok!(Pallet::<MockRuntime>::switch(
				RawOrigin::Signed(user.clone()).into(),
				99_999,
				Box::new(get_asset_hub_location().into())
			));
			let total_currency_issuance_after = <Balances as Inspect<AccountId32>>::total_issuance();
			// Total issuance of currency has not changed
			assert_eq!(total_currency_issuance_after, total_currency_issuance_before);
			// User's currency balance is reduced by switch amount
			assert_eq!(<Balances as Inspect<AccountId32>>::total_balance(&user), 2);
			// User's frozen balance has remained unchanged.
			assert!(<Balances as InspectFreeze<AccountId32>>::balance_frozen(&FREEZE_REASON, &user).is_zero());
			// User's held balance has remained unchanged.
			assert!(<Balances as InspectHold<AccountId32>>::balance_on_hold(&HOLD_REASON, &user).is_one());
			// Pool's currency balance (previously only ED) is increased by switch amount
			assert_eq!(
				<Balances as Inspect<AccountId32>>::total_balance(&pool_account),
				100_000
			);
			// Pool's remote balance is decreased by switch amount
			assert!(SwitchPair::<MockRuntime>::get()
				.unwrap()
				.reducible_remote_balance()
				.is_one());
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
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::LocalToRemoteSwitchExecuted {
					amount: 99_999,
					from: user.clone(),
					to: get_asset_hub_location().into()
				}
				.into()));
		});
}

#[test]
fn fails_on_invalid_origin() {
	ExtBuilder::default().build_and_execute_with_sanity_tests(|| {
		assert_noop!(
			Pallet::<MockRuntime>::switch(RawOrigin::Root.into(), 1, Box::new(get_asset_hub_location().into())),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn fails_on_non_existing_pool() {
	let user = AccountId32::from([0; 32]);
	ExtBuilder::default().build_and_execute_with_sanity_tests(|| {
		assert_noop!(
			Pallet::<MockRuntime>::switch(
				RawOrigin::Signed(user).into(),
				1,
				Box::new(get_asset_hub_location().into())
			),
			Error::<MockRuntime>::SwitchPairNotFound
		);
	});
}

#[test]
fn fails_on_pool_not_running() {
	let user = AccountId32::from([0; 32]);
	let pool_account = AccountId32::from([1; 32]);
	ExtBuilder::default()
		.with_switch_pair_info(NewSwitchPairInfoOf::<MockRuntime> {
			pool_account,
			remote_asset_circulating_supply: 0,
			remote_asset_ed: 0,
			remote_asset_id: get_remote_erc20_asset_id().into(),
			remote_asset_total_supply: 100_000,
			remote_reserve_location: get_asset_hub_location().into(),
			remote_xcm_fee: XCM_ASSET_FEE.into(),
			status: SwitchPairStatus::Paused,
		})
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Pallet::<MockRuntime>::switch(
					RawOrigin::Signed(user).into(),
					1,
					Box::new(get_asset_hub_location().into())
				),
				Error::<MockRuntime>::SwitchPairNotEnabled
			);
		});
}

#[test]
fn fails_on_not_enough_user_local_balance() {
	let user = AccountId32::from([0; 32]);
	let pool_account = AccountId32::from([1; 32]);
	// Fails if user has not enough balance.
	ExtBuilder::default()
		.with_switch_pair_info(NewSwitchPairInfoOf::<MockRuntime> {
			pool_account: pool_account.clone(),
			remote_asset_circulating_supply: 0,
			remote_asset_ed: 0,
			remote_asset_id: get_remote_erc20_asset_id().into(),
			remote_asset_total_supply: 100_000,
			remote_reserve_location: get_asset_hub_location().into(),
			remote_xcm_fee: XCM_ASSET_FEE.into(),
			status: SwitchPairStatus::Running,
		})
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Pallet::<MockRuntime>::switch(
					RawOrigin::Signed(user.clone()).into(),
					100_000,
					Box::new(get_asset_hub_location().into())
				),
				Error::<MockRuntime>::UserSwitchBalance
			);
		});
	// Fails if user has frozen balance.
	ExtBuilder::default()
		.with_balances(vec![(user.clone(), 100_000, 1, 0)])
		.with_switch_pair_info(NewSwitchPairInfoOf::<MockRuntime> {
			pool_account: pool_account.clone(),
			remote_asset_circulating_supply: 0,
			remote_asset_ed: 0,
			remote_asset_id: get_remote_erc20_asset_id().into(),
			remote_asset_total_supply: 100_000,
			remote_reserve_location: get_asset_hub_location().into(),
			remote_xcm_fee: XCM_ASSET_FEE.into(),
			status: SwitchPairStatus::Running,
		})
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Pallet::<MockRuntime>::switch(
					RawOrigin::Signed(user.clone()).into(),
					100_000,
					Box::new(get_asset_hub_location().into())
				),
				Error::<MockRuntime>::UserSwitchBalance
			);
		});
	// Fails if user has held balance.
	ExtBuilder::default()
		.with_balances(vec![(user.clone(), 100_000, 0, 1)])
		.with_switch_pair_info(NewSwitchPairInfoOf::<MockRuntime> {
			pool_account: pool_account.clone(),
			remote_asset_circulating_supply: 0,
			remote_asset_ed: 0,
			remote_asset_id: get_remote_erc20_asset_id().into(),
			remote_asset_total_supply: 100_000,
			remote_reserve_location: get_asset_hub_location().into(),
			remote_xcm_fee: XCM_ASSET_FEE.into(),
			status: SwitchPairStatus::Running,
		})
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Pallet::<MockRuntime>::switch(
					RawOrigin::Signed(user.clone()).into(),
					100_000,
					Box::new(get_asset_hub_location().into())
				),
				Error::<MockRuntime>::UserSwitchBalance
			);
		});
	// Fails if user goes under their ED.
	ExtBuilder::default()
		.with_balances(vec![(user.clone(), 100_000, 0, 0)])
		.with_switch_pair_info(NewSwitchPairInfoOf::<MockRuntime> {
			pool_account,
			remote_asset_circulating_supply: 0,
			remote_asset_ed: 0,
			remote_asset_id: get_remote_erc20_asset_id().into(),
			remote_asset_total_supply: 1_000_000,
			remote_reserve_location: get_asset_hub_location().into(),
			remote_xcm_fee: XCM_ASSET_FEE.into(),
			status: SwitchPairStatus::Running,
		})
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Pallet::<MockRuntime>::switch(
					RawOrigin::Signed(user).into(),
					100_000,
					Box::new(get_asset_hub_location().into())
				),
				Error::<MockRuntime>::UserSwitchBalance
			);
		});
}

#[test]
fn fails_on_not_enough_remote_balance() {
	let user = AccountId32::from([0; 32]);
	let pool_account = AccountId32::from([1; 32]);
	// Case where min remote balance is `0`
	ExtBuilder::default()
		.with_balances(vec![(user.clone(), 100_000, 0, 1)])
		.with_switch_pair_info(NewSwitchPairInfoOf::<MockRuntime> {
			pool_account: pool_account.clone(),
			remote_asset_circulating_supply: 0,
			remote_asset_ed: 0,
			remote_asset_id: get_remote_erc20_asset_id().into(),
			remote_asset_total_supply: 50_000,
			remote_reserve_location: get_asset_hub_location().into(),
			remote_xcm_fee: XCM_ASSET_FEE.into(),
			status: SwitchPairStatus::Running,
		})
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Pallet::<MockRuntime>::switch(
					RawOrigin::Signed(user.clone()).into(),
					50_001,
					Box::new(get_asset_hub_location().into())
				),
				Error::<MockRuntime>::Liquidity
			);
		});
	// Case where min remote balance is `1`
	ExtBuilder::default()
		.with_balances(vec![(user.clone(), 100_000, 0, 1)])
		.with_switch_pair_info(NewSwitchPairInfoOf::<MockRuntime> {
			pool_account,
			remote_asset_circulating_supply: 0,
			remote_asset_ed: 1,
			remote_asset_id: get_remote_erc20_asset_id().into(),
			remote_asset_total_supply: 50_000,
			remote_reserve_location: get_asset_hub_location().into(),
			remote_xcm_fee: XCM_ASSET_FEE.into(),
			status: SwitchPairStatus::Running,
		})
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Pallet::<MockRuntime>::switch(
					RawOrigin::Signed(user.clone()).into(),
					// Tradeable are only 49_999 because of the remote ED.
					50_000,
					Box::new(get_asset_hub_location().into())
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
			Asset {
				// 1 unit less than required
				fun: Fungibility::Fungible(999),
				..XCM_ASSET_FEE
			},
		)])
		.with_switch_pair_info(NewSwitchPairInfoOf::<MockRuntime> {
			pool_account,
			remote_asset_circulating_supply: 0,
			remote_asset_ed: 0,
			remote_asset_id: get_remote_erc20_asset_id().into(),
			remote_asset_total_supply: 100_000,
			remote_reserve_location: get_asset_hub_location().into(),
			remote_xcm_fee: XCM_ASSET_FEE.into(),
			status: SwitchPairStatus::Running,
		})
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Pallet::<MockRuntime>::switch(
					RawOrigin::Signed(user.clone()).into(),
					50_001,
					Box::new(get_asset_hub_location().into())
				),
				Error::<MockRuntime>::UserXcmBalance
			);
		});
}
