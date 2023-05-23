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
		fungible::hold::{Inspect, Mutate},
		ReservableCurrency,
	},
};
use frame_system::Config;
use sp_runtime::SaturatedConversion;

use crate::deposit;

pub fn ensure_upgraded<T: Config, Currency: ReservableCurrency<T::AccountId> + Mutate<T::AccountId>>(
	account: &T::AccountId,
	reason: &<Currency as Inspect<T::AccountId>>::Reason,
) {
}

/// Mutate the balance of the given account.
/// Moves all reserved balance to holds. This is a migration function and should
/// be deleted once all accounts are updated.
pub fn switch_reserved_to_holds<T: Config, Currency: ReservableCurrency<T::AccountId> + Mutate<T::AccountId>>(
	deposit: &T::AccountId,
	deposit: <Currency as ReservableCurrency<T::AccountId>>::Balance,
	reason: &<Currency as Inspect<T::AccountId>>::Reason,
) -> DispatchResult {
	let reserved_balance = Currency::reserved_balance(account);
	Currency::unreserve(account, reserved_balance);
	let hold_balance = reserved_balance.saturated_into::<u128>();
	Currency::hold(reason, account, hold_balance.saturated_into())
}

fn switch_locked_to_freezes<T: Config, Currency: ReservableCurrency<T::AccountId> + Mutate<T::AccountId>>(
	account: &T::AccountId,
) -> DispatchResult {
	Ok(())
}
