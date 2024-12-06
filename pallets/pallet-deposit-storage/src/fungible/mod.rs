// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

use frame_support::traits::{
	fungible::{Dust, Inspect, InspectHold, MutateHold, Unbalanced, UnbalancedHold},
	tokens::{Fortitude, Precision, Preservation, Provenance, WithdrawConsequence},
};
use kilt_support::Deposit;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{CheckedSub, Zero},
	DispatchError, DispatchResult,
};

use crate::{deposit::DepositEntry, Config, DepositKeyOf, Error, HoldReason, Pallet, SystemDeposits};

const LOG_TARGET: &str = "runtime::pallet_deposit_storage::mutate-hold";

#[cfg(test)]
mod tests;

// This trait is implemented by forwarding everything to the `Currency`
// implementation.
impl<T> Inspect<T::AccountId> for Pallet<T>
where
	T: Config,
{
	type Balance = <T::Currency as Inspect<T::AccountId>>::Balance;

	fn total_issuance() -> Self::Balance {
		<T::Currency as Inspect<T::AccountId>>::total_issuance()
	}

	fn minimum_balance() -> Self::Balance {
		<T::Currency as Inspect<T::AccountId>>::minimum_balance()
	}

	fn total_balance(who: &T::AccountId) -> Self::Balance {
		<T::Currency as Inspect<T::AccountId>>::total_balance(who)
	}

	fn balance(who: &T::AccountId) -> Self::Balance {
		<T::Currency as Inspect<T::AccountId>>::balance(who)
	}

	fn reducible_balance(who: &T::AccountId, preservation: Preservation, force: Fortitude) -> Self::Balance {
		<T::Currency as Inspect<T::AccountId>>::reducible_balance(who, preservation, force)
	}

	fn can_deposit(
		who: &T::AccountId,
		amount: Self::Balance,
		provenance: Provenance,
	) -> frame_support::traits::tokens::DepositConsequence {
		<T::Currency as Inspect<T::AccountId>>::can_deposit(who, amount, provenance)
	}

	fn can_withdraw(who: &T::AccountId, amount: Self::Balance) -> WithdrawConsequence<Self::Balance> {
		<T::Currency as Inspect<T::AccountId>>::can_withdraw(who, amount)
	}
}

/// The `HoldReason` consumers must use in order to rely on this pallet's
/// implementation of `MutateHold`.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Copy, Debug, Default)]
pub struct PalletDepositStorageReason<Namespace, Key> {
	pub(crate) namespace: Namespace,
	pub(crate) key: Key,
}

impl<Namespace, Key> From<PalletDepositStorageReason<Namespace, Key>> for HoldReason {
	fn from(_value: PalletDepositStorageReason<Namespace, Key>) -> Self {
		// All the deposits ever taken like this will count towards the same hold
		// reason.
		Self::Deposit
	}
}

impl<T> InspectHold<T::AccountId> for Pallet<T>
where
	T: Config,
{
	type Reason = PalletDepositStorageReason<T::Namespace, DepositKeyOf<T>>;

	fn total_balance_on_hold(who: &T::AccountId) -> Self::Balance {
		<T::Currency as InspectHold<T::AccountId>>::total_balance_on_hold(who)
	}

	// We return the balance of hold of a specific (namespace, key), which prevents
	// mistakes since everything in the end is converted to `HoldReason`.
	fn balance_on_hold(reason: &Self::Reason, who: &T::AccountId) -> Self::Balance {
		let Some(deposit_entry) = SystemDeposits::<T>::get(&reason.namespace, &reason.key) else {
			return Zero::zero();
		};
		if deposit_entry.deposit.owner == *who {
			deposit_entry.deposit.amount
		} else {
			Zero::zero()
		}
	}
}

// This trait is implemented by forwarding everything to the `Currency`
// implementation.
impl<T> Unbalanced<T::AccountId> for Pallet<T>
where
	T: Config,
{
	fn handle_dust(dust: Dust<T::AccountId, Self>) {
		<T::Currency as Unbalanced<T::AccountId>>::handle_dust(Dust(dust.0));
	}

	fn write_balance(who: &T::AccountId, amount: Self::Balance) -> Result<Option<Self::Balance>, DispatchError> {
		<T::Currency as Unbalanced<T::AccountId>>::write_balance(who, amount)
	}

	fn set_total_issuance(amount: Self::Balance) {
		<T::Currency as Unbalanced<T::AccountId>>::set_total_issuance(amount);
	}
}

