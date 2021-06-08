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

use parity_scale_codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::{
	cmp::Ordering,
	ops::{Index, IndexMut, Range, RangeFull},
	prelude::*,
};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// An ordered set backed by `Vec`.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, PartialEq, Eq, Encode, Decode, Default, Clone)]
pub struct OrderedSet<T>(Vec<T>);

impl<T: Ord> OrderedSet<T> {
	/// Create a new empty set.
	pub fn new() -> Self {
		Self(Vec::new())
	}

	/// Create a set from a `Vec`.
	///
	/// `v` will be sorted and dedup first.
	pub fn from(mut v: Vec<T>) -> Self {
		v.sort();
		v.dedup();
		Self::from_sorted_set(v)
	}

	/// Create a set from a `Vec`.
	///
	/// Assume `v` is sorted and contain unique elements.
	pub fn from_sorted_set(v: Vec<T>) -> Self {
		Self(v)
	}

	/// Insert an element, if no equal item exist in the set.
	///
	/// Return true if the item is unique in the set, otherwise returns false.
	pub fn insert(&mut self, value: T) -> bool {
		match self.0.binary_search(&value) {
			Ok(_) => false,
			Err(loc) => {
				self.0.insert(loc, value);
				true
			}
		}
	}

	/// Insert or replaces an element.
	///
	/// Returns the old value if existing.
	pub fn upsert(&mut self, value: T) -> Option<T> {
		match self.0.binary_search(&value) {
			Ok(i) => {
				let old = sp_std::mem::replace(&mut self.0[i], value);
				Some(old)
			}
			Err(i) => {
				self.0.insert(i, value);
				None
			}
		}
	}

	/// Remove an element.
	///
	/// Return true if removal happened.
	pub fn remove(&mut self, value: &T) -> Option<T> {
		match self.0.binary_search(value) {
			Ok(loc) => Some(self.0.remove(loc)),
			Err(_) => None,
		}
	}

	/// Remove an element.
	///
	/// Return true if removal happened.
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
		self.0.binary_search(value).is_ok()
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
		self.0.clear();
	}

	/// Return the length of the set.
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Return whether the set is empty.
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	/// Convert the set to a vector.
	pub fn into_vec(self) -> Vec<T> {
		self.0
	}
}

impl<T: Ord> From<Vec<T>> for OrderedSet<T> {
	fn from(v: Vec<T>) -> Self {
		Self::from(v)
	}
}

impl<T: Ord> Index<usize> for OrderedSet<T> {
	type Output = T;

	fn index(&self, index: usize) -> &Self::Output {
		&self.0[index]
	}
}

impl<T: Ord> Index<Range<usize>> for OrderedSet<T> {
	type Output = [T];

	fn index(&self, range: Range<usize>) -> &Self::Output {
		&self.0[range]
	}
}

impl<T: Ord> Index<RangeFull> for OrderedSet<T> {
	type Output = [T];

	fn index(&self, range: RangeFull) -> &Self::Output {
		&self.0[range]
	}
}

impl<T: Ord> IndexMut<usize> for OrderedSet<T> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.0[index]
	}
}

impl<T: Ord> IntoIterator for OrderedSet<T> {
	type Item = T;
	type IntoIter = sp_std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<T: Ord> From<OrderedSet<T>> for Vec<T> {
	fn from(s: OrderedSet<T>) -> Self {
		s.0
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn from() {
		let v = vec![4, 2, 3, 4, 3, 1];
		let set: OrderedSet<i32> = v.into();
		assert_eq!(set, OrderedSet::from(vec![1, 2, 3, 4]));
	}

	#[test]
	fn insert() {
		let mut set: OrderedSet<i32> = OrderedSet::new();
		assert_eq!(set, OrderedSet::from(vec![]));

		assert!(set.insert(1));
		assert_eq!(set, OrderedSet::from(vec![1]));

		assert!(set.insert(5));
		assert_eq!(set, OrderedSet::from(vec![1, 5]));

		assert!(set.insert(3));
		assert_eq!(set, OrderedSet::from(vec![1, 3, 5]));

		assert!(!set.insert(3));
		assert_eq!(set, OrderedSet::from(vec![1, 3, 5]));
	}

	#[test]
	fn remove() {
		let mut set: OrderedSet<i32> = OrderedSet::from(vec![1, 2, 3, 4]);

		assert_eq!(set.remove(&5), None);
		assert_eq!(set, OrderedSet::from(vec![1, 2, 3, 4]));

		assert_eq!(set.remove(&1), Some(1));
		assert_eq!(set, OrderedSet::from(vec![2, 3, 4]));

		assert_eq!(set.remove(&3), Some(3));
		assert_eq!(set, OrderedSet::from(vec![2, 4]));

		assert_eq!(set.remove(&3), None);
		assert_eq!(set, OrderedSet::from(vec![2, 4]));

		assert_eq!(set.remove(&4), Some(4));
		assert_eq!(set, OrderedSet::from(vec![2]));

		assert_eq!(set.remove(&2), Some(2));
		assert_eq!(set, OrderedSet::from(vec![]));

		assert_eq!(set.remove(&2), None);
		assert_eq!(set, OrderedSet::from(vec![]));
	}

	#[test]
	fn contains() {
		let set: OrderedSet<i32> = OrderedSet::from(vec![1, 2, 3, 4]);

		assert!(!set.contains(&5));

		assert!(set.contains(&1));

		assert!(set.contains(&3));
	}

	#[test]
	fn clear() {
		let mut set: OrderedSet<i32> = OrderedSet::from(vec![1, 2, 3, 4]);
		set.clear();
		assert_eq!(set, OrderedSet::new());
	}
}
