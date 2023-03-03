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
	use frame_support::weights::Weight;
	use frame_system::{pallet_prelude::OriginFor, RawOrigin};
	use sp_std::marker::PhantomData;
	use xcm_executor::traits::Convert;

	pub trait IdentityProofDispatcher<Identifier, AccountId, IdentityRoot, Details = ()> {
		type Error;

		fn dispatch<B: TxBuilder<Identifier, IdentityRoot, Details>>(
			action: VersionedIdentityProofAction<Identifier, IdentityRoot, Details>,
			dispatcher: AccountId,
			asset: MultiAsset,
			destination: MultiLocation,
		) -> Result<(), Self::Error>;
	}

	pub struct NullIdentityProofDispatcher;

	impl<Identifier, AccountId, IdentityRoot, Details>
		IdentityProofDispatcher<Identifier, AccountId, IdentityRoot, Details> for NullIdentityProofDispatcher
	{
		type Error = &'static str;

		fn dispatch<_B>(
			_action: VersionedIdentityProofAction<Identifier, IdentityRoot, Details>,
			_dispatcher: AccountId,
			_asset: MultiAsset,
			_destination: MultiLocation,
		) -> Result<(), Self::Error> {
			Ok(())
		}
	}

	// Dispatcher wrapping the XCM pallet.
	// It basically properly encodes the Transact operation, then delegates
	// everything else to the pallet's `send_xcm` function, similarly to what the
	// pallet's `send` extrinsic does.
	pub struct DidXcmV3ViaXcmPalletDispatcher<T, I, P, C, D = ()>(PhantomData<(T, I, P, C, D)>);

	impl<T, I, P, C, D> IdentityProofDispatcher<I, <T as frame_system::Config>::AccountId, P, D>
		for DidXcmV3ViaXcmPalletDispatcher<T, I, P, C, D>
	where
		T: pallet_xcm::Config,
		I: Encode,
		P: Encode,
		C: Convert<OriginFor<T>, MultiLocation>,
	{
		type Error = SendError;

		fn dispatch<B: TxBuilder<I, P, D>>(
			action: VersionedIdentityProofAction<I, P, D>,
			dispatcher: T::AccountId,
			asset: MultiAsset,
			destination: MultiLocation,
		) -> Result<(), Self::Error> {
			// Check that destination is a chain, or alternatively make sure statically it
			// can only be a chain.
			println!("DidXcmV3ViaXcmPalletDispatcher::dispatch 1");
			let origin_location =
				C::convert(RawOrigin::Signed(dispatcher).into()).map_err(|_| SendError::DestinationUnsupported)?;
			println!(
				"DidXcmV3ViaXcmPalletDispatcher::dispatch 2 with origin_location: {:?}",
				origin_location
			);
			let interior: Junctions = origin_location
				.try_into()
				.map_err(|_| SendError::DestinationUnsupported)?;
			println!(
				"DidXcmV3ViaXcmPalletDispatcher::dispatch 3 with interior: {:?}",
				interior
			);
			// TODO: Replace with proper error handling
			let dest_tx = B::build(destination, action)
				.map_err(|_| ())
				.expect("Failed to build call");
			// Catch-case if anything goes wrong.
			let refund_and_deposit = vec![
				RefundSurplus,
				DepositAsset {
					assets: Wild(All),
					beneficiary: destination
						.pushed_with_interior(*origin_location.last().unwrap())
						.unwrap(),
				},
			];
			let operation = [
				vec![
					WithdrawAsset(asset.clone().into()),
					// Refund all and deposit back to owner if anything goes wrong.
					SetErrorHandler(refund_and_deposit.clone().into()),
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
				refund_and_deposit,
			]
			.concat();
			let dest_xcm = Xcm(operation);
			println!("DidXcmV3ViaXcmPalletDispatcher::dispatch 4");
			let res = pallet_xcm::Pallet::<T>::send_xcm(interior, destination, dest_xcm).map(|_| ());
			println!("DidXcmV3ViaXcmPalletDispatcher::dispatch 5");
			res
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
