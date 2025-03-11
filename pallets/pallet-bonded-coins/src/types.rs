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
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// Locks applied to a pool.
#[derive(Default, Clone, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen, Debug)]
pub struct Locks {
	pub allow_mint: bool,
	pub allow_burn: bool,
}

impl Locks {
	pub const fn any_lock_set(&self) -> bool {
		!(self.allow_mint && self.allow_burn)
	}
}

/// Status of a pool.
#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen, Debug)]
pub enum PoolStatus<LockType> {
	Active,
	Locked(LockType),
	Refunding,
	Destroying,
}

impl<LockType: Default> Default for PoolStatus<LockType> {
	fn default() -> Self {
		Self::Locked(LockType::default())
	}
}

impl<LockType> PoolStatus<LockType> {
	/// Checks if the pool is in a live state.
	pub const fn is_live(&self) -> bool {
		matches!(self, Self::Active | Self::Locked(_))
	}

	/// Checks if the pool is in a destroying state.
	pub const fn is_destroying(&self) -> bool {
		matches!(self, Self::Destroying)
	}

	/// Checks if the pool is in a refunding state.
	pub const fn is_refunding(&self) -> bool {
		matches!(self, Self::Refunding)
	}

	/// Freezes the pool with the given locks.
	pub fn freeze(&mut self, lock: LockType) {
		*self = Self::Locked(lock);
	}

	/// Starts the destruction process for the pool.
	pub fn start_destroy(&mut self) {
		*self = Self::Destroying;
	}

	/// Starts the refund process for the pool.
	pub fn start_refund(&mut self) {
		*self = Self::Refunding;
	}
}

/// Details of a pool.
#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen, Debug)]
pub struct PoolDetails<AccountId, ParametrizedCurve, Currencies, BaseCurrencyId, DepositBalance> {
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
	/// Whether the pool is transferable or not.
	pub transferable: bool,
	/// The denomination of the pool.
	pub denomination: u8,
	/// The minimum amount that can be minted/burnt.
	pub min_operation_balance: u128,
	/// The deposit to be returned upon destruction of this pool.
	pub deposit: DepositBalance,
	/// Whether asset management changes are allowed.
	pub enable_asset_management: bool,
}

impl<AccountId, ParametrizedCurve, Currencies, BaseCurrencyId, DepositBalance>
	PoolDetails<AccountId, ParametrizedCurve, Currencies, BaseCurrencyId, DepositBalance>
where
	AccountId: PartialEq + Clone,
{
	#[allow(clippy::too_many_arguments)]
	/// Creates a new pool with the given parameters.
	pub fn new(
		owner: AccountId,
		curve: ParametrizedCurve,
		collateral: BaseCurrencyId,
		bonded_currencies: Currencies,
		transferable: bool,
		enable_asset_management: bool,
		denomination: u8,
		min_operation_balance: u128,
		deposit: DepositBalance,
	) -> Self {
		Self {
			manager: Some(owner.clone()),
			owner,
			curve,
			collateral,
			bonded_currencies,
			transferable,
			enable_asset_management,
			state: PoolStatus::default(),
			denomination,
			min_operation_balance,
			deposit,
		}
	}

	/// Checks if the given account is the owner of the pool.
	pub fn is_owner(&self, who: &AccountId) -> bool {
		who == &self.owner
	}

	/// Checks if the given account is the manager of the pool.
	pub fn is_manager(&self, who: &AccountId) -> bool {
		Some(who) == self.manager.as_ref()
	}

	/// Checks if the given account can mint tokens in the pool, if the pool is
	/// locked.
	pub fn can_mint(&self, who: &AccountId) -> bool {
		match &self.state {
			PoolStatus::Locked(locks) => locks.allow_mint || self.is_manager(who),
			PoolStatus::Active => true,
			_ => false,
		}
	}

	/// Checks if the given account can burn tokens in the pool, if the pool is
	/// locked.
	pub fn can_burn(&self, who: &AccountId) -> bool {
		match &self.state {
			PoolStatus::Locked(locks) => locks.allow_burn || self.is_manager(who),
			PoolStatus::Active => true,
			_ => false,
		}
	}
}

/// Metadata of a bonded token.
#[derive(Debug, Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct TokenMeta<Balance, Symbol, Name> {
	/// The name of the token.
	pub name: Name,
	/// The symbol of the token.
	pub symbol: Symbol,
	/// min required balance
	pub min_balance: Balance,
}

/// Managing team of a pool.
#[derive(Debug, Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct PoolManagingTeam<AccountId> {
	/// The admin of the pool.
	pub admin: AccountId,
	/// The freezer of the pool.
	pub freezer: AccountId,
}

/// Enum, to specify the rounding direction.
#[derive(PartialEq, Clone, Copy, Eq)]
pub enum Round {
	/// Round up.
	Up,
	/// Round down.
	Down,
}
