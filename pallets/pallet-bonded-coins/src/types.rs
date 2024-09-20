use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{ArithmeticError, FixedPointNumber};

use crate::curves_parameters::{self, BondingFunction, SquareRoot};

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
	pub decimals: u8,
	pub min_balance: Balance,
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum Curve<F> {
	LinearRatioCurve(curves_parameters::LinearBondingFunctionParameters<F>),
	QuadraticRatioCurve(curves_parameters::QuadraticBondingFunctionParameters<F>),
	SquareRootBondingFunction(curves_parameters::SquareRootBondingFunctionParameters<F>),
	RationalBondingFunction(curves_parameters::RationalBondingFunctionParameters<F>),
}

pub enum DiffKind {
	Mint,
	Burn,
}

impl<F> Curve<F>
where
	F: FixedPointNumber + SquareRoot,
{
	pub fn calculate_cost(
		&self,
		active_issuance_pre: F,
		active_issuance_post: F,
		passive_issuance: F,
	) -> Result<F, ArithmeticError> {
		let active_issuance_pre_with_passive = active_issuance_pre.saturating_add(passive_issuance);
		let active_issuance_post_with_passive = active_issuance_post.saturating_add(passive_issuance);

		match self {
			Curve::LinearRatioCurve(params) => {
				params.calculate_costs(active_issuance_pre_with_passive, active_issuance_post_with_passive)
			}
			Curve::QuadraticRatioCurve(params) => {
				params.calculate_costs(active_issuance_pre_with_passive, active_issuance_post_with_passive)
			}
			Curve::SquareRootBondingFunction(params) => {
				params.calculate_costs(active_issuance_pre_with_passive, active_issuance_post_with_passive)
			}
			Curve::RationalBondingFunction(params) => params.calculate_costs(
				(active_issuance_pre, passive_issuance),
				(active_issuance_post, passive_issuance),
			),
		}
	}
}