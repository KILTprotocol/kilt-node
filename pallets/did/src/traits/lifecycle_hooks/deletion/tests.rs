// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

//! Test module for the `RequireBoth` type. It verifies that the type works as
//! expected in case of failure of one of its components.

use sp_runtime::AccountId32;
use sp_weights::Weight;

use crate::{
	traits::lifecycle_hooks::{deletion::RequireBoth, mock::TestRuntime, DidDeletionHook},
	DidIdentifierOf,
};

struct AlwaysDeny;

impl DidDeletionHook<TestRuntime> for AlwaysDeny {
	const MAX_WEIGHT: Weight = Weight::from_all(10);

	fn can_delete(_did: &DidIdentifierOf<TestRuntime>) -> Result<(), Weight> {
		Err(Weight::from_all(5))
	}
}

struct AlwaysAllow;

impl DidDeletionHook<TestRuntime> for AlwaysAllow {
	const MAX_WEIGHT: Weight = Weight::from_all(20);

	fn can_delete(_did: &DidIdentifierOf<TestRuntime>) -> Result<(), Weight> {
		Ok(())
	}
}

#[test]
fn first_false() {
	type TestSubject = RequireBoth<AlwaysDeny, AlwaysAllow>;

	// Max weight is the sum.
	assert_eq!(TestSubject::MAX_WEIGHT, Weight::from_all(30));
	// Failure consumes `False`'s weight.
	assert_eq!(
		TestSubject::can_delete(&AccountId32::new([0u8; 32])),
		Err(Weight::from_all(5))
	);
}

#[test]
fn second_false() {
	type TestSubject = RequireBoth<AlwaysAllow, AlwaysDeny>;

	// Max weight is the sum.
	assert_eq!(TestSubject::MAX_WEIGHT, Weight::from_all(30));
	// Failure consumes the sum of `True`'s max weight and `False`'s weight.
	assert_eq!(
		TestSubject::can_delete(&AccountId32::new([0u8; 32])),
		Err(Weight::from_all(25))
	);
}

#[test]
fn both_true() {
	type TestSubject = RequireBoth<AlwaysAllow, AlwaysAllow>;

	// Max weight is the sum.
	assert_eq!(TestSubject::MAX_WEIGHT, Weight::from_all(40));
	// Overall result is `Ok`.
	assert_eq!(TestSubject::can_delete(&AccountId32::new([0u8; 32])), Ok(()));
}
