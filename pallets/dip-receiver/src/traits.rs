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

use dip_support::VersionedIdentityProof;
use sp_std::marker::PhantomData;

pub trait IdentityProofVerifier {
	type ProofDigest;
	type LeafKey;
	type LeafValue;
	type VerificationResult;
	type Error;

	fn verify_proof_against_digest(
		proof: VersionedIdentityProof<Self::LeafKey, Self::LeafValue>,
		digest: Self::ProofDigest,
	) -> Result<Self::VerificationResult, Self::Error>;
}

// Always returns success.
pub struct SuccessfulProofVerifier<ProofDigest, LeafKey, LeafValue>(PhantomData<(ProofDigest, LeafKey, LeafValue)>);
impl<ProofDigest, LeafKey, LeafValue> IdentityProofVerifier
	for SuccessfulProofVerifier<ProofDigest, LeafKey, LeafValue>
{
	type ProofDigest = ProofDigest;
	type Error = ();
	type LeafKey = LeafKey;
	type LeafValue = LeafValue;
	type VerificationResult = ();

	fn verify_proof_against_digest(
		_proof: VersionedIdentityProof<Self::LeafKey, Self::LeafValue>,
		_digest: Self::ProofDigest,
	) -> Result<Self::VerificationResult, Self::Error> {
		Ok(())
	}
}
