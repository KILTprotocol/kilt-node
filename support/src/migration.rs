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
	traits::{fungible::hold::Mutate as MutateHold, ReservableCurrency},
};
use sp_runtime::{traits::Zero, SaturatedConversion};

/// Checks some precondition of the migrations.
pub fn has_user_reserved_balance<AccountId, Currency: ReservableCurrency<AccountId> + MutateHold<AccountId>>(
	owner: &AccountId,
	reason: &Currency::Reason,
) -> bool {
	Currency::balance_on_hold(reason, owner).is_zero() && Currency::reserved_balance(owner) > Zero::zero()
}

pub fn switch_reserved_to_hold<AccountId, Currency: ReservableCurrency<AccountId> + MutateHold<AccountId>>(
	owner: AccountId,
	reason: &Currency::Reason,
	amount: u128,
) -> DispatchResult {
	let remaining_balance = Currency::unreserve(&owner, amount.saturated_into());
	debug_assert!(
		remaining_balance.is_zero(),
		"Could not unreserve balance. Remaining: {:?}. To unreserve: {:?}",
		remaining_balance,
		amount
	);
	let to_hold_balance = amount.saturating_sub(remaining_balance.saturated_into());
	Currency::hold(reason, &owner, to_hold_balance.saturated_into())
}
