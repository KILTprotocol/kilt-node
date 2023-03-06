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

use dip_support::VersionedIdentityProofAction;
use xcm::{latest::prelude::*, DoubleEncoded};

pub use identity_generation::*;
pub mod identity_generation {
	use sp_runtime::DispatchError;

	pub trait IdentityProofGenerator<Identifier, Identity, Output> {
		fn generate_proof(identifier: &Identifier, identity: &Identity) -> Result<Output, DispatchError>;
	}

	pub struct DefaultIdentityProofGenerator;

	impl<Identifier, Identity, Output> IdentityProofGenerator<Identifier, Identity, Output>
		for DefaultIdentityProofGenerator
	where
		Output: Default,
	{
		fn generate_proof(_identifier: &Identifier, _identity: &Identity) -> Result<Output, DispatchError> {
			Ok(Output::default())
		}
	}
}

pub use identity_dispatch::*;
pub mod identity_dispatch {
	use super::*;

	use codec::Encode;
	use frame_support::{traits::Get, weights::Weight};
	use sp_std::marker::PhantomData;
	use xcm::latest::opaque::Instruction;

	pub trait IdentityProofDispatcher<Identifier, IdentityRoot, Details = ()> {
		type PreDispatchOutput;
		type Error;

		fn pre_dispatch<B: TxBuilder<Identifier, IdentityRoot, Details>>(
			action: VersionedIdentityProofAction<Identifier, IdentityRoot, Details>,
			asset: MultiAsset,
			destination: MultiLocation,
		) -> Result<(Self::PreDispatchOutput, MultiAssets), Self::Error>;

		fn dispatch(pre_output: Self::PreDispatchOutput) -> Result<(), Self::Error>;
	}

	pub struct NullIdentityProofDispatcher;

	impl<Identifier, IdentityRoot, Details> IdentityProofDispatcher<Identifier, IdentityRoot, Details>
		for NullIdentityProofDispatcher
	{
		type PreDispatchOutput = ();
		type Error = &'static str;

		fn pre_dispatch<_B>(
			_action: VersionedIdentityProofAction<Identifier, IdentityRoot, Details>,
			_asset: MultiAsset,
			_destination: MultiLocation,
		) -> Result<((), MultiAssets), Self::Error> {
			Ok(((), MultiAssets::default()))
		}

		fn dispatch(_pre_output: Self::PreDispatchOutput) -> Result<(), Self::Error> {
			Ok(())
		}
	}

	fn catch_instructions(beneficiary: MultiLocation) -> Vec<Instruction> {
		vec![
			RefundSurplus,
			DepositAsset {
				assets: Wild(All),
				beneficiary,
			},
		]
	}

	// Dispatcher wrapping the XCM pallet.
	// It basically properly encodes the Transact operation, then delegates
	// everything else to the pallet's `send_xcm` function, similarly to what the
	// pallet's `send` extrinsic does.
	pub struct XcmRouterDispatcher<R, I, P, L, D = ()>(PhantomData<(R, I, P, L, D)>);

	impl<R, I, P, L, D> IdentityProofDispatcher<I, P, D> for XcmRouterDispatcher<R, I, P, L, D>
	where
		R: SendXcm,
		I: Encode,
		P: Encode,
		L: Get<MultiLocation>,
	{
		type PreDispatchOutput = R::Ticket;
		type Error = SendError;

		fn pre_dispatch<B: TxBuilder<I, P, D>>(
			action: VersionedIdentityProofAction<I, P, D>,
			asset: MultiAsset,
			destination: MultiLocation,
		) -> Result<(Self::PreDispatchOutput, MultiAssets), Self::Error> {
			println!("DidXcmV3ViaXcmPalletDispatcher::dispatch 1");
			// TODO: Replace with proper error handling
			let dest_tx = B::build(destination, action)
				.map_err(|_| ())
				.expect("Failed to build call");

			let operation = [
				vec![
					WithdrawAsset(asset.clone().into()),
					// Refund all and deposit back to owner if anything goes wrong.
					SetErrorHandler(catch_instructions(L::get()).into()),
					BuyExecution {
						fees: asset,
						weight_limit: Limited(Weight::from_parts(1_000_000, 1_000_000)),
					},
					Transact {
						origin_kind: OriginKind::SovereignAccount,
						require_weight_at_most: Weight::from_ref_time(1_000_000),
						call: dest_tx,
					},
				],
				catch_instructions(L::get()),
			]
			.concat();
			println!("DidXcmV3ViaXcmPalletDispatcher::dispatch 4");
			let op = Xcm(operation);
			R::validate(&mut Some(destination), &mut Some(op))
		}

		fn dispatch(pre_output: Self::PreDispatchOutput) -> Result<(), Self::Error> {
			R::deliver(pre_output).map(|_| ())
		}
	}
}

pub use identity_provision::*;
pub mod identity_provision {
	use sp_runtime::DispatchError;

	pub trait IdentityProvider<Identifier, Identity, Details = ()> {
		fn retrieve(identifier: &Identifier) -> Result<Option<(Identity, Details)>, DispatchError>;
	}

	pub struct DefaultIdentityProvider;

	impl<Identifier, Identity, Details> IdentityProvider<Identifier, Identity, Details> for DefaultIdentityProvider
	where
		Identity: Default,
		Details: Default,
	{
		fn retrieve(_identifier: &Identifier) -> Result<Option<(Identity, Details)>, DispatchError> {
			Ok(Some((Identity::default(), Details::default())))
		}
	}

	pub struct NoneIdentityProvider;

	impl<Identifier, Identity, Details> IdentityProvider<Identifier, Identity, Details> for NoneIdentityProvider {
		fn retrieve(_identifier: &Identifier) -> Result<Option<(Identity, Details)>, DispatchError> {
			Ok(None)
		}
	}
}

pub trait TxBuilder<Identifier, Proof, Details = ()> {
	type Error;

	fn build(
		dest: MultiLocation,
		action: VersionedIdentityProofAction<Identifier, Proof, Details>,
	) -> Result<DoubleEncoded<()>, Self::Error>;
}
