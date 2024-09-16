use frame_support::BoundedVec;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::Get;
use sp_runtime::{ArithmeticError, FixedPointNumber};

use crate::curves_parameters::{self, BondingFunction};

#[derive(Default, Clone, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct Locks {
	pub allow_mint: bool,
	pub allow_burn: bool,
	pub allow_swap: bool,
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum PoolStatus<LockType> {
	Active,
	Frozen(LockType),
	Destroying,
}
impl<LockType: Default> Default for PoolStatus<LockType> {
	fn default() -> Self {
		Self::Frozen(LockType::default())
	}
}

#[derive(Default, Clone, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct PoolDetails<AccountId, CurrencyId, ParametrizedCurve, MaxOptions: Get<u32>> {
	pub creator: AccountId,
	pub curve: ParametrizedCurve,
	pub bonded_currencies: BoundedVec<CurrencyId, MaxOptions>,
	pub state: PoolStatus<Locks>,
	pub transferable: bool,
}

impl<AccountId, CurrencyId, ParametrizedCurve, MaxOptions: Get<u32>>
	PoolDetails<AccountId, CurrencyId, ParametrizedCurve, MaxOptions>
{
	pub fn new(
		creator: AccountId,
		curve: ParametrizedCurve,
		bonded_currencies: BoundedVec<CurrencyId, MaxOptions>,
		transferable: bool,
	) -> Self {
		Self {
			creator,
			curve,
			bonded_currencies,
			transferable,
			state: PoolStatus::default(),
		}
	}
}

#[derive(Debug, Encode, Decode, Clone, PartialEq, TypeInfo)]
pub struct TokenMeta<Balance, AssetId> {
	pub id: AssetId,
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub decimals: u8,
	pub min_balance: Balance,
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum Curve<F> {
	/// Price scales linearly with the ratio of the total issuance of the active currency to the sum of all total issuances.
	/// `f(i_active) = s * i_active / (i_active + i_passive)`, where s is a scaling factor.
	/// Parameters:
	/// - Scaling Factor
	LinearRatioCurve(curves_parameters::LinearBondingFunctionParameters<F>),
}

impl<F> Curve<F>
where
	F: FixedPointNumber,
{
	pub fn calculate_cost(
		&self,
		active_issuance_pre: F,
		active_issuance_post: F,
		passive_issuance: F,
	) -> Result<F, ArithmeticError> {
		let calculation_param = match self {
			Curve::LinearRatioCurve(params) => params,
		};

		let active_issuance_pre_with_passive = active_issuance_pre.saturating_add(passive_issuance);
		let active_issuance_post_with_passive = active_issuance_post.saturating_add(passive_issuance);

		calculation_param.calculate_costs(active_issuance_pre_with_passive, active_issuance_post_with_passive)
	}
}
