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

mod dip_revealed_details_and_unverified_did_signature {
	use frame_support::{assert_err, assert_ok};

	use crate::{DipRevealedDetailsAndUnverifiedDidSignature, Error};

	#[test]
	fn verify_signature_time_successful() {
		let signature =
			DipRevealedDetailsAndUnverifiedDidSignature::<(), (), (), (), (), _, 1>::with_signature_time(10u32);
		assert_ok!(signature.clone().verify_signature_time(&0));
		assert_ok!(signature.clone().verify_signature_time(&1));
		assert_ok!(signature.clone().verify_signature_time(&9));
		assert_ok!(signature.verify_signature_time(&10));
	}

	#[test]
	fn verify_signature_time_too_old() {
		let signature =
			DipRevealedDetailsAndUnverifiedDidSignature::<(), (), (), (), (), _, 1>::with_signature_time(10u32);
		assert_err!(
			signature.clone().verify_signature_time(&11),
			Error::InvalidSignatureTime
		);
		assert_err!(signature.verify_signature_time(&u32::MAX), Error::InvalidSignatureTime);
	}
}

mod dip_revealed_details_and_verified_did_signature_freshness {
	#[test]
	fn retrieve_signing_leaf_for_payload_successful() {
		unimplemented!()
	}

	#[test]
	fn retrieve_signing_leaf_for_payload_no_key_present() {
		unimplemented!()
	}
}
