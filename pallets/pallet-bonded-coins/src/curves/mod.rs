pub(crate) mod lmsr;
pub(crate) mod polynomial;
pub(crate) mod square_root;

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use sp_std::ops::{AddAssign, BitOrAssign, ShlAssign};
use substrate_fixed::traits::{Fixed, FixedSigned, ToFixed};

use crate::{
	curves::{lmsr::LMSRParameters, polynomial::PolynomialParameters, square_root::SquareRootParameters},
	PassiveSupply, Precision,
};

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum Curve<Parameter> {
	Polynomial(PolynomialParameters<Parameter>),
	SquareRoot(SquareRootParameters<Parameter>),
	LMSR(LMSRParameters<Parameter>),
}

pub enum Operation<PassiveSupply> {
	Mint(PassiveSupply),
	Burn(PassiveSupply),
}

impl<Balance> Operation<PassiveSupply<Balance>> {
	pub fn inner_value(&self) -> &PassiveSupply<Balance> {
		match self {
			Operation::Mint(x) => x,
			Operation::Burn(x) => x,
		}
	}
}

impl<Parameter> BondingFunction<Parameter> for Curve<Parameter>
where
	Parameter: FixedSigned + PartialOrd<Precision> + From<Precision> + ToFixed,
	<Parameter as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
	fn calculate_costs(
		&self,
		active_issuance_pre: Parameter,
		active_issuance_post: Parameter,
		op: Operation<PassiveSupply<Parameter>>,
	) -> Result<Parameter, ArithmeticError> {
		match self {
			Curve::Polynomial(params) => params.calculate_costs(active_issuance_pre, active_issuance_post, op),
			Curve::SquareRoot(params) => params.calculate_costs(active_issuance_pre, active_issuance_post, op),
			Curve::LMSR(params) => params.calculate_costs(active_issuance_pre, active_issuance_post, op),
		}
	}
}

pub trait BondingFunction<Balance> {
	fn calculate_costs(
		&self,
		active_issuance_pre: Balance,
		active_issuance_post: Balance,
		op: Operation<PassiveSupply<Balance>>,
	) -> Result<Balance, ArithmeticError>;
}

fn square<FixedType: Fixed>(x: FixedType) -> Result<FixedType, ArithmeticError> {
	x.checked_mul(x).ok_or(ArithmeticError::Overflow)
}

fn calculate_integral_bounds<FixedType: Fixed>(
	op: Operation<PassiveSupply<FixedType>>,
	active_issuance_pre: FixedType,
	active_issuance_post: FixedType,
) -> (FixedType, FixedType) {
	match op {
		Operation::Burn(passive) => {
			let accumulated_passive_issuance = calculate_accumulated_passive_issuance(&passive);
			(
				active_issuance_post.saturating_add(accumulated_passive_issuance),
				active_issuance_pre.saturating_add(accumulated_passive_issuance),
			)
		}
		Operation::Mint(passive) => {
			let accumulated_passive_issuance = calculate_accumulated_passive_issuance(&passive);
			(
				active_issuance_pre.saturating_add(accumulated_passive_issuance),
				active_issuance_post.saturating_add(accumulated_passive_issuance),
			)
		}
	}
}

fn calculate_accumulated_passive_issuance<Balance: Fixed>(passive_issuance: &[Balance]) -> Balance {
	passive_issuance
		.iter()
		.fold(Balance::from_num(0), |sum, x| sum.saturating_add(*x))
}
