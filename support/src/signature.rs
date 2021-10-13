// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

use frame_support::dispatch::Weight;

#[cfg(any(test, feature = "mock", feature = "runtime-benchmarks"))]
use {sp_runtime::traits::Zero, sp_std::marker::PhantomData};

/// The result that the delegation pallet expects from the implementer of the
/// delegate's signature verification operation.
pub type SignatureVerificationResult = Result<(), SignatureVerificationError>;

/// Types of errors the signature verification is expected to generate.
#[derive(Debug, Clone, Copy)]
pub enum SignatureVerificationError {
	/// The delegate's information is not present on chain.
	SignerInformationNotPresent,
	/// The signature over the delegation information is invalid.
	SignatureInvalid,
}

/// Trait to implement to provide to the delegation pallet signature
/// verification over a delegation details.
pub trait VerifyDelegateSignature {
	/// The type of the delegate identifier.
	type DelegateId;
	/// The type of the encoded delegation details.
	type Payload;
	/// The type of the signature generated.
	type Signature;

	/// Verifies that the signature matches the payload and has been generated
	/// by the delegate.
	fn verify(
		delegate: &Self::DelegateId,
		payload: &Self::Payload,
		signature: &Self::Signature,
	) -> SignatureVerificationResult;

	fn weight(payload_byte_length: u32) -> Weight;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct AlwaysVerify<A, P, S>(PhantomData<(A, P, S)>);
#[cfg(feature = "runtime-benchmarks")]
impl<Account, Payload, Signature: Default> VerifyDelegateSignature for AlwaysVerify<Account, Payload, Signature> {
	type DelegateId = Account;

	type Payload = Payload;

	type Signature = Signature;

	fn verify(
		_delegate: &Self::DelegateId,
		_payload: &Self::Payload,
		_signature: &Self::Signature,
	) -> SignatureVerificationResult {
		SignatureVerificationResult::Ok(())
	}

	fn weight(_: u32) -> Weight {
		Weight::zero()
	}
}

#[cfg(any(test, feature = "mock", feature = "runtime-benchmarks"))]
pub struct EqualVerify<A, B>(PhantomData<(A, B)>);
#[cfg(any(test, feature = "mock", feature = "runtime-benchmarks"))]
impl<Account, Payload> VerifyDelegateSignature for EqualVerify<Account, Payload>
where
	Account: PartialEq + Clone,
	Payload: PartialEq + Clone,
{
	type DelegateId = Account;

	type Payload = Payload;

	type Signature = (Account, Payload);

	fn verify(
		delegate: &Self::DelegateId,
		payload: &Self::Payload,
		signature: &Self::Signature,
	) -> SignatureVerificationResult {
		if (delegate, payload) == (&signature.0, &signature.1) {
			SignatureVerificationResult::Ok(())
		} else {
			SignatureVerificationResult::Err(SignatureVerificationError::SignatureInvalid)
		}
	}

	fn weight(_: u32) -> Weight {
		Weight::zero()
	}
}
