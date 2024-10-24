// todo: send help!
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(Default, Clone, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen, Debug)]
pub struct Locks {
	pub allow_mint: bool,
	pub allow_burn: bool,
	pub allow_swap: bool,
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen, Debug)]
pub enum PoolStatus<LockType> {
	Active,
	Locked(LockType),
	Refunding,
	Destroying,
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

	pub fn destroy(&mut self) {
		*self = Self::Destroying;
	}

	pub fn refunding(&mut self) {
		*self = Self::Refunding;
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
		state: PoolStatus<Locks>,
	) -> Self {
		Self {
			manager,
			curve,
			bonded_currencies,
			transferable,
			state,
		}
	}

	pub fn is_manager(&self, who: &AccountId) -> bool {
		who == &self.manager
	}

	pub fn is_minting_authorized(&self, who: &AccountId) -> bool {
		match &self.state {
			PoolStatus::Locked(locks) => locks.allow_mint || self.is_manager(who),
			PoolStatus::Active => true,
			_ => false,
		}
	}

	pub fn is_swapping_authorized(&self, who: &AccountId) -> bool {
		match &self.state {
			PoolStatus::Locked(locks) => locks.allow_swap || self.is_manager(who),
			PoolStatus::Active => true,
			_ => false,
		}
	}

	pub fn is_burning_authorized(&self, who: &AccountId) -> bool {
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
