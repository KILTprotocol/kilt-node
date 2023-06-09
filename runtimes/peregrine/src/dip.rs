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

use did::{DidRawOrigin, EnsureDidOrigin};
use dip_support::IdentityDetailsAction;
use kilt_dip_support::xcm::XcmRouterIdentityDispatcher;
use pallet_dip_provider::traits::TxBuilder;
use parity_scale_codec::{Decode, Encode};
use runtime_common::dip::{did::LinkedDidInfoProviderOf, merkle::DidMerkleRootGenerator};
use xcm::{v3::MultiLocation, DoubleEncoded};

use crate::{
	xcm_config::{UniversalLocation, XcmRouter},
	AccountId, DidIdentifier, Hash, Runtime, RuntimeEvent,
};

#[derive(Encode, Decode)]
enum ConsumerParachainCalls {
	#[codec(index = 50)]
	DipConsumer(ConsumerParachainDipConsumerCalls),
}

#[derive(Encode, Decode)]
enum ConsumerParachainDipConsumerCalls {
	#[codec(index = 0)]
	ProcessIdentityAction(IdentityDetailsAction<DidIdentifier, Hash>),
}

pub struct ConsumerParachainTxBuilder;
impl TxBuilder<DidIdentifier, Hash> for ConsumerParachainTxBuilder {
	type Error = ();

	fn build(
		_dest: MultiLocation,
		action: IdentityDetailsAction<DidIdentifier, Hash>,
	) -> Result<DoubleEncoded<()>, Self::Error> {
		let double_encoded: DoubleEncoded<()> =
			ConsumerParachainCalls::DipConsumer(ConsumerParachainDipConsumerCalls::ProcessIdentityAction(action))
				.encode()
				.into();
		Ok(double_encoded)
	}
}

impl pallet_dip_provider::Config for Runtime {
	type CommitOriginCheck = EnsureDidOrigin<DidIdentifier, AccountId>;
	type CommitOrigin = DidRawOrigin<DidIdentifier, AccountId>;
	type Identifier = DidIdentifier;
	type IdentityProofDispatcher = XcmRouterIdentityDispatcher<XcmRouter, UniversalLocation>;
	type IdentityProofGenerator = DidMerkleRootGenerator<Runtime>;
	type IdentityProvider = LinkedDidInfoProviderOf<Runtime>;
	type ProofOutput = Hash;
	type RuntimeEvent = RuntimeEvent;
	type TxBuilder = ConsumerParachainTxBuilder;
}
