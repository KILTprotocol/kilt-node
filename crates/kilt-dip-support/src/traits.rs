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

use sp_runtime::traits::{CheckedAdd, One, Zero};

/// A trait for "bumpable" types, i.e., types that have some notion of order of
/// its members.
pub trait Bump {
	/// Bump the type instance to its next value. Overflows are assumed to be
	/// taken care of by the type internal logic.
	fn bump(&mut self);
}

impl<T> Bump for T
where
	T: CheckedAdd + Zero + One,
{
	// FIXME: Better implementation?
	fn bump(&mut self) {
		if let Some(new) = self.checked_add(&Self::one()) {
			*self = new;
		} else {
			*self = Self::zero();
		}
	}
}

/// A trait for types that implement some sort of access control logic on the
/// provided input `Call` type.
pub trait DidDipOriginFilter<Call> {
	/// The error type for cases where the checks fail.
	type Error;
	/// The type of additional information required by the type to perform the
	/// checks on the `Call` input.
	type OriginInfo;
	/// The success type for cases where the checks succeed.
	type Success;

	fn check_call_origin_info(call: &Call, info: &Self::OriginInfo) -> Result<Self::Success, Self::Error>;
}
