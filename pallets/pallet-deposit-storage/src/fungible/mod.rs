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
	tokens::{Fortitude, Preservation, Provenance, WithdrawConsequence},
};
use kilt_support::Deposit;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, DispatchError, DispatchResult};

use crate::{deposit::DepositEntry, Config, DepositKeyOf, Deposits, Pallet};

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
		<T::Currency as Inspect<T::AccountId>>::minimum_balance()
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

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Copy, Debug)]
pub struct PalletDepositStorageReason<Namespace, Key> {
	pub(crate) namespace: Namespace,
	pub(crate) key: Key,
}

// This trait is implemented by forwarding everything to the `Currency`
// implementation.
impl<T> InspectHold<T::AccountId> for Pallet<T>
where
	T: Config,
{
	type Reason = PalletDepositStorageReason<T::Namespace, DepositKeyOf<T>>;

	fn total_balance_on_hold(who: &T::AccountId) -> Self::Balance {
		<T::Currency as InspectHold<T::AccountId>>::total_balance_on_hold(who)
	}

	fn balance_on_hold(reason: &Self::Reason, who: &T::AccountId) -> Self::Balance {
		<T::Currency as InspectHold<T::AccountId>>::balance_on_hold(&reason.clone().into(), who)
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
	// Implements this trait function by first dispatching to the underlying
	// `Currency` and then overriding the relevant storage entry.
	fn set_balance_on_hold(reason: &Self::Reason, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<T::Currency as UnbalancedHold<T::AccountId>>::set_balance_on_hold(&reason.clone().into(), who, amount)?;
		Deposits::<T>::mutate_exists(&reason.namespace, &reason.key, |maybe_existing_deposit_entry| {
			match maybe_existing_deposit_entry {
				// If this is the first registered deposit, create a new storage entry.
				None => {
					if amount > Zero::zero() {
						*maybe_existing_deposit_entry = Some(DepositEntry {
							deposit: Deposit {
								amount,
								owner: who.clone(),
							},
							reason: reason.clone().into(),
							reclaimable: false,
						})
					}
				}
				// If this is a previous deposit, update the storage entry or remove completely if balance on hold is
				// set to `0`.
				Some(existing_deposit_entry) => {
					if amount.is_zero() {
						*maybe_existing_deposit_entry = None;
					} else {
						existing_deposit_entry.deposit.amount = amount;
					}
				}
			}
		});
		Ok(())
	}
}

impl<T> MutateHold<T::AccountId> for Pallet<T> where T: Config {}
