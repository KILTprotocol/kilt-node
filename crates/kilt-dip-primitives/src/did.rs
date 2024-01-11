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

/// A DID signature anchored to a specific block height.
#[derive(Encode, Decode, RuntimeDebug, Clone, Eq, PartialEq, TypeInfo)]
pub struct TimeBoundDidSignature<BlockNumber> {
	/// The signature.
	pub signature: DidSignature,
	/// The block number until the signature is to be considered valid.
	pub valid_until: BlockNumber,
}

impl<BlockNumber> TimeBoundDidSignature<BlockNumber>
where
	BlockNumber: PartialOrd,
{
	/// Verifies if the DID signature is expired and returns the signature
	/// information after stripping time-related information.
	pub fn extract_signature_if_not_expired(
		self,
		current_block_number: BlockNumber,
	) -> Result<DidSignature, DidSignatureVerificationError> {
		ensure!(
			self.valid_until >= current_block_number,
			DidSignatureVerificationError::SignatureExpired
		);

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
			valid_until: BlockNumber::default(),
		}
	}
}

pub enum DidSignatureVerificationError {
	SignatureExpired,
}
