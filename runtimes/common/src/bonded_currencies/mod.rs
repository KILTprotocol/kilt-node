use frame_support::{
	dispatch::DispatchResult,
	traits::{
		fungible::{self, UnionOf},
		fungibles::{self, metadata::Inspect},
		tokens::{
			AssetId as AssetIdTraits, DepositConsequence, Fortitude, Precision, Preservation, Provenance,
			WithdrawConsequence,
		},
		AccountTouch, Get,
	},
};
use sp_runtime::{
	traits::Convert,
	DispatchError, Either,
	Either::{Left, Right},
};
use sp_std::{marker::PhantomData, vec::Vec};
use substrate_fixed::types::{I75F53, U75F53};
use xcm::v4::Location;

/// The AssetId for bonded assets.
pub type AssetId = u32;

/// Fixed point number used for creating bonding curves.
pub type FloatInput = U75F53;

/// Fixed point number used for doing calculation steps in the bonding curves.
pub type Float = I75F53;

pub struct TargetFromLeft<Target, L = Location>(PhantomData<(Target, L)>);
impl<Target: Get<L>, L: PartialEq + Eq> Convert<L, Either<(), L>> for TargetFromLeft<Target, L> {
	fn convert(l: L) -> Either<(), L> {
		Target::get().eq(&l).then(|| Left(())).map_or(Right(l), |n| n)
	}
}

pub type NativeAndForeignAssets<Balances, Fungibles, Criterion, AssetKind, AccountId> =
	UnionOf<Balances, Fungibles, Criterion, AssetKind, AccountId>;

pub struct WrapperNativeAndForeignAssets<Left, Right, Criterion, AssetKind, AccountId>(
	sp_std::marker::PhantomData<(Left, Right, Criterion, AssetKind, AccountId)>,
);

impl<Left, Right, Criterion, AssetKind, AccountId> fungibles::Inspect<AccountId>
	for WrapperNativeAndForeignAssets<Left, Right, Criterion, AssetKind, AccountId>
where
	Left: fungible::Inspect<AccountId>,
	Right: fungibles::Inspect<AccountId, Balance = Left::Balance>,
	Criterion: Convert<AssetKind, Either<(), Right::AssetId>>,
	AssetKind: AssetIdTraits,
{
	type AssetId = AssetKind;
	type Balance = Left::Balance;

	fn total_issuance(asset: Self::AssetId) -> Self::Balance {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::total_issuance(asset)
	}

	fn active_issuance(asset: Self::AssetId) -> Self::Balance {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::active_issuance(asset)
	}
	fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::minimum_balance(asset)
	}
	fn balance(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::balance(asset, who)
	}
	fn total_balance(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::total_balance(asset, who)
	}
	fn reducible_balance(
		asset: Self::AssetId,
		who: &AccountId,
		preservation: Preservation,
		force: Fortitude,
	) -> Self::Balance {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::reducible_balance(
			asset,
			who,
			preservation,
			force,
		)
	}
	fn can_deposit(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
		provenance: Provenance,
	) -> DepositConsequence {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::can_deposit(
			asset, who, amount, provenance,
		)
	}
	fn can_withdraw(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
	) -> WithdrawConsequence<Self::Balance> {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::can_withdraw(asset, who, amount)
	}

	fn asset_exists(asset: Self::AssetId) -> bool {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::asset_exists(asset)
	}
}

impl<
		Left: fungible::Inspect<AccountId>,
		Right: fungibles::Inspect<AccountId, Balance = Left::Balance> + fungibles::Create<AccountId>,
		Criterion: Convert<AssetKind, Either<(), Right::AssetId>>,
		AssetKind: AssetIdTraits,
		AccountId,
	> fungibles::Create<AccountId> for WrapperNativeAndForeignAssets<Left, Right, Criterion, AssetKind, AccountId>
{
	fn create(asset: AssetKind, admin: AccountId, is_sufficient: bool, min_balance: Self::Balance) -> DispatchResult {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::create(
			asset,
			admin,
			is_sufficient,
			min_balance,
		)
	}
}

