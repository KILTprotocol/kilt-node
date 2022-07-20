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

use serde::{Deserialize, Serialize};

use public_credentials::{BlockNumberOf, AccountIdOf, BalanceOf, CredentialEntryOf};
use kilt_support::deposit::Deposit;

#[derive(Serialize, Deserialize)]
#[serde(bound(
	serialize = "
	BlockNumber: Serialize,
	Balance: std::fmt::Display,
	AccountId: Serialize",
	deserialize = "
	BlockNumber: Deserialize<'de>,
	Balance: std::str::FromStr,
	AccountId: Deserialize<'de>",
))]
pub struct OuterCredentialEntry<BlockNumber, AccountId, Balance, T> {
	pub block_number: BlockNumber,
	pub deposit: Deposit<AccountId, Balance>,
	_phantom: Option<std::marker::PhantomData<T>>,
}

impl<T: public_credentials::Config> From<CredentialEntryOf<T>> for OuterCredentialEntry<BlockNumberOf<T>, AccountIdOf<T>, BalanceOf<T>, T> {
	fn from(value: CredentialEntryOf<T>) -> Self {
		Self {
			block_number: value.block_number,
			deposit: value.deposit,
			_phantom: None,
		}
	}
}
