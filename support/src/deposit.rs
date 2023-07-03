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

use frame_support::traits::{
	fungible::{hold::Mutate, Inspect},
	tokens::Precision,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::DispatchError;

/// An amount of balance reserved by the specified address.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Encode, Decode, Eq, PartialEq, Ord, PartialOrd, TypeInfo, MaxEncodedLen)]
pub struct Deposit<Account, Balance> {
	pub owner: Account,
	pub amount: Balance,
}

pub(crate) fn reserve_deposit<Account, Currency: Mutate<Account>>(
	account: Account,
	deposit_amount: Currency::Balance,
	reason: &Currency::Reason,
) -> Result<Deposit<Account, Currency::Balance>, DispatchError> {
	Currency::hold(reason, &account, deposit_amount)?;
	Ok(Deposit {
		owner: account,
		amount: deposit_amount,
	})
}

pub(crate) fn free_deposit<Account, Currency: Mutate<Account>>(
	deposit: &Deposit<Account, Currency::Balance>,
	reason: &Currency::Reason,
) -> Result<<Currency as Inspect<Account>>::Balance, DispatchError> {
	let result = Currency::release(reason, &deposit.owner, deposit.amount, Precision::BestEffort);
	debug_assert!(
		result == Ok(deposit.amount),
		"Released deposit amount does not match with expected amount. Expected: {:?}, Released amount: {:?}  Error: {:?}",
		deposit.amount,
		result.ok(),
		result.err(),
	);
	result
}
