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

use xcm::{VersionedAssetId, VersionedInteriorMultiLocation};

pub trait SwapHooks {
	type Error: Into<u8>;

	fn on_swap_pair_created(
		local_asset_id: &VersionedInteriorMultiLocation,
		remote_asset_id: &VersionedAssetId,
	) -> Result<(), Self::Error>;

	fn on_swap_pair_removed(
		local_asset_id: &VersionedInteriorMultiLocation,
		remote_asset_id: &VersionedAssetId,
	) -> Result<(), Self::Error>;

	fn on_swap_pair_paused(
		local_asset_id: &VersionedInteriorMultiLocation,
		remote_asset_id: &VersionedAssetId,
	) -> Result<(), Self::Error>;

	fn on_swap_pair_resumed(
		local_asset_id: &VersionedInteriorMultiLocation,
		remote_asset_id: &VersionedAssetId,
	) -> Result<(), Self::Error>;
}

impl SwapHooks for () {
	type Error = u8;

	fn on_swap_pair_created(
		_local_asset_id: &VersionedInteriorMultiLocation,
		_remote_asset_id: &VersionedAssetId,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn on_swap_pair_paused(
		_local_asset_id: &VersionedInteriorMultiLocation,
		_remote_asset_id: &VersionedAssetId,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn on_swap_pair_removed(
		_local_asset_id: &VersionedInteriorMultiLocation,
		_remote_asset_id: &VersionedAssetId,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn on_swap_pair_resumed(
		_local_asset_id: &VersionedInteriorMultiLocation,
		_remote_asset_id: &VersionedAssetId,
	) -> Result<(), Self::Error> {
		Ok(())
	}
}
