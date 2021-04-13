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

use codec::{Decode, Encode};
use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure, StorageMap};
use frame_system::{self, ensure_signed};
use sp_std::fmt::Debug;

/// The CTYPE Config
pub trait Config: frame_system::Config + did::Config {
	/// CTYPE specific event type
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

	/// Weight information for extrinsics in this pallet.
	type WeightInfo: WeightInfo;
}

decl_event!(
	/// Events for CTYPEs
	pub enum Event<T> where <T as did::Config>::DidIdentifier, <T as frame_system::Config>::Hash {
		/// A CTYPE has been added
		CTypeCreated(DidIdentifier, Hash),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Config> {
		DidOperationError,
		AlreadyExists,
	}
}

#[derive(Clone, Decode, Encode, PartialEq)]
pub struct CtypeCreationOperation<T: Config> {
	creator_did: <T as did::Config>::DidIdentifier,
	hash: <T as frame_system::Config>::Hash,
	tx_counter: u64,
}

impl<T: Config> did::DidOperation<T> for CtypeCreationOperation<T> {
	fn get_verification_key_type(&self) -> did::DidVerificationKeyType {
		did::DidVerificationKeyType::AssertionMethod
	}

	fn get_did(&self) -> &T::DidIdentifier {
		&self.creator_did
	}

	// Irrelevant for creation operations.
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
		pub fn submit_ctype_creation_operation(origin, creation_operation: CtypeCreationOperation<T>, operation_signature: did::DidSignature) -> DispatchResult {
			// origin of the transaction needs to be a signed sender account
			ensure_signed(origin)?;

			let mut did_details = <did::Did<T>>::get(&creation_operation.creator_did).ok_or(<did::Error<T>>::DidNotPresent)?;

			did::pallet::Pallet::verify_operation_validity_for_did(&creation_operation, &operation_signature, &did_details).map_err(|_| <Error<T>>::DidOperationError)?;

			// check if CTYPE already exists
			ensure!(!<CTYPEs<T>>::contains_key(creation_operation.hash), Error::<T>::AlreadyExists);

			// add CTYPE to storage
			log::debug!("insert CTYPE");
			<CTYPEs<T>>::insert(creation_operation.hash, creation_operation.creator_did.clone());

			// Update tx counter in DID details and save to DID pallet
			did_details.increase_tx_counter().expect("Increasing DID tx counter should be a safe operation.");
			<did::Did<T>>::insert(creation_operation.creator_did.clone(), did_details);

			// deposit event that the CTYPE has been added
			Self::deposit_event(RawEvent::CTypeCreated(creation_operation.creator_did, creation_operation.hash));
			Ok(())
		}
	}
}

decl_storage! {
	trait Store for Module<T: Config> as Ctype {
		// CTYPEs: ctype-hash -> account-id?
		pub CTYPEs get(fn ctypes):map hasher(opaque_blake2_256) T::Hash => Option<<T as did::Config>::DidIdentifier>;
	}
}
