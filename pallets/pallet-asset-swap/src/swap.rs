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

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;
use xcm::{VersionedAssetId, VersionedInteriorMultiLocation};

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, RuntimeDebug, Clone)]
pub struct SwapPairRatio {
	pub local_asset: u128,
	pub remote_asset: u128,
}

impl SwapPairRatio {
	pub fn from_components((local_asset, remote_asset): (u128, u128)) -> Self {
		Self {
			local_asset,
			remote_asset,
		}
	}
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, RuntimeDebug, Clone)]
pub struct SwapPairInfo<AccountId> {
	pub pool_account: AccountId,
	pub ratio: SwapPairRatio,
	pub remote_asset_balance: u128,
	pub remote_asset_id: VersionedAssetId,
	pub running: bool,
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, RuntimeDebug, Clone)]
pub struct SwapRequestLocalAsset<Balance> {
	local_asset_id: VersionedInteriorMultiLocation,
	local_asset_amount: Balance,
}
