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

//! # Relay migration initialization pallet
//!
//! This pallet changes the para ID and sends an XCM message to swap the
//! parachain lease.
//!
//! - [`Pallet`]

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use kilt_support::traits::RelayCallBuilder;
	use xcm::v2::{Junctions::Here, Parent};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	// TODO: Check whether tight coupling is fine
	pub trait Config:
		frame_system::Config
		+ parachain_info::Config
		+ pallet_xcm::Config
		+ polkadot_runtime_common::paras_registrar::Config
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type AdminOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;
		/// The Call builder for communicating with RelayChain via XCM
		/// messaging.
		type RelayChainCallBuilder: RelayCallBuilder<
			AccountId = Self::AccountId,
			Balance = polkadot_core_primitives::Balance,
		>;
	}

	#[pallet::storage]
	#[pallet::getter(fn para_id_changed)]
	pub type ParaIdChanged<T> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn lease_swap_pending)]
	pub type LeaseSwapPending<T> = StorageValue<_, bool, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		//// The parachain id was changed.
		ParaIdChanged {
			/// The old parachain id.
			old_id: u32,
			/// The new parachain id.
			new_id: u32,
		},
		/// The parachain lease swap was initiated.
		LeaseSwapInitiated,
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The lease swap has already been initiated.
		LeaseSwapAlreadyInitiated,
		/// The para id has already been changed throughout the lifetime of the
		/// parachain.
		ParaIdAlreadyChanged,
		/// The origin is not authorized to initiate the migration.
		Unauthorized,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn send_swap_call(
			origin: OriginFor<T>,
			relay_call: <<T as Config>::RelayChainCallBuilder as RelayCallBuilder>::RelayChainCall,
			relay_balance: u128,
			max_weight: u64,
		) -> DispatchResult {
			// FIXME: Check for authorization
			let who = ensure_signed(origin)?;
			ensure!(!LeaseSwapPending::<T>::get(), Error::<T>::LeaseSwapAlreadyInitiated);

			let xcm_message =
				T::RelayChainCallBuilder::finalize_call_into_xcm_message(relay_call, relay_balance, max_weight);

			let result = pallet_xcm::Pallet::<T>::send_xcm(Here, Parent, xcm_message);
			log::debug!("Sending XCM to swap para lease with result: {:?}", result);

			// TODO: Should probably just remove this storage entirely
			if result.is_ok() {
				LeaseSwapPending::<T>::put(true);
			}

			Self::deposit_event(Event::LeaseSwapInitiated);

			Ok(())
		}
		// TODO: Docs
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn initiate_relay_migration(origin: OriginFor<T>, new_para_id: u32) -> DispatchResult {
			// FIXME: Check for authorization
			let who = ensure_signed(origin)?;

			ensure!(!ParaIdChanged::<T>::get(), Error::<T>::ParaIdAlreadyChanged);

			// FIXME: Add raw writing of parachain ID (is declared as private 🥲)
			// <T as parachain_info::Config>::Pallet::ParachainId::insert(new_para_id);

			let old_para_id = parachain_info::Pallet::<T>::parachain_id();

			ParaIdChanged::<T>::put(true);

			Self::deposit_event(Event::ParaIdChanged {
				old_id: old_para_id.into(),
				new_id: new_para_id,
			});
			Ok(())
		}
	}
}
