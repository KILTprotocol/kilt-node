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

//! Module to deal with cross-chain KILT DIDs.

use did::DidSignature;
use frame_support::ensure;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;
use sp_runtime::traits::CheckedSub;

/// A DID signature anchored to a specific block height.
#[derive(Encode, Decode, RuntimeDebug, Clone, Eq, PartialEq, TypeInfo)]
pub struct TimeBoundDidSignature<BlockNumber> {
	/// The signature.
	pub signature: DidSignature,
	/// The block number, in the context of the local executor, to which the
	/// signature is anchored.
	pub block_number: BlockNumber,
}

impl<BlockNumber> TimeBoundDidSignature<BlockNumber>
where
	BlockNumber: CheckedSub + Ord,
{
	/// Verifies the time bounds of the DID signatures and returns the signature
	/// information after stripping time-related information.
	pub(crate) fn verify_time_bounds(
		self,
		current_block_number: BlockNumber,
		max_offset: BlockNumber,
	) -> Result<DidSignature, DidSignatureVerificationError> {
		let signature_block_number = self.block_number;

		let is_signature_fresh =
			if let Some(blocks_ago_from_now) = current_block_number.checked_sub(&signature_block_number) {
				// False if the signature is too old.
				blocks_ago_from_now <= max_offset
			} else {
				// Signature generated for a future time, not possible to verify.
				false
			};
		ensure!(is_signature_fresh, DidSignatureVerificationError::SignatureNotFresh);

		Ok(self.signature)
	}
}

#[cfg(feature = "runtime-benchmarks")]
impl<BlockNumber, Context> kilt_support::traits::GetWorstCase<Context> for TimeBoundDidSignature<BlockNumber>
where
	DidSignature: kilt_support::traits::GetWorstCase<Context>,
	BlockNumber: Default,
{
	fn worst_case(context: Context) -> Self {
		Self {
			signature: DidSignature::worst_case(context),
			block_number: BlockNumber::default(),
		}
	}
}

pub enum DidSignatureVerificationError {
	SignatureNotFresh,
	SignatureUnverifiable,
	OriginCheckFailed,
	Internal,
}

impl From<DidSignatureVerificationError> for u8 {
	fn from(value: DidSignatureVerificationError) -> Self {
		match value {
			DidSignatureVerificationError::SignatureNotFresh => 1,
			DidSignatureVerificationError::SignatureUnverifiable => 2,
			DidSignatureVerificationError::OriginCheckFailed => 3,
			DidSignatureVerificationError::Internal => u8::MAX,
		}
	}
}
