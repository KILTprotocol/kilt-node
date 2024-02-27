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

mod parachain_dip_did_proof {
	#[test]
	fn verify_provider_head_proof_with_state_root_successful() {
		unimplemented!()
	}

	#[test]
	fn verify_provider_head_proof_with_state_root_wrong_relay_hasher() {
		unimplemented!()
	}

	#[test]
	fn verify_provider_head_proof_with_state_root_wrong_provider_header_type() {
		unimplemented!()
	}

	#[test]
	fn verify_provider_head_proof_with_state_root_different_storage_key() {
		// Valid proof but on a different storage key than the expected one
		unimplemented!()
	}

	#[test]
	fn verify_provider_head_proof_with_state_root_invalid_proof() {
		// Invalid proof for the given storage key
		unimplemented!()
	}
}

mod dip_did_proof_with_verified_relay_state_root {
	#[test]
	fn verify_dip_commitment_proof_for_subject_successful() {
		unimplemented!()
	}

	#[test]
	fn verify_dip_commitment_proof_for_subject_successful_wrong_provider_hasher() {
		unimplemented!()
	}

	#[test]
	fn verify_dip_commitment_proof_for_subject_successful_wrong_provider_runtime() {
		unimplemented!()
	}

	#[test]
	fn verify_dip_commitment_proof_for_subject_different_storage_key() {
		// Valid proof but on a different storage key than the expected one
		unimplemented!()
	}

	#[test]
	fn verify_dip_commitment_proof_for_subject_invalid_proof() {
		// Invalid proof for the given storage key
		unimplemented!()
	}
}

mod dip_did_proof_with_verified_subject_commitment {
	#[test]
	fn verify_dip_proof_successful() {
		unimplemented!()
	}

	#[test]
	fn verify_dip_proof_wrong_merkle_hasher() {
		unimplemented!()
	}

	#[test]
	fn verify_dip_proof_too_many_leaves() {
		unimplemented!()
	}

	#[test]
	fn verify_dip_proof_invalid_proof() {
		unimplemented!()
	}
}
