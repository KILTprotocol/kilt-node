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

//! # DID lookup pallet
//!
//! This pallet stores a map from account IDs to DIDs.
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

pub use crate::{default_weights::WeightInfo, pallet::*};

#[frame_support::pallet]
pub mod pallet {
	use super::WeightInfo;
	use frame_support::{ensure, pallet_prelude::*, traits::StorageVersion};
	use frame_system::pallet_prelude::*;

	use kilt_support::traits::CallSources;
	use sp_runtime::traits::{IdentifyAccount, Verify};

	/// The identifier to which the accounts can be associated.
	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	/// The identifier to which the accounts can be associated.
	pub(crate) type DidAccountOf<T> = <T as Config>::DidAccount;

	pub(crate) type SignatureOf<T> = <T as Config>::Signature;

	pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Signature: Verify<Signer = Self::Signer> + Parameter;
		type Signer: IdentifyAccount<AccountId = AccountIdOf<Self>> + Parameter;

		/// The origin that can associate accounts to itself.
		type EnsureOrigin: EnsureOrigin<Success = Self::OriginSuccess, <Self as frame_system::Config>::Origin>;

		/// The information that is returned by the origin check.
		type OriginSuccess: CallSources<AccountIdOf<Self>, DidAccountOf<Self>>;

		/// The identifier to which accounts can get associated.
		type DidAccount: Parameter + Default;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	/// Mapping from account identifiers to DIDs.
	#[pallet::storage]
	#[pallet::getter(fn connected_dids)]
	pub type ConnectedDids<T> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, DidAccountOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new association between a DID and an account ID was created.
		AssociationEstablished(AccountIdOf<T>, DidAccountOf<T>),

		/// An association between a DID and an account ID was removed.
		AssociationRemoved(AccountIdOf<T>, DidAccountOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The association does not exist.
		AssociationNotFound,

		/// The origin was not allowed to manage the association between the DID
		/// and the account ID.
		NotAuthorized,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Associate the given account to the DID that authorized this call.
		///
		/// The account has to sign the DID in order to authorize the
		/// association.
		///
		/// Emits `AssociationEstablished` and, optionally, `AssociationRemoved`
		/// if there was a previous association for the account.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: ConnectedDids + DID Origin Check
		/// - Writes: ConnectedDids
		/// # </weight>
		#[pallet::weight(<T as Config>::WeightInfo::associate_account())]
		pub fn associate_account(
			origin: OriginFor<T>,
			account: AccountIdOf<T>,
			proof: SignatureOf<T>,
		) -> DispatchResult {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;
			let did_account = source.subject();

			ensure!(
				proof.verify(&did_account.encode()[..], &account),
				Error::<T>::NotAuthorized
			);
			Self::add_association(did_account, account);

			Ok(())
		}

		/// Associate the sender of the call to the DID that authorized this
		/// call.
		///
		/// Emits `AssociationEstablished` and, optionally, `AssociationRemoved`
		/// if there was a previous association for the account.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: ConnectedDids + DID Origin Check
		/// - Writes: ConnectedDids
		/// # </weight>
		#[pallet::weight(<T as Config>::WeightInfo::associate_sender())]
		pub fn associate_sender(origin: OriginFor<T>) -> DispatchResult {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;

			Self::add_association(source.subject(), source.sender());
			Ok(())
		}

		/// Remove the association of the sender account. This call doesn't
		/// require the authorization of the DID.
		///
		/// Emits `AssociationRemoved`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: ConnectedDids
		/// - Writes: ConnectedDids
		/// # </weight>
		#[pallet::weight(<T as Config>::WeightInfo::remove_sender_association())]
		pub fn remove_sender_association(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::remove_association(who)
		}

		/// Remove the association of the provided account ID. This call doesn't
		/// require the authorization of the account ID, but the associated DID
		/// needs to match the DID that authorized this call.
		///
		/// Emits `AssociationRemoved`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: ConnectedDids + DID Origin Check
		/// - Writes: ConnectedDids
		/// # </weight>
		#[pallet::weight(<T as Config>::WeightInfo::remove_account_association())]
		pub fn remove_account_association(origin: OriginFor<T>, account: AccountIdOf<T>) -> DispatchResult {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;

			let did_account = ConnectedDids::<T>::get(&account).ok_or(Error::<T>::AssociationNotFound)?;
			ensure!(did_account == source.subject(), Error::<T>::NotAuthorized);

			Self::remove_association(account)
		}
	}

	impl<T: Config> Pallet<T> {
		fn add_association(did_account: DidAccountOf<T>, account: AccountIdOf<T>) {
			ConnectedDids::<T>::mutate(&account, |did_entry| {
				if let Some(old_did) = did_entry.replace(did_account.clone()) {
					Self::deposit_event(Event::<T>::AssociationRemoved(account.clone(), old_did));
				}
			});
			Self::deposit_event(Event::AssociationEstablished(account, did_account));
		}

		fn remove_association(account: AccountIdOf<T>) -> DispatchResult {
			if let Some(did_account) = ConnectedDids::<T>::take(&account) {
				Self::deposit_event(Event::AssociationRemoved(account, did_account));
				Ok(())
			} else {
				Err(Error::<T>::AssociationNotFound.into())
			}
		}
	}
}
