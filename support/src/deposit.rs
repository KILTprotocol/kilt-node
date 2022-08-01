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
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// An amount of balance reserved by the specified address.
#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Deposit<Account, Balance> {
	pub owner: Account,
	#[cfg_attr(
		feature = "std",
		serde(bound(serialize = "Balance: std::fmt::Display", deserialize = "Balance: std::str::FromStr"))
	)]
	#[cfg_attr(feature = "std", serde(with = "serde_balance"))]
	pub amount: Balance,
}

// This code was copied from https://github.com/paritytech/substrate/blob/ded44948e2d5a398abcb4e342b0513cb690961bb/frame/transaction-payment/src/types.rs#L113
// It is needed because u128 cannot be serialized by serde_json out of the box.
#[cfg(feature = "std")]
mod serde_balance {
	use serde::{Deserialize, Deserializer, Serializer};

	pub fn serialize<S: Serializer, T: std::fmt::Display>(t: &T, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&t.to_string())
	}

	pub fn deserialize<'de, D: Deserializer<'de>, T: std::str::FromStr>(deserializer: D) -> Result<T, D::Error> {
		let s = String::deserialize(deserializer)?;
		s.parse::<T>()
			.map_err(|_| serde::de::Error::custom("Parse from string failed"))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_serialize_and_deserialize_properly_with_string() {
		let deposit = Deposit {
			owner: 0_u8,
			amount: 0_u128,
		};

		let json_str = r#"{"owner":0,"amount":"0"}"#;

		assert_eq!(serde_json::to_string(&deposit).unwrap(), json_str);
		assert_eq!(serde_json::from_str::<Deposit<u8, u128>>(json_str).unwrap(), deposit);

		// should not panic
		serde_json::to_value(&deposit).unwrap();
	}

	#[test]
	fn should_serialize_and_deserialize_properly_large_value() {
		let deposit = Deposit {
			owner: 0_u8,
			amount: u128::MAX,
		};

		let json_str = r#"{"owner":0,"amount":"340282366920938463463374607431768211455"}"#;

		assert_eq!(serde_json::to_string(&deposit).unwrap(), json_str);
		assert_eq!(serde_json::from_str::<Deposit<u8, u128>>(json_str).unwrap(), deposit);

		// should not panic
		serde_json::to_value(&deposit).unwrap();
	}
}
