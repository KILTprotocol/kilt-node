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

use frame_support::weights::WeightToFee;
use xcm::{
	v3::{AssetId, Fungibility, MultiAsset, MultiLocation, Weight},
	VersionedAssetId, VersionedMultiAsset, VersionedMultiLocation,
};
use xcm_executor::traits::WeightTrader;

use crate::{Config, NewSwitchPairInfoOf};

pub(super) fn get_switch_pair_info_for_remote_location<Runtime>(
	location: &MultiLocation,
) -> NewSwitchPairInfoOf<Runtime>
where
	Runtime: Config,
	Runtime::AccountId: From<[u8; 32]>,
{
	NewSwitchPairInfoOf::<Runtime> {
		pool_account: Runtime::AccountId::from([1; 32]),
		remote_asset_id: VersionedAssetId::V3(AssetId::Concrete(*location)),
		remote_reserve_location: VersionedMultiLocation::V3(*location),
		remote_xcm_fee: VersionedMultiAsset::V3(MultiAsset {
			id: AssetId::Concrete(*location),
			fun: Fungibility::Fungible(1),
		}),
		remote_asset_circulating_supply: Default::default(),
		remote_asset_ed: Default::default(),
		remote_asset_total_supply: Default::default(),
		status: Default::default(),
	}
}

#[derive(Debug, Clone)]
pub(super) struct SumTimeAndProofValues;

impl WeightToFee for SumTimeAndProofValues {
	type Balance = u128;

	fn weight_to_fee(weight: &Weight) -> Self::Balance {
		(weight.ref_time() + weight.proof_size()) as u128
	}
}

pub(super) fn is_weigher_unchanged<Weigher>(weigher: &Weigher) -> bool
where
	Weigher: WeightTrader + PartialEq,
{
	weigher == &Weigher::new()
}
