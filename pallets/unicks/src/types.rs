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

use sp_std::{marker::PhantomData, str};

use codec::{Decode, Encode};
use frame_support::{ensure, BoundedVec};
use scale_info::TypeInfo;

use crate::{Config, Error};

pub mod traits {
	pub trait Normalizable<Output = Self> {
		fn normalize(&self) -> Output;
	}
}

pub struct Unick<T, S>(BoundedVec<u8, S>, PhantomData<T>);

impl<T: Config> TryFrom<Vec<u8>> for Unick<T, T::MaxUnickLength> {
	type Error = Error<T>;

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		let bounded_vec =
			BoundedVec::<u8, T::MaxUnickLength>::try_from(value).map_err(|_| Self::Error::InvalidUnickFormat)?;
		ensure!(
			crate::utils::is_byte_array_ascii_string(&bounded_vec),
			Self::Error::InvalidUnickFormat
		);

		Ok(Self(bounded_vec.try_into().unwrap(), PhantomData))
	}
}

impl<T: Config> traits::Normalizable<Self> for Unick<T, T::MaxUnickLength> {
	fn normalize(&self) -> Self {
		let lowercase_bytes = str::from_utf8(&self.0).unwrap().to_lowercase().into_bytes();
		Self::try_from(lowercase_bytes).unwrap()
	}
}

#[derive(Clone, Encode, Decode, PartialEq, TypeInfo)]
pub struct UnickOwnership<Owner, Deposit, BlockNumber> {
	pub(crate) owner: Owner,
	pub(crate) claimed_at: BlockNumber,
	pub(crate) deposit: Deposit,
}