impl<
		Left: fungible::Unbalanced<AccountId>,
		Right: fungibles::Unbalanced<AccountId, Balance = Left::Balance>,
		Criterion: Convert<AssetKind, Either<(), Right::AssetId>>,
		AssetKind: AssetIdTraits,
		AccountId,
	> fungibles::Unbalanced<AccountId> for WrapperNativeAndForeignAssets<Left, Right, Criterion, AssetKind, AccountId>
{
	fn handle_dust(dust: fungibles::Dust<AccountId, Self>)
	where
		Self: Sized,
	{
		match Criterion::convert(dust.0) {
			Left(()) => <Left as fungible::Unbalanced<AccountId>>::handle_dust(fungible::Dust(dust.1)),
			Right(a) => <Right as fungibles::Unbalanced<AccountId>>::handle_dust(fungibles::Dust(a, dust.1)),
		}
	}

	fn write_balance(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
	) -> Result<Option<Self::Balance>, DispatchError> {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::write_balance(asset, who, amount)
	}
	fn set_total_issuance(asset: Self::AssetId, amount: Self::Balance) -> () {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::set_total_issuance(asset, amount);
	}
	fn decrease_balance(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
		precision: Precision,
		preservation: Preservation,
		force: Fortitude,
	) -> Result<Self::Balance, DispatchError> {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::decrease_balance(
			asset,
			who,
			amount,
			precision,
			preservation,
			force,
		)
	}
	fn increase_balance(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
		precision: Precision,
	) -> Result<Self::Balance, DispatchError> {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::increase_balance(
			asset, who, amount, precision,
		)
	}
}

impl<
		Left: fungible::Mutate<AccountId>,
		Right: fungibles::Mutate<AccountId, Balance = Left::Balance>,
		Criterion: Convert<AssetKind, Either<(), Right::AssetId>>,
		AssetKind: AssetIdTraits,
		AccountId: Eq,
	> fungibles::Mutate<AccountId> for WrapperNativeAndForeignAssets<Left, Right, Criterion, AssetKind, AccountId>
{
	fn mint_into(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> Result<Self::Balance, DispatchError> {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::mint_into(asset, who, amount)
	}
	fn burn_from(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
		precision: Precision,
		force: Fortitude,
	) -> Result<Self::Balance, DispatchError> {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::burn_from(
			asset, who, amount, precision, force,
		)
	}
	fn shelve(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> Result<Self::Balance, DispatchError> {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::shelve(asset, who, amount)
	}
	fn restore(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> Result<Self::Balance, DispatchError> {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::restore(asset, who, amount)
	}
	fn transfer(
		asset: Self::AssetId,
		source: &AccountId,
		dest: &AccountId,
		amount: Self::Balance,
		preservation: Preservation,
	) -> Result<Self::Balance, DispatchError> {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::transfer(
			asset,
			source,
			dest,
			amount,
			preservation,
		)
	}

	fn set_balance(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> Self::Balance {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::set_balance(asset, who, amount)
	}
}

impl<
		Left: fungible::Inspect<AccountId>
			+ AccountTouch<(), AccountId, Balance = <Left as fungible::Inspect<AccountId>>::Balance>,
		Right: fungibles::Inspect<AccountId>
			+ AccountTouch<Right::AssetId, AccountId, Balance = <Left as fungible::Inspect<AccountId>>::Balance>,
		Criterion: Convert<AssetKind, Either<(), Right::AssetId>>,
		AssetKind: AssetIdTraits,
		AccountId,
	> AccountTouch<AssetKind, AccountId> for WrapperNativeAndForeignAssets<Left, Right, Criterion, AssetKind, AccountId>
{
	type Balance = <Left as fungible::Inspect<AccountId>>::Balance;

	fn deposit_required(asset: AssetKind) -> Self::Balance {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::deposit_required(asset)
	}

	fn should_touch(asset: AssetKind, who: &AccountId) -> bool {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::should_touch(asset, who)
	}

	fn touch(asset: AssetKind, who: &AccountId, depositor: &AccountId) -> DispatchResult {
		NativeAndForeignAssets::<Left, Right, Criterion, AssetKind, AccountId>::touch(asset, who, depositor)
	}
}

impl<
		Left: fungible::Inspect<AccountId>,
		Right: Inspect<AccountId, Balance = <Left as fungible::Inspect<AccountId>>::Balance>,
		Criterion: Convert<AssetKind, Either<(), Right::AssetId>>,
		AssetKind: AssetIdTraits,
		AccountId,
	> Inspect<AccountId> for WrapperNativeAndForeignAssets<Left, Right, Criterion, AssetKind, AccountId>
{
	fn decimals(asset: Self::AssetId) -> u8 {
		match Criterion::convert(asset) {
			// TODO: CHANGE THAT
			Left(()) => 15u8,
			Right(a) => Right::decimals(a),
		}
	}

	fn name(asset: Self::AssetId) -> Vec<u8> {
		match Criterion::convert(asset) {
			// TODO: CHANGE THAT
			Left(()) => "KILT".as_bytes().to_vec(),
			Right(a) => Right::name(a),
		}
	}

	fn symbol(asset: Self::AssetId) -> Vec<u8> {
		match Criterion::convert(asset) {
			// TODO: CHANGE THAT
			Left(()) => "KILT".as_bytes().to_vec(),
			Right(a) => Right::symbol(a),
		}
	}
}
