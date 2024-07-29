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
	mock::{get_asset_hub_location, get_remote_erc20_asset_id, ExtBuilder, MockRuntime, System, XCM_ASSET_FEE},
	switch::SwitchPairStatus,
	Error, Event, NewSwitchPairInfoOf, Pallet, SwitchPair,
};

#[test]
fn successful() {
	// Resuming a non-running switch pair generates an event.
	ExtBuilder::default()
		.with_switch_pair_info(NewSwitchPairInfoOf::<MockRuntime> {
			pool_account: [0u8; 32].into(),
			remote_asset_circulating_supply: 0,
			remote_asset_ed: 0,
			remote_asset_id: get_remote_erc20_asset_id().into(),
			remote_asset_total_supply: 1_000,
			remote_reserve_location: get_asset_hub_location().into(),
			remote_xcm_fee: XCM_ASSET_FEE.into(),
			status: SwitchPairStatus::Paused,
		})
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(Pallet::<MockRuntime>::resume_switch_pair(RawOrigin::Root.into()));
			assert_eq!(
				SwitchPair::<MockRuntime>::get().unwrap().status,
				SwitchPairStatus::Running
			);
			assert!(System::events().into_iter().map(|e| e.event).any(|e| e
				== Event::<MockRuntime>::SwitchPairResumed {
					remote_asset_id: get_remote_erc20_asset_id().into()
				}
				.into()));
		});
	// Resuming a running switch pair generates no event.
	ExtBuilder::default()
		.with_switch_pair_info(NewSwitchPairInfoOf::<MockRuntime> {
			pool_account: [0u8; 32].into(),
			remote_asset_circulating_supply: 0,
			remote_asset_ed: 0,
			remote_asset_id: get_remote_erc20_asset_id().into(),
			remote_asset_total_supply: 1_000,
			remote_reserve_location: get_asset_hub_location().into(),
			remote_xcm_fee: XCM_ASSET_FEE.into(),
			status: SwitchPairStatus::Running,
		})
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(Pallet::<MockRuntime>::resume_switch_pair(RawOrigin::Root.into()));
			assert_eq!(
				SwitchPair::<MockRuntime>::get().unwrap().status,
				SwitchPairStatus::Running
			);
			assert!(System::events().into_iter().map(|e| e.event).all(|e| e
				!= Event::<MockRuntime>::SwitchPairResumed {
					remote_asset_id: get_remote_erc20_asset_id().into()
				}
				.into()));
		});
}

#[test]
fn fails_on_non_existing_pair() {
	ExtBuilder::default().build_and_execute_with_sanity_tests(|| {
		assert_noop!(
			Pallet::<MockRuntime>::resume_switch_pair(RawOrigin::Root.into()),
			Error::<MockRuntime>::SwitchPairNotFound
		);
	});
}

#[test]
fn fails_on_invalid_origin() {
	ExtBuilder::default().build_and_execute_with_sanity_tests(|| {
		assert_noop!(
			Pallet::<MockRuntime>::resume_switch_pair(RawOrigin::None.into()),
			DispatchError::BadOrigin
		);
	});
}
