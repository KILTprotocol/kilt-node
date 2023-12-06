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

	use crate::{did::TimeBoundDidSignature, merkle::DidMerkleProof, BoundedBlindedValue};

	#[derive(Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo, Clone)]
	pub struct ParachainRootStateProof<RelayBlockHeight> {
		/// The relaychain block height for which the proof has been generated.
		pub(crate) relay_block_height: RelayBlockHeight,
		/// The raw state proof.
		pub(crate) proof: BoundedBlindedValue<u8>,
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<RelayBlockHeight, Context> kilt_support::traits::GetWorstCase<Context>
		for ParachainRootStateProof<RelayBlockHeight>
	where
		RelayBlockHeight: Default,
	{
		fn worst_case(context: Context) -> Self {
			Self {
				relay_block_height: RelayBlockHeight::default(),
				proof: BoundedBlindedValue::worst_case(context),
			}
		}
	}

	#[derive(Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, Clone)]
	pub struct DipMerkleProofAndDidSignature<BlindedValues, Leaf, BlockNumber> {
		/// The DIP Merkle proof revealing some leaves about the DID subject's
		/// identity.
		pub(crate) leaves: DidMerkleProof<BlindedValues, Leaf>,
		/// The cross-chain DID signature.
		pub(crate) signature: TimeBoundDidSignature<BlockNumber>,
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<BlindedValues, Leaf, BlockNumber, Context> kilt_support::traits::GetWorstCase<Context>
		for DipMerkleProofAndDidSignature<BlindedValues, Leaf, BlockNumber>
	where
		BlindedValues: kilt_support::traits::GetWorstCase<Context>,
		Leaf: Default + Clone,
		BlockNumber: Default,
		Context: Clone,
	{
		fn worst_case(context: Context) -> Self {
			Self {
				leaves: DidMerkleProof::worst_case(context.clone()),
				signature: TimeBoundDidSignature::worst_case(context),
			}
		}
	}
}
