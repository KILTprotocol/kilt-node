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

use sp_runtime::traits::Zero;
use xcm::{
	v4::{Asset, AssetId, Fungibility, Location},
	VersionedAsset, VersionedAssetId, VersionedLocation,
};

use crate::{Config, NewSwitchPairInfoOf, SwitchPairStatus};

pub(super) fn get_switch_pair_info_for_remote_location_with_pool_usable_balance<Runtime>(
	location: &Location,
	pool_usable_balance: u64,
	status: SwitchPairStatus,
) -> NewSwitchPairInfoOf<Runtime>
where
	Runtime: Config,
	Runtime::AccountId: From<[u8; 32]>,
{
	NewSwitchPairInfoOf::<Runtime> {
		pool_account: Runtime::AccountId::from([1; 32]),
		remote_asset_id: VersionedAssetId::V4(AssetId(location.clone())),
		remote_reserve_location: VersionedLocation::V4(location.clone()),
		remote_xcm_fee: VersionedAsset::V4(Asset {
			id: AssetId(location.clone()),
			fun: Fungibility::Fungible(1),
		}),
		remote_asset_total_supply: (u64::MAX as u128) + pool_usable_balance as u128,
		remote_asset_circulating_supply: pool_usable_balance as u128,
		remote_asset_ed: u128::zero(),
		status,
	}
}

pub(super) fn get_switch_pair_info_for_remote_location<Runtime>(
	location: &Location,
	status: SwitchPairStatus,
) -> NewSwitchPairInfoOf<Runtime>
where
	Runtime: Config,
	Runtime::AccountId: From<[u8; 32]>,
{
	get_switch_pair_info_for_remote_location_with_pool_usable_balance::<Runtime>(location, 0, status)
}