impl<T> UnbalancedHold<T::AccountId> for Pallet<T>
where
	T: Config,
{
	/// This function gets called inside all invocations of `hold`, `release`,
	/// and `release_all`. The value of `amount` is always the final amount that
	/// should be kept on hold, meaning that a `release_all` will result in a
	/// value of `0`. An hold increase from `1` to `2` will imply a value of
	/// `amount` of `2`, i.e., the total amount held at the end of the
	/// evaluation. Similarly, a decrease from `3` to `1` will imply a `amount`
	/// value of `1`.
	fn set_balance_on_hold(reason: &Self::Reason, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		SystemDeposits::<T>::try_mutate_exists(&reason.namespace, &reason.key, |maybe_existing_deposit_entry| {
			match maybe_existing_deposit_entry {
				// If this is the first registered deposit for this reason, create a new storage entry.
				None => {
					if amount > Zero::zero() {
						// Increase the held amount for the final runtime hold reason by `amount`.
						<T::Currency as UnbalancedHold<T::AccountId>>::increase_balance_on_hold(
							&T::RuntimeHoldReason::from(reason.clone().into()),
							who,
							amount,
							Precision::Exact,
						)?;
						*maybe_existing_deposit_entry = Some(DepositEntry {
							deposit: Deposit {
								amount,
								owner: who.clone(),
							},
							reason: T::RuntimeHoldReason::from(reason.clone().into()),
						})
					}
					Ok(())
				}
				Some(existing_deposit_entry) => {
					// The pallet assumes each (namespace, key) points to a unique deposit, so the
					// same combination cannot be used for a different account.
					if existing_deposit_entry.deposit.owner != *who {
						return Err(DispatchError::from(Error::<T>::DepositExisting));
					}
					if amount.is_zero() {
						// If trying to remove all holds for this reason (`amount` == 0), decrease the
						// held amount for the final runtime hold reason by the amount that was held by
						// this reason.
						<T::Currency as UnbalancedHold<T::AccountId>>::decrease_balance_on_hold(
							&T::RuntimeHoldReason::from(reason.clone().into()),
							who,
							existing_deposit_entry.deposit.amount,
							Precision::Exact,
						)?;
						// Since we're setting the held amount to `0`, remove the storage entry.
						*maybe_existing_deposit_entry = None;
					// We're trying to update (i.e., not creating, not deleting)
					// the held amount for this hold reason.
					} else {
						// If trying to increase the amount held, increase the held amount for the final
						// runtime hold reason by the (amount - stored amount) difference.
						if amount >= existing_deposit_entry.deposit.amount {
							let amount_to_hold = amount
								.checked_sub(&existing_deposit_entry.deposit.amount)
								.ok_or_else(|| {
									log::error!(target: LOG_TARGET, "Failed to evaluate {:?} - {:?}", amount, existing_deposit_entry.deposit.amount);
									Error::<T>::Internal
								})?;
							<T::Currency as UnbalancedHold<T::AccountId>>::increase_balance_on_hold(
								&T::RuntimeHoldReason::from(reason.clone().into()),
								who,
								amount_to_hold,
								Precision::Exact,
							)?;
						// Else, decrease the held amount for the final runtime
						// hold reason by the (stored amount - amount)
						// difference.
						} else {
							let amount_to_release = existing_deposit_entry
								.deposit
								.amount
								.checked_sub(&amount)
								.ok_or_else(|| {
									log::error!(target: LOG_TARGET, "Failed to evaluate {:?} - {:?}", existing_deposit_entry.deposit.amount, amount);
									Error::<T>::Internal
								})?;
							<T::Currency as UnbalancedHold<T::AccountId>>::decrease_balance_on_hold(
								&T::RuntimeHoldReason::from(reason.clone().into()),
								who,
								amount_to_release,
								Precision::Exact,
							)?;
						}
						// In either case, update the storage entry with the specified amount.
						existing_deposit_entry.deposit.amount = amount;
					}
					Ok(())
				}
			}
		})?;
		Ok(())
	}
}

impl<T> MutateHold<T::AccountId> for Pallet<T> where T: Config {}
