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
use pallet_dip_sender::traits::{
	DefaultIdentityProofGenerator, DefaultIdentityProvider, TxBuilder, XcmRouterDispatcher,
};
use parity_scale_codec::{Decode, Encode};
use xcm::{latest::MultiLocation, DoubleEncoded};

use crate::{DidIdentifier, Runtime, RuntimeEvent, XcmRouter};

#[derive(Encode, Decode)]
enum ReceiverParachainCalls {
	#[codec(index = 50)]
	DipReceiver(ReceiverParachainDipReceiverCalls),
}

#[derive(Encode, Decode)]
enum ReceiverParachainDipReceiverCalls {
	#[codec(index = 0)]
	ProcessIdentityAction(VersionedIdentityProofAction<DidIdentifier, [u8; 32]>),
}

pub struct ReceiverParachainTxBuilder;
impl TxBuilder<DidIdentifier, [u8; 32]> for ReceiverParachainTxBuilder {
	type Error = ();

	fn build(
		_dest: MultiLocation,
		action: VersionedIdentityProofAction<DidIdentifier, [u8; 32]>,
	) -> Result<DoubleEncoded<()>, Self::Error> {
		let double_encoded: DoubleEncoded<()> =
			ReceiverParachainCalls::DipReceiver(ReceiverParachainDipReceiverCalls::ProcessIdentityAction(action))
				.encode()
				.into();
		Ok(double_encoded)
	}
}

impl pallet_dip_sender::Config for Runtime {
	type Identifier = DidIdentifier;
	// TODO: Change with right one
	type Identity = u32;
	// TODO: Change with right one
	type IdentityProofDispatcher = XcmRouterDispatcher<XcmRouter, DidIdentifier, [u8; 32]>;
	// TODO: Change with right one
	type IdentityProofGenerator = DefaultIdentityProofGenerator;
	// TODO: Change with right one
	type IdentityProvider = DefaultIdentityProvider;
	// TODO: Change with right one
	type ProofOutput = [u8; 32];
	type RuntimeEvent = RuntimeEvent;
	type TxBuilder = ReceiverParachainTxBuilder;
}
