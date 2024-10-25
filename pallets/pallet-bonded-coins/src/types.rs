// todo: send help!
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(Default, Clone, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen, Debug)]
pub struct Locks {
	pub allow_mint: bool,
	pub allow_burn: bool,
	pub allow_swap: bool,
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen, Debug, Default)]
pub enum PoolStatus<LockType> {
	#[default]
	Active,
	Locked(LockType),
	Destroying,
}

impl<LockType> PoolStatus<LockType> {
	pub fn is_active(&self) -> bool {
		matches!(self, Self::Active)
	}

	pub fn is_destroying(&self) -> bool {
		matches!(self, Self::Destroying)
	}

	pub fn freeze(&mut self, lock: LockType) {
		*self = Self::Locked(lock);
	}

	pub fn destroy(&mut self) {
		*self = Self::Destroying;
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct PoolDetails<AccountId, ParametrizedCurve, Currencies> {
	pub manager: AccountId,
	pub curve: ParametrizedCurve,
	pub bonded_currencies: Currencies,
	pub state: PoolStatus<Locks>,
	pub transferable: bool,
}

impl<AccountId, ParametrizedCurve, Currencies> PoolDetails<AccountId, ParametrizedCurve, Currencies>
where
	AccountId: PartialEq,
{
	pub fn new(
		manager: AccountId,
		curve: ParametrizedCurve,
		bonded_currencies: Currencies,
		transferable: bool,
	) -> Self {
		Self {
			manager,
			curve,
			bonded_currencies,
			transferable,
			state: PoolStatus::default(),
		}
	}

	pub fn is_manager(&self, who: &AccountId) -> bool {
		who == &self.manager
	}

	pub fn is_minting_authorized(&self, who: &AccountId) -> bool {
		match &self.state {
			PoolStatus::Locked(locks) => locks.allow_mint || self.is_manager(&who),
			PoolStatus::Active => true,
			_ => false,
		}
	}

	pub fn is_swapping_authorized(&self, who: &AccountId) -> bool {
		match &self.state {
			PoolStatus::Locked(locks) => locks.allow_swap || self.is_manager(&who),
			PoolStatus::Active => true,
			_ => false,
		}
	}

	pub fn is_burning_authorized(&self, who: &AccountId) -> bool {
		match &self.state {
			PoolStatus::Locked(locks) => locks.allow_burn || self.is_manager(&who),
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
	pub issuer: AccountId,
	pub freezer: AccountId,
}
