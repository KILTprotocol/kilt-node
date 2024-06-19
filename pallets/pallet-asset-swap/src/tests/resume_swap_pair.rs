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
	mock::{ExtBuilder, MockRuntime, System, ASSET_HUB_LOCATION, REMOTE_ERC20_ASSET_ID, XCM_ASSET_FEE},
	swap::SwapPairStatus,
	Error, Event, Pallet, SwapPair, SwapPairInfoOf,
};

#[test]
fn successful() {
	// Resuming a non-running swap pair generates an event.
	ExtBuilder::default()
		.with_swap_pair_info(SwapPairInfoOf::<MockRuntime> {
			pool_account: [0u8; 32].into(),
			remote_asset_balance: 1_000,
			remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
			remote_fee: XCM_ASSET_FEE.into(),
			remote_reserve_location: ASSET_HUB_LOCATION.into(),
			status: SwapPairStatus::Paused,
		})
		.build()
		.execute_with(|| {
			assert_ok!(Pallet::<MockRuntime>::resume_swap_pair(RawOrigin::Root.into()));
			assert_eq!(SwapPair::<MockRuntime>::get().unwrap().status, SwapPairStatus::Running);
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::SwapPairResumed {
					remote_asset_id: REMOTE_ERC20_ASSET_ID.into()
				}
				.into()));
		});
	// Resuming a running swap pair generates no event.
	ExtBuilder::default()
		.with_swap_pair_info(SwapPairInfoOf::<MockRuntime> {
			pool_account: [0u8; 32].into(),
			remote_asset_balance: 1_000,
			remote_asset_id: REMOTE_ERC20_ASSET_ID.into(),
			remote_fee: XCM_ASSET_FEE.into(),
			remote_reserve_location: ASSET_HUB_LOCATION.into(),
			status: SwapPairStatus::Running,
		})
		.build()
		.execute_with(|| {
			assert_ok!(Pallet::<MockRuntime>::resume_swap_pair(RawOrigin::Root.into()));
			assert_eq!(SwapPair::<MockRuntime>::get().unwrap().status, SwapPairStatus::Running);
			assert!(System::events().into_iter().map(|e| e.event).all(|e| e
				!= Event::<MockRuntime>::SwapPairResumed {
					remote_asset_id: REMOTE_ERC20_ASSET_ID.into()
				}
				.into()));
		});
}

#[test]
fn fails_on_non_existing_pair() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Pallet::<MockRuntime>::resume_swap_pair(RawOrigin::Root.into()),
			Error::<MockRuntime>::NotFound
		);
	});
}

#[test]
fn fails_on_invalid_origin() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Pallet::<MockRuntime>::resume_swap_pair(RawOrigin::None.into()),
			DispatchError::BadOrigin
		);
	});
}
