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

use sp_runtime::AccountId32;
use sp_weights::Weight;

use crate::{
	traits::lifecycle_hooks::{deletion::EvaluateAll, mock::TestRuntime, DidDeletionHook},
	DidIdentifierOf,
};

struct False;

impl DidDeletionHook<TestRuntime> for False {
	const MAX_WEIGHT: Weight = Weight::from_all(10);

	fn can_delete(_did: &DidIdentifierOf<TestRuntime>) -> Result<(), Weight> {
		Err(Weight::from_all(5))
	}
}

struct True;

impl DidDeletionHook<TestRuntime> for True {
	const MAX_WEIGHT: Weight = Weight::from_all(20);

	fn can_delete(did: &DidIdentifierOf<TestRuntime>) -> Result<(), Weight> {
		Ok(())
	}
}

#[test]
fn first_false() {
	type TestSubject = EvaluateAll<False, True>;

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
	type TestSubject = EvaluateAll<True, False>;

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
	type TestSubject = EvaluateAll<True, True>;

	// Max weight is the sum.
	assert_eq!(TestSubject::MAX_WEIGHT, Weight::from_all(40));
	assert_eq!(TestSubject::can_delete(&AccountId32::new([0u8; 32])), Ok(()));
}
