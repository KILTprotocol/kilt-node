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

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::ReservableCurrency;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{traits::Zero, DispatchError};

/// An amount of balance reserved by the specified address.
#[derive(Clone, Debug, Encode, Decode, Eq, PartialEq, Ord, PartialOrd, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Deposit<Account, Balance> {
	pub owner: Account,
	pub amount: Balance,
}

pub fn reserve_deposit<Account, Currency: ReservableCurrency<Account>>(
	account: Account,
	deposit_amount: Currency::Balance,
) -> Result<Deposit<Account, Currency::Balance>, DispatchError> {
	Currency::reserve(&account, deposit_amount)?;
	Ok(Deposit {
		owner: account,
		amount: deposit_amount,
	})
}

pub fn free_deposit<Account, Currency: ReservableCurrency<Account>>(deposit: &Deposit<Account, Currency::Balance>) {
	let err_amount = Currency::unreserve(&deposit.owner, deposit.amount);
	debug_assert!(err_amount.is_zero());
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_serialize_and_deserialize_properly_with_string() {
		let deposit = Deposit {
			owner: 0_u8,
			amount: 1_000_000_u128,
		};

		let json_str = r#"{"owner":0,"amount":"1000000"}"#;

		assert_eq!(serde_json::to_string(&deposit).unwrap(), json_str);
		assert_eq!(serde_json::from_str::<Deposit<u8, u128>>(json_str).unwrap(), deposit);

		// should not panic
		serde_json::to_value(&deposit).unwrap();
	}

	#[test]
	fn should_serialize_and_deserialize_properly_large_value() {
		let deposit = Deposit {
			owner: 0_u8,
			amount: 1_000_000_u128,
		};

		let json_str = r#"{"owner":0,"amount":"1000000"}"#;

		assert_eq!(serde_json::to_string(&deposit).unwrap(), json_str);
		assert_eq!(serde_json::from_str::<Deposit<u8, u128>>(json_str).unwrap(), deposit);

		// should not panic
		serde_json::to_value(&deposit).unwrap();
	}
}
