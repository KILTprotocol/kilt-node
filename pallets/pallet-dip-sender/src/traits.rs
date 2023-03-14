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

	pub trait IdentityProofGenerator<Identifier, Identity, Output> {
		type Error;

		fn generate_proof(identifier: &Identifier, identity: &Identity) -> Result<Output, Self::Error>;
	}

	// Implement the `IdentityProofGenerator` by returning the `Default` value for
	// the `Output` type.
	pub struct DefaultIdentityProofGenerator;

	impl<Identifier, Identity, Output> IdentityProofGenerator<Identifier, Identity, Output>
		for DefaultIdentityProofGenerator
	where
		Output: Default,
	{
		type Error = ();

		fn generate_proof(_identifier: &Identifier, _identity: &Identity) -> Result<Output, Self::Error> {
			Ok(Output::default())
		}
	}
}

pub use identity_dispatch::*;
pub mod identity_dispatch {
	use super::*;

	use codec::Encode;
	use frame_support::{traits::Get, weights::Weight};
	use sp_std::{marker::PhantomData, vec, vec::Vec};
	use xcm::latest::Instruction;

	pub trait IdentityProofDispatcher<Identifier, IdentityRoot, Details = ()> {
		type PreDispatchOutput;
		type Error;

		fn pre_dispatch<B: TxBuilder<Identifier, IdentityRoot, Details>>(
			action: VersionedIdentityProofAction<Identifier, IdentityRoot, Details>,
			asset: MultiAsset,
			weight: Weight,
			destination: MultiLocation,
		) -> Result<(Self::PreDispatchOutput, MultiAssets), Self::Error>;

		fn dispatch(pre_output: Self::PreDispatchOutput) -> Result<(), Self::Error>;
	}

	// Returns `Ok` without doing anything.
	pub struct NullIdentityProofDispatcher;

	impl<Identifier, IdentityRoot, Details> IdentityProofDispatcher<Identifier, IdentityRoot, Details>
		for NullIdentityProofDispatcher
	{
		type PreDispatchOutput = ();
		type Error = ();

		fn pre_dispatch<_B>(
			_action: VersionedIdentityProofAction<Identifier, IdentityRoot, Details>,
			_asset: MultiAsset,
			_weight: Weight,
			_destination: MultiLocation,
		) -> Result<((), MultiAssets), Self::Error> {
			Ok(((), MultiAssets::default()))
		}

		fn dispatch(_pre_output: Self::PreDispatchOutput) -> Result<(), Self::Error> {
			Ok(())
		}
	}

	// Dispatcher using a type implementing the `SendXcm` trait.
	// It properly encodes the `Transact` operation, then delegates everything else
	// to the sender, similarly to what the XCM pallet's `send` extrinsic does.
	pub struct XcmRouterDispatcher<Router, Identifier, ProofOutput, Location, Details = ()>(
		PhantomData<(Router, Identifier, ProofOutput, Location, Details)>,
	);

	impl<Router, Identifier, ProofOutput, Location, Details> IdentityProofDispatcher<Identifier, ProofOutput, Details>
		for XcmRouterDispatcher<Router, Identifier, ProofOutput, Location, Details>
	where
		Router: SendXcm,
		Identifier: Encode,
		ProofOutput: Encode,
		Location: Get<MultiLocation>,
	{
		type PreDispatchOutput = Router::Ticket;
		type Error = SendError;

		fn pre_dispatch<Builder: TxBuilder<Identifier, ProofOutput, Details>>(
			action: VersionedIdentityProofAction<Identifier, ProofOutput, Details>,
			asset: MultiAsset,
			weight: Weight,
			destination: MultiLocation,
		) -> Result<(Self::PreDispatchOutput, MultiAssets), Self::Error> {
			// TODO: Replace with proper error handling
			let dest_tx = Builder::build(destination, action)
				.map_err(|_| ())
				.expect("Failed to build call");

			fn catch_instructions(beneficiary: MultiLocation) -> Vec<Instruction<()>> {
				vec![
					RefundSurplus,
					DepositAsset {
						assets: Wild(All),
						beneficiary,
					},
				]
			}

			// Set an error handler to refund any leftover in case anything goes wrong.
			let operation = [
				vec![
					WithdrawAsset(asset.clone().into()),
					// Refund all and deposit back to owner if anything goes wrong.
					SetErrorHandler(catch_instructions(Location::get()).into()),
					BuyExecution {
						fees: asset,
						weight_limit: Unlimited,
					},
					Transact {
						origin_kind: OriginKind::Native,
						require_weight_at_most: weight,
						call: dest_tx,
					},
				],
				catch_instructions(Location::get()),
			]
			.concat();
			let op = Xcm(operation);
			Router::validate(&mut Some(destination), &mut Some(op))
		}

		fn dispatch(pre_output: Self::PreDispatchOutput) -> Result<(), Self::Error> {
			Router::deliver(pre_output).map(|_| ())
		}
	}
}

pub use identity_provision::*;
pub mod identity_provision {

	pub trait IdentityProvider<Identifier, Identity, Details = ()> {
		type Error;

		fn retrieve(identifier: &Identifier) -> Result<Option<(Identity, Details)>, Self::Error>;
	}

	// Return the `Default` value if `Identity` adn `Details` both implement it.
	pub struct DefaultIdentityProvider;

	impl<Identifier, Identity, Details> IdentityProvider<Identifier, Identity, Details> for DefaultIdentityProvider
	where
		Identity: Default,
		Details: Default,
	{
		type Error = ();

		fn retrieve(_identifier: &Identifier) -> Result<Option<(Identity, Details)>, Self::Error> {
			Ok(Some((Identity::default(), Details::default())))
		}
	}

	// Always return `None`. Might be useful for tests.
	pub struct NoneIdentityProvider;

	impl<Identifier, Identity, Details> IdentityProvider<Identifier, Identity, Details> for NoneIdentityProvider {
		type Error = ();

		fn retrieve(_identifier: &Identifier) -> Result<Option<(Identity, Details)>, Self::Error> {
			Ok(None)
		}
	}
}

// Given a destination and an identity action, creates and encodes the proper
// `Transact` call.
pub trait TxBuilder<Identifier, Proof, Details = ()> {
	type Error;

	fn build(
		dest: MultiLocation,
		action: VersionedIdentityProofAction<Identifier, Proof, Details>,
	) -> Result<DoubleEncoded<()>, Self::Error>;
}
