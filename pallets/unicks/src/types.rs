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

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{fmt::Debug, marker::PhantomData, vec::Vec};

use codec::{Decode, Encode};
use frame_support::{ensure, BoundedVec};
use scale_info::TypeInfo;

use crate::{Config, Error};

/// A KILT Unick.
///
/// It is bounded in size and can only contain a subset of ASCII characters.
#[derive(Encode, Decode, TypeInfo)]
#[scale_info(skip_type_params(T, MaxLength))]
pub struct AsciiUnick<T, MaxLength>(BoundedVec<u8, MaxLength>, PhantomData<T>);

impl<T: Config> TryFrom<Vec<u8>> for AsciiUnick<T, T::MaxUnickLength> {
	type Error = Error<T>;

	/// Fallible initialization from a provided byte vector if it exceeds the
	/// maximum length or contains invalid ASCII characters.
	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		let bounded_vec: BoundedVec<u8, T::MaxUnickLength> =
			BoundedVec::try_from(value).map_err(|_| Self::Error::InvalidUnickFormat)?;
		ensure!(
			crate::utils::is_byte_array_ascii_string(&bounded_vec),
			Self::Error::InvalidUnickFormat
		);
		Ok(Self(bounded_vec, PhantomData))
	}
}

impl<T: Config> Debug for AsciiUnick<T, T::MaxUnickLength> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		f.debug_tuple("AsciiUnick").field(&self.0).finish()
	}
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for AsciiUnick<T, T::MaxUnickLength> {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for AsciiUnick<T, T::MaxUnickLength> {
	fn clone(&self) -> Self {
		Self(self.0.clone(), self.1)
	}
}

/// KILT unick ownership details.
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo)]
pub struct UnickOwnership<Owner, Deposit, BlockNumber> {
	/// The owner of the unick.
	pub(crate) owner: Owner,
	/// The block number at which the unick was claimed.
	pub(crate) claimed_at: BlockNumber,
	/// The deposit associated with the unick.
	pub(crate) deposit: Deposit,
}
