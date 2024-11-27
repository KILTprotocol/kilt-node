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

use frame_support::{pallet_prelude::OptionQuery, storage_alias, traits::PalletInfoAccess};
use sp_core::Get;
use sp_std::marker::PhantomData;

use crate::AccountId;

// TODO: When upgrading to 1.8.0, which introduces a new `pallet-parameteres`,
// migrate this custom implementation into the pallet.
// The downside of this approach is that no try-runtime checks will be run on
// this piece of storage.
#[storage_alias]
type AllowedNameClaimerStorage<DotNamesDeployment: PalletInfoAccess> =
	StorageValue<DotNamesDeployment, AccountId, OptionQuery>;

/// Stored information about the allowed claimer inside the DotNames pallet,
/// without the pallet knowing about it.
pub struct AllowedNameClaimer<DotNamesDeployment>(PhantomData<DotNamesDeployment>);

impl<DotNamesDeployment> Get<Option<AccountId>> for AllowedNameClaimer<DotNamesDeployment>
where
	DotNamesDeployment: PalletInfoAccess,
{
	fn get() -> Option<AccountId> {
		AllowedNameClaimerStorage::<DotNamesDeployment>::get()
	}
}
