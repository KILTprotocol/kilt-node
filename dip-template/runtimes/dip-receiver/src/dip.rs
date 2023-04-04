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

use pallet_dip_receiver::traits::SuccessfulProofVerifier;

use crate::{DidIdentifier, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin};

impl pallet_dip_receiver::Config for Runtime {
	type Identifier = DidIdentifier;
	// TODO: Change with right one
	type ProofDigest = [u8; 32];
	// TODO: Change with right one
	type ProofLeafKey = [u8; 4];
	// TODO: Change with right one
	type ProofLeafValue = [u8; 4];
	// TODO: Change with right one
	type ProofVerifier = SuccessfulProofVerifier<Self::ProofDigest, Self::ProofLeafKey, Self::ProofLeafValue>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
}
