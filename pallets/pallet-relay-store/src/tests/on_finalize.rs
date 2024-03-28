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

use frame_support::traits::Hooks;
use sp_runtime::traits::Zero;

use crate::{
	mock::{ExtBuilder, System, TestRuntime},
	relay::RelayParentInfo,
	LatestBlockHeights, LatestRelayHeads, Pallet,
};

#[test]
fn on_finalize_empty_state() {
	ExtBuilder::default()
		.with_new_relay_state_root((1, [100; 32].into()))
		.build()
		.execute_with(|| {
			assert!(LatestRelayHeads::<TestRuntime>::iter().count().is_zero());
			assert!(LatestBlockHeights::<TestRuntime>::get().is_empty());

			Pallet::<TestRuntime>::on_finalize(System::block_number());

			assert_eq!(
				LatestRelayHeads::<TestRuntime>::iter().collect::<Vec<_>>(),
				vec![(
					1,
					RelayParentInfo {
						relay_parent_storage_root: [100; 32].into()
					}
				)]
			);
			assert_eq!(LatestBlockHeights::<TestRuntime>::get(), vec![1]);
		});
}

// This should never happen, but we add a test to make sure the code in here
// never panics.
#[test]
fn on_finalize_empty_validation_data() {
	ExtBuilder::default().build().execute_with(|| {
		Pallet::<TestRuntime>::on_finalize(System::block_number());
		assert!(LatestRelayHeads::<TestRuntime>::iter().count().is_zero());
		assert!(LatestBlockHeights::<TestRuntime>::get().is_empty());
	});
}

#[test]
fn on_finalize_full_state() {
	ExtBuilder::default()
		.with_stored_relay_roots(vec![
			(1, [1; 32].into()),
			(2, [2; 32].into()),
			(3, [3; 32].into()),
			(4, [4; 32].into()),
			(5, [5; 32].into()),
		])
		.with_new_relay_state_root((6, [6; 32].into()))
		.build()
		.execute_with(|| {
			Pallet::<TestRuntime>::on_finalize(System::block_number());
			assert!(LatestRelayHeads::<TestRuntime>::get(1).is_none(),);
			assert_eq!(
				LatestRelayHeads::<TestRuntime>::get(6),
				Some(RelayParentInfo {
					relay_parent_storage_root: [6; 32].into()
				})
			);
			assert_eq!(LatestBlockHeights::<TestRuntime>::get(), vec![2, 3, 4, 5, 6]);
		});
}
