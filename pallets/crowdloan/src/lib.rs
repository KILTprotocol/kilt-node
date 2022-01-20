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

//! # Crowdloan contributions Pallet
//!
//! Provides means of registering the contributors to the KILT crowdloan.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Genesis config
//!
//! The crowdloan contributions pallet depends on the [`GenesisConfig`].
//!
//! The genesis config sets the initial registrar account that can update the
//! pallet's storage.
//!
//! ## Assumptions
//!
//! - At any time, there is one and only one registrar account which can manage
//!   the pallet storage.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod default_weights;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod storage;

pub use crate::{
	default_weights::WeightInfo,
	pallet::*,
	storage::{GratitudeConfig, ReserveAccounts},
};

#[frame_support::pallet]
pub mod pallet {
	use super::WeightInfo;
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement, StorageVersion, VestingSchedule, WithdrawReasons},
		PalletId,
	};
	use frame_system::{pallet_prelude::*, EnsureOneOf, EnsureSigned};
	use sp_runtime::{
		traits::{AccountIdConversion, BadOrigin, CheckedDiv, CheckedSub, Saturating, StaticLookup},
		Either,
	};
	use sp_std::vec;

	use crate::storage::{GratitudeConfig, ReserveAccounts};

	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	pub(crate) type CurrencyOf<T> = <T as Config>::Currency;
	pub(crate) type VestingOf<T> = <T as Config>::Vesting;
	pub(crate) type WeightInfoOf<T> = <T as Config>::WeightInfo;
	pub(crate) type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

	pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::config]
	pub trait Config: frame_system::Config
	where
		Self::Balance: From<BlockNumberOf<Self>>,
	{
		/// The crowdloan's pallet id, used for deriving its sovereign account
		/// ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		/// Currency type.
		type Currency: Currency<AccountIdOf<Self>, Balance = Self::Balance>;
		type Vesting: VestingSchedule<AccountIdOf<Self>, Currency = Self::Currency, Moment = BlockNumberOf<Self>>;
		type Balance: sp_runtime::traits::AtLeast32BitUnsigned + Parameter + Copy + Default + From<u64>;
		/// Overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The origin allowed to change the registrar account.
		type EnsureRegistrarOrigin: EnsureOrigin<Self::Origin>;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub registrar_account: AccountIdOf<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				registrar_account: Pallet::<T>::account_id(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			RegistrarAccount::<T>::set(self.registrar_account.clone());
		}
	}

	/// The registrar account allowed to manage the pallet storage.
	#[pallet::storage]
	#[pallet::getter(fn registrar_account)]
	pub type RegistrarAccount<T> = StorageValue<_, AccountIdOf<T>, ValueQuery>;

	/// The account from which the vested amount is transferred.
	#[pallet::storage]
	#[pallet::getter(fn reserve)]
	pub type Reserve<T> = StorageValue<_, ReserveAccounts<AccountIdOf<T>>, ValueQuery>;

	/// The account from which the free amount is transferred.
	#[pallet::storage]
	#[pallet::getter(fn configuration)]
	pub type Configuration<T> = StorageValue<_, GratitudeConfig<<T as frame_system::Config>::BlockNumber>, ValueQuery>;

	/// The set of contributions.
	///
	/// It maps from contributor's account to amount contributed.
	#[pallet::storage]
	#[pallet::getter(fn contributions)]
	pub type Contributions<T> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, BalanceOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new registrar has been set.
		/// \[old registrar account, new registrar account\]
		NewRegistrarAccountSet(AccountIdOf<T>, AccountIdOf<T>),

		/// New reserve accounts have been set.
		/// \[old vested reserve account, old free reserve account, new vested
		/// reserve account, new free reserve account\]
		NewReserveAccounts(AccountIdOf<T>, AccountIdOf<T>, AccountIdOf<T>, AccountIdOf<T>),

		/// A contribution has been set.
		/// \[contributor account, old amount (OPTIONAL), new amount\]
		ContributionSet(AccountIdOf<T>, Option<BalanceOf<T>>, BalanceOf<T>),

		/// A contribution has been removed.
		/// \[contributor account\]
		ContributionRemoved(AccountIdOf<T>),

		/// Our gratitude goes out to the \[account\].
		/// \[contributor account\]
		GratitudeReceived(AccountIdOf<T>),

		/// There was an error while sending the gratitude.
		/// \[contributor account\]
		GratitudeError(AccountIdOf<T>),

		/// A new configuration was set.
		/// \[new configuration\]
		UpdatedConfig(GratitudeConfig<BlockNumberOf<T>>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The contribution is not present.
		ContributorNotPresent,

		/// The reserve account run out of funds.
		InsufficientBalance,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Sets a new account as the registrar of this pallet.
		///
		/// The dispatch origin can be either Sudo or the current registrar
		/// account.
		///
		/// Emits `NewRegistrarAccountSet`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: [Origin Account], RegistrarAccount
		/// - Writes: RegistrarAccount
		#[pallet::weight(WeightInfoOf::<T>::set_registrar_account())]
		pub fn set_registrar_account(
			origin: OriginFor<T>,
			new_account: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let who =
				EnsureOneOf::<AccountIdOf<T>, T::EnsureRegistrarOrigin, EnsureSigned<AccountIdOf<T>>>::ensure_origin(
					origin,
				)?;

			let old_account = RegistrarAccount::<T>::get();

			if let Either::Right(signed_origin) = who {
				ensure!(signed_origin == old_account, BadOrigin);
			}

			let looked_up_account = <T as frame_system::Config>::Lookup::lookup(new_account)?;

			// *** No Fail beyond this point ***

			RegistrarAccount::<T>::set(looked_up_account.clone());

			Self::deposit_event(Event::NewRegistrarAccountSet(old_account, looked_up_account));

			Ok(())
		}

		/// Change the reserve accounts.
		///
		/// Only the registrar can change the reserve accounts.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: RegistrarAccount, Reserve
		/// - Writes: Reserve
		#[pallet::weight(WeightInfoOf::<T>::set_reserve_accounts())]
		pub fn set_reserve_accounts(
			origin: OriginFor<T>,
			new_vested_reserve: <T::Lookup as StaticLookup>::Source,
			new_free_reserve: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(who == RegistrarAccount::<T>::get(), BadOrigin);

			let new_vested_reserve_acc = <T as frame_system::Config>::Lookup::lookup(new_vested_reserve)?;
			let new_free_reserve_acc = <T as frame_system::Config>::Lookup::lookup(new_free_reserve)?;

			let ReserveAccounts {
				vested: old_vested,
				free: old_free,
			} = Reserve::<T>::get();

			Reserve::<T>::set(ReserveAccounts {
				vested: new_vested_reserve_acc.clone(),
				free: new_free_reserve_acc.clone(),
			});

			Self::deposit_event(Event::NewReserveAccounts(
				old_vested,
				old_free,
				new_vested_reserve_acc,
				new_free_reserve_acc,
			));

			Ok(())
		}

		/// Change the configuration of this pallet.
		///
		/// Only the registrar can change the configuration.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: RegistrarAccount
		/// - Writes: Configuration
		#[pallet::weight(WeightInfoOf::<T>::set_config())]
		pub fn set_config(origin: OriginFor<T>, new_config: GratitudeConfig<BlockNumberOf<T>>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(who == RegistrarAccount::<T>::get(), BadOrigin);

			Configuration::<T>::set(new_config.clone());

			Self::deposit_event(Event::UpdatedConfig(new_config));

			Ok(())
		}

		/// Sets a new contribution amount for a given contributor's account.
		///
		/// If a previous contribution is present, it is overridden.
		///
		/// The dispatch origin must be the current registrar account.
		///
		/// Emits `NewContributionSet`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: [Origin Account], RegistrarAccount, Contributions
		/// - Writes: Contributions
		#[pallet::weight(WeightInfoOf::<T>::set_contribution())]
		pub fn set_contribution(
			origin: OriginFor<T>,
			contributor: AccountIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(who == RegistrarAccount::<T>::get(), BadOrigin);

			// *** No Fail beyond this point ***

			let old_amount = Contributions::<T>::mutate(&contributor, |entry| entry.replace(amount));

			Self::deposit_event(Event::ContributionSet(contributor, old_amount, amount));

			Ok(())
		}

		/// Removes a contribution entry from the storage, if present.
		///
		/// It returns an error if there is no contribution for the given
		/// contributor's account.
		///
		/// The dispatch origin must be the current registrar account.
		///
		/// Emits `ContributionRemoved`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: [Origin Account], RegistrarAccount, Contributions
		/// - Writes: Contributions
		#[pallet::weight(WeightInfoOf::<T>::remove_contribution())]
		pub fn remove_contribution(origin: OriginFor<T>, contributor: AccountIdOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(who == RegistrarAccount::<T>::get(), BadOrigin);

			// *** No Fail except ContributorNotPresent beyond this point ***

			Contributions::<T>::take(&contributor).ok_or(Error::<T>::ContributorNotPresent)?;

			Self::deposit_event(Event::ContributionRemoved(contributor));

			Ok(())
		}

		/// Receive the gratitude.
		///
		/// Moves tokens to the given account according to the vote that was
		/// giving in favour of our parachain.
		///
		/// This is an unsigned extrinsic. The validity needs to be checked with
		/// `ValidateUnsigned`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: Contributions, Reserve, Configuration, [receiver account],
		///   [free reserve account], [vested reserve account]
		/// - Writes: Contributions, [free reserve account], [vested reserve
		///   account], [receiver account]
		#[pallet::weight(WeightInfoOf::<T>::receive_gratitude())]
		pub fn receive_gratitude(origin: OriginFor<T>, receiver: AccountIdOf<T>) -> DispatchResult {
			ensure_none(origin)?;

			let gratitude = Self::split_gratitude_for(&receiver)?;
			Self::ensure_can_send_gratitude(&receiver, gratitude)?;

			Contributions::<T>::remove(&receiver);

			let SplitGratitude { vested, free } = gratitude;
			let config = Configuration::<T>::get();
			let reserve = Reserve::<T>::get();

			// Transfer the free amount. Should not fail since checked we
			// ensure_can_withdraw.
			let result_free_transfer =
				CurrencyOf::<T>::transfer(&reserve.free, &receiver, free, ExistenceRequirement::AllowDeath);
			debug_assert!(
				result_free_transfer.is_ok(),
				"free transfer failed after we checked in ensure_can_withdraw"
			);

			// Transfer the vested amount and set the vesting schedule. Should not fail
			// since checked we ensure_can_withdraw.
			let result_vest_transfer =
				CurrencyOf::<T>::transfer(&reserve.vested, &receiver, vested, ExistenceRequirement::AllowDeath);
			debug_assert!(
				result_vest_transfer.is_ok(),
				"vested transfer failed after we checked in ensure_can_withdraw"
			);

			let per_block = vested
				.checked_div(&BalanceOf::<T>::from(config.vesting_length))
				.unwrap_or(vested);
			// vesting should not fail since we have transferred enough free balance.
			let result_vesting = VestingOf::<T>::add_vesting_schedule(&receiver, vested, per_block, config.start_block);
			debug_assert!(
				result_vesting.is_ok(),
				"vesting failed for vested coins after we transferred enough coins that ought to be vestable."
			);

			if result_vesting.is_ok() && result_vest_transfer.is_ok() && result_free_transfer.is_ok() {
				Self::deposit_event(Event::GratitudeReceived(receiver));
			} else {
				Self::deposit_event(Event::GratitudeError(receiver));
			}

			Ok(())
		}
	}

	#[derive(Clone, Copy, Debug)]
	struct SplitGratitude<Balance> {
		vested: Balance,
		free: Balance,
	}

	impl<T: Config> Pallet<T> {
		/// The account ID of the initial registrar account.
		///
		/// This actually does computation. If you need to keep using it, then
		/// make sure you cache the value and only call this once.
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account()
		}

		fn split_gratitude_for(receiver: &AccountIdOf<T>) -> Result<SplitGratitude<BalanceOf<T>>, DispatchError> {
			let amount = Contributions::<T>::get(receiver).ok_or(Error::<T>::ContributorNotPresent)?;
			let config = Configuration::<T>::get();

			// A two without any trait bounds (no From<u32>).
			let vested = config.vested_share.mul_floor(amount);
			let free = amount.saturating_sub(vested);

			Ok(SplitGratitude { vested, free })
		}

		fn ensure_can_send_gratitude(
			receiver: &AccountIdOf<T>,
			SplitGratitude { vested, free }: SplitGratitude<BalanceOf<T>>,
		) -> DispatchResult {
			let reserve = Reserve::<T>::get();
			let config = Configuration::<T>::get();

			if reserve.free == reserve.vested {
				let amount = free.saturating_add(vested);
				let new_acc_balance = CurrencyOf::<T>::free_balance(&reserve.free)
					.checked_sub(&amount)
					.ok_or(Error::<T>::InsufficientBalance)?;

				CurrencyOf::<T>::ensure_can_withdraw(
					&reserve.free,
					amount,
					WithdrawReasons::TRANSFER,
					new_acc_balance,
				)?;
			} else {
				let new_free_acc_balance = CurrencyOf::<T>::free_balance(&reserve.free)
					.checked_sub(&free)
					.ok_or(Error::<T>::InsufficientBalance)?;
				let new_vested_acc_balance = CurrencyOf::<T>::free_balance(&reserve.vested)
					.checked_sub(&vested)
					.ok_or(Error::<T>::InsufficientBalance)?;

				CurrencyOf::<T>::ensure_can_withdraw(
					&reserve.free,
					free,
					WithdrawReasons::TRANSFER,
					new_free_acc_balance,
				)?;
				CurrencyOf::<T>::ensure_can_withdraw(
					&reserve.vested,
					vested,
					WithdrawReasons::TRANSFER,
					new_vested_acc_balance,
				)?;
			}

			let per_block = vested
				.checked_div(&BalanceOf::<T>::from(config.vesting_length))
				.unwrap_or(vested);
			// vesting should not fail since we have transferred enough free balance.
			VestingOf::<T>::can_add_vesting_schedule(receiver, vested, per_block, config.start_block)?;

			Ok(())
		}
	}

	/// Custom validity errors while validating transactions.
	#[repr(u8)]
	pub enum ValidityError {
		/// The account is not registered and therefore not allowed to make this
		/// call.
		NoContributor = 0,
		/// An internal error prevents the call from being submitted.
		CannotSendGratitude = 1,
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			const PRIORITY: u64 = 100;

			let receiver = match call {
				// <weight>
				// The weight of this logic must be included in the `receive_gratitude` call.
				// </weight>
				Call::receive_gratitude { receiver: account } => account,
				_ => return Err(InvalidTransaction::Call.into()),
			};

			let gratitude = Self::split_gratitude_for(receiver)
				.map_err(|_| InvalidTransaction::Custom(ValidityError::NoContributor as u8))?;
			Self::ensure_can_send_gratitude(receiver, gratitude)
				.map_err(|_| InvalidTransaction::Custom(ValidityError::CannotSendGratitude as u8))?;

			Ok(ValidTransaction {
				priority: PRIORITY,
				requires: vec![],
				provides: vec![("gratitude", receiver).encode()],
				longevity: TransactionLongevity::max_value(),
				propagate: true,
			})
		}
	}
}
