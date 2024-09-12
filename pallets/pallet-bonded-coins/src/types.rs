use frame_support::BoundedVec;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::Get;
use sp_runtime::traits::Saturating;

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
pub struct LinearRatioCurveParams<ParamType> {
	pub scaling_factor: ParamType,
}
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum Curve<ParamType> {
	/// Price scales linearly with the ratio of the total issuance of the active currency to the sum of all total issuances.
	/// `f(i_active) = s * i_active / (i_active + i_passive)`, where s is a scaling factor.
	/// Parameters:
	/// - Scaling Factor
	LinearRatioCurve(LinearRatioCurveParams<ParamType>),
}

pub struct MockCurve {}

impl MockCurve {
	pub fn new() -> Self {
		Self {}
	}

	pub fn calculate_cost<Balance: Saturating>(
		self,
		active_issuance_pre: Balance,
		active_issuance_post: Balance,
		_: Balance,
	) -> Balance {
		active_issuance_pre.saturating_sub(active_issuance_post)
	}
}
