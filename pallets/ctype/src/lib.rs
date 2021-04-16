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

#[cfg(test)]
mod mock;

pub mod default_weights;
pub use default_weights::WeightInfo;

use codec::{Decode, Encode};
use frame_support::ensure;
use frame_system::{self, ensure_signed};
use sp_std::fmt::Debug;

pub use pallet::*;

/// An operation to create a new CTYPE.
/// The struct implements the DidOperation trait, and as such it must contain
/// information about the creator's DID, the type of DID key required to
/// verify the operation signature, and the tx counter to protect against replay
/// attacks. The struct has the following fields:
/// * creator_did: the DID of the CTYPE creator
/// * hash: the CTYPE hash
/// * tx_counter: the DID tx counter to mitigate replay attacks
#[derive(Clone, Decode, Encode, PartialEq)]
pub struct CtypeCreationOperation<T: Config> {
	creator_did: T::DidIdentifier,
	hash: T::Hash,
	tx_counter: u64,
}

impl<T: Config> did::DidOperation<T> for CtypeCreationOperation<T> {
	fn get_verification_key_type(&self) -> did::DidVerificationKeyType {
		did::DidVerificationKeyType::AssertionMethod
	}

	fn get_did(&self) -> &T::DidIdentifier {
		&self.creator_did
	}

	fn get_tx_counter(&self) -> u64 {
		self.tx_counter
	}
}

// Required to use a struct as an extrinsic parameter, and since Config does not
// implement Debug, the derive macro does not work.
impl<T: Config> Debug for CtypeCreationOperation<T> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		f.debug_tuple("CtypeCreationOperation")
			.field(&self.creator_did)
			.field(&self.hash)
			.field(&self.tx_counter)
			.finish()
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		traits::{Hooks, IsType},
	};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + did::Config {
		/// CTYPE specific event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::storage]
	#[pallet::getter(fn ctypes)]
	pub type Ctype<T> =
		StorageMap<_, Blake2_128Concat, <T as frame_system::Config>::Hash, <T as did::Config>::DidIdentifier>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CTypeCreated(T::DidIdentifier, T::Hash),
	}

	#[pallet::error]
	pub enum Error<T> {
		NotFound,
		AlreadyExists,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Adds a CTYPE on chain, where
		/// origin - the origin of the transaction
		/// hash - hash of the CTYPE of the claim
		#[pallet::weight(<T as Config>::WeightInfo::submit_ctype_creation_operation())]
		pub fn submit_ctype_creation_operation(
			origin: OriginFor<T>,
			operation: CtypeCreationOperation<T>,
			signature: did::DidSignature,
		) -> DispatchResultWithPostInfo {
			// origin of the transaction needs to be a signed sender account
			ensure_signed(origin)?;

			let mut did_details =
				<did::Did<T>>::get(&operation.creator_did).ok_or(<did::Error<T>>::DidNotPresent)?;

			did::pallet::Pallet::verify_operation_validity_for_did(
				&operation,
				&signature,
				&did_details,
			)
			.map_err(<did::Error<T>>::from)?;

			// check if CTYPE already exists
			ensure!(
				!<Ctype<T>>::contains_key(&operation.hash),
				Error::<T>::AlreadyExists
			);

			// add CTYPE to storage
			log::debug!("insert CTYPE");
			<Ctype<T>>::insert(&operation.hash, operation.creator_did.clone());

			// Update tx counter in DID details and save to DID pallet
			did_details
				.increase_tx_counter()
				.expect("Increasing DID tx counter should be a safe operation.");
			<did::Did<T>>::insert(&operation.creator_did, did_details);

			// deposit event that the CTYPE has been added
			Self::deposit_event(Event::CTypeCreated(
				operation.creator_did,
				operation.hash,
			));
			Ok(().into())
		}
	}
}
