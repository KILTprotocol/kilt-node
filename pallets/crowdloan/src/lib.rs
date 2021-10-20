// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

//! # Crowdloan contributions Pallet
//!
//! Provides means of registering the contributors to the KILT crowdloan.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Genesis config
//!
//! The crowdloan contributions pallet depends on the [`GenesisConfig`].
//!
//! The genesis config sets the initial registrar account that can update the
//! pallet's storage.
//!
//! ## Assumptions
//!
//! - At any time, there is one and only one registrar account which can manage
//!   the pallet storage.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod default_weights;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use crate::{default_weights::WeightInfo, pallet::*};

#[frame_support::pallet]
pub mod pallet {
	use super::WeightInfo;
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, StorageVersion},
	};
	use frame_system::{pallet_prelude::*, EnsureOneOf, EnsureSigned};
	use sp_runtime::{
		traits::{BadOrigin, StaticLookup},
		Either,
	};

	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

	pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Currency type.
		type Currency: Currency<AccountIdOf<Self>>;
		/// Overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The origin allowed to change the registrar account.
		type EnsureRegistrarOrigin: EnsureOrigin<Self::Origin>;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub registrar_account: AccountIdOf<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				registrar_account: AccountIdOf::<T>::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			RegistrarAccount::<T>::set(self.registrar_account.clone());
		}
	}

	/// The registrar account allowed to manage the pallet storage.
	#[pallet::storage]
	#[pallet::getter(fn registrar_account)]
	pub type RegistrarAccount<T> = StorageValue<_, AccountIdOf<T>, ValueQuery>;

	/// The set of contributions.
	///
	/// It maps from contributor's account to amount contributed.
	#[pallet::storage]
	#[pallet::getter(fn contributions)]
	pub type Contributions<T> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, BalanceOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new registrar has been set.
		/// \[old registrar account, new registrar account\]
		NewRegistrarAccountSet(AccountIdOf<T>, AccountIdOf<T>),
		/// A contribution has been set.
		/// \[contributor account, old amount (OPTIONAL), new amount\]
		ContributionSet(AccountIdOf<T>, Option<BalanceOf<T>>, BalanceOf<T>),
		/// A contribution has been removed.
		/// \[contributor account\]
		ContributionRemoved(AccountIdOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The contribution is not present.
		ContributorNotPresent,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Sets a new account as the registrar of this pallet.
		///
		/// The dispatch origin can be either Sudo or the current registrar
		/// account.
		///
		/// Emits `NewRegistrarAccountSet`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: [Origin Account], RegistrarAccount
		/// - Writes: RegistrarAccount
		#[pallet::weight(T::WeightInfo::set_registrar_account())]
		pub fn set_registrar_account(
			origin: OriginFor<T>,
			new_account: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let who =
				EnsureOneOf::<AccountIdOf<T>, T::EnsureRegistrarOrigin, EnsureSigned<AccountIdOf<T>>>::ensure_origin(
					origin,
				)?;

			let old_account = RegistrarAccount::<T>::get();

			if let Either::Right(signed_origin) = who {
				ensure!(signed_origin == old_account, BadOrigin);
			}

			let looked_up_account = <T as frame_system::Config>::Lookup::lookup(new_account)?;
			RegistrarAccount::<T>::set(looked_up_account.clone());

			Self::deposit_event(Event::NewRegistrarAccountSet(old_account, looked_up_account));

			Ok(())
		}

		/// Sets a new contribution amount for a given contributor's account.
		///
		/// If a previous contribution is present, it is overridden.
		///
		/// The dispatch origin must be the current registrar account.
		///
		/// Emits `NewContributionSet`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: [Origin Account], RegistrarAccount, Contributions
		/// - Writes: Contributions
		#[pallet::weight(T::WeightInfo::set_contribution())]
		pub fn set_contribution(
			origin: OriginFor<T>,
			contributor_account: <T::Lookup as StaticLookup>::Source,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(who == RegistrarAccount::<T>::get(), BadOrigin);

			let looked_up_account = <T as frame_system::Config>::Lookup::lookup(contributor_account)?;
			let old_amount = Contributions::<T>::mutate(&looked_up_account, |entry| entry.replace(amount));

			Self::deposit_event(Event::ContributionSet(looked_up_account, old_amount, amount));

			Ok(())
		}

		/// Removes a contribution entry from the storage, if present.
		///
		/// It returns an error if there is no contribution for the given
		/// contributor's account.
		///
		/// The dispatch origin must be the current registrar account.
		///
		/// Emits `ContributionRemoved`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: [Origin Account], RegistrarAccount, Contributions
		/// - Writes: Contributions
		#[pallet::weight(T::WeightInfo::remove_contribution())]
		pub fn remove_contribution(
			origin: OriginFor<T>,
			contributor_account: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(who == RegistrarAccount::<T>::get(), BadOrigin);

			let looked_up_account = <T as frame_system::Config>::Lookup::lookup(contributor_account)?;
			Contributions::<T>::take(&looked_up_account).ok_or(Error::<T>::ContributorNotPresent)?;

			Self::deposit_event(Event::ContributionRemoved(looked_up_account));

			Ok(())
		}
	}
}
