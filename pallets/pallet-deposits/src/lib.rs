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

//! # deposit pallet
//!
//! This pallet stores deposit pools.
//!
//! - [`Pallet`]

#![cfg_attr(not(feature = "std"), no_std)]

pub mod default_weights;
pub mod deposit_change_handler;

pub use deposit_change_handler::*;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		ensure,
		pallet_prelude::*,
		traits::{Currency, ReservableCurrency, StorageVersion},
	};
	use frame_system::{pallet_prelude::*};

	use sp_runtime::traits::{MaybeDisplay, Saturating};
	use sp_std::fmt::Debug;

	use crate::DepositChangeHandler;

	pub use crate::default_weights::WeightInfo;

	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

	pub(crate) type DepositIdOf<T> = <T as Config>::DepositId;

	pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The event type for this pallet
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The id type for deposits
		type DepositId: Parameter + Member + MaybeSerializeDeserialize + Debug + MaybeDisplay + Ord + MaxEncodedLen;

		/// The currency that is used to reserve funds for each did.
		type Currency: ReservableCurrency<AccountIdOf<Self>>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Handler for deposit changes.
		type ChangeHandler: crate::DepositChangeHandler<Self>;

		/// Minimal amount of blocks that a deposit take has to be announced in advance
		type MinimalNoticePeriod: Get<Self::BlockNumber>;
	}

	/// Announcment that at `block` the `amount` of the deposit will we
	/// withdrawable
	#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
	pub struct DepositTakeAnnouncement<BlockNumber, Balance> {
		pub block: BlockNumber,
		pub amount: Balance,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	/// Mapping from deposit id to total amount of deposits
	#[pallet::storage]
	#[pallet::getter(fn deposits_total)]
	pub type DepositsTotal<T> = StorageMap<_, Blake2_128Concat, DepositIdOf<T>, BalanceOf<T>>;

	/// Mapping from (deposit id + account id) -> balance.
	/// Holds the info who paid how much for the deposit
	#[pallet::storage]
	#[pallet::getter(fn deposits_by_account)]
	pub type DepositsByAccount<T> =
		StorageDoubleMap<_, Blake2_128Concat, DepositIdOf<T>, Blake2_128Concat, AccountIdOf<T>, BalanceOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn announcments)]
	pub type Announcments<T> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		DepositIdOf<T>,
		Blake2_128Concat,
		AccountIdOf<T>,
		DepositTakeAnnouncement<<T as frame_system::Config>::BlockNumber, BalanceOf<T>>,
	>;

	/// Event definitions
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A deposit was paid (deposit id + account id + new total amount)
		DepositPaid(DepositIdOf<T>, AccountIdOf<T>, BalanceOf<T>),
		/// A deposit is about to be withdrawn 
		DepositTakeAnnouncement(
			DepositIdOf<T>, 
			AccountIdOf<T>, 
			<T as frame_system::Config>::BlockNumber,
			BalanceOf<T>,
		),
		/// A deposit was taken (deposit id + account id + new total amount)
		DepositTaken(DepositIdOf<T>, AccountIdOf<T>, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		InsufficientFunds,
		NoticePeriodTooShort,
		NotAnnounced,
		StillInNoticePeriod,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Pay a deposit
		/// # Arguments
		/// * `origin`: The account that is paying the deposit
		/// * `deposit_id` - The id of the deposit
		/// * `amount` - The amount to pay
		#[pallet::weight(<T as Config>::WeightInfo::pay())]
		pub fn pay(origin: OriginFor<T>, deposit_id: DepositIdOf<T>, amount: BalanceOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// check if sender has enough funds
			ensure!(
				<T::Currency as ReservableCurrency<AccountIdOf<T>>>::can_reserve(&sender, amount,),
				Error::<T>::InsufficientFunds
			);

			// reserve the deposit
			<T::Currency as ReservableCurrency<AccountIdOf<T>>>::reserve(&sender, amount)?;

			// update the account balance for this deposit pot
			DepositsByAccount::<T>::mutate(&deposit_id, &sender, |balance| {
				*balance = Some(balance.unwrap_or_default().saturating_add(amount))
			});

			// update the total amount for this deposit pot
			let new_total = DepositsTotal::<T>::mutate(&deposit_id, |total| {
				*total = Some(total.unwrap_or_default().saturating_add(amount));
				total.unwrap() // safe because we make it Some() one line above
			});

			// call the change handler with the new total
			<T::ChangeHandler>::on_deposit_paid(&deposit_id, &sender, new_total);

			// emit event
			Self::deposit_event(Event::DepositPaid(deposit_id, sender, new_total));

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::announce_take())]
		pub fn announce_take(
			origin: OriginFor<T>,
			deposit_id: DepositIdOf<T>,
			amount: BalanceOf<T>,
			block: <T as frame_system::Config>::BlockNumber,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			
			// check minimal notice period
			let current_block = frame_system::Pallet::<T>::block_number();
			if block.saturating_sub(current_block) < T::MinimalNoticePeriod::get() {
				return Err(Error::<T>::NoticePeriodTooShort.into());
			}

			// create the announcement
			let announcment = DepositTakeAnnouncement {
				block,
				amount,
			};
			Announcments::<T>::insert(&deposit_id, &sender,  announcment);
			
			// emit event
			Self::deposit_event(Event::DepositTakeAnnouncement(deposit_id, sender, block, amount));
			
			Ok(())
		}
		/// Take a deposit
		/// # Arguments
		/// * `origin`: The account that is taking the deposit
		/// * `deposit_id` - The id of the deposit
		#[pallet::weight(<T as Config>::WeightInfo::take())]
		pub fn take(origin: OriginFor<T>, deposit_id: DepositIdOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// check if the take was announced before
			let announcment = Announcments::<T>::get(&deposit_id, &sender).ok_or(Error::<T>::NotAnnounced)?;
			let current_block = frame_system::Pallet::<T>::block_number();
			if current_block < announcment.block {
				return Err(Error::<T>::StillInNoticePeriod.into());
			}

			// check if sender has paid more or equal amount of deposit as requested to take
			let amount = announcment.amount;
			let sender_balance = DepositsByAccount::<T>::get(&deposit_id, &sender).unwrap_or_default();
			ensure!(sender_balance <= amount, Error::<T>::InsufficientFunds);

			// unreserve the deposit
			<T::Currency as ReservableCurrency<AccountIdOf<T>>>::unreserve(&sender, amount);

			// substract the amount from the balance for this account
			DepositsByAccount::<T>::mutate(&deposit_id, &sender, |balance| {
				*balance = Some(balance.unwrap_or_default().saturating_sub(amount))
			});

			// update the total amount for this deposit pot
			let new_total = DepositsTotal::<T>::mutate(&deposit_id, |total| {
				*total = Some(total.unwrap_or_default().saturating_sub(amount));
				total.unwrap()
			});

			// cleanup the announcement
			Announcments::<T>::remove(&deposit_id, &sender);

			// call the change handler with the new total
			<T::ChangeHandler>::on_deposit_taken(&deposit_id, &sender, new_total);

			// emit event
			Self::deposit_event(Event::DepositTaken(deposit_id, sender, new_total));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// retrieve the total amount of a deposit pot
		pub fn check_deposit(deposit_id: &DepositIdOf<T>) -> BalanceOf<T> {
			DepositsTotal::<T>::get(&deposit_id).unwrap_or_default()
		}

		/// ensure that the deposit is already paid and if not try to get it from the caller
		pub fn ensure_deposit(origin: OriginFor<T>, deposit_id: DepositIdOf<T>, min_amount: BalanceOf<T>) -> DispatchResult {
			let deposit = Self::check_deposit(&deposit_id);
			if deposit < min_amount {
				Self::pay(origin, deposit_id, min_amount - deposit)
			} else {
				Ok(())
			}
		}
	}
}
