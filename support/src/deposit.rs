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

use frame_support::{pallet_prelude::DispatchResult, traits::fungible::hold::Mutate};

use frame_support::traits::tokens::Precision;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::DispatchError;

#[derive(Clone, Debug, Encode, Decode, Eq, PartialEq, Ord, PartialOrd, TypeInfo, MaxEncodedLen, Copy)]
pub enum HFIdentifier {
	Deposit(Pallets),
	Staking,
	Misc,
}

#[derive(Clone, Debug, Encode, Decode, Eq, PartialEq, Ord, PartialOrd, TypeInfo, MaxEncodedLen, Copy)]
pub enum Pallets {
	Attestation,
	Ctype,
	Delegation,
	Did,
	DidLookup,
	W3n,
	Staking,
	PublicCredentials,
}

/// An amount of balance reserved by the specified address.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Encode, Decode, Eq, PartialEq, Ord, PartialOrd, TypeInfo, MaxEncodedLen)]
pub struct Deposit<Account, Balance> {
	pub owner: Account,
	pub amount: Balance,
}

pub fn reserve_deposit<Account, Currency: Mutate<Account, Reason = HFIdentifier>>(
	account: Account,
	deposit_amount: Currency::Balance,
	reason: &HFIdentifier,
) -> Result<Deposit<Account, Currency::Balance>, DispatchError> {
	let q = Currency::hold(reason, &account, deposit_amount);
	q?;
	Ok(Deposit {
		owner: account,
		amount: deposit_amount,
	})
}

pub fn free_deposit<Account, Currency: Mutate<Account, Reason = HFIdentifier>>(
	deposit: &Deposit<Account, Currency::Balance>,
	reason: &HFIdentifier,
) -> DispatchResult {
	Currency::release(reason, &deposit.owner, deposit.amount, Precision::Exact)?;
	Ok(())
}
