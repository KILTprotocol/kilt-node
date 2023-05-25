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

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::AccountId32;

use crate::account::AccountId20;

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum LinkableAccountId {
	AccountId20(AccountId20),
	AccountId32(AccountId32),
}

impl From<AccountId20> for LinkableAccountId {
	fn from(account_id: AccountId20) -> Self {
		Self::AccountId20(account_id)
	}
}

impl From<AccountId32> for LinkableAccountId {
	fn from(account_id: AccountId32) -> Self {
		Self::AccountId32(account_id)
	}
}

#[cfg(feature = "std")]
impl std::fmt::Display for LinkableAccountId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::AccountId20(account_id) => write!(f, "{}", account_id),
			Self::AccountId32(account_id) => write!(f, "{}", account_id),
		}
	}
}

// Default implementation required by the DipDidOrigin origin type, only for
// benchmarks.
#[cfg(feature = "runtime-benchmarks")]
impl Default for LinkableAccountId {
	fn default() -> Self {
		AccountId32::new([0u8; 32]).into()
	}
}
