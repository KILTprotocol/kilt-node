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

use dip_support::IdentityProofAction;
use pallet_dip_provider::traits::{TxBuilder, XcmRouterDispatcher};
use parity_scale_codec::{Decode, Encode};
use runtime_common::dip::{
	did::{LinkedDidInfoOf, LinkedDidInfoProviderOf},
	merkle::DidMerkleRootGenerator,
};
use xcm::{latest::MultiLocation, DoubleEncoded};

use crate::{DidIdentifier, Hash, Runtime, RuntimeEvent, XcmRouter};

#[derive(Encode, Decode)]
enum ConsumerParachainCalls {
	#[codec(index = 50)]
	DipConsumer(ConsumerParachainDipConsumerCalls),
}

#[derive(Encode, Decode)]
enum ConsumerParachainDipConsumerCalls {
	#[codec(index = 0)]
	ProcessIdentityAction(IdentityProofAction<DidIdentifier, Hash>),
}

pub struct ConsumerParachainTxBuilder;
impl TxBuilder<DidIdentifier, Hash> for ConsumerParachainTxBuilder {
	type Error = ();

	fn build(
		_dest: MultiLocation,
		action: IdentityProofAction<DidIdentifier, Hash>,
	) -> Result<DoubleEncoded<()>, Self::Error> {
		let double_encoded: DoubleEncoded<()> =
			ConsumerParachainCalls::DipConsumer(ConsumerParachainDipConsumerCalls::ProcessIdentityAction(action))
				.encode()
				.into();
		Ok(double_encoded)
	}
}

impl pallet_dip_provider::Config for Runtime {
	type Identifier = DidIdentifier;
	type Identity = LinkedDidInfoOf<Runtime>;
	type IdentityProofDispatcher = XcmRouterDispatcher<XcmRouter, DidIdentifier, Hash>;
	type IdentityProofGenerator = DidMerkleRootGenerator<Runtime>;
	type IdentityProvider = LinkedDidInfoProviderOf<Runtime>;
	type ProofOutput = Hash;
	type RuntimeEvent = RuntimeEvent;
	type TxBuilder = ConsumerParachainTxBuilder;
}
