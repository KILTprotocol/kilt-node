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

use cumulus_primitives_core::{Junction::AccountKey20, OriginKind, ParaId};
use frame_support::ensure;
use sp_core::Get;
use sp_std::marker::PhantomData;
use xcm::latest::{
	Instruction,
	Instruction::{BuyExecution, DescendOrigin, Transact, WithdrawAsset},
	Junction::{AccountId32, Parachain},
	Junctions::{X1, X2},
	MultiLocation, Weight,
};
use xcm_executor::traits::{ConvertOrigin, ShouldExecute};

// Allows a parachain to descend to an `X1(AccountId32)` or `X1(AccountId20)`
// junction, withdraw fees from their balance, and then carry on with a
// `Transact`.
pub struct AllowParachainProviderAsSubaccount<ProviderParaId>(PhantomData<ProviderParaId>);

impl<ProviderParaId> ShouldExecute for AllowParachainProviderAsSubaccount<ProviderParaId>
where
	ProviderParaId: Get<ParaId>,
{
	fn should_execute<RuntimeCall>(
		origin: &MultiLocation,
		instructions: &mut [Instruction<RuntimeCall>],
		_max_weight: Weight,
		_weight_credit: &mut Weight,
	) -> Result<(), ()> {
		// Ensure that the origin is a parachain allowed to act as identity provider.
		ensure!(*origin == Parachain(ProviderParaId::get().into()).into(), ());
		let mut iter = instructions.iter();
		// This must match the implementation of the `IdentityProofDispatcher` trait.
		// TODO: Refactor so that they depend on each other and we avoid duplication
		match (iter.next(), iter.next(), iter.next(), iter.next(), iter.next()) {
			(
				Some(DescendOrigin(X1(AccountId32 { .. } | AccountKey20 { .. }))),
				Some(WithdrawAsset { .. }),
				Some(BuyExecution { .. }),
				Some(Transact {
					origin_kind: OriginKind::Native,
					..
				}),
				None,
			) => Ok(()),
			_ => Err(()),
		}
	}
}

// Decorate an existing Barrier to add one more check to allow a sibling
// parachain as the DIP provider.
pub struct OrElseCheckForParachainProvider<Barrier, ProviderParaId>(PhantomData<(Barrier, ProviderParaId)>);

impl<Barrier, ProviderParaId> ShouldExecute for OrElseCheckForParachainProvider<Barrier, ProviderParaId>
where
	Barrier: ShouldExecute,
	ProviderParaId: Get<ParaId>,
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

pub struct AccountIdJunctionToParachainOriginConverter<ProviderParaId, ParachainOrigin, RuntimeOrigin>(
	PhantomData<(ProviderParaId, ParachainOrigin, RuntimeOrigin)>,
);

impl<ProviderParaId, ParachainOrigin, RuntimeOrigin> ConvertOrigin<RuntimeOrigin>
	for AccountIdJunctionToParachainOriginConverter<ProviderParaId, ParachainOrigin, RuntimeOrigin>
where
	ProviderParaId: Get<ParaId>,
	ParachainOrigin: From<ParaId>,
	RuntimeOrigin: From<ParachainOrigin>,
{
	fn convert_origin(origin: impl Into<MultiLocation>, kind: OriginKind) -> Result<RuntimeOrigin, MultiLocation> {
		let origin = origin.into();
		let provider_para_id: u32 = ProviderParaId::get().into();
		match (kind, origin) {
			(
				OriginKind::Native,
				MultiLocation {
					parents: 1,
					interior: X2(Parachain(para_id), AccountId32 { .. } | AccountKey20 { .. }),
				},
			) if para_id == provider_para_id => Ok(ParachainOrigin::from(ProviderParaId::get()).into()),
			_ => Err(origin),
		}
	}
}

// // Decorate an existing OriginConverter to add the conversion of a sibling
// // parachain as the DIP provider.
// pub struct OrElseSiblingParachainProviderConverter<OriginConverter,
// ProviderParaId, ParachainOrigin, RuntimeOrigin>(
// 	PhantomData<(OriginConverter, ProviderParaId, ParachainOrigin,
// RuntimeOrigin)>, );

// impl<OriginConverter, ProviderParaId, ParachainOrigin, RuntimeOrigin>
// ConvertOrigin<RuntimeOrigin>
// 	for OrElseSiblingParachainProviderConverter<OriginConverter, ProviderParaId,
// ParachainOrigin, RuntimeOrigin> where
// 	OriginConverter: ConvertOrigin<RuntimeOrigin>,
// 	ProviderParaId: Get<ParaId>,
// 	ParachainOrigin: From<ParaId>,
// 	RuntimeOrigin: From<ParachainOrigin>,
// {
// 	fn convert_origin(origin: impl Into<MultiLocation>, kind: OriginKind) ->
// Result<RuntimeOrigin, MultiLocation> {
// 		OriginConverter::convert_origin(origin, kind)?;
// 		AccountIdJunctionToParachainOriginConverter::<ProviderParaId,
// ParachainOrigin, RuntimeOrigin>::convert_origin( 			origin, kind,
// 		)
// 	}
// }
