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

use frame_support::{
	pallet_prelude::DispatchResult,
	traits::{
		fungible::{hold::Mutate as MutateHold, Inspect},
		ReservableCurrency,
	},
};
use sp_runtime::{traits::Zero, Saturating};

#[cfg(any(feature = "mock", feature = "runtime-benchmarks"))]
use pallet_balances::{Config, Holds, Pallet};

pub fn switch_reserved_to_hold<AccountId, Currency>(
	owner: &AccountId,
	reason: &Currency::Reason,
	amount: <Currency as Inspect<AccountId>>::Balance,
) -> DispatchResult
where
	Currency: ReservableCurrency<AccountId>
		+ MutateHold<AccountId, Balance = <Currency as frame_support::traits::Currency<AccountId>>::Balance>,
{
	let remaining_balance = Currency::unreserve(owner, amount);
	debug_assert!(
		remaining_balance.is_zero(),
		"Could not unreserve balance. Remaining: {:?}. To unreserve: {:?}",
		remaining_balance,
		amount
	);
	let to_hold_balance = amount.saturating_sub(remaining_balance);
	Currency::hold(reason, owner, to_hold_balance)
}

#[cfg(any(feature = "mock", feature = "runtime-benchmarks"))]
pub fn translate_holds_to_reserve<T: Config>(hold_id: T::RuntimeHoldReason)
where
	T: Config,
{
	use frame_support::traits::tokens::Precision;

	Holds::<T>::iter().for_each(|(user, holds)| {
		holds.iter().filter(|hold| hold.id == hold_id).for_each(|hold| {
			Pallet::<T>::release(&hold_id, &user, hold.amount, Precision::Exact)
				.expect("Translation to reserves should not fail");

			Pallet::<T>::reserve(&user, hold.amount).expect("Reserving Balance should not fail.");
		})
	});
}
