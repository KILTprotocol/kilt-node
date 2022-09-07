// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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

//! # Dynamic Call Filter
//!
//! Enable or disable specific features without a runtime upgrade.
//!
//! - [`Pallet`]

#![cfg_attr(not(feature = "std"), no_std)]

pub mod default_weights;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod setting;

pub use crate::{default_weights::WeightInfo, pallet::*};

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		pallet_prelude::*,
		traits::{Contains, StorageVersion},
	};
	use frame_system::pallet_prelude::*;

	use crate::{setting::FilterSettings, WeightInfo};

	pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The origin check for the authorised entities that can change the
		/// filter.
		type ApproveOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

		/// TransferCall filters all calls that allow to transfer funds.
		type TransferCall: Contains<<Self as frame_system::Config>::Call>;

		/// FeatureCall filters all calls that provide the utility of the chain.
		type FeatureCall: Contains<<Self as frame_system::Config>::Call>;

		/// XcmCall filters all calls that send messages to other chains
		type XcmCall: Contains<<Self as frame_system::Config>::Call>;

		/// System calls are not filtered. (SystemCall contains all calls that
		/// are needed for block production, return true if system call)
		type SystemCall: Contains<<Self as frame_system::Config>::Call>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn filter_setting)]
	pub type Filter<T> = StorageValue<_, FilterSettings, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewFilterRules { rules: FilterSettings },
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::set_filter())]
		pub fn set_filter(origin: OriginFor<T>, filter: FilterSettings) -> DispatchResult {
			T::ApproveOrigin::ensure_origin(origin)?;

			Filter::<T>::set(filter);
			Self::deposit_event(Event::<T>::NewFilterRules { rules: filter });
			Ok(())
		}
	}

	impl<T: Config> Contains<T::Call> for Pallet<T> {
		/// The provided call goes through if this returns `true`. Else, it
		/// fails.
		fn contains(t: &T::Call) -> bool {
			// System relevant calls cannot be filtered
			if T::SystemCall::contains(t) {
				return true;
			}

			let FilterSettings {
				transfer_disabled,
				feature_disabled,
				xcm_disabled,
			} = Filter::<T>::get();

			!((transfer_disabled && T::TransferCall::contains(t))
				|| (feature_disabled && T::FeatureCall::contains(t))
				|| (xcm_disabled && T::XcmCall::contains(t)))
		}
	}
}
