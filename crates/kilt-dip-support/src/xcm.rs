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

use dip_support::IdentityDetailsAction;
use pallet_dip_provider::traits::{IdentityProofDispatcher, TxBuilder};
use parity_scale_codec::Encode;
use sp_core::Get;
use sp_std::marker::PhantomData;
use xcm::v3::{
	Instruction::{BuyExecution, DepositAsset, DescendOrigin, ExpectOrigin, RefundSurplus, Transact, WithdrawAsset},
	InteriorMultiLocation,
	Junction::AccountId32,
	Junctions::{Here, X1},
	MultiAsset,
	MultiAssetFilter::Wild,
	MultiAssets, MultiLocation, OriginKind, SendError, SendXcm, Weight,
	WeightLimit::Limited,
	WildMultiAsset::All,
	Xcm,
};

// Dispatcher using a type implementing the `SendXcm` trait.
// It properly encodes the `Transact` operation, then delegates everything else
// to the sender, similarly to what the XCM pallet's `send` extrinsic does.
pub struct XcmRouterIdentityDispatcher<Router, UniversalLocationProvider>(
	PhantomData<(Router, UniversalLocationProvider)>,
);

impl<Router, UniversalLocationProvider, Identifier, ProofOutput, AccountId, Details>
	IdentityProofDispatcher<Identifier, ProofOutput, AccountId, Details>
	for XcmRouterIdentityDispatcher<Router, UniversalLocationProvider>
where
	Router: SendXcm,
	UniversalLocationProvider: Get<InteriorMultiLocation>,
	Identifier: Encode,
	ProofOutput: Encode,
	AccountId: Into<[u8; 32]> + Clone,
{
	type PreDispatchOutput = Router::Ticket;
	type Error = SendError;

	fn pre_dispatch<Builder: TxBuilder<Identifier, ProofOutput, Details>>(
		action: IdentityDetailsAction<Identifier, ProofOutput, Details>,
		source: AccountId,
		asset: MultiAsset,
		weight: Weight,
		destination: MultiLocation,
	) -> Result<(Self::PreDispatchOutput, MultiAssets), Self::Error> {
		// TODO: Replace with proper error handling
		let dest_tx = Builder::build(destination, action)
			.map_err(|_| ())
			.expect("Failed to build call");

		// TODO: Set an error handler and an appendix to refund any leftover funds to
		// the provider parachain sovereign account.
		let operation = [[
			ExpectOrigin(Some(
				Here.into_location()
					.reanchored(&destination, UniversalLocationProvider::get())
					.unwrap(),
			)),
			DescendOrigin(X1(AccountId32 {
				network: None,
				id: source.clone().into(),
			})),
			WithdrawAsset(asset.clone().into()),
			BuyExecution {
				fees: asset,
				weight_limit: Limited(weight),
			},
			Transact {
				origin_kind: OriginKind::Native,
				require_weight_at_most: weight,
				call: dest_tx,
			},
			RefundSurplus,
			DepositAsset {
				assets: Wild(All),
				beneficiary: MultiLocation {
					parents: 1,
					// Re-anchor the same account junction as seen from the destination.
					// TODO: Error handling
					interior: Here
						.into_location()
						.reanchored(&destination, UniversalLocationProvider::get())
						.unwrap()
						.pushed_with_interior(AccountId32 {
							network: None,
							id: source.into(),
						})
						.unwrap()
						.interior,
				},
			},
		]]
		.concat();
		// TODO: Restructure the trait to be able to inject the [Instruction] provider,
		// and unit test that.
		debug_assert!(barriers::check_expected_dip_instruction_order(&operation).is_ok());
		let op = Xcm(operation);
		Router::validate(&mut Some(destination), &mut Some(op))
	}

	fn dispatch(pre_output: Self::PreDispatchOutput) -> Result<(), Self::Error> {
		Router::deliver(pre_output).map(|_| ())
	}
}

pub mod barriers {
	use super::*;

