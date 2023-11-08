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

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::fmt::Debug;

use crate::{Config, RuntimeCallOf};

pub trait IdentityProofVerifier<Runtime>
where
	Runtime: Config,
{
	type Error: Into<u16>;
	type Proof: TypeInfo + Encode + Decode + Clone + Debug + PartialEq;
	type VerificationResult;

	fn verify_proof_for_call_against_details(
		call: &RuntimeCallOf<Runtime>,
		subject: &Runtime::Identifier,
		submitter: &Runtime::AccountId,
		identity_details: &mut Option<Runtime::LocalIdentityInfo>,
		proof: Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error>;
}

// Always returns success.
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
