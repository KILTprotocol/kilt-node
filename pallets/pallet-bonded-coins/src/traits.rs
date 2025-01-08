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
use frame_support::{dispatch::DispatchResult, traits::fungibles::roles::Inspect};
use frame_system::RawOrigin;
use pallet_assets::{Config as AssetConfig, Pallet as AssetsPallet};
use sp_runtime::{traits::StaticLookup, DispatchError};
use sp_std::{fmt::Debug, prelude::*, vec::Vec};

use crate::{AccountIdOf, Config, FungiblesAssetIdOf};

/// A trait for freezing and thawing accounts.
pub trait FreezeAccounts<AccountId, AssetId> {
	type Error: Into<DispatchError> + Debug;

	/// Freeze the account `who` for the asset `asset_id`.
	fn freeze(asset_id: &AssetId, who: &AccountId) -> Result<(), Self::Error>;

	/// Thaw the account `who` for the asset `asset_id`.
	fn thaw(asset_id: &AssetId, who: &AccountId) -> Result<(), Self::Error>;
}

type AssetIdOf<T, I> = <T as AssetConfig<I>>::AssetId;

impl<T, I> FreezeAccounts<AccountIdOf<T>, <T as AssetConfig<I>>::AssetId> for AssetsPallet<T, I>
where
	I: 'static,
	T: AssetConfig<I>,
	AccountIdOf<T>: Clone,
	<<T as frame_system::Config>::Lookup as StaticLookup>::Source: From<AccountIdOf<T>>,
{
	type Error = DispatchError;

	fn freeze(asset_id: &AssetIdOf<T, I>, who: &AccountIdOf<T>) -> Result<(), Self::Error> {
		let owned_asset_id: <T as AssetConfig<I>>::AssetId = asset_id.to_owned();
		let freezer = AssetsPallet::<T, I>::freezer(owned_asset_id.clone()).ok_or(Self::Error::Unavailable)?;
		let origin = RawOrigin::Signed(freezer);
		AssetsPallet::<T, I>::freeze(origin.into(), owned_asset_id.into(), who.to_owned().into())
	}

	fn thaw(asset_id: &AssetIdOf<T, I>, who: &AccountIdOf<T>) -> Result<(), Self::Error> {
		let owned_asset_id: <T as AssetConfig<I>>::AssetId = asset_id.to_owned();
		let admin = AssetsPallet::<T, I>::admin(owned_asset_id.clone()).ok_or(Self::Error::Unavailable)?;
		let origin = RawOrigin::Signed(admin);
		AssetsPallet::<T, I>::thaw(origin.into(), owned_asset_id.into(), who.to_owned().into())
	}
}

/// Copy from the Polkadot SDK. once we are at version 1.13.0, we can remove
/// this.
pub trait ResetTeam<AccountId>: Inspect<AccountId> {
	/// Reset the team for the asset with the given `id`.
	///
	/// ### Parameters
	/// - `id`: The identifier of the asset for which the team is being reset.
	/// - `owner`: The new `owner` account for the asset.
	/// - `admin`: The new `admin` account for the asset.
	/// - `issuer`: The new `issuer` account for the asset.
	/// - `freezer`: The new `freezer` account for the asset.
	fn reset_team(
		id: Self::AssetId,
		owner: AccountId,
		admin: AccountId,
		issuer: AccountId,
		freezer: AccountId,
	) -> DispatchResult;
}

impl<T, I: 'static> ResetTeam<AccountIdOf<T>> for AssetsPallet<T, I>
where
	T: AssetConfig<I>,
	<<T as frame_system::Config>::Lookup as StaticLookup>::Source: From<AccountIdOf<T>>,
{
	fn reset_team(
		id: Self::AssetId,
		_owner: AccountIdOf<T>,
		admin: AccountIdOf<T>,
		issuer: AccountIdOf<T>,
		freezer: AccountIdOf<T>,
	) -> DispatchResult {
		let owner = AssetsPallet::<T, I>::owner(id.clone()).ok_or(DispatchError::Unavailable)?;
		let origin = RawOrigin::Signed(owner);
		AssetsPallet::<T, I>::set_team(origin.into(), id.into(), issuer.into(), admin.into(), freezer.into())
	}
}

/// A trait for getting the next n asset ids to be used during pool creation.
pub trait NextAssetIds<T: Config> {
	/// Generic error type.
	type Error: Into<DispatchError>;
	/// Get the next `n` asset ids.
	fn try_get(n: u32) -> Result<Vec<FungiblesAssetIdOf<T>>, Self::Error>;
}
