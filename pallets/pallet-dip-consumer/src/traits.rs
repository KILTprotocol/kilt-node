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
	type BlindedValue;
	type Error;
	type ProofDigest;
	type ProofLeaf;
	type VerificationResult;

	fn verify_proof_against_digest(
		proof: VersionedIdentityProof<Self::BlindedValue, Self::ProofLeaf>,
		digest: Self::ProofDigest,
	) -> Result<Self::VerificationResult, Self::Error>;
}

// Always returns success.
pub struct SuccessfulProofVerifier<ProofDigest, Leaf, BlindedValue>(PhantomData<(ProofDigest, Leaf, BlindedValue)>);
impl<ProofDigest, Leaf, BlindedValue> IdentityProofVerifier
	for SuccessfulProofVerifier<ProofDigest, Leaf, BlindedValue>
{
	type BlindedValue = BlindedValue;
	type Error = ();
	type ProofDigest = ProofDigest;
	type ProofLeaf = Leaf;
	type VerificationResult = ();

	fn verify_proof_against_digest(
		_proof: VersionedIdentityProof<Self::BlindedValue, Self::ProofLeaf>,
		_digest: Self::ProofDigest,
	) -> Result<Self::VerificationResult, Self::Error> {
		Ok(())
	}
}

pub trait DipCallOriginFilter<Call> {
	type Error;
	type Proof;
	type Success;

	fn check_proof(call: Call, proof: Self::Proof) -> Result<Self::Success, Self::Error>;
}
