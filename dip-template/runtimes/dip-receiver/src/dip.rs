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

use runtime_common::dip::{receiver::DidMerkleProofVerifier, LeafKey, LeafValue};
use sp_std::vec::Vec;

use crate::{BlockNumber, DidIdentifier, Hash, Hasher, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin};

impl pallet_dip_receiver::Config for Runtime {
	type BlindedValue = Vec<Vec<u8>>;
	type Identifier = DidIdentifier;
	type ProofDigest = Hash;
	type ProofLeafKey = LeafKey<Hash>;
	type ProofLeafValue = LeafValue<Hash, BlockNumber>;
	type ProofVerifier = DidMerkleProofVerifier<Hash, BlockNumber, Hasher>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
}
