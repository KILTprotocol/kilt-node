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

use frame_support::{traits::Get, BoundedVec, DefaultNoBound};
use parity_scale_codec::{Decode, Encode};
use sp_std::{
	cmp::Ordering,
	convert::TryInto,
	ops::{Index, IndexMut, Range, RangeFull},
};
#[cfg(feature = "std")]
use sp_std::{fmt, prelude::*};

/// An ordered set backed by `BoundedVec`.
#[derive(PartialEq, Eq, Encode, Decode, DefaultNoBound, Clone)]
pub struct OrderedSet<T, S>(BoundedVec<T, S>);

impl<T: Ord + Clone, S: Get<u32>> OrderedSet<T, S> {
	/// Create a new empty set.
	pub fn new() -> Self {
		Self(BoundedVec::default())
	}

	/// Creates an ordered set from a `BoundedVec`.
	///
	/// The vector will be sorted reversily (from greatest to lowest) and
	/// deduped first.
	pub fn from(bv: BoundedVec<T, S>) -> Self {
		let mut v = bv.into_inner();
		v.sort_by(|a, b| b.cmp(a));
		v.dedup();
		Self::from_sorted_set(v.try_into().expect("No values were added"))
	}

	/// Create a set from a `BoundedVec`.
	///
	/// Assumes that `v` is sorted reversely (from greatest to lowest) and only
	/// contains unique elements.
	pub fn from_sorted_set(bv: BoundedVec<T, S>) -> Self {
		Self(bv)
	}

	/// Inserts an element, if no equal item exist in the set.
	///
	/// Throws if insertion would exceed the bounded vec's max size.
	///
	/// Returns true if the item is unique in the set, otherwise returns false.
	pub fn try_insert(&mut self, value: T) -> Result<bool, ()> {
		match self.linear_search(&value) {
			Ok(_) => Ok(false),
			Err(loc) => {
				self.0.try_insert(loc, value)?;
				Ok(true)
			}
		}
	}

	/// Attempts to replace the last element of the set with the provided value.
	/// Assumes the set to have reached its bounded size.
	///
	/// Throws with `false` if the value already exists in the set.
	/// Throws with `true` if the value has the least order in the set, i.e.,
	/// it would be appended (inserted at i == length).
	///
	/// Returns the replaced element upon success.
	pub fn try_insert_replace(&mut self, value: T) -> Result<T, bool> {
		let last_idx = self.len().saturating_sub(1);
		match self.linear_search(&value) {
			Ok(_) => Err(false),
			Err(i) if i < self.len() => {
				// always replace the last element
				let old = sp_std::mem::replace(&mut self.0[last_idx], value);
				self.sort_greatest_to_lowest();
				Ok(old)
			}
			_ => Err(true),
		}
	}

	/// Inserts a new element or updates the value of an existing one.
	///
	/// Throws if the maximum size of the bounded vec would be exceeded
	/// upon insertion.
	///
	/// Returns the old value if existing or None if the value did not exist
	/// before.
	pub fn try_upsert(&mut self, value: T) -> Result<Option<T>, ()> {
		match self.linear_search(&value) {
			Ok(i) => {
				let old = sp_std::mem::replace(&mut self.0[i], value);
				self.sort_greatest_to_lowest();
				Ok(Some(old))
			}
			Err(i) => {
				// Delegator
				self.0.try_insert(i, value)?;
				Ok(None)
			}
		}
	}

	/// Removes an element.
	///
	/// Returns true if removal happened.
	pub fn remove(&mut self, value: &T) -> Option<T> {
		match self.linear_search(value) {
			Ok(loc) => Some(self.0.remove(loc)),
			Err(_) => None,
		}
	}

	/// Removes an element.
	///
	/// Returns true if removal happened.
	pub fn remove_by<F>(&mut self, f: F) -> Option<T>
	where
		F: FnMut(&T) -> Ordering,
	{
		match self.0.binary_search_by(f) {
			Ok(loc) => Some(self.0.remove(loc)),
			Err(_) => None,
		}
	}

