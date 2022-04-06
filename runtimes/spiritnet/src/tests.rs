// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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

use codec::MaxEncodedLen;
use frame_support::{traits::Currency, BoundedVec};

use did::DeriveDidCallAuthorizationVerificationType;
use pallet_treasury::BalanceOf;
use pallet_web3_names::{Web3NameOf, Web3OwnershipOf};
use runtime_common::constants::{
	attestation::MAX_ATTESTATION_BYTE_LENGTH, did::MAX_DID_BYTE_LENGTH, did_lookup::MAX_CONNECTION_BYTE_LENGTH,
	web3_names::MAX_NAME_BYTE_LENGTH, MAX_INDICES_BYTE_LENGTH,
};

#[cfg(test)]
use runtime_common::{AccountId, BlockNumber};

use super::{Call, Runtime};

#[test]
fn call_size() {
	assert!(
		core::mem::size_of::<Call>() <= 240,
		"size of Call is more than 240 bytes: some calls have too big arguments, use Box to reduce \
		the size of Call.
		If the limit is too strong, maybe consider increase the limit to 300.",
	);
}

#[test]
fn attestation_storage_sizes() {
	type DelegationRecord =
		BoundedVec<<Runtime as frame_system::Config>::Hash, <Runtime as attestation::Config>::MaxDelegatedAttestations>;

	let attestation_record = attestation::AttestationDetails::<Runtime>::max_encoded_len();
	let delegation_record = DelegationRecord::max_encoded_len()
		/ (<Runtime as attestation::Config>::MaxDelegatedAttestations::get() as usize);
	assert_eq!(
		attestation_record + delegation_record,
		MAX_ATTESTATION_BYTE_LENGTH as usize
	)
}

#[test]
fn did_storage_sizes() {
	let did_size = did::did_details::DidDetails::<Runtime>::max_encoded_len();

	// service endpoints and counter
	let did_endpoint_size = did::service_endpoints::DidEndpoint::<Runtime>::max_encoded_len()
		* (<Runtime as did::Config>::MaxNumberOfServicesPerDid::get() as usize)
		+ u32::max_encoded_len();

	assert_eq!(did_size + did_endpoint_size, MAX_DID_BYTE_LENGTH as usize)
}

#[test]
fn did_lookup_storage_sizes() {
	type DidConnection =
		pallet_did_lookup::ConnectionRecord<
			<Runtime as pallet_did_lookup::Config>::DidIdentifier,
			<Runtime as frame_system::Config>::AccountId,
			<<Runtime as pallet_did_lookup::Config>::Currency as Currency<
				<Runtime as frame_system::Config>::AccountId,
			>>::Balance,
		>;

	let did_connection_size = DidConnection::max_encoded_len();

	assert_eq!(did_connection_size, MAX_CONNECTION_BYTE_LENGTH as usize)
}

#[test]
fn web3_name_storage_sizes() {
	let owner_size = Web3NameOf::<Runtime>::max_encoded_len();
	let name_size = Web3OwnershipOf::<Runtime>::max_encoded_len();

	assert_eq!(owner_size + name_size, MAX_NAME_BYTE_LENGTH as usize)
}

#[test]
fn indices_storage_sizes() {
	type Indices = (<Runtime as frame_system::Config>::AccountId, BalanceOf<Runtime>, bool);

	let size = Indices::max_encoded_len();
	assert_eq!(size, MAX_INDICES_BYTE_LENGTH as usize)
}

#[test]
fn test_derive_did_verification_relation_ctype() {
	let c1 = Call::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3],
	});
	let c2 = Call::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3, 3],
	});
	let c3 = Call::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3, 3],
	});
	let c4 = Call::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 100],
	});

	let cb = Call::Utility(pallet_utility::Call::batch {
		calls: vec![c1, c2, c3, c4],
	});
	assert_eq!(
		cb.derive_verification_key_relationship(),
		Ok(did::DidVerificationKeyRelationship::AssertionMethod)
	);
}

