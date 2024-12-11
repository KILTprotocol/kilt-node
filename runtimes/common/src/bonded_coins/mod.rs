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

/// Wrapper struct for [UnionOf] to implement the [metadata::Inspect] trait,
/// needed for the pallet_bonded_coins module.
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
use substrate_fixed::{
	traits::Fixed,
	types::{I75F53, U75F53},
};

pub mod hooks;

pub mod runtime_api;

/// The AssetId for bonded assets.
pub type AssetId = u32;

/// Fixed point number used for creating bonding curves.
pub type FixedPointInput = U75F53;

/// Fixed point number used for doing calculation steps in the bonding curves.
pub type FixedPoint = I75F53;

/// For a I75F53, the underlying type is a i128.
pub type FixedPointUnderlyingType = <FixedPoint as Fixed>::Bits;

/// Struct to implement the desired [Convert] trait needed for the
/// [NativeAndForeignAssets] type.
/// The generic type Target is used to determine the type of the asset id
/// for the Either::Left variant.
pub struct TargetFromLeft<Target>(PhantomData<Target>);

/// Metadata trait for native asset.
pub trait InspectMetadata<AssetId> {
	// Get name for native asset.
	fn name() -> Vec<u8>;
	// Get symbol for native asset.
	fn symbol() -> Vec<u8>;
	// Get decimals for native asset.
	fn decimals() -> u8;
}

/// Implements the Convert trait for the [TargetFromLeft] struct.
/// This is used to convert an asset id of type L to an [Either] type.
impl<Target: Get<L>, L: PartialEq + Eq> Convert<L, Either<(), L>> for TargetFromLeft<Target> {
	fn convert(l: L) -> Either<(), L> {
		// If l equals the target asset id, return Left(()), otherwise return
		// Right(l).
		Target::get().eq(&l).then(|| Left(())).map_or(Right(l), |n| n)
	}
}

pub struct NativeAndForeignAssets<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId, MetadataProvider = ()>(
	sp_std::marker::PhantomData<(
		NativeAsset,
		ForeignAssets,
		Criterion,
		AssetKind,
		AccountId,
		MetadataProvider,
	)>,
);

impl<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId, MetadataProvider> fungibles::Inspect<AccountId>
	for NativeAndForeignAssets<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId, MetadataProvider>
where
	NativeAsset: fungible::Inspect<AccountId>,
	ForeignAssets: fungibles::Inspect<AccountId, Balance = NativeAsset::Balance>,
	Criterion: Convert<AssetKind, Either<(), ForeignAssets::AssetId>>,
	AssetKind: AssetIdTraits,
{
	type AssetId = AssetKind;
	type Balance = NativeAsset::Balance;

	fn total_issuance(asset: Self::AssetId) -> Self::Balance {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::total_issuance(asset)
	}

	fn active_issuance(asset: Self::AssetId) -> Self::Balance {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::active_issuance(asset)
	}
	fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::minimum_balance(asset)
	}
	fn balance(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::balance(asset, who)
	}
	fn total_balance(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::total_balance(asset, who)
	}
	fn reducible_balance(
		asset: Self::AssetId,
		who: &AccountId,
		preservation: Preservation,
		force: Fortitude,
	) -> Self::Balance {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::reducible_balance(
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
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::can_deposit(
			asset, who, amount, provenance,
		)
	}
	fn can_withdraw(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
	) -> WithdrawConsequence<Self::Balance> {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::can_withdraw(asset, who, amount)
	}

	fn asset_exists(asset: Self::AssetId) -> bool {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::asset_exists(asset)
	}
}

