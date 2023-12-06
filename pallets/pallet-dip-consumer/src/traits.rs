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

use frame_support::Parameter;

use crate::{Config, RuntimeCallOf};

/// A trait to verify a given DIP identity proof. The trait depends on the
/// runtime definition of the consumer pallet's `Identifier` and of the system
/// pallet's `AccountId`. The type of proof expected and the type returned upon
/// successful verification is defined as an associated type.
pub trait IdentityProofVerifier<Runtime>
where
	Runtime: Config,
{
	/// The error returned upon failed DIP proof verification.
	type Error: Into<u16>;
	/// The accepted type for a DIP identity proof.
	type Proof: Parameter;
	/// The type returned upon successful DIP proof verification.
	type VerificationResult;

	/// Verify a given DIP proof given the calling context, including the call
	/// being dispatched, the DIP subject dispatching it, the account submitting
	/// the DIP tx, and the identity details of the DIP subject as stored in the
	/// consumer pallet.
	fn verify_proof_for_call_against_details(
		call: &RuntimeCallOf<Runtime>,
		subject: &Runtime::Identifier,
		submitter: &Runtime::AccountId,
		identity_details: &mut Option<Runtime::LocalIdentityInfo>,
		proof: Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error>;
}

/// Dummy implementation of the [`IdentityProofVerifier`] trait which always
/// returns `Ok(())`.
pub struct SuccessfulProofVerifier;
impl<Runtime> IdentityProofVerifier<Runtime> for SuccessfulProofVerifier
where
	Runtime: Config,
{
	type Error = u16;
	type Proof = ();
	type VerificationResult = ();

	fn verify_proof_for_call_against_details(
		_call: &RuntimeCallOf<Runtime>,
		_subject: &Runtime::Identifier,
		_submitter: &Runtime::AccountId,
		_identity_details: &mut Option<Runtime::LocalIdentityInfo>,
		_proof: Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		Ok(())
	}
}