	use frame_support::{ensure, traits::ProcessMessageError};
	use xcm::v3::{Instruction, Junction::Parachain, ParentThen};
	use xcm_executor::traits::ShouldExecute;

	// Must match the order of instructions as produced by the provider's
	// implementation of the `IdentityProofDispatcher` trait.
	pub(crate) fn check_expected_dip_instruction_order<RuntimeCall>(
		instructions: &[Instruction<RuntimeCall>],
	) -> Result<(), ()> {
		let mut iter = instructions.iter();
		match (
			iter.next(),
			iter.next(),
			iter.next(),
			iter.next(),
			iter.next(),
			iter.next(),
			iter.next(),
			iter.next(),
		) {
			(
				// A first instruction different than `DescendOrigin` is needed to distinguish between user-triggered
				// and parachain-triggered XCM messages, since also the XCM pallet always preprends user-created XCM
				// messages with a `DescendOrigin` instruction.
				Some(ExpectOrigin(..)),
				// Go down to user level to charge them for the XCM fees.
				Some(DescendOrigin(X1(AccountId32 { .. }))),
				// Expect the user to first withdraw an asset to pay for the fees.
				Some(WithdrawAsset { .. }),
				// Buy execution time.
				Some(BuyExecution { .. }),
				// Although this is irrelevant since `origin_kind` can also be specified by a user, we use
				// `OriginKind::Native` here to make clear this is a parachain-dispatched XCM message.
				Some(Transact {
					origin_kind: OriginKind::Native,
					..
				}),
				// Any unused weight is refunded.
				Some(RefundSurplus),
				// Any unused assets are refunded back into the user's account.
				Some(DepositAsset { .. }),
				// No more instructions are allowed.
				None,
			) => Ok(()),
			_ => Err(()),
		}
	}

	// Allows a parachain to descend to an `X1(AccountId32)` junction, withdraw fees
	// from their balance, and then carry on with a `Transact`.
	// Must be used **ONLY** in conjunction with the `AccountIdJunctionAsParachain`
	// origin converter.
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
		) -> Result<(), ProcessMessageError> {
			#[cfg(feature = "std")]
			println!(
				"AllowParachainProviderAsSubaccount::should_execute(origin = {:?}, instructions = {:?}",
				origin, instructions
			);
			// Ensure that the origin is a parachain allowed to act as identity provider.
			ensure!(
				*origin == ParentThen(Parachain(ProviderParaId::get()).into()).into(),
				ProcessMessageError::Yield
			);
			check_expected_dip_instruction_order(instructions).map_err(|_| ProcessMessageError::Yield)
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
		) -> Result<(), ProcessMessageError> {
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
	pub struct ErrOrElseCheckForParachainProvider<Barrier, ProviderParaId>(PhantomData<(Barrier, ProviderParaId)>);

	impl<Barrier, ProviderParaId> ShouldExecute for ErrOrElseCheckForParachainProvider<Barrier, ProviderParaId>
	where
		Barrier: ShouldExecute,
		ProviderParaId: Get<u32>,
	{
		fn should_execute<RuntimeCall>(
			origin: &MultiLocation,
			instructions: &mut [Instruction<RuntimeCall>],
			max_weight: Weight,
			weight_credit: &mut Weight,
		) -> Result<(), ProcessMessageError> {
			Barrier::should_execute(origin, instructions, max_weight, weight_credit)?;
			AllowParachainProviderAsSubaccount::<ProviderParaId>::should_execute(
				origin,
				instructions,
				max_weight,
				weight_credit,
			)
		}
	}
}

pub mod origins {
	use super::*;

	use xcm::v3::{Junction::Parachain, Junctions::X2};
	use xcm_executor::traits::ConvertOrigin;

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
						interior: X2(Parachain(para_id), AccountId32 { .. }),
					},
				) if para_id == provider_para_id => Ok(ParachainOrigin::from(provider_para_id).into()),
				_ => Err(origin),
			}
		}
	}
}
