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

//! CTYPE: Handles CTYPEs on chain,
//! adding CTYPEs.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod benchmarking;
/// Test module for CTYPEs
#[cfg(test)]
mod tests;

pub mod default_weights;
pub use default_weights::WeightInfo;

use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure, StorageMap};
use frame_system::{self, ensure_signed};

/// The CTYPE Config
pub trait Config: frame_system::Config {
	/// CTYPE specific event type
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

	/// Weight information for extrinsics in this pallet.
	type WeightInfo: WeightInfo;
}

decl_event!(
	/// Events for CTYPEs
	pub enum Event<T> where <T as frame_system::Config>::AccountId, <T as frame_system::Config>::Hash {
		/// A CTYPE has been added
		CTypeCreated(AccountId, Hash),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Config> {
		NotFound,
		AlreadyExists,
	}
}

decl_module! {
	/// The CTYPE runtime module
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		/// Deposit events
		fn deposit_event() = default;

		// Initializing errors
		// this includes information about your errors in the node's metadata.
		// it is needed only if you are using errors in your pallet
		type Error = Error<T>;

		/// Adds a CTYPE on chain, where
		/// origin - the origin of the transaction
		/// hash - hash of the CTYPE of the claim
		#[weight = <T as Config>::WeightInfo::add()]
		pub fn add(origin, hash: T::Hash) -> DispatchResult {
			// origin of the transaction needs to be a signed sender account
			let sender = ensure_signed(origin)?;

			// check if CTYPE already exists
			ensure!(!<CTYPEs<T>>::contains_key(hash), Error::<T>::AlreadyExists);

			// add CTYPE to storage
			log::debug!("insert CTYPE");
			<CTYPEs<T>>::insert(hash, sender.clone());
			// deposit event that the CTYPE has been added
			Self::deposit_event(RawEvent::CTypeCreated(sender, hash));
			Ok(())
		}
	}
}

decl_storage! {
	trait Store for Module<T: Config> as Ctype {
		// CTYPEs: ctype-hash -> account-id?
		pub CTYPEs get(fn ctypes):map hasher(opaque_blake2_256) T::Hash => Option<T::AccountId>;
	}
}
