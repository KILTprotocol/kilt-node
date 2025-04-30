// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

use pallet_bonded_coins::{curves::Curve, BondedCurrenciesSettings, Locks, PoolStatus};
use parity_scale_codec::{alloc::string::String, Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

/// Curve representation
pub type CurveOf = Curve<String>;

/// Currencies representation
pub type CurrenciesOf<AssetId, Balance> = Vec<BondedCurrencyDetails<AssetId, Balance>>;

/// Human readable pool details.
pub type PoolDetailsOf<AccountId, Balance, AssetId, CollateralAssetId> = PoolDetails<
	AccountId,
	AccountId,
	CurveOf,
	CurrenciesOf<AssetId, Balance>,
	CollateralDetails<CollateralAssetId>,
	Balance,
	Balance,
>;

#[derive(Default, Clone, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen, Debug)]
pub struct PoolDetails<
	AccountId,
	PoolId,
	ParametrizedCurve,
	Currencies,
	BaseCurrencyId,
	DepositBalance,
	FungiblesBalance,
> {
	/// The ID of the pool.
	pub id: PoolId,
	/// The owner of the pool.
	pub owner: AccountId,
	/// The manager of the pool. If a manager is set, the pool is permissioned.
	pub manager: Option<AccountId>,
	/// The curve of the pool.
	pub curve: ParametrizedCurve,
	/// The collateral currency of the pool.
	pub collateral: BaseCurrencyId,
	/// The bonded currencies of the pool.
	pub bonded_currencies: Currencies,
	/// The status of the pool.
	pub state: PoolStatus<Locks>,
	/// Shared settings of the currencies in the pool.
	pub currencies_settings: BondedCurrenciesSettings<FungiblesBalance>,
	/// The deposit to be returned upon destruction of this pool.
	pub deposit: DepositBalance,
}
/// Collateral currency details used for the runtime API.
#[derive(Decode, Encode, TypeInfo)]
pub struct CollateralDetails<AssetId> {
	pub id: AssetId,
	pub name: String,
	pub symbol: String,
	pub denomination: u8,
}

/// Bonded currency details used for the runtime API.
#[derive(Decode, Encode, TypeInfo)]
pub struct BondedCurrencyDetails<AssetId, Balance> {
	pub id: AssetId,
	pub name: String,
	pub symbol: String,
	pub supply: Balance,
}
