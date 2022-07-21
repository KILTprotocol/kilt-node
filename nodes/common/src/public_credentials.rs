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

use kilt_support::deposit::Deposit;
use public_credentials::CredentialEntry;

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
pub struct OuterCredentialEntry<BlockNumber, AccountId, Balance> {
	pub block_number: BlockNumber,
	pub deposit: Deposit<AccountId, Balance>,
}

impl<BlockNumber, AccountId, Balance> From<CredentialEntry<BlockNumber, AccountId, Balance>>
	for OuterCredentialEntry<BlockNumber, AccountId, Balance>
{
	fn from(value: CredentialEntry<BlockNumber, AccountId, Balance>) -> Self {
		Self {
			block_number: value.block_number,
			deposit: value.deposit,
		}
	}
}
