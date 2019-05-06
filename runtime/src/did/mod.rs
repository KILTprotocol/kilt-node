
//! DID: Handles decentralized identifiers on chain,
//! adding and removing DIDs.

/// Test module for attestations
#[cfg(test)]
mod tests;

use rstd::prelude::*;
use runtime_primitives::traits::{Member};
use support::{dispatch::Result, StorageMap, Parameter, decl_module, decl_storage, decl_event};
use runtime_primitives::codec::Codec;
use {system, system::ensure_signed};

/// The DID trait
pub trait Trait: system::Trait {
	/// DID specific event type
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	/// Public signing key type for DIDs
    type PublicSigningKey : Parameter + Member + Codec + Default;
	/// Public boxing key type for DIDs
    type PublicBoxKey : Parameter + Member + Codec + Default;
}

decl_event!(
	/// Events for DIDs
	pub enum Event<T> where <T as system::Trait>::AccountId {
		/// A did has been created
		DidCreated(AccountId),
		/// A did has been removed
		DidRemoved(AccountId),
	}
);

decl_module! {
	/// The DID runtime module
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		/// Deposit events
		fn deposit_event<T>() = default;

		/// Adds a DID on chain, where
		/// origin - the origin of the transaction
		/// sign_key - public signing key of the DID
		/// box_key - public boxing key of the DID
		/// doc_ref - optional reference to the DID document storage
		pub fn add(origin, sign_key: T::PublicSigningKey, box_key: T::PublicBoxKey, doc_ref: Option<Vec<u8>>) -> Result {
			// origin of the transaction needs to be a signed sender account
			let sender = ensure_signed(origin)?;
			// add DID to the storage
			<DIDs<T>>::insert(sender.clone(), (sign_key, box_key, doc_ref));
			// deposit an event that the DID has been created
			Self::deposit_event(RawEvent::DidCreated(sender.clone()));
            Ok(())
		}
		
		/// Removes a DID from chain storage, where
		/// origin - the origin of the transaction
        pub fn remove(origin) -> Result {
			// origin of the transaction needs to be a signed sender account
			let sender = ensure_signed(origin)?;
			// remove DID from storage
			<DIDs<T>>::remove(sender.clone());
			// deposit an event that the DID has been removed
			Self::deposit_event(RawEvent::DidRemoved(sender.clone()));
            Ok(())
		}
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as DID {
		// DID: account-id -> (public-signing-key, public-encryption-key, did-reference?)
		DIDs get(dids): map T::AccountId => (T::PublicSigningKey, T::PublicBoxKey, Option<Vec<u8>>);
	}
}

