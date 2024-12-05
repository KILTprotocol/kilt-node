// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

use pallet_bonded_coins::{curves::Curve, PoolDetails};
use parity_scale_codec::{alloc::string::String, Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;
use xcm::v4::AssetId as AssetIdLocation;

use crate::{bonded_currencies::AssetId, AccountId, Balance};

/// Enum to represent the operation of minting or burning tokens.
#[derive(Decode, Encode, TypeInfo)]
pub enum Operation {
	Mint,
	Burn,
}

/// Human readable curve type.
pub type HumanReadableCurve = Curve<String>;

/// Human readable bonded currencies
pub type HumanReadableCurrencies = Vec<BondedCurrencyDetails>;

/// Human readable pool details.
pub type HumanReadablePoolDetails =
	PoolDetails<AccountId, HumanReadableCurve, HumanReadableCurrencies, CollateralDetails, Balance>;

/// Collateral currency details used for the runtime API.
#[derive(Decode, Encode, TypeInfo)]
pub struct CollateralDetails {
	pub id: AssetIdLocation,
	pub name: String,
	pub symbol: String,
	pub denomination: u8,
}

/// Bonded currency details used for the runtime API.
#[derive(Decode, Encode, TypeInfo)]
pub struct BondedCurrencyDetails {
	pub id: AssetId,
	pub name: String,
	pub symbol: String,
	pub supply: Balance,
}

#[derive(Debug, Encode, Decode, TypeInfo, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
	PoolNotFound,
	CurrencyNotFound,
	BalanceConversion,
	CalculationError,
	Internal,
	InvalidInput,
}