	/// Return whether the set contains `value`.
	pub fn contains(&self, value: &T) -> bool {
		self.linear_search(value).is_ok()
	}

	/// Binary searches this ordered OrderedSet for a given element.
	///
	/// 1. If the value is found, then Result::Ok is returned, containing the
	/// index of the matching element.
	/// 2. If there are multiple matches, then any one of the matches could be
	/// returned.
	/// 3. If the value is not found then Result::Err is returned, containing
	/// the index where a matching element could be inserted while maintaining
	/// sorted order.
	pub fn binary_search(&self, value: &T) -> Result<usize, usize> {
		self.0.binary_search(value)
	}

	/// Iteratively searches this (from greatest to lowest) ordered set for a
	/// given element.
	///
	/// 1. If the value is found, then Result::Ok is returned, containing the
	/// index of the matching element.
	/// 2. If the value is not found, then Result::Err is returned, containing
	/// the index where a matching element could be inserted while maintaining
	/// sorted order.
	pub fn linear_search(&self, value: &T) -> Result<usize, usize> {
		let size = self.0.len();
		let mut loc: usize = size;
		// keep running until we find a smaller item
		self.0
			.iter()
			.enumerate()
			.find_map(|(i, v)| {
				match (v.cmp(value), loc == size) {
					// prevent to have same items
					(Ordering::Equal, _) => Some(Ok(i)),
					// eventually, we want to return this index but we need to keep checking for Ordering::Equal in case
					// value is still in the set
					(Ordering::Less, true) => {
						// insert after current element
						loc = i;
						None
					}
					_ => None,
				}
			})
			.unwrap_or(Err(loc))
	}

	/// Binary searches this ordered OrderedSet for a given element with the
	/// provided comparator.
	///
	/// 1. If the value is found, then Result::Ok is returned, containing the
	/// index of the matching element.
	/// 2. If there are multiple matches, then any one of the matches could be
	/// returned.
	/// 3. If the value is not found then Result::Err is returned, containing
	/// the index where a matching element could be inserted while maintaining
	/// sorted order.
	pub fn binary_search_by<'a, F>(&'a self, f: F) -> Result<usize, usize>
	where
		F: FnMut(&'a T) -> Ordering,
	{
		self.0.binary_search_by(f)
	}

	/// Clear the set.
	pub fn clear(&mut self) {
		self.0 = BoundedVec::default();
	}

	/// Return the length of the set.
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Return whether the set is empty.
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	/// Convert the set to a bounded vector.
	pub fn into_bounded_vec(self) -> BoundedVec<T, S> {
		self.0
	}

	/// Sorts from greatest to lowest.
	pub fn sort_greatest_to_lowest(&mut self) {
		// NOTE: BoundedVec does not implement DerefMut because it would allow for
		// unchecked extension of the inner vector. Thus, we have to work with a
		// clone unfortunately.
		let mut sorted_v: sp_std::vec::Vec<T> = sp_std::mem::take(&mut self.0).into();
		sorted_v.sort_by(|a, b| b.cmp(a));
		self.0 = sorted_v.try_into().expect("Did not extend size of bounded vec");
	}
}

#[cfg(feature = "std")]
impl<T, S> fmt::Debug for OrderedSet<T, S>
where
	T: fmt::Debug,
	S: Get<u32>,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_tuple("OrderedSet").field(&self.0).finish()
	}
}

impl<T: Ord + Clone, S: Get<u32>> From<BoundedVec<T, S>> for OrderedSet<T, S> {
	fn from(bv: BoundedVec<T, S>) -> Self {
		Self::from(bv)
	}
}

impl<T: Ord + Clone, S: Get<u32>> Index<usize> for OrderedSet<T, S> {
	type Output = T;

	fn index(&self, index: usize) -> &Self::Output {
		&self.0[index]
	}
}

