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

use crate::{mock::Test, types::StakeOf};
use frame_support::parameter_types;
use sp_runtime::RuntimeDebug;

use super::*;

parameter_types! {
	#[derive(Eq, PartialEq, RuntimeDebug)]
	pub const Zero: u32 = 0;
	#[derive(Eq, PartialEq, RuntimeDebug)]
	pub const One: u32 = 1;
	#[derive(Eq, PartialEq, RuntimeDebug)]
	pub const Eight: u32 = 8;
	#[derive(Clone, Eq, PartialEq, RuntimeDebug)]
	pub const Five: u32 = 5;
}

#[test]
fn from() {
	let v: BoundedVec<i32, Eight> = vec![4, 2, 3, 4, 3, 1].try_into().unwrap();
	let set: OrderedSet<i32, Eight> = v.into();
	assert_eq!(
		set,
		OrderedSet::<i32, Eight>::from(vec![1, 2, 3, 4].try_into().unwrap())
	);
}

#[test]
fn insert() {
	let mut set: OrderedSet<i32, Eight> = OrderedSet::new();
	assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![].try_into().unwrap()));

	assert_eq!(set.try_insert(1), Ok(true));
	assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![1].try_into().unwrap()));

	assert_eq!(set.try_insert(5), Ok(true));
	assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![1, 5].try_into().unwrap()));

	assert_eq!(set.try_insert(3), Ok(true));
	assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![1, 3, 5].try_into().unwrap()));

	assert_eq!(set.try_insert(3), Ok(false));
	assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![1, 3, 5].try_into().unwrap()));
}

#[test]
fn remove() {
	let mut set: OrderedSet<i32, Eight> = OrderedSet::from(vec![1, 2, 3, 4].try_into().unwrap());

	assert_eq!(set.remove(&5), None);
	assert_eq!(
		set,
		OrderedSet::<i32, Eight>::from(vec![1, 2, 3, 4].try_into().unwrap())
	);

	assert_eq!(set.remove(&1), Some(1));
	assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![2, 3, 4].try_into().unwrap()));

	assert_eq!(set.remove(&3), Some(3));
	assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![2, 4].try_into().unwrap()));

	assert_eq!(set.remove(&3), None);
	assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![2, 4].try_into().unwrap()));

	assert_eq!(set.remove(&4), Some(4));
	assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![2].try_into().unwrap()));

	assert_eq!(set.remove(&2), Some(2));
	assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![].try_into().unwrap()));

	assert_eq!(set.remove(&2), None);
	assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![].try_into().unwrap()));
}

#[test]
fn contains() {
	let set: OrderedSet<i32, Eight> = OrderedSet::from(vec![1, 2, 3, 4].try_into().unwrap());
	assert!(!set.contains(&5));
	assert!(set.contains(&1));
	assert!(set.contains(&3));
}

#[test]
fn clear() {
	let mut set: OrderedSet<i32, Eight> = OrderedSet::from(vec![1, 2, 3, 4].try_into().unwrap());
	set.clear();
	assert_eq!(set, OrderedSet::new());
}

#[test]
fn try_insert_replace_integer() {
	let mut set: OrderedSet<i32, Zero> = OrderedSet::from(vec![].try_into().unwrap());
	assert_eq!(set.try_insert_replace(10), Err(true));

	let mut set: OrderedSet<i32, One> = OrderedSet::from(vec![].try_into().unwrap());
	assert_eq!(set.try_insert_replace(10), Ok(None));
	assert_eq!(set.try_insert_replace(9), Err(true));
	assert_eq!(set.try_insert_replace(11), Ok(Some(10)));

	let mut set: OrderedSet<i32, Five> = OrderedSet::from(vec![].try_into().unwrap());
	assert_eq!(set.try_insert_replace(10), Ok(None));
	assert_eq!(set.try_insert_replace(7), Ok(None));
	assert_eq!(set.try_insert_replace(9), Ok(None));
	assert_eq!(set.try_insert_replace(8), Ok(None));

	assert_eq!(set.clone().into_bounded_vec().into_inner(), vec![10, 9, 8, 7]);
	assert_eq!(set.try_insert_replace(5), Ok(None));
	assert!(set.try_insert(11).is_err());

	assert_eq!(set.try_insert_replace(6), Ok(Some(5)));
	assert_eq!(set.clone().into_bounded_vec().into_inner(), vec![10, 9, 8, 7, 6]);

	assert_eq!(set.try_insert_replace(6), Err(false));
	assert_eq!(set.try_insert_replace(5), Err(true));

	assert_eq!(set.try_insert_replace(10), Err(false));
	assert_eq!(set.try_insert_replace(11), Ok(Some(6)));
	assert_eq!(set.into_bounded_vec().into_inner(), vec![11, 10, 9, 8, 7]);
}

