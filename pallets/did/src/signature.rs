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

use frame_support::dispatch;
use kilt_support::signature::{SignatureVerificationError, SignatureVerificationResult, VerifySignature};
use sp_runtime::SaturatedConversion;
use sp_std::{marker::PhantomData, vec::Vec};

use crate::{
	did_details::{DidSignature, DidVerificationKeyRelationship},
	errors::DidError,
	Config, Did, Pallet, WeightInfo,
};

pub struct DidSignatureVerify<T>(PhantomData<T>);
impl<T: Config> VerifySignature for DidSignatureVerify<T> {
	type SignerId = <T as Config>::DidIdentifier;
	type Payload = Vec<u8>;
	type Signature = DidSignature;

	fn verify(
		delegate: &Self::SignerId,
		payload: &Self::Payload,
		signature: &Self::Signature,
	) -> SignatureVerificationResult {
		let delegate_details =
			Did::<T>::get(delegate).ok_or(SignatureVerificationError::SignerInformationNotPresent)?;

		Pallet::verify_payload_signature_with_did_key_type(
			payload,
			signature,
			&delegate_details,
			DidVerificationKeyRelationship::Authentication,
		)
		.map_err(|err| match err {
			// Should never happen as a DID has always a valid authentication key and UrlErrors are never thrown here.
			DidError::Signature(_) => SignatureVerificationError::SignatureInvalid,
			_ => SignatureVerificationError::SignerInformationNotPresent,
		})
	}

	fn weight(payload_byte_length: usize) -> dispatch::Weight {
		<T as Config>::WeightInfo::signature_verification_sr25519(payload_byte_length.saturated_into())
			.max(<T as Config>::WeightInfo::signature_verification_ed25519(
				payload_byte_length.saturated_into(),
			))
			.max(<T as Config>::WeightInfo::signature_verification_ecdsa(
				payload_byte_length.saturated_into(),
			))
	}
}