impl<T: Ord + Clone, S: Get<u32>> Index<Range<usize>> for OrderedSet<T, S> {
	type Output = [T];

	fn index(&self, range: Range<usize>) -> &Self::Output {
		&self.0[range]
	}
}

impl<T: Ord + Clone, S: Get<u32>> Index<RangeFull> for OrderedSet<T, S> {
	type Output = [T];

	fn index(&self, range: RangeFull) -> &Self::Output {
		&self.0[range]
	}
}

impl<T: Ord + Clone, S: Get<u32>> IndexMut<usize> for OrderedSet<T, S> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.0[index]
	}
}

impl<T: Ord + Clone, S: Get<u32>> IntoIterator for OrderedSet<T, S> {
	type Item = T;
	type IntoIter = sp_std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<T: Ord + Clone, S: Get<u32>> From<OrderedSet<T, S>> for BoundedVec<T, S> {
	fn from(s: OrderedSet<T, S>) -> Self {
		s.0
	}
}

impl<T: Ord + Clone, S: Get<u32>> IndexMut<Range<usize>> for OrderedSet<T, S> {
	fn index_mut(&mut self, range: Range<usize>) -> &mut Self::Output {
		&mut self.0[range]
	}
}

impl<T: Ord + Clone, S: Get<u32>> IndexMut<RangeFull> for OrderedSet<T, S> {
	fn index_mut(&mut self, range: RangeFull) -> &mut Self::Output {
		&mut self.0[range]
	}
}

#[cfg(test)]
mod tests {
	use crate::{mock::Test, types::StakeOf};
	use frame_support::parameter_types;
	use sp_runtime::RuntimeDebug;

	use super::*;

	parameter_types! {
		#[derive(PartialEq, RuntimeDebug)]
		pub const Eight: u32 = 8;
		#[derive(PartialEq, RuntimeDebug, Clone)]
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
	fn try_insert_replace() {
		let mut set: OrderedSet<i32, Five> = OrderedSet::from(vec![10, 9, 8, 7, 5].try_into().unwrap());
		assert_eq!(set.clone().into_bounded_vec().into_inner(), vec![10, 9, 8, 7, 5]);
		assert!(set.try_insert(11).is_err());

		assert_eq!(set.try_insert_replace(6), Ok(5));
		assert_eq!(set.clone().into_bounded_vec().into_inner(), vec![10, 9, 8, 7, 6]);

		assert_eq!(set.try_insert_replace(6), Err(false));
		assert_eq!(set.try_insert_replace(5), Err(true));

		assert_eq!(set.try_insert_replace(10), Err(false));
		assert_eq!(set.try_insert_replace(11), Ok(6));
		assert_eq!(set.into_bounded_vec().into_inner(), vec![11, 10, 9, 8, 7]);
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
				StakeOf::<Test> { owner: 9, amount: 60 },
			]
			.try_into()
			.unwrap(),
		);
		assert_eq!(set.linear_search(&StakeOf::<Test> { owner: 1, amount: 0 }), Ok(0));
		assert_eq!(set.linear_search(&StakeOf::<Test> { owner: 7, amount: 100 }), Ok(3));
		assert_eq!(set.linear_search(&StakeOf::<Test> { owner: 7, amount: 50 }), Ok(3));
		assert_eq!(set.linear_search(&StakeOf::<Test> { owner: 2, amount: 100 }), Err(1));
		assert_eq!(set.linear_search(&StakeOf::<Test> { owner: 2, amount: 90 }), Err(2));
		assert_eq!(set.linear_search(&StakeOf::<Test> { owner: 2, amount: 65 }), Err(4));
		assert_eq!(set.linear_search(&StakeOf::<Test> { owner: 2, amount: 60 }), Err(5));
		assert_eq!(set.linear_search(&StakeOf::<Test> { owner: 2, amount: 59 }), Err(5));
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
}
