// todo: send help!
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(Default, Clone, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen, Debug)]
pub struct Locks {
	pub allow_mint: bool,
	pub allow_burn: bool,
}

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
	pub fn is_live(&self) -> bool {
		matches!(self, Self::Active | Self::Locked(_))
	}

	pub fn is_destroying(&self) -> bool {
		matches!(self, Self::Destroying)
	}

	pub fn is_refunding(&self) -> bool {
		matches!(self, Self::Refunding)
	}

	pub fn freeze(&mut self, lock: LockType) {
		*self = Self::Locked(lock);
	}

	pub fn start_destroy(&mut self) {
		*self = Self::Destroying;
	}

	pub fn start_refund(&mut self) {
		*self = Self::Refunding;
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct PoolDetails<AccountId, ParametrizedCurve, Currencies, BaseCurrencyId> {
	pub owner: AccountId,
	pub manager: Option<AccountId>,
	pub curve: ParametrizedCurve,
	pub collateral_id: BaseCurrencyId,
	pub bonded_currencies: Currencies,
	pub state: PoolStatus<Locks>,
	pub transferable: bool,
	pub denomination: u8,
}

impl<AccountId, ParametrizedCurve, Currencies, BaseCurrencyId>
	PoolDetails<AccountId, ParametrizedCurve, Currencies, BaseCurrencyId>
where
	AccountId: PartialEq + Clone,
{
	pub fn new(
		owner: AccountId,
		curve: ParametrizedCurve,
		collateral_id: BaseCurrencyId,
		bonded_currencies: Currencies,
		transferable: bool,
		denomination: u8,
	) -> Self {
		Self {
			manager: Some(owner.clone()),
			owner,
			curve,
			collateral_id,
			bonded_currencies,
			transferable,
			state: PoolStatus::default(),
			denomination,
		}
	}

	pub fn is_owner(&self, who: &AccountId) -> bool {
		who == &self.owner
	}

	pub fn is_manager(&self, who: &AccountId) -> bool {
		Some(who) == self.manager.as_ref()
	}

	pub fn can_mint(&self, who: &AccountId) -> bool {
		match &self.state {
			PoolStatus::Locked(locks) => locks.allow_mint || self.is_manager(who),
			PoolStatus::Active => true,
			_ => false,
		}
	}

	pub fn can_burn(&self, who: &AccountId) -> bool {
		match &self.state {
			PoolStatus::Locked(locks) => locks.allow_burn || self.is_manager(who),
			PoolStatus::Active => true,
			_ => false,
		}
	}
}

#[derive(Debug, Encode, Decode, Clone, PartialEq, TypeInfo)]
pub struct TokenMeta<Balance, Symbol, Name> {
	pub name: Name,
	pub symbol: Symbol,
	pub min_balance: Balance,
}

#[derive(Debug, Encode, Decode, Clone, PartialEq, TypeInfo)]
pub struct PoolManagingTeam<AccountId> {
	pub admin: AccountId,
	pub freezer: AccountId,
}

/// Enum, to specify the rounding direction.
#[derive(PartialEq)]
pub(crate) enum Round {
	/// Round up.
	Up,
	/// Round down.
	Down,
}
