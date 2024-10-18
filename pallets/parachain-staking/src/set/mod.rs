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

#[cfg(test)]
mod tests;

use frame_support::{traits::Get, BoundedVec, DefaultNoBound};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, RuntimeDebug, SaturatedConversion};
use sp_std::{
	cmp::Ordering,
	convert::TryInto,
	ops::{Index, Range, RangeFull},
};

#[cfg(feature = "std")]
use sp_std::prelude::*;

/// An ordered set backed by `BoundedVec`.
#[derive(PartialEq, Eq, Encode, Decode, DefaultNoBound, Clone, TypeInfo, MaxEncodedLen, RuntimeDebug)]
#[scale_info(skip_type_params(S))]
#[codec(mel_bound(T: MaxEncodedLen))]
pub struct OrderedSet<T, S: Get<u32>>(BoundedVec<T, S>);

impl<T: Ord + Clone, S: Get<u32>> OrderedSet<T, S> {
	/// Create a new empty set.
	pub fn new() -> Self {
		Self(BoundedVec::default())
	}

	pub fn iter(&self) -> sp_std::slice::Iter<'_, T> {
		self.0.iter()
	}

	/// Creates an ordered set from a `BoundedVec`.
	///
	/// The vector will be sorted reversily (from greatest to lowest) and
	/// deduped first.
	pub fn from(bv: BoundedVec<T, S>) -> Self {
		let mut v = bv.into_inner();
		v.sort_by(|a, b| b.cmp(a));
		v.dedup();
		#[allow(clippy::expect_used)]
		Self::from_sorted_set(v.try_into().map_err(|_| ()).expect("No values were added"))
	}

	/// Create a set from a `BoundedVec`.
	///
	/// Assumes that `v` is sorted reversely (from greatest to lowest) and only
	/// contains unique elements.
	pub fn from_sorted_set(bv: BoundedVec<T, S>) -> Self {
		Self(bv)
	}

	/// Mutate the set without restrictions. After the set was mutated it will
	/// be resorted and deduplicated.
	pub fn mutate<F: FnOnce(&mut BoundedVec<T, S>)>(&mut self, function: F) {
		function(&mut self.0);
		(self.0[..]).sort_by(|a, b| b.cmp(a));

		// TODO: add dedup to BoundedVec
		let mut i: usize = 0;
		let mut next = i.saturating_add(1);
		while next < self.len() {
			#[allow(clippy::indexing_slicing)]
			if self[i] == self[next] {
				self.0.remove(next);
			} else {
				i = next;
				next = next.saturating_add(1);
			}
		}
	}

	/// Inserts an element, if no equal item exist in the set.
	///
	/// Returns an error if insertion would exceed the bounded vec's max size.
	/// The error contains the index where the element would be inserted, if
	/// enough space would be left.
	///
	/// Returns true if the item is unique in the set, otherwise returns false.
	pub fn try_insert(&mut self, value: T) -> Result<bool, usize> {
		match self.linear_search(&value) {
			Ok(_) => Ok(false),
			Err(loc) => {
				self.0.try_insert(loc, value).map_err(|_| loc)?;
				Ok(true)
			}
		}
	}

	/// Inserts an element, if no equal item exist in the set. If the set is
	/// full, but an element with a lower rank is in the set, the element with
	/// the lowest rank will be removed and the new element will be added.
	///
	/// Returns
	/// * Ok(Some(old_element)) if the new element was added and an old element
	///   had to be removed.
	/// * Ok(None) if the element was added without removing an element.
	/// * Err(true) if the set is full and the new element has a lower rank than
	///   the lowest element in the set.
	/// * Err(false) if the element is already in the set.
	pub fn try_insert_replace(&mut self, value: T) -> Result<Option<T>, bool> {
		// the highest allowed index
		let highest_index: usize = S::get().saturating_sub(1).saturated_into();
		if S::get().is_zero() {
			return Err(true);
		}
		match self.try_insert(value.clone()) {
			Err(loc) if loc <= highest_index => {
				// always replace the last element
				let last_idx = self.len().saturating_sub(1);
				// accessing by index wont panic since we checked the index, inserting the item
				// at the end of the list to ensure last-in-least-priority-rule for collators.
				// sorting algorithm must be stable!
				#[allow(clippy::indexing_slicing)]
				let old = sp_std::mem::replace(&mut self.0[last_idx], value);
				self.sort_greatest_to_lowest();
				Ok(Some(old))
			}
			Err(_) => Err(true),
			Ok(false) => Err(false),
			Ok(_) => Ok(None),
		}
	}

	/// Inserts a new element or updates the value of an existing one.
	///
	/// Returns an error if the maximum size of the bounded vec would be
	/// exceeded upon insertion.
	///
	/// Returns the old value if existing or None if the value did not exist
	/// before.
	pub fn try_upsert(&mut self, value: T) -> Result<Option<T>, ()> {
		match self.linear_search(&value) {
			Ok(i) => {
				#[allow(clippy::indexing_slicing)]
				let old = sp_std::mem::replace(&mut self.0[i], value);
				self.sort_greatest_to_lowest();
				Ok(Some(old))
			}
			Err(i) => {
				// Delegator
				self.0.try_insert(i, value).map_err(|_| ())?;
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

	/// Return whether the set contains `value`.
	pub fn contains(&self, value: &T) -> bool {
		self.linear_search(value).is_ok()
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

	/// Returns a reference to an element or None if out of bounds.
	pub fn get(&self, index: usize) -> Option<&T> {
		self.0.get(index)
	}

	/// Sorts from greatest to lowest.
	pub fn sort_greatest_to_lowest(&mut self) {
		(self.0[..]).sort_by(|a, b| b.cmp(a));
	}
}

impl<T: Ord + Clone, S: Get<u32>> From<BoundedVec<T, S>> for OrderedSet<T, S> {
	fn from(bv: BoundedVec<T, S>) -> Self {
		Self::from(bv)
	}
}

impl<T: Ord + Clone, S: Get<u32>> Index<usize> for OrderedSet<T, S> {
	type Output = T;

	#[allow(clippy::indexing_slicing)]
	fn index(&self, index: usize) -> &Self::Output {
		&self.0[index]
	}
}

impl<T: Ord + Clone, S: Get<u32>> Index<Range<usize>> for OrderedSet<T, S> {
	type Output = [T];

	#[allow(clippy::indexing_slicing)]
	fn index(&self, range: Range<usize>) -> &Self::Output {
		&self.0[range]
	}
}

impl<T: Ord + Clone, S: Get<u32>> Index<RangeFull> for OrderedSet<T, S> {
	type Output = [T];

	#[allow(clippy::indexing_slicing)]
	fn index(&self, range: RangeFull) -> &Self::Output {
		&self.0[range]
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
