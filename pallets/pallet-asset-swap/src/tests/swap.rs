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

use frame_support::{assert_ok, traits::fungible::Inspect};
use frame_system::RawOrigin;
use sp_runtime::{traits::Zero, AccountId32};

use crate::{
	mock::{Balances, ExtBuilder, MockRuntime, ASSET_HUB_LOCATION, REMOTE_ERC20_ASSET_ID, XCM_ASSET_FEE},
	swap::SwapPairStatus,
	Pallet, SwapPairInfoOf,
};

#[test]
fn successful() {
	let user = AccountId32::from([0; 32]);
	let pool_account = AccountId32::from([1; 32]);
	ExtBuilder::default()
		.with_balances(vec![(user.clone(), 100_000, 0, 0)])
		.with_swap_pair_info(SwapPairInfoOf::<MockRuntime> {
			pool_account,
			remote_asset_balance: 100_000,
			remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
			remote_fee: XCM_ASSET_FEE.into(),
			remote_reserve_location: ASSET_HUB_LOCATION.into(),
			status: SwapPairStatus::Running,
		})
		.build()
		.execute_with(|| {
			assert_ok!(Pallet::<MockRuntime>::swap(
				RawOrigin::Signed(user.clone()).into(),
				100_000,
				Box::new(ASSET_HUB_LOCATION.into())
			));
		});
	// User's currency balance is reduced by swap amount
	assert!(<Balances as Inspect<AccountId32>>::total_balance(&user).is_zero());
}

#[test]
fn fails_on_invalid_origin() {}

#[test]
fn fails_on_non_existing_pool() {}

#[test]
fn fails_on_pool_not_running() {}

#[test]
fn fails_on_not_enough_user_local_balance() {}

#[test]
fn fails_on_pool_balance_overflow() {}

#[test]
fn fails_on_not_enough_remote_balance() {}

#[test]
fn fails_on_not_enough_user_xcm_balance() {}