#[test]
fn test_derive_did_key_web3name() {
	assert_eq!(
		Call::Web3Names(pallet_web3_names::Call::claim {
			name: b"test-name".to_vec().try_into().unwrap()
		})
		.derive_verification_key_relationship(),
		Ok(did::DidVerificationKeyRelationship::Authentication)
	);

	assert_eq!(
		Call::Web3Names(pallet_web3_names::Call::release_by_owner {}).derive_verification_key_relationship(),
		Ok(did::DidVerificationKeyRelationship::Authentication)
	);
}

#[test]
fn test_derive_did_key_lookup() {
	assert_eq!(
		Call::DidLookup(pallet_did_lookup::Call::associate_account {
			account: AccountId::new([1u8; 32]),
			expiration: BlockNumber::default(),
			proof: sp_runtime::MultiSignature::from(sp_core::ed25519::Signature([0; 64])),
		})
		.derive_verification_key_relationship(),
		Ok(did::DidVerificationKeyRelationship::Authentication)
	);

	assert_eq!(
		Call::DidLookup(pallet_did_lookup::Call::remove_account_association {
			account: AccountId::new([1u8; 32]),
		})
		.derive_verification_key_relationship(),
		Ok(did::DidVerificationKeyRelationship::Authentication)
	);
}

#[test]
fn test_derive_did_verification_relation_fail() {
	let c1 = Call::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3],
	});
	let c2 = Call::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3, 3],
	});
	let c3 = Call::System(frame_system::Call::remark {
		remark: vec![0, 1, 2, 3, 3],
	});
	let c4 = Call::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 100],
	});

	let cb = Call::Utility(pallet_utility::Call::batch {
		calls: vec![c1, c2, c3, c4],
	});

	#[cfg(feature = "runtime-benchmarks")]
	assert_eq!(
		cb.derive_verification_key_relationship(),
		Err(did::RelationshipDeriveError::InvalidCallParameter)
	);
	#[cfg(not(feature = "runtime-benchmarks"))]
	assert_eq!(
		cb.derive_verification_key_relationship(),
		Err(did::RelationshipDeriveError::NotCallableByDid)
	);
}

#[test]
fn test_derive_did_verification_relation_nested_fail() {
	let c1 = Call::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3],
	});
	let c2 = Call::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3, 3],
	});
	let f3 = Call::System(frame_system::Call::remark {
		remark: vec![0, 1, 2, 3, 3],
	});
	let c4 = Call::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 100],
	});

	let cb = Call::Utility(pallet_utility::Call::batch {
		calls: vec![c1.clone(), c2.clone(), c4.clone()],
	});

	let cb = Call::Utility(pallet_utility::Call::batch {
		calls: vec![c1, c2, cb, f3, c4],
	});

	#[cfg(feature = "runtime-benchmarks")]
	assert_eq!(
		cb.derive_verification_key_relationship(),
		Err(did::RelationshipDeriveError::InvalidCallParameter)
	);
	#[cfg(not(feature = "runtime-benchmarks"))]
	assert_eq!(
		cb.derive_verification_key_relationship(),
		Err(did::RelationshipDeriveError::NotCallableByDid)
	);
}

#[test]
fn test_derive_did_verification_relation_nested() {
	let c1 = Call::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3],
	});
	let c2 = Call::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3, 3],
	});
	let c4 = Call::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 100],
	});

	let cb = Call::Utility(pallet_utility::Call::batch {
		calls: vec![c1.clone(), c2.clone(), c4.clone()],
	});

	let cb = Call::Utility(pallet_utility::Call::batch {
		calls: vec![c1, c2, cb, c4],
	});
	assert_eq!(
		cb.derive_verification_key_relationship(),
		Ok(did::DidVerificationKeyRelationship::AssertionMethod)
	);
}

#[test]
fn test_derive_did_verification_relation_single() {
	let c1 = Call::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3],
	});

	let cb = Call::Utility(pallet_utility::Call::batch { calls: vec![c1] });

	assert_eq!(
		cb.derive_verification_key_relationship(),
		Ok(did::DidVerificationKeyRelationship::AssertionMethod)
	);
}

#[test]
fn test_derive_did_verification_relation_empty() {
	let cb = Call::Utility(pallet_utility::Call::batch { calls: vec![] });

	assert_eq!(
		cb.derive_verification_key_relationship(),
		Err(did::RelationshipDeriveError::InvalidCallParameter)
	);
}
