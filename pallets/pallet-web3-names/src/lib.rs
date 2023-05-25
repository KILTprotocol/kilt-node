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

//! # Pallet storing unique nickname <-> DID links for user-friendly DID
//! nicknames.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod web3_name;

mod default_weights;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod mock;

#[cfg(any(test, feature = "try-runtime"))]
mod try_state;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use crate::{default_weights::WeightInfo, pallet::*};

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::SaturatedConversion,
		traits::{Currency, ReservableCurrency, StorageVersion},
		Blake2_128Concat,
	};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::FullCodec;
	use sp_std::{fmt::Debug, vec::Vec};

	use kilt_support::{
		deposit::Deposit,
		traits::{CallSources, StorageDepositCollector},
	};

	use super::WeightInfo;
	use crate::web3_name::Web3NameOwnership;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type BlockNumberFor<T> = <T as frame_system::Config>::BlockNumber;
	pub type Web3NameOwnerOf<T> = <T as Config>::Web3NameOwner;
	pub type Web3NameInput<T> = BoundedVec<u8, <T as Config>::MaxNameLength>;
	pub type Web3NameOf<T> = <T as Config>::Web3Name;
	pub type Web3OwnershipOf<T> =
		Web3NameOwnership<Web3NameOwnerOf<T>, Deposit<AccountIdOf<T>, BalanceOf<T>>, BlockNumberFor<T>>;

	pub(crate) type CurrencyOf<T> = <T as Config>::Currency;
	pub type BalanceOf<T> = <CurrencyOf<T> as Currency<AccountIdOf<T>>>::Balance;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	/// Map of name -> ownership details.
	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type Owner<T> = StorageMap<_, Blake2_128Concat, Web3NameOf<T>, Web3OwnershipOf<T>>;

	/// Map of owner -> name.
	#[pallet::storage]
	#[pallet::getter(fn names)]
	pub type Names<T> = StorageMap<_, Blake2_128Concat, Web3NameOwnerOf<T>, Web3NameOf<T>>;

	/// Map of name -> ().
	///
	/// If a name key is present, the name is currently banned.
	#[pallet::storage]
	#[pallet::getter(fn is_banned)]
	pub type Banned<T> = StorageMap<_, Blake2_128Concat, Web3NameOf<T>, ()>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The origin allowed to ban names.
		type BanOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// The origin allowed to perform regular operations.
		type OwnerOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin, Success = Self::OriginSuccess>;
		/// The type of origin after a successful origin check.
		type OriginSuccess: CallSources<AccountIdOf<Self>, Web3NameOwnerOf<Self>>;
		/// The currency type to reserve and release deposits.
		type Currency: ReservableCurrency<AccountIdOf<Self>>;
		/// The amount of KILT to deposit to claim a name.
		#[pallet::constant]
		type Deposit: Get<BalanceOf<Self>>;
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The min encoded length of a name.
		#[pallet::constant]
		type MinNameLength: Get<u32>;
		/// The max encoded length of a name.
		#[pallet::constant]
		type MaxNameLength: Get<u32>;
		// FIXME: Refactor the definition of AsciiWeb3Name so that we don't need to
		// require `Ord` here
		/// The type of a name.
		type Web3Name: FullCodec
			+ Debug
			+ PartialEq
			+ Clone
			+ TypeInfo
			+ TryFrom<Vec<u8>, Error = Error<Self>>
			+ MaxEncodedLen
			+ Ord;
		/// The type of a name owner.
		type Web3NameOwner: Parameter + MaxEncodedLen;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new name has been claimed.
		Web3NameClaimed {
			owner: Web3NameOwnerOf<T>,
			name: Web3NameOf<T>,
		},
		/// A name has been released.
		Web3NameReleased {
			owner: Web3NameOwnerOf<T>,
			name: Web3NameOf<T>,
		},
		/// A name has been banned.
		Web3NameBanned { name: Web3NameOf<T> },
		/// A name has been unbanned.
		Web3NameUnbanned { name: Web3NameOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The tx submitter does not have enough funds to pay for the deposit.
		InsufficientFunds,
		/// The specified name has already been previously claimed.
		AlreadyExists,
		/// The specified name does not exist.
		NotFound,
		/// The specified owner already owns a name.
		OwnerAlreadyExists,
		/// The specified owner does not own any names.
		OwnerNotFound,
		/// The specified name has been banned and cannot be interacted
		/// with.
		Banned,
		/// The specified name is not currently banned.
		NotBanned,
		/// The specified name has already been previously banned.
		AlreadyBanned,
		/// The actor cannot performed the specified operation.
		NotAuthorized,
		/// A name that is too short is being claimed.
		TooShort,
		/// A name that is too long is being claimed.
		TooLong,
		/// A name that contains not allowed characters is being claimed.
		InvalidCharacter,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		#[cfg(feature = "try-runtime")]
		fn try_state(_n: BlockNumberFor<T>) -> Result<(), &'static str> {
			crate::try_state::do_try_state::<T>()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Assign the specified name to the owner as specified in the
		/// origin.
		///
		/// The name must not have already been claimed by someone else and the
		/// owner must not already own another name.
		///
		/// Emits `Web3NameClaimed` if the operation is carried out
		/// successfully.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: Names, Owner, Banned storage entries + available currency
		///   check + origin check
		/// - Writes: Names, Owner storage entries + currency deposit reserve
		/// # </weight>
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::claim(name.len().saturated_into()))]
		pub fn claim(origin: OriginFor<T>, name: Web3NameInput<T>) -> DispatchResult {
			let origin = T::OwnerOrigin::ensure_origin(origin)?;
			let payer = origin.sender();
			let owner = origin.subject();

			let decoded_name = Self::check_claiming_preconditions(name, &owner, &payer)?;

			Self::register_name(decoded_name.clone(), owner.clone(), payer);
			Self::deposit_event(Event::<T>::Web3NameClaimed {
				owner,
				name: decoded_name,
			});

			Ok(())
		}

		/// Release the provided name from its owner.
		///
		/// The origin must be the owner of the specified name.
		///
		/// Emits `Web3NameReleased` if the operation is carried out
		/// successfully.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: Names storage entry + origin check
		/// - Writes: Names, Owner storage entries + currency deposit release
		/// # </weight>
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::release_by_owner())]
		pub fn release_by_owner(origin: OriginFor<T>) -> DispatchResult {
			let origin = T::OwnerOrigin::ensure_origin(origin)?;
			let owner = origin.subject();

			let owned_name = Self::check_releasing_preconditions(&owner)?;

			Self::unregister_name(&owned_name);
			Self::deposit_event(Event::<T>::Web3NameReleased {
				owner,
				name: owned_name,
			});

			Ok(())
		}

		/// Release the provided name from its owner.
		///
		/// The origin must be the account that paid for the name's deposit.
		///
		/// Emits `Web3NameReleased` if the operation is carried out
		/// successfully.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: Owner storage entry + origin check
		/// - Writes: Names, Owner storage entries + currency deposit release
		/// # </weight>
		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::reclaim_deposit(name.len().saturated_into()))]
		pub fn reclaim_deposit(origin: OriginFor<T>, name: Web3NameInput<T>) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			let decoded_name = Self::check_reclaim_deposit_preconditions(name, &caller)?;

			let Web3OwnershipOf::<T> { owner, .. } = Self::unregister_name(&decoded_name);
			Self::deposit_event(Event::<T>::Web3NameReleased {
				owner,
				name: decoded_name,
			});

			Ok(())
		}

		/// Ban a name.
		///
		/// A banned name cannot be claimed by anyone. The name's deposit
		/// is returned to the original payer.
		///
		/// The origin must be the ban origin.
		///
		/// Emits `Web3NameBanned` if the operation is carried out
		/// successfully.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: Banned, Owner, Names storage entries + origin check
		/// - Writes: Names, Owner, Banned storage entries + currency deposit
		///   release
		/// # </weight>
		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::ban(name.len().saturated_into()))]
		pub fn ban(origin: OriginFor<T>, name: Web3NameInput<T>) -> DispatchResult {
			T::BanOrigin::ensure_origin(origin)?;

			let (decoded_name, is_claimed) = Self::check_banning_preconditions(name)?;

			if is_claimed {
				Self::unregister_name(&decoded_name);
			}

			Self::ban_name(&decoded_name);
			Self::deposit_event(Event::<T>::Web3NameBanned { name: decoded_name });

			Ok(())
		}

		/// Unban a name.
		///
		/// Make a name claimable again.
		///
		/// The origin must be the ban origin.
		///
		/// Emits `Web3NameUnbanned` if the operation is carried out
		/// successfully.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: Banned storage entry + origin check
		/// - Writes: Banned storage entry deposit release
		/// # </weight>
		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config>::WeightInfo::unban(name.len().saturated_into()))]
		pub fn unban(origin: OriginFor<T>, name: Web3NameInput<T>) -> DispatchResult {
			T::BanOrigin::ensure_origin(origin)?;

			let decoded_name = Self::check_unbanning_preconditions(name)?;

			Self::unban_name(&decoded_name);
			Self::deposit_event(Event::<T>::Web3NameUnbanned { name: decoded_name });

			Ok(())
		}

		/// Changes the deposit owner.
		///
		/// The balance that is reserved by the current deposit owner will be
		/// freed and balance of the new deposit owner will get reserved.
		///
		/// The subject of the call must be the owner of the web3name.
		/// The sender of the call will be the new deposit owner.
		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::change_deposit_owner())]
		pub fn change_deposit_owner(origin: OriginFor<T>) -> DispatchResult {
			let source = <T as Config>::OwnerOrigin::ensure_origin(origin)?;
			let w3n_owner = source.subject();
			let name = Names::<T>::get(&w3n_owner).ok_or(Error::<T>::NotFound)?;
			Web3NameStorageDepositCollector::<T>::change_deposit_owner(&name, source.sender())?;

			Ok(())
		}

		/// Updates the deposit amount to the current deposit rate.
		///
		/// The sender must be the deposit owner.
		#[pallet::call_index(6)]
		#[pallet::weight(<T as Config>::WeightInfo::update_deposit())]
		pub fn update_deposit(origin: OriginFor<T>, name_input: Web3NameInput<T>) -> DispatchResult {
			let source = ensure_signed(origin)?;
			let name = Web3NameOf::<T>::try_from(name_input.into_inner()).map_err(DispatchError::from)?;
			let w3n_entry = Owner::<T>::get(&name).ok_or(Error::<T>::NotFound)?;
			ensure!(w3n_entry.deposit.owner == source, Error::<T>::NotAuthorized);

			Web3NameStorageDepositCollector::<T>::update_deposit(&name)?;

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Verify that the claiming preconditions are verified. Specifically:
		/// - The name input data can be decoded as a valid name
		/// - The name does not already exist
		/// - The owner does not already own a name
		/// - The name has not been banned
		/// - The tx submitter has enough funds to pay the deposit
		fn check_claiming_preconditions(
			name_input: Web3NameInput<T>,
			owner: &Web3NameOwnerOf<T>,
			deposit_payer: &AccountIdOf<T>,
		) -> Result<Web3NameOf<T>, DispatchError> {
			let name = Web3NameOf::<T>::try_from(name_input.into_inner()).map_err(DispatchError::from)?;

			ensure!(!Names::<T>::contains_key(owner), Error::<T>::OwnerAlreadyExists);
			ensure!(!Owner::<T>::contains_key(&name), Error::<T>::AlreadyExists);
			ensure!(!Banned::<T>::contains_key(&name), Error::<T>::Banned);

			ensure!(
				<T::Currency as ReservableCurrency<AccountIdOf<T>>>::can_reserve(deposit_payer, T::Deposit::get()),
				Error::<T>::InsufficientFunds
			);

			Ok(name)
		}

		/// Assign a name to the provided owner reserving the deposit from
		/// the provided account. This function must be called after
		/// `check_claiming_preconditions` as it does not verify all the
		/// preconditions again.
		pub(crate) fn register_name(name: Web3NameOf<T>, owner: Web3NameOwnerOf<T>, deposit_payer: AccountIdOf<T>) {
			let deposit = Deposit {
				owner: deposit_payer,
				amount: T::Deposit::get(),
			};
			let block_number = frame_system::Pallet::<T>::block_number();

			CurrencyOf::<T>::reserve(&deposit.owner, deposit.amount).unwrap();

			Names::<T>::insert(&owner, name.clone());
			Owner::<T>::insert(
				&name,
				Web3OwnershipOf::<T> {
					owner,
					claimed_at: block_number,
					deposit,
				},
			);
		}

		/// Verify that the releasing preconditions for an owner are verified.
		/// Specifically:
		/// - The owner has a previously claimed name
		fn check_releasing_preconditions(owner: &Web3NameOwnerOf<T>) -> Result<Web3NameOf<T>, DispatchError> {
			let name = Names::<T>::get(owner).ok_or(Error::<T>::OwnerNotFound)?;

			Ok(name)
		}

		/// Verify that the releasing preconditions for a deposit payer are
		/// verified. Specifically:
		/// - The name input data can be decoded as a valid name
		/// - The name exists (i.e., it has been previous claimed)
		/// - The caller owns the name's deposit
		fn check_reclaim_deposit_preconditions(
			name_input: Web3NameInput<T>,
			caller: &AccountIdOf<T>,
		) -> Result<Web3NameOf<T>, DispatchError> {
			let name = Web3NameOf::<T>::try_from(name_input.into_inner()).map_err(DispatchError::from)?;
			let Web3NameOwnership { deposit, .. } = Owner::<T>::get(&name).ok_or(Error::<T>::NotFound)?;

			ensure!(caller == &deposit.owner, Error::<T>::NotAuthorized);

			Ok(name)
		}

		/// Release the provided name and returns the deposit to the
		/// original payer. This function must be called after
		/// `check_releasing_preconditions` as it does not verify all the
		/// preconditions again.
		fn unregister_name(name: &Web3NameOf<T>) -> Web3OwnershipOf<T> {
			let name_ownership = Owner::<T>::take(name).unwrap();
			Names::<T>::remove(&name_ownership.owner);

			kilt_support::free_deposit::<AccountIdOf<T>, CurrencyOf<T>>(&name_ownership.deposit);

			name_ownership
		}

		/// Verify that the banning preconditions are verified.
		/// Specifically:
		/// - The name input data can be decoded as a valid name
		/// - The name must not be already banned
		///
		/// If the preconditions are verified, return
		/// a tuple containing the parsed name value and whether the name
		/// being banned is currently assigned to someone or not.
		fn check_banning_preconditions(name_input: Web3NameInput<T>) -> Result<(Web3NameOf<T>, bool), DispatchError> {
			let name = Web3NameOf::<T>::try_from(name_input.into_inner()).map_err(DispatchError::from)?;

			ensure!(!Banned::<T>::contains_key(&name), Error::<T>::AlreadyBanned);

			let is_claimed = Owner::<T>::contains_key(&name);

			Ok((name, is_claimed))
		}

		/// Ban the provided name. This function must be called after
		/// `check_banning_preconditions` as it does not verify all the
		/// preconditions again.
		pub(crate) fn ban_name(name: &Web3NameOf<T>) {
			Banned::<T>::insert(name, ());
		}

		/// Verify that the unbanning preconditions are verified.
		/// Specifically:
		/// - The name input data can be decoded as a valid name
		/// - The name must have already been banned
		fn check_unbanning_preconditions(name_input: Web3NameInput<T>) -> Result<Web3NameOf<T>, DispatchError> {
			let name = Web3NameOf::<T>::try_from(name_input.into_inner()).map_err(DispatchError::from)?;

			ensure!(Banned::<T>::contains_key(&name), Error::<T>::NotBanned);

			Ok(name)
		}

		/// Unban the provided name. This function must be called after
		/// `check_unbanning_preconditions` as it does not verify all the
		/// preconditions again.
		fn unban_name(name: &Web3NameOf<T>) {
			Banned::<T>::remove(name);
		}
	}

	struct Web3NameStorageDepositCollector<T: Config>(PhantomData<T>);
	impl<T: Config> StorageDepositCollector<AccountIdOf<T>, T::Web3Name> for Web3NameStorageDepositCollector<T> {
		type Currency = T::Currency;

		fn deposit(
			key: &T::Web3Name,
		) -> Result<Deposit<AccountIdOf<T>, <Self::Currency as Currency<AccountIdOf<T>>>::Balance>, DispatchError> {
			let w3n_entry = Owner::<T>::get(key).ok_or(Error::<T>::NotFound)?;

			Ok(w3n_entry.deposit)
		}

		fn deposit_amount(_key: &T::Web3Name) -> <Self::Currency as Currency<AccountIdOf<T>>>::Balance {
			T::Deposit::get()
		}

		fn store_deposit(
			key: &T::Web3Name,
			deposit: Deposit<AccountIdOf<T>, <Self::Currency as Currency<AccountIdOf<T>>>::Balance>,
		) -> Result<(), DispatchError> {
			let w3n_entry = Owner::<T>::get(key).ok_or(Error::<T>::NotFound)?;
			Owner::<T>::insert(key, Web3OwnershipOf::<T> { deposit, ..w3n_entry });

			Ok(())
		}
	}
}
