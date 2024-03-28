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

	use crate::{DipRevealedDetailsAndUnverifiedDidSignature, Error, TimeBoundDidSignature};

	impl<
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			ConsumerBlockNumber,
			const MAX_REVEALED_LEAVES_COUNT: u32,
		>
		DipRevealedDetailsAndUnverifiedDidSignature<
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			ConsumerBlockNumber,
			MAX_REVEALED_LEAVES_COUNT,
		> where
		KiltDidKeyId: Default,
		KiltBlockNumber: Default,
		ConsumerBlockNumber: Default,
	{
		fn with_signature_time(valid_until: ConsumerBlockNumber) -> Self {
			Self {
				signature: TimeBoundDidSignature {
					valid_until,
					..Default::default()
				},
				revealed_leaves: Default::default(),
			}
		}
	}

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
	use did::{
		did_details::{DidPublicKeyDetails, DidVerificationKey},
		DidVerificationKeyRelationship,
	};
	use frame_support::assert_err;
	use parity_scale_codec::Encode;
	use sp_core::{ed25519, ConstU32, Pair};
	use sp_runtime::{AccountId32, BoundedVec};

	use crate::{
		DipOriginInfo, DipRevealedDetailsAndVerifiedDidSignatureFreshness, Error, RevealedDidKey,
		RevealedDidMerkleProofLeaf,
	};

	#[test]
	fn retrieve_signing_leaves_for_payload_single_leaf_successful() {
		let payload = b"Hello, world!";
		let (did_key_pair, _) = ed25519::Pair::generate();
		let did_auth_key: DidVerificationKey<AccountId32> = did_key_pair.public().into();
		let revealed_leaves: BoundedVec<RevealedDidMerkleProofLeaf<u32, AccountId32, u32, (), ()>, ConstU32<1>> =
			vec![RevealedDidKey {
				id: 0u32,
				relationship: DidVerificationKeyRelationship::Authentication.into(),
				details: DidPublicKeyDetails {
					key: did_auth_key.into(),
					block_number: 0u32,
				},
			}
			.into()]
			.try_into()
			.unwrap();
		let revealed_details: DipRevealedDetailsAndVerifiedDidSignatureFreshness<_, _, _, _, _, 1> =
			DipRevealedDetailsAndVerifiedDidSignatureFreshness {
				revealed_leaves: revealed_leaves.clone(),
				signature: did_key_pair.sign(&payload.encode()).into(),
			};
		assert_eq!(
			revealed_details.retrieve_signing_leaves_for_payload(&payload.encode()),
			Ok(DipOriginInfo {
				signing_leaves_indices: vec![0].try_into().unwrap(),
				revealed_leaves,
			})
		);
	}

	#[test]
	fn retrieve_signing_leaves_for_payload_multiple_leaves_successful() {
		let payload = b"Hello, world!";
		let (did_key_pair, _) = ed25519::Pair::generate();
		let did_auth_key: DidVerificationKey<AccountId32> = did_key_pair.public().into();
		let revealed_leaves: BoundedVec<RevealedDidMerkleProofLeaf<u32, AccountId32, u32, (), ()>, ConstU32<3>> = vec![
			RevealedDidKey {
				id: 0u32,
				relationship: DidVerificationKeyRelationship::Authentication.into(),
				details: DidPublicKeyDetails {
					key: did_auth_key.clone().into(),
					block_number: 0u32,
				},
			}
			.into(),
			RevealedDidKey {
				id: 0u32,
				relationship: DidVerificationKeyRelationship::CapabilityDelegation.into(),
				details: DidPublicKeyDetails {
					// This key should be filtered out from the result, since it does not verify successfully for the
					// provided payload and signature.
					key: DidVerificationKey::from(ed25519::Public([100; 32])).into(),
					block_number: 0u32,
				},
			}
			.into(),
			RevealedDidKey {
				id: 0u32,
				relationship: DidVerificationKeyRelationship::AssertionMethod.into(),
				details: DidPublicKeyDetails {
					key: did_auth_key.into(),
					block_number: 0u32,
				},
			}
			.into(),
		]
		.try_into()
		.unwrap();
		let revealed_details: DipRevealedDetailsAndVerifiedDidSignatureFreshness<_, _, _, _, _, 3> =
			DipRevealedDetailsAndVerifiedDidSignatureFreshness {
				revealed_leaves: revealed_leaves.clone(),
				signature: did_key_pair.sign(&payload.encode()).into(),
			};
		assert_eq!(
			revealed_details.retrieve_signing_leaves_for_payload(&payload.encode()),
			Ok(DipOriginInfo {
				signing_leaves_indices: vec![0, 2].try_into().unwrap(),
				revealed_leaves,
			})
		);
	}

	#[test]
	fn retrieve_signing_leaves_for_payload_no_key_present() {
		let did_auth_key: DidVerificationKey<AccountId32> = ed25519::Public([0u8; 32]).into();
		let revealed_leaves: BoundedVec<RevealedDidMerkleProofLeaf<u32, AccountId32, u32, (), ()>, ConstU32<1>> =
			vec![RevealedDidKey {
				id: 0u32,
				relationship: DidVerificationKeyRelationship::Authentication.into(),
				details: DidPublicKeyDetails {
					key: did_auth_key.into(),
					block_number: 0u32,
				},
			}
			.into()]
			.try_into()
			.unwrap();
		let revealed_details: DipRevealedDetailsAndVerifiedDidSignatureFreshness<_, _, _, _, _, 1> =
			DipRevealedDetailsAndVerifiedDidSignatureFreshness {
				revealed_leaves,
				signature: ed25519::Signature([100u8; 64]).into(),
			};
		assert_err!(
			revealed_details.retrieve_signing_leaves_for_payload(&().encode()),
			Error::InvalidDidKeyRevealed
		);
	}
}
