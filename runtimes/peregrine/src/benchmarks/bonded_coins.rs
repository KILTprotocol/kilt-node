// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.org>

use frame_support::traits::fungible::Mutate;
use pallet_bonded_coins::{BenchmarkHelper, CollateralAssetIdOf, Config, FungiblesAssetIdOf};
use sp_runtime::SaturatedConversion;
use xcm::v4::{Junction, Junctions, Location};

use crate::kilt::BondedFungiblesInstance;
pub struct BondedFungiblesBenchmarkHelper<T>(sp_std::marker::PhantomData<T>);

impl<T: Config + pallet_assets::Config + pallet_assets::Config<BondedFungiblesInstance> + pallet_balances::Config>
	BenchmarkHelper<T> for BondedFungiblesBenchmarkHelper<T>
where
	FungiblesAssetIdOf<T>: From<u32>,
	CollateralAssetIdOf<T>: From<Location>,
{
	fn calculate_bonded_asset_id(seed: u32) -> FungiblesAssetIdOf<T> {
		FungiblesAssetIdOf::<T>::from(seed)
	}

	fn calculate_collateral_asset_id(seed: u32) -> CollateralAssetIdOf<T> {
		CollateralAssetIdOf::<T>::from(Location {
			parents: 0,
			interior: Junctions::X1([Junction::GeneralIndex(seed.into())].into()),
		})
	}

	fn set_native_balance(who: &<T as frame_system::Config>::AccountId, amount: u128) {
		pallet_balances::Pallet::<T>::set_balance(who, amount.saturated_into());
	}
}
