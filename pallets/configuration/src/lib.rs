// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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

//! # Configuration Pallet
//!
//! This pallet allows to change configurations without performing a runtime
//! upgrade.
//!
//! Currently the following configurations are supported:
//!
//! * `CheckAssociatedRelayNumber` of the parachain-system pallet

#![cfg_attr(not(feature = "std"), no_std)]

pub mod configuration;
pub mod default_weights;

#[cfg(any(feature = "mock", test))]
pub mod mock;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

/// Test module for CTypes
#[cfg(test)]
mod tests;

pub use crate::{configuration::Configuration, default_weights::WeightInfo, pallet::*};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::{
		pallet_prelude::*,
		traits::{EnsureOrigin, StorageVersion},
	};
	use frame_system::pallet_prelude::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;
		type EnsureOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// CTypes stored on chain.
	///
	/// It maps from a CType hash to its creator and block number in which it
	/// was created.
	#[pallet::storage]
	pub type ConfigurationStore<T> = StorageValue<_, Configuration, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new CType has been created.
		/// \[creator identifier, CType hash\]
		ConfigurationUpdate(Configuration),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// There is no CType with the given hash.
		Invalid,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new CType from the given unique CType hash and associates
		/// it with its creator.
		///
		/// A CType with the same hash must not be stored on chain.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_configuration())]
		pub fn set_configuration(origin: OriginFor<T>, configuration: Configuration) -> DispatchResult {
			<T as pallet::Config>::EnsureOrigin::ensure_origin(origin)?;

			ConfigurationStore::<T>::set(configuration.clone());
			Self::deposit_event(Event::<T>::ConfigurationUpdate(configuration));

			Ok(())
		}
	}
}
