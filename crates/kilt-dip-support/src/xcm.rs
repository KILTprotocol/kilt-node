// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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

use cumulus_primitives_core::{Junction::AccountKey20, OriginKind};
use frame_support::ensure;
use sp_core::Get;
use sp_std::marker::PhantomData;
use xcm::latest::{
	Instruction,
	Instruction::{BuyExecution, DepositAsset, DescendOrigin, RefundSurplus, Transact, WithdrawAsset},
	Junction::{AccountId32, Parachain},
	Junctions::{X1, X2},
	MultiLocation, ParentThen, Weight,
};
use xcm_executor::traits::{ConvertOrigin, ShouldExecute};

// Allows a parachain to descend to an `X1(AccountId32)` or `X1(AccountId20)`
// junction, withdraw fees from their balance, and then carry on with a
// `Transact`.
pub struct AllowParachainProviderAsSubaccount<ProviderParaId>(PhantomData<ProviderParaId>);

impl<ProviderParaId> ShouldExecute for AllowParachainProviderAsSubaccount<ProviderParaId>
where
	ProviderParaId: Get<u32>,
{
	fn should_execute<RuntimeCall>(
		origin: &MultiLocation,
		instructions: &mut [Instruction<RuntimeCall>],
		_max_weight: Weight,
		_weight_credit: &mut Weight,
	) -> Result<(), ()> {
		// Ensure that the origin is a parachain allowed to act as identity provider.
		ensure!(
			*origin == ParentThen(Parachain(ProviderParaId::get()).into()).into(),
			()
		);
		let mut iter = instructions.iter();
		// This must match the implementation of the `IdentityProofDispatcher` trait.
		// TODO: Refactor so that they depend on each other and we avoid duplication
		match (
			iter.next(),
			iter.next(),
			iter.next(),
			iter.next(),
			iter.next(),
			iter.next(),
			iter.next(),
		) {
			(
				Some(DescendOrigin(X1(AccountId32 { .. }))),
				Some(WithdrawAsset { .. }),
				Some(BuyExecution { .. }),
				Some(Transact {
					origin_kind: OriginKind::Native,
					..
				}),
				Some(RefundSurplus),
				Some(DepositAsset { .. }),
				None,
			) => Ok(()),
			_ => Err(()),
		}
	}
}

// Decorate an existing barrier to add one more check in case all the previous
// barriers fail.
pub struct OkOrElseCheckForParachainProvider<Barrier, ProviderParaId>(PhantomData<(Barrier, ProviderParaId)>);

impl<Barrier, ProviderParaId> ShouldExecute for OkOrElseCheckForParachainProvider<Barrier, ProviderParaId>
where
	Barrier: ShouldExecute,
	ProviderParaId: Get<u32>,
{
	fn should_execute<RuntimeCall>(
		origin: &MultiLocation,
		instructions: &mut [Instruction<RuntimeCall>],
		max_weight: Weight,
		weight_credit: &mut Weight,
	) -> Result<(), ()> {
		Barrier::should_execute(origin, instructions, max_weight, weight_credit).or_else(|_| {
			AllowParachainProviderAsSubaccount::<ProviderParaId>::should_execute(
				origin,
				instructions,
				max_weight,
				weight_credit,
			)
		})
	}
}

// Decorate an existing barrier to check for the provider parachain origin only
// in case none of the previous barriers fail.
pub struct OkAndThenCheckForParachainProvider<Barrier, ProviderParaId>(PhantomData<(Barrier, ProviderParaId)>);

impl<Barrier, ProviderParaId> ShouldExecute for OkAndThenCheckForParachainProvider<Barrier, ProviderParaId>
where
	Barrier: ShouldExecute,
	ProviderParaId: Get<u32>,
{
	fn should_execute<RuntimeCall>(
		origin: &MultiLocation,
		instructions: &mut [Instruction<RuntimeCall>],
		max_weight: Weight,
		weight_credit: &mut Weight,
	) -> Result<(), ()> {
		Barrier::should_execute(origin, instructions, max_weight, weight_credit)?;
		AllowParachainProviderAsSubaccount::<ProviderParaId>::should_execute(
			origin,
			instructions,
			max_weight,
			weight_credit,
		)
	}
}

pub struct AccountIdJunctionAsParachain<ProviderParaId, ParachainOrigin, RuntimeOrigin>(
	PhantomData<(ProviderParaId, ParachainOrigin, RuntimeOrigin)>,
);

impl<ProviderParaId, ParachainOrigin, RuntimeOrigin> ConvertOrigin<RuntimeOrigin>
	for AccountIdJunctionAsParachain<ProviderParaId, ParachainOrigin, RuntimeOrigin>
where
	ProviderParaId: Get<u32>,
	ParachainOrigin: From<u32>,
	RuntimeOrigin: From<ParachainOrigin>,
{
	fn convert_origin(origin: impl Into<MultiLocation>, kind: OriginKind) -> Result<RuntimeOrigin, MultiLocation> {
		let origin = origin.into();
		let provider_para_id = ProviderParaId::get();
		match (kind, origin) {
			(
				OriginKind::Native,
				MultiLocation {
					parents: 1,
					interior: X2(Parachain(para_id), AccountId32 { .. } | AccountKey20 { .. }),
				},
			) if para_id == provider_para_id => Ok(ParachainOrigin::from(provider_para_id).into()),
			_ => Err(origin),
		}
	}
}
