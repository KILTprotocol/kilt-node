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
//! This pallet provides means of sending an XCM messages to the relay chain by
//! a configurable origin and switching the associated relay number block checks
//! between strictly and any.
//!
//! - [`Pallet`]

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use kilt_support::traits::RelayCallBuilder;
	use sp_std::vec::Vec;
	use xcm::v2::{Junctions::Here, Parent};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_xcm::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Origin from which calls of this pallet can be made.
		type ApproveOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

		/// The Call builder for communicating with RelayChain via XCM
		/// messaging.
		type RelayChainCallBuilder: RelayCallBuilder<
			AccountId = Self::AccountId,
			Balance = polkadot_core_primitives::Balance,
		>;
	}

	/// Switch between RelayNumberStrictlyIncreases and AnyRelayNumber.
	#[pallet::storage]
	#[pallet::getter(fn relay_block_number_strictly_increases)]
	pub(crate) type RelayNumberStrictlyIncreases<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The parachain lease swap was initiated.
		LeaseSwapInitiated,
		/// The requirement for associated relay block numbers was set
		RelayNumberCheckSet {
			/// Reflects setting to RelayNumberStrictlyIncreases
			strict: bool,
		},
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set an XCM call to the relay chain.
		///
		/// Has to be done pre migration.
		#[pallet::weight(1_000_000 + T::DbWeight::get().reads_writes(10, 10))]
		pub fn send_swap_call_bytes(
			origin: OriginFor<T>,
			relay_call: Vec<u8>,
			relay_balance: u128,
			max_weight: u64,
		) -> DispatchResult {
			T::ApproveOrigin::ensure_origin(origin)?;
			let xcm_message =
				T::RelayChainCallBuilder::finalize_call_into_xcm_message(relay_call, relay_balance, max_weight);

			let result = pallet_xcm::Pallet::<T>::send_xcm(Here, Parent, xcm_message);
			log::debug!("Sending XCM with result: {:?}", result);

			Self::deposit_event(Event::LeaseSwapInitiated);

			Ok(())
		}

		/// Set the associated relay block number to be
		/// RelayNumberStrictlyIncreases.
		///
		/// Has to be done post migration.
		#[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn enable_strict_relay_number_check(origin: OriginFor<T>) -> DispatchResult {
			T::ApproveOrigin::ensure_origin(origin)?;
			RelayNumberStrictlyIncreases::<T>::put(true);

			Self::deposit_event(Event::RelayNumberCheckSet { strict: true });

			Ok(())
		}

		/// Set the associated relay block number to be AnyRelayNumber.
		///
		/// Has to be done pre migration.
		#[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn disable_strict_relay_number_check(origin: OriginFor<T>) -> DispatchResult {
			T::ApproveOrigin::ensure_origin(origin)?;
			RelayNumberStrictlyIncreases::<T>::put(false);

			Self::deposit_event(Event::RelayNumberCheckSet { strict: false });

			Ok(())
		}
	}

	impl<T: Config> cumulus_pallet_parachain_system::CheckAssociatedRelayNumber for Pallet<T> {
		fn check_associated_relay_number(
			current: polkadot_core_primitives::BlockNumber,
			previous: polkadot_core_primitives::BlockNumber,
		) {
			if RelayNumberStrictlyIncreases::<T>::get() {
				cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases::check_associated_relay_number(
					current, previous,
				)
			} else {
				cumulus_pallet_parachain_system::AnyRelayNumber::check_associated_relay_number(current, previous)
			}
		}
	}
}
