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

use did::DeriveDidCallAuthorizationVerificationKeyRelationship;

use super::Call;

#[test]
fn call_size() {
	assert!(
		core::mem::size_of::<Call>() <= 272,
		"size of Call is {:?} bytes which is more than 240 bytes: some calls have too big arguments, use Box to reduce the size of Call.
		If the limit is too strong, maybe consider increase the limit to 300.",
		core::mem::size_of::<Call>(),
	);
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
