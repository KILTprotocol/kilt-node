// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

use crate::*;
use frame_support::{dispatch::Weight, weights::constants::WEIGHT_PER_SECOND};
use sp_runtime::Perbill;
use tests::*;

/// We assume that ~10% of the block weight is consumed by `on_initalize`
/// handlers. This is used to limit the maximal weight of a single extrinsic.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be
/// used by  Operational  extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}

// A test DID operation which can be crated to require any dUD verification key
// type.
#[derive(Clone, Decode, Debug, Encode, PartialEq)]
pub struct TestDIDOperation<DIDIdentifier: Parameter + Encode + Decode + Debug> {
	pub did: DIDIdentifier,
	pub verification_key_type: DIDVerificationKeyType,
}

impl<DIDIdentifier> DIDOperation<DIDIdentifier> for TestDIDOperation<DIDIdentifier>
where
	DIDIdentifier: Parameter + Encode + Decode + Debug,
{
	fn get_verification_key_type(&self) -> DIDVerificationKeyType {
		self.verification_key_type.clone()
	}

	fn get_did(&self) -> &DIDIdentifier {
		&self.did
	}
}
