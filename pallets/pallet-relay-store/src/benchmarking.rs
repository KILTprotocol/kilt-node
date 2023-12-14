// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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

use crate::{Config, Pallet};
use frame_benchmarking::v2::*;

#[benchmarks(
	where
	T: cumulus_pallet_parachain_system::Config
)]
mod benchmarks {
	use cumulus_pallet_parachain_system::RelaychainDataProvider;
	use sp_core::H256;
	use sp_runtime::{
		traits::{BlockNumberProvider, Get},
		BoundedVec,
	};

	use crate::{relay::RelayParentInfo, LatestBlockHeights, LatestRelayHeads};

	use super::*;

	#[benchmark]
	fn on_finalize() {
		let max_blocks_stored = T::MaxRelayBlocksStored::get();
		let latest_block_heights: BoundedVec<u32, T::MaxRelayBlocksStored> = (1..=max_blocks_stored)
			.collect::<Vec<_>>()
			.try_into()
			.expect("Should not fail to build BoundedVec for LatestBlockHeights");
		let latest_block_heads_iter = latest_block_heights.iter().map(|block_height| {
			(
				block_height,
				RelayParentInfo {
					relay_parent_storage_root: H256::default(),
				},
			)
		});
		latest_block_heads_iter
			.for_each(|(block_height, parent_info)| LatestRelayHeads::<T>::insert(block_height, parent_info));
		LatestBlockHeights::<T>::put(latest_block_heights);

		assert_eq!(
			LatestRelayHeads::<T>::iter().count(),
			max_blocks_stored as usize,
			"The maximum allowed number of blocks should be stored in storage."
		);

		let new_block_number = max_blocks_stored + 1;
		frame_system::Pallet::<T>::set_block_number(new_block_number.into());
		RelaychainDataProvider::<T>::set_block_number(new_block_number);

		#[block]
		{
			Pallet::<T>::on_finalize_internal(new_block_number.into())
		}

		assert!(
			LatestBlockHeights::<T>::get().contains(&new_block_number),
			"LatestBlockHeights should contain the information about the new block"
		);
		assert!(
			LatestRelayHeads::<T>::contains_key(new_block_number),
			"LatestRelayHeads should contain the information about the new block"
		);
	}

	#[cfg(test)]
	mod benchmarks_tests {
		use crate::Pallet;
		use frame_benchmarking::impl_benchmark_test_suite;

		impl_benchmark_test_suite!(
			Pallet,
			crate::mock::ExtBuilder::default().build_with_keystore(),
			crate::mock::TestRuntime,
		);
	}
}
