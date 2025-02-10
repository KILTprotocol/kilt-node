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

pub(super) mod dot_names {
	use did::origin::AuthorisedSubmitter;
	use frame_support::{pallet_prelude::OptionQuery, storage_alias, traits::PalletInfoAccess};
	use sp_core::Get;
	use sp_std::marker::PhantomData;

	use crate::AccountId;

	const LOG_TARGET: &str = "runtime::DotNames::AllowedDotNameClaimer";

	// TODO: When upgrading to 1.8.0, which introduces a new `pallet-parameteres`,
	// migrate this custom implementation into the pallet.
	// The downside of this approach is that no try-runtime checks will be run on
	// this piece of storage.
	#[storage_alias]
	type AllowedDotNameClaimerStorage<DotNamesDeployment: PalletInfoAccess> =
		StorageValue<DotNamesDeployment, AccountId, OptionQuery>;

	/// Stored information about the allowed claimer inside the DotNames pallet,
	/// without the pallet knowing about it.
	pub struct AllowedDotNameClaimer<DotNamesDeployment>(PhantomData<DotNamesDeployment>);

	impl<DotNamesDeployment> Get<AuthorisedSubmitter<AccountId>> for AllowedDotNameClaimer<DotNamesDeployment>
	where
		DotNamesDeployment: PalletInfoAccess,
	{
		fn get() -> AuthorisedSubmitter<AccountId> {
			let stored_account = AllowedDotNameClaimerStorage::<DotNamesDeployment>::get();
			log::trace!(target: LOG_TARGET, "Stored value for DotNames authorized account: {:#?}", stored_account);
			stored_account.map_or(AuthorisedSubmitter::None, |authorised_account| {
				authorised_account.into()
			})
		}
	}
}

pub(super) mod unique_linking {
	use did::origin::AuthorisedSubmitter;
	use frame_support::{pallet_prelude::OptionQuery, storage_alias, traits::PalletInfoAccess};
	use sp_core::Get;
	use sp_std::marker::PhantomData;

	use crate::AccountId;

	const LOG_TARGET: &str = "runtime::UniqueLinking::AllowedUniqueLinkingAssociator";

	// TODO: When upgrading to 1.8.0, which introduces a new `pallet-parameteres`,
	// migrate this custom implementation into the pallet.
	// The downside of this approach is that no try-runtime checks will be run on
	// this piece of storage.
	#[storage_alias]
	type AllowedUniqueLinkingAssociatorStorage<UniqueLinkingDeployment: PalletInfoAccess> =
		StorageValue<UniqueLinkingDeployment, AccountId, OptionQuery>;

	/// Stored information about the allowed claimer inside the UniqueLinking
	/// pallet, without the pallet knowing about it.
	pub struct AllowedUniqueLinkingAssociator<UniqueLinkingDeployment>(PhantomData<UniqueLinkingDeployment>);

	impl<UniqueLinkingDeployment> Get<AuthorisedSubmitter<AccountId>>
		for AllowedUniqueLinkingAssociator<UniqueLinkingDeployment>
	where
		UniqueLinkingDeployment: PalletInfoAccess,
	{
		fn get() -> AuthorisedSubmitter<AccountId> {
			let stored_account = AllowedUniqueLinkingAssociatorStorage::<UniqueLinkingDeployment>::get();
			log::trace!(target: LOG_TARGET, "Stored value for UniqueLinking authorized account: {:#?}", stored_account);
			stored_account.map_or(AuthorisedSubmitter::None, |authorised_account| {
				authorised_account.into()
			})
		}
	}
}
