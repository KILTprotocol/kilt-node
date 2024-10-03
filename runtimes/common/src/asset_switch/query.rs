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

use frame_support::{pallet_prelude::ValueQuery, storage_alias, traits::DefensiveSaturating};
use pallet_asset_switch::traits::QueryIdProvider;
use sp_runtime::traits::One;
use sp_std::marker::PhantomData;
use xcm::v4::QueryId;

/// Query ID provider using the `pallet_xcm` `QueryCounter` storage element to
/// return unique IDs.
pub struct QueryIdProviderViaXcmPallet<Runtime>(PhantomData<Runtime>);

// Must match the definition of the `QueryCounter` storage value from pallet
// XCM.
#[storage_alias]
type QueryCounter<Runtime: pallet_xcm::Config> = StorageValue<pallet_xcm::Pallet<Runtime>, QueryId, ValueQuery>;

impl<Runtime> QueryIdProvider for QueryIdProviderViaXcmPallet<Runtime>
where
	Runtime: pallet_xcm::Config,
{
	fn next_id() -> QueryId {
		QueryCounter::<Runtime>::mutate(|id| {
			let ret = *id;
			id.defensive_saturating_accrue(QueryId::one());
			ret
		})
	}
}
