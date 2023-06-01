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
use xcm::{latest::prelude::*, DoubleEncoded};

pub use identity_generation::*;
pub mod identity_generation {
	use sp_std::marker::PhantomData;

	pub trait IdentityProofGenerator<Identifier, Identity> {
		type Error;
		type Output;

		fn generate_commitment(identifier: &Identifier, identity: &Identity) -> Result<Self::Output, Self::Error>;
	}

	// Implement the `IdentityProofGenerator` by returning the `Default` value for
	// the `Output` type.
	pub struct DefaultIdentityProofGenerator<Output>(PhantomData<Output>);

	impl<Identifier, Identity, Output> IdentityProofGenerator<Identifier, Identity>
		for DefaultIdentityProofGenerator<Output>
	where
		Output: Default,
	{
		type Error = ();
		type Output = Output;

		fn generate_commitment(_identifier: &Identifier, _identity: &Identity) -> Result<Self::Output, Self::Error> {
			Ok(Output::default())
		}
	}
}

pub use identity_dispatch::*;
pub mod identity_dispatch {
	use super::*;

	use frame_support::{pallet_prelude::Get, weights::Weight};
	use parity_scale_codec::Encode;
	use sp_std::marker::PhantomData;

	pub trait IdentityProofDispatcher<Identifier, IdentityRoot, AccountId, Details = ()> {
		type PreDispatchOutput;
		type Error;

		fn pre_dispatch<B: TxBuilder<Identifier, IdentityRoot, Details>>(
			action: IdentityDetailsAction<Identifier, IdentityRoot, Details>,
			source: AccountId,
			asset: MultiAsset,
			weight: Weight,
			destination: MultiLocation,
		) -> Result<(Self::PreDispatchOutput, MultiAssets), Self::Error>;

		fn dispatch(pre_output: Self::PreDispatchOutput) -> Result<(), Self::Error>;
	}

	// Returns `Ok` without doing anything.
	pub struct NullIdentityProofDispatcher;

	impl<Identifier, IdentityRoot, AccountId, Details>
		IdentityProofDispatcher<Identifier, IdentityRoot, AccountId, Details> for NullIdentityProofDispatcher
	{
		type PreDispatchOutput = ();
		type Error = ();

		fn pre_dispatch<_B>(
			_action: IdentityDetailsAction<Identifier, IdentityRoot, Details>,
			_source: AccountId,
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
	pub struct XcmRouterDispatcher<Router, UniversalLocationProvider>(PhantomData<(Router, UniversalLocationProvider)>);

	impl<Router, UniversalLocationProvider, Identifier, ProofOutput, AccountId, Details>
		IdentityProofDispatcher<Identifier, ProofOutput, AccountId, Details>
		for XcmRouterDispatcher<Router, UniversalLocationProvider>
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
	use sp_std::marker::PhantomData;

	pub trait IdentityProvider<Identifier> {
		type Error;
		type Success;

		fn retrieve(identifier: &Identifier) -> Result<Option<Self::Success>, Self::Error>;
	}

	// Return the `Default` value if `Identity` adn `Details` both implement it.
	pub struct DefaultIdentityProvider<Identity>(PhantomData<Identity>);

	impl<Identifier, Identity> IdentityProvider<Identifier> for DefaultIdentityProvider<Identity>
	where
		Identity: Default,
	{
		type Error = ();
		type Success = Identity;

		fn retrieve(_identifier: &Identifier) -> Result<Option<Self::Success>, Self::Error> {
			Ok(Some(Identity::default()))
		}
	}

	// Always return `None`. Might be useful for tests.
	pub struct NoneIdentityProvider;

	impl<Identifier> IdentityProvider<Identifier> for NoneIdentityProvider {
		type Error = ();
		type Success = ();

		fn retrieve(_identifier: &Identifier) -> Result<Option<Self::Success>, Self::Error> {
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
		action: IdentityDetailsAction<Identifier, Proof, Details>,
	) -> Result<DoubleEncoded<()>, Self::Error>;
}
