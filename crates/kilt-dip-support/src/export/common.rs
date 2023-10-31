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

pub mod latest {
	pub use super::v0::{DipMerkleProofAndDidSignature, ParachainRootStateProof};
}

pub mod v0 {
	use parity_scale_codec::{Decode, Encode};
	use scale_info::TypeInfo;
	use sp_core::RuntimeDebug;
	use sp_std::vec::Vec;

	use crate::{did::TimeBoundDidSignature, merkle::DidMerkleProof};

	#[derive(Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo, Clone)]
	pub struct ParachainRootStateProof<RelayBlockHeight> {
		pub(crate) relay_block_height: RelayBlockHeight,
		pub(crate) proof: Vec<Vec<u8>>,
	}

	#[derive(Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, Clone)]
	pub struct DipMerkleProofAndDidSignature<BlindedValues, Leaf, BlockNumber> {
		pub(crate) leaves: DidMerkleProof<BlindedValues, Leaf>,
		pub(crate) signature: TimeBoundDidSignature<BlockNumber>,
	}
}
