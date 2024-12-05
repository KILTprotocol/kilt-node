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
	pallet_prelude::{PalletInfoAccess, ValueQuery},
	storage_alias,
};
use pallet_bonded_coins::{traits::NextAssetIds, FungiblesAssetIdOf};
use sp_runtime::{ArithmeticError, DispatchError};
use sp_std::{marker::PhantomData, vec::Vec};

use crate::bonded_currencies::AssetId;

#[storage_alias]
pub type NextAssetId<BondedFungibles: PalletInfoAccess, T> =
	StorageValue<BondedFungibles, FungiblesAssetIdOf<T>, ValueQuery>;

const LOG_TARGET: &str = "runtime::pallet_bonded_coins::hooks";

/// Struct to implement desired traits for [NextAssetId].
pub struct NextAssetIdGenerator<T>(PhantomData<T>);

/// impl NetAssetId for GetNextAssetIdStruct
impl<T: pallet_bonded_coins::Config, BondedFungibles: PalletInfoAccess> NextAssetIds<T>
	for NextAssetIdGenerator<BondedFungibles>
where
	FungiblesAssetIdOf<T>: From<AssetId> + Into<AssetId> + Default,
{
	type Error = DispatchError;
	fn try_get(n: u32) -> Result<Vec<FungiblesAssetIdOf<T>>, Self::Error> {
		let next_asset_id: AssetId = NextAssetId::<BondedFungibles, T>::get().into();

		let new_next_asset_id = next_asset_id.checked_add(n).ok_or(ArithmeticError::Overflow)?;

		let asset_ids = (next_asset_id..new_next_asset_id)
			.map(FungiblesAssetIdOf::<T>::from)
			.collect::<Vec<FungiblesAssetIdOf<T>>>();

		NextAssetId::<BondedFungibles, T>::set(new_next_asset_id.into());
		Ok(asset_ids)
	}
}