#[test]
fn try_insert_replace_stake() {
	let mut set: OrderedSet<StakeOf<Test>, Eight> = OrderedSet::from(
		vec![
			StakeOf::<Test> { owner: 1, amount: 100 },
			StakeOf::<Test> { owner: 3, amount: 90 },
			StakeOf::<Test> { owner: 5, amount: 80 },
			StakeOf::<Test> { owner: 7, amount: 70 },
			StakeOf::<Test> { owner: 8, amount: 70 },
			StakeOf::<Test> { owner: 9, amount: 60 },
		]
		.try_into()
		.unwrap(),
	);
	assert_eq!(
		set.try_insert_replace(StakeOf::<Test> { owner: 1, amount: 0 }),
		Err(false)
	);
	assert_eq!(
		set.try_insert_replace(StakeOf::<Test> { owner: 7, amount: 100 }),
		Err(false)
	);
	assert_eq!(
		set.try_insert_replace(StakeOf::<Test> { owner: 7, amount: 50 }),
		Err(false)
	);
	assert_eq!(
		set.try_insert_replace(StakeOf::<Test> { owner: 8, amount: 50 }),
		Err(false)
	);
	assert_eq!(
		set.try_insert_replace(StakeOf::<Test> { owner: 2, amount: 100 }),
		Ok(None)
	);
	assert_eq!(
		set.try_insert_replace(StakeOf::<Test> { owner: 2, amount: 90 }),
		Err(false)
	);
	assert_eq!(
		set.try_insert_replace(StakeOf::<Test> { owner: 10, amount: 65 }),
		Ok(None)
	);
	assert_eq!(
		set.try_insert_replace(StakeOf::<Test> { owner: 11, amount: 60 }),
		Err(true)
	);
	assert_eq!(
		set.try_insert_replace(StakeOf::<Test> { owner: 11, amount: 100 }),
		Ok(Some(StakeOf::<Test> { owner: 9, amount: 60 }))
	);
}

#[test]
fn exceeding_max_size_should_fail() {
	let mut set: OrderedSet<i32, Five> = OrderedSet::from(vec![1, 2, 3, 4, 5].try_into().unwrap());
	let inserted = set.try_insert(6);

	assert!(inserted.is_err());
}

#[test]
fn linear_search() {
	let set: OrderedSet<StakeOf<Test>, Eight> = OrderedSet::from(
		vec![
			StakeOf::<Test> { owner: 1, amount: 100 },
			StakeOf::<Test> { owner: 3, amount: 90 },
			StakeOf::<Test> { owner: 5, amount: 80 },
			StakeOf::<Test> { owner: 7, amount: 70 },
			StakeOf::<Test> { owner: 8, amount: 70 },
			StakeOf::<Test> { owner: 9, amount: 60 },
		]
		.try_into()
		.unwrap(),
	);
	assert_eq!(set.linear_search(&StakeOf::<Test> { owner: 1, amount: 0 }), Ok(0));
	assert_eq!(set.linear_search(&StakeOf::<Test> { owner: 7, amount: 100 }), Ok(3));
	assert_eq!(set.linear_search(&StakeOf::<Test> { owner: 7, amount: 50 }), Ok(3));
	assert_eq!(set.linear_search(&StakeOf::<Test> { owner: 8, amount: 50 }), Ok(4));
	assert_eq!(set.linear_search(&StakeOf::<Test> { owner: 2, amount: 100 }), Err(1));
	assert_eq!(set.linear_search(&StakeOf::<Test> { owner: 2, amount: 90 }), Err(2));
	assert_eq!(set.linear_search(&StakeOf::<Test> { owner: 2, amount: 65 }), Err(5));
	assert_eq!(set.linear_search(&StakeOf::<Test> { owner: 2, amount: 60 }), Err(6));
	assert_eq!(set.linear_search(&StakeOf::<Test> { owner: 2, amount: 59 }), Err(6));
}

#[test]
fn upsert_set() {
	let mut set: OrderedSet<StakeOf<Test>, Eight> = OrderedSet::from(
		vec![
			StakeOf::<Test> { owner: 1, amount: 100 },
			StakeOf::<Test> { owner: 3, amount: 90 },
			StakeOf::<Test> { owner: 5, amount: 80 },
			StakeOf::<Test> { owner: 7, amount: 70 },
			StakeOf::<Test> { owner: 9, amount: 60 },
		]
		.try_into()
		.unwrap(),
	);
	assert_eq!(set.try_insert(StakeOf::<Test> { owner: 2, amount: 75 }), Ok(true));
	assert_eq!(
		set,
		OrderedSet::from(
			vec![
				StakeOf::<Test> { owner: 1, amount: 100 },
				StakeOf::<Test> { owner: 3, amount: 90 },
				StakeOf::<Test> { owner: 5, amount: 80 },
				StakeOf::<Test> { owner: 2, amount: 75 },
				StakeOf::<Test> { owner: 7, amount: 70 },
				StakeOf::<Test> { owner: 9, amount: 60 },
			]
			.try_into()
			.unwrap()
		)
	);
	assert_eq!(
		set.try_upsert(StakeOf::<Test> { owner: 2, amount: 90 }),
		Ok(Some(StakeOf::<Test> { owner: 2, amount: 75 }))
	);
	assert_eq!(
		set,
		OrderedSet::from(
			vec![
				StakeOf::<Test> { owner: 1, amount: 100 },
				StakeOf::<Test> { owner: 3, amount: 90 },
				StakeOf::<Test> { owner: 2, amount: 90 },
				StakeOf::<Test> { owner: 5, amount: 80 },
				StakeOf::<Test> { owner: 7, amount: 70 },
				StakeOf::<Test> { owner: 9, amount: 60 },
			]
			.try_into()
			.unwrap()
		)
	);
	assert_eq!(
		set.try_upsert(StakeOf::<Test> { owner: 2, amount: 60 }),
		Ok(Some(StakeOf::<Test> { owner: 2, amount: 90 }))
	);
	assert_eq!(
		set,
		OrderedSet::from(
			vec![
				StakeOf::<Test> { owner: 1, amount: 100 },
				StakeOf::<Test> { owner: 3, amount: 90 },
				StakeOf::<Test> { owner: 5, amount: 80 },
				StakeOf::<Test> { owner: 7, amount: 70 },
				StakeOf::<Test> { owner: 2, amount: 60 },
				StakeOf::<Test> { owner: 9, amount: 60 },
			]
			.try_into()
			.unwrap()
		)
	);
}
