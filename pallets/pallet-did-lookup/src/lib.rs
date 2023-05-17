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

//! # DID lookup pallet
//!
//! This pallet stores a map from account IDs to DIDs.
//!
//! - [`Pallet`]

#![cfg_attr(not(feature = "std"), no_std)]

pub mod account;
pub mod associate_account_request;
pub mod default_weights;
pub mod linkable_account;
pub mod migrations;

mod connection_record;
mod signature;

#[cfg(all(test, feature = "std"))]
mod tests;

#[cfg(all(test, feature = "std"))]
mod mock;

#[cfg(any(feature = "try-runtime", test))]
mod try_state;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use crate::{default_weights::WeightInfo, pallet::*};

#[frame_support::pallet]
pub mod pallet {
	use crate::{
		associate_account_request::AssociateAccountRequest, default_weights::WeightInfo,
		linkable_account::LinkableAccountId,
	};

	use frame_support::{
		ensure,
		pallet_prelude::*,
		traits::{Currency, ReservableCurrency, StorageVersion},
	};
	use frame_system::pallet_prelude::*;
	use kilt_support::{
		deposit::Deposit,
		traits::{CallSources, StorageDepositCollector},
	};

	use sp_runtime::traits::BlockNumberProvider;

	pub use crate::connection_record::ConnectionRecord;

	/// The native identifier for accounts in this runtime.
	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	/// The identifier to which the accounts can be associated.
	pub(crate) type DidIdentifierOf<T> = <T as Config>::DidIdentifier;

	/// The type used to describe a balance.
	pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

	/// The currency module that keeps track of balances.
	pub(crate) type CurrencyOf<T> = <T as Config>::Currency;

	/// The connection record type.
	pub(crate) type ConnectionRecordOf<T> = ConnectionRecord<DidIdentifierOf<T>, AccountIdOf<T>, BalanceOf<T>>;

	pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(4);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The origin that can associate accounts to itself.
		type EnsureOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin, Success = Self::OriginSuccess>;

		/// The information that is returned by the origin check.
		type OriginSuccess: CallSources<AccountIdOf<Self>, DidIdentifierOf<Self>>;

		/// The identifier to which accounts can get associated.
		type DidIdentifier: Parameter + AsRef<[u8]> + MaxEncodedLen + MaybeSerializeDeserialize;

		/// The currency that is used to reserve funds for each did.
		type Currency: ReservableCurrency<AccountIdOf<Self>>;

		/// The amount of balance that will be taken for each DID as a deposit
		/// to incentivise fair use of the on chain storage. The deposit can be
		/// reclaimed when the DID is deleted.
		#[pallet::constant]
		type Deposit: Get<BalanceOf<Self>>;

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
	pub type ConnectedDids<T> = StorageMap<_, Blake2_128Concat, LinkableAccountId, ConnectionRecordOf<T>>;

	/// Mapping from (DID + account identifier) -> ().
	/// The empty tuple is used as a sentinel value to simply indicate the
	/// presence of a given tuple in the map.
	#[pallet::storage]
	#[pallet::getter(fn connected_accounts)]
	pub type ConnectedAccounts<T> =
		StorageDoubleMap<_, Blake2_128Concat, DidIdentifierOf<T>, Blake2_128Concat, LinkableAccountId, ()>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new association between a DID and an account ID was created.
		AssociationEstablished(LinkableAccountId, DidIdentifierOf<T>),

		/// An association between a DID and an account ID was removed.
		AssociationRemoved(LinkableAccountId, DidIdentifierOf<T>),

		/// There was some progress in the migration process.
		MigrationProgress,

		/// All AccountIds have been migrated to LinkableAccountId.
		MigrationCompleted,
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The association does not exist.
		NotFound,

		/// The origin was not allowed to manage the association between the DID
		/// and the account ID.
		NotAuthorized,

		/// The supplied proof of ownership was outdated.
		OutdatedProof,

		/// The account has insufficient funds and can't pay the fees or reserve
		/// the deposit.
		InsufficientFunds,