impl<
		NativeAsset: fungible::Unbalanced<AccountId>,
		ForeignAssets: fungibles::Unbalanced<AccountId, Balance = NativeAsset::Balance>,
		Criterion: Convert<AssetKind, Either<(), ForeignAssets::AssetId>>,
		AssetKind: AssetIdTraits,
		AccountId,
		MetadataProvider,
	> fungibles::Unbalanced<AccountId>
	for NativeAndForeignAssets<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId, MetadataProvider>
{
	fn handle_dust(dust: fungibles::Dust<AccountId, Self>)
	where
		Self: Sized,
	{
		match Criterion::convert(dust.0) {
			Left(()) => <NativeAsset as fungible::Unbalanced<AccountId>>::handle_dust(fungible::Dust(dust.1)),
			Right(a) => <ForeignAssets as fungibles::Unbalanced<AccountId>>::handle_dust(fungibles::Dust(a, dust.1)),
		}
	}

	fn write_balance(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
	) -> Result<Option<Self::Balance>, DispatchError> {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::write_balance(asset, who, amount)
	}
	fn set_total_issuance(asset: Self::AssetId, amount: Self::Balance) {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::set_total_issuance(asset, amount);
	}
	fn decrease_balance(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
		precision: Precision,
		preservation: Preservation,
		force: Fortitude,
	) -> Result<Self::Balance, DispatchError> {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::decrease_balance(
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
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::increase_balance(
			asset, who, amount, precision,
		)
	}
}

impl<
		NativeAsset: fungible::Mutate<AccountId>,
		ForeignAssets: fungibles::Mutate<AccountId, Balance = NativeAsset::Balance>,
		Criterion: Convert<AssetKind, Either<(), ForeignAssets::AssetId>>,
		AssetKind: AssetIdTraits,
		AccountId: Eq,
		MetadataProvider,
	> fungibles::Mutate<AccountId>
	for NativeAndForeignAssets<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId, MetadataProvider>
{
	fn mint_into(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> Result<Self::Balance, DispatchError> {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::mint_into(asset, who, amount)
	}
	fn burn_from(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
		precision: Precision,
		force: Fortitude,
	) -> Result<Self::Balance, DispatchError> {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::burn_from(
			asset, who, amount, precision, force,
		)
	}
	fn shelve(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> Result<Self::Balance, DispatchError> {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::shelve(asset, who, amount)
	}
	fn restore(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> Result<Self::Balance, DispatchError> {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::restore(asset, who, amount)
	}
	fn transfer(
		asset: Self::AssetId,
		source: &AccountId,
		dest: &AccountId,
		amount: Self::Balance,
		preservation: Preservation,
	) -> Result<Self::Balance, DispatchError> {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::transfer(
			asset,
			source,
			dest,
			amount,
			preservation,
		)
	}

	fn set_balance(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> Self::Balance {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::set_balance(asset, who, amount)
	}
}

impl<
		NativeAsset: fungible::Inspect<AccountId>
			+ AccountTouch<(), AccountId, Balance = <NativeAsset as fungible::Inspect<AccountId>>::Balance>,
		ForeignAssets: fungibles::Inspect<AccountId>
			+ AccountTouch<
				ForeignAssets::AssetId,
				AccountId,
				Balance = <NativeAsset as fungible::Inspect<AccountId>>::Balance,
			>,
		Criterion: Convert<AssetKind, Either<(), ForeignAssets::AssetId>>,
		AssetKind: AssetIdTraits,
		AccountId,
		MetadataProvider,
	> AccountTouch<AssetKind, AccountId>
	for NativeAndForeignAssets<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId, MetadataProvider>
{
	type Balance = <NativeAsset as fungible::Inspect<AccountId>>::Balance;

	fn deposit_required(asset: AssetKind) -> Self::Balance {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::deposit_required(asset)
	}

	fn should_touch(asset: AssetKind, who: &AccountId) -> bool {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::should_touch(asset, who)
	}

	fn touch(asset: AssetKind, who: &AccountId, depositor: &AccountId) -> DispatchResult {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::touch(asset, who, depositor)
	}
}

impl<
		NativeAsset: fungible::Inspect<AccountId>,
		ForeignAssets: Inspect<AccountId, Balance = <NativeAsset as fungible::Inspect<AccountId>>::Balance>,
		Criterion: Convert<AssetKind, Either<(), ForeignAssets::AssetId>>,
		AssetKind: AssetIdTraits,
		AccountId,
		MetadataProvider: InspectMetadata<AssetKind>,
	> Inspect<AccountId>
	for NativeAndForeignAssets<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId, MetadataProvider>
{
	fn decimals(asset: Self::AssetId) -> u8 {
		match Criterion::convert(asset) {
			Left(()) => MetadataProvider::decimals(),
			Right(a) => ForeignAssets::decimals(a),
		}
	}

	fn name(asset: Self::AssetId) -> Vec<u8> {
		match Criterion::convert(asset) {
			Left(()) => MetadataProvider::name(),
			Right(a) => ForeignAssets::name(a),
		}
	}

	fn symbol(asset: Self::AssetId) -> Vec<u8> {
		match Criterion::convert(asset) {
			Left(()) => MetadataProvider::symbol(),
			Right(a) => ForeignAssets::symbol(a),
		}
	}
}

#[cfg(feature = "runtime-benchmarks")]
impl<
		NativeAsset: fungible::Inspect<AccountId>,
		ForeignAssets: fungibles::Inspect<AccountId, Balance = NativeAsset::Balance> + fungibles::Create<AccountId>,
		Criterion: Convert<AssetKind, Either<(), ForeignAssets::AssetId>>,
		AssetKind: AssetIdTraits,
		AccountId,
		MetadataProvider,
	> fungibles::Create<AccountId>
	for NativeAndForeignAssets<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId, MetadataProvider>
{
	fn create(asset: AssetKind, admin: AccountId, is_sufficient: bool, min_balance: Self::Balance) -> DispatchResult {
		UnionOf::<NativeAsset, ForeignAssets, Criterion, AssetKind, AccountId>::create(
			asset,
			admin,
			is_sufficient,
			min_balance,
		)
	}
}
