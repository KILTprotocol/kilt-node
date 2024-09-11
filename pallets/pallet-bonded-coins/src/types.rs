use frame_support::BoundedVec;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::Get;

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

#[derive(Default, Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum Curve {
	#[default]
	LinearRatioCurve,
}