		/// The ConnectedAccounts and ConnectedDids storage are out of sync.
		///
		/// NOTE: this will only be returned if the storage has inconsistencies.
		Migration,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub links: Vec<(LinkableAccountId, ConnectionRecordOf<T>)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				links: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// populate link records
			for (acc, connection) in &self.links {
				ConnectedDids::<T>::insert(acc, connection);
				ConnectedAccounts::<T>::insert(&connection.did, acc, ());
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		#[cfg(feature = "try-runtime")]
		fn try_state(_n: BlockNumberFor<T>) -> Result<(), &'static str> {
			crate::try_state::do_try_state::<T>()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T::AccountId: Into<LinkableAccountId>,
		T::AccountId: From<sp_runtime::AccountId32>,
		T::AccountId: Into<sp_runtime::AccountId32>,
	{
		/// Associate the given account to the DID that authorized this call.
		///
		/// The account has to sign the DID and a blocknumber after which the
		/// signature expires in order to authorize the association.
		///
		/// The signature will be checked against the scale encoded tuple of the
		/// method specific id of the did identifier and the block number after
		/// which the signature should be regarded invalid.
		///
		/// Emits `AssociationEstablished` and, optionally, `AssociationRemoved`
		/// if there was a previous association for the account.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: ConnectedDids + ConnectedAccounts + DID Origin Check
		/// - Writes: ConnectedDids + ConnectedAccounts
		/// # </weight>
		#[pallet::call_index(0)]
		#[pallet::weight(
			<T as Config>::WeightInfo::associate_account_multisig_sr25519().max(
			<T as Config>::WeightInfo::associate_account_multisig_ed25519().max(
			<T as Config>::WeightInfo::associate_account_multisig_ecdsa().max(
			<T as Config>::WeightInfo::associate_eth_account()
		))))]
		pub fn associate_account(
			origin: OriginFor<T>,
			req: AssociateAccountRequest,
			expiration: <T as frame_system::Config>::BlockNumber,
		) -> DispatchResult {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;
			let did_identifier = source.subject();
			let sender = source.sender();

			ensure!(
				frame_system::Pallet::<T>::current_block_number() <= expiration,
				Error::<T>::OutdatedProof
			);

			ensure!(
				<T::Currency as ReservableCurrency<AccountIdOf<T>>>::can_reserve(
					&sender,
					<T as Config>::Deposit::get()
				),
				Error::<T>::InsufficientFunds
			);

			ensure!(
				req.verify::<T::DidIdentifier, T::BlockNumber>(&did_identifier, expiration),
				Error::<T>::NotAuthorized
			);

			Self::add_association(sender, did_identifier, req.get_linkable_account())?;

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
		/// - Reads: ConnectedDids + ConnectedAccounts + DID Origin Check
		/// - Writes: ConnectedDids + ConnectedAccounts
		/// # </weight>
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::associate_sender())]
		pub fn associate_sender(origin: OriginFor<T>) -> DispatchResult {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;

			ensure!(
				<T::Currency as ReservableCurrency<AccountIdOf<T>>>::can_reserve(
					&source.sender(),
					<T as Config>::Deposit::get()
				),
				Error::<T>::InsufficientFunds
			);

			Self::add_association(source.sender(), source.subject(), source.sender().into())?;
			Ok(())
		}

		/// Remove the association of the sender account. This call doesn't
		/// require the authorization of the DID, but requires a signed origin.
		///
		/// Emits `AssociationRemoved`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: ConnectedDids + ConnectedAccounts + DID Origin Check
		/// - Writes: ConnectedDids + ConnectedAccounts
		/// # </weight>
		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::remove_sender_association())]
		pub fn remove_sender_association(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::remove_association(who.into())
		}

		/// Remove the association of the provided account ID. This call doesn't
		/// require the authorization of the account ID, but the associated DID
		/// needs to match the DID that authorized this call.
		///
		/// Emits `AssociationRemoved`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: ConnectedDids + ConnectedAccounts + DID Origin Check
		/// - Writes: ConnectedDids + ConnectedAccounts
		/// # </weight>
		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::remove_account_association())]
		pub fn remove_account_association(origin: OriginFor<T>, account: LinkableAccountId) -> DispatchResult {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;

			let connection_record = ConnectedDids::<T>::get(&account).ok_or(Error::<T>::NotFound)?;
			ensure!(connection_record.did == source.subject(), Error::<T>::NotAuthorized);

			Self::remove_association(account)
		}

		/// Remove the association of the provided account. This call can only
		/// be called from the deposit owner. The reserved deposit will be
		/// freed.
		///
		/// Emits `AssociationRemoved`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: ConnectedDids
		/// - Writes: ConnectedDids
		/// # </weight>
		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config>::WeightInfo::remove_sender_association())]
		pub fn reclaim_deposit(origin: OriginFor<T>, account: LinkableAccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let record = ConnectedDids::<T>::get(&account).ok_or(Error::<T>::NotFound)?;
			ensure!(record.deposit.owner == who, Error::<T>::NotAuthorized);
			Self::remove_association(account)
		}

		/// Changes the deposit owner.
		///
		/// The balance that is reserved by the current deposit owner will be
		/// freed and balance of the new deposit owner will get reserved.
		///
		/// The subject of the call must be linked to the account.
		/// The sender of the call will be the new deposit owner.
		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::change_deposit_owner())]
		pub fn change_deposit_owner(origin: OriginFor<T>, account: LinkableAccountId) -> DispatchResult {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;
			let subject = source.subject();

			let record = ConnectedDids::<T>::get(&account).ok_or(Error::<T>::NotFound)?;
			ensure!(record.did == subject, Error::<T>::NotAuthorized);

			LinkableAccountDepositCollector::<T>::change_deposit_owner(&account, source.sender())
		}

		/// Updates the deposit amount to the current deposit rate.
		///
		/// The sender must be the deposit owner.
		#[pallet::call_index(6)]
		#[pallet::weight(<T as Config>::WeightInfo::update_deposit())]
		pub fn update_deposit(origin: OriginFor<T>, account: LinkableAccountId) -> DispatchResult {
			let source = ensure_signed(origin)?;

			let record = ConnectedDids::<T>::get(&account).ok_or(Error::<T>::NotFound)?;
			ensure!(record.deposit.owner == source, Error::<T>::NotAuthorized);

			LinkableAccountDepositCollector::<T>::update_deposit(&account)
		}

		// Old call that was used to migrate
		// #[pallet::call_index(254)]
		// pub fn migrate(origin: OriginFor<T>, limit: u32) -> DispatchResult
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn add_association(
			sender: AccountIdOf<T>,
			did_identifier: DidIdentifierOf<T>,
			account: LinkableAccountId,
		) -> DispatchResult {
			let deposit = Deposit {
				owner: sender,
				amount: T::Deposit::get(),
			};
			let record = ConnectionRecord {
				deposit,
				did: did_identifier.clone(),
			};

			CurrencyOf::<T>::reserve(&record.deposit.owner, record.deposit.amount)?;

			ConnectedDids::<T>::mutate(&account, |did_entry| {
				if let Some(old_connection) = did_entry.replace(record) {
					ConnectedAccounts::<T>::remove(&old_connection.did, &account);
					Self::deposit_event(Event::<T>::AssociationRemoved(account.clone(), old_connection.did));
					kilt_support::free_deposit::<AccountIdOf<T>, CurrencyOf<T>>(&old_connection.deposit);
				}
			});
			ConnectedAccounts::<T>::insert(&did_identifier, &account, ());
			Self::deposit_event(Event::AssociationEstablished(account, did_identifier));

			Ok(())
		}

		pub(crate) fn remove_association(account: LinkableAccountId) -> DispatchResult {
			if let Some(connection) = ConnectedDids::<T>::take(&account) {
				ConnectedAccounts::<T>::remove(&connection.did, &account);
				kilt_support::free_deposit::<AccountIdOf<T>, CurrencyOf<T>>(&connection.deposit);
				Self::deposit_event(Event::AssociationRemoved(account, connection.did));

				Ok(())
			} else {
				Err(Error::<T>::NotFound.into())
			}
		}
	}

	struct LinkableAccountDepositCollector<T: Config>(PhantomData<T>);
	impl<T: Config> StorageDepositCollector<AccountIdOf<T>, LinkableAccountId> for LinkableAccountDepositCollector<T> {
		type Currency = T::Currency;

		fn deposit(
			key: &LinkableAccountId,
		) -> Result<Deposit<AccountIdOf<T>, <Self::Currency as Currency<AccountIdOf<T>>>::Balance>, DispatchError> {
			let record = ConnectedDids::<T>::get(key).ok_or(Error::<T>::NotFound)?;
			Ok(record.deposit)
		}

		fn deposit_amount(_key: &LinkableAccountId) -> <Self::Currency as Currency<AccountIdOf<T>>>::Balance {
			T::Deposit::get()
		}

		fn store_deposit(
			key: &LinkableAccountId,
			deposit: Deposit<AccountIdOf<T>, <Self::Currency as Currency<AccountIdOf<T>>>::Balance>,
		) -> Result<(), DispatchError> {
			let record = ConnectedDids::<T>::get(key).ok_or(Error::<T>::NotFound)?;
			ConnectedDids::<T>::insert(key, ConnectionRecord { deposit, ..record });

			Ok(())
		}
	}
}
