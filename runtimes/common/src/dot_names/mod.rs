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

use frame_support::ensure;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;
use sp_runtime::{traits::Get, BoundedVec, SaturatedConversion};

use pallet_web3_names::{Config, Error};

#[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T, I))]
#[codec(mel_bound())]
pub struct DotName<T, I>(pub BoundedVec<u8, <T as Config<I>>::MaxNameLength>)
where
	T: Config<I>,
	I: 'static;

impl<T, I> TryFrom<Vec<u8>> for DotName<T, I>
where
	T: Config<I>,
	I: 'static,
{
	type Error = Error<T, I>;

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		ensure!(
			value.len() >= T::MinNameLength::get().saturated_into(),
			Self::Error::TooShort
		);
		let bounded_vec: BoundedVec<u8, T::MaxNameLength> =
			BoundedVec::try_from(value).map_err(|_| Self::Error::TooLong)?;
		ensure!(is_valid_dot_name(&bounded_vec), Self::Error::InvalidCharacter);
		Ok(Self(bounded_vec))
	}
}

// FIXME: did not find a way to automatically implement this. Runtime would need
// to implement PartialEq.
impl<T, I> PartialEq for DotName<T, I>
where
	T: Config<I>,
	I: 'static,
{
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}

// FIXME: did not find a way to automatically implement this. Runtime would need
// to implement Eq.
impl<T, I> Eq for DotName<T, I>
where
	T: Config<I>,
	I: 'static,
{
	fn assert_receiver_is_total_eq(&self) {
		self.0.assert_receiver_is_total_eq()
	}
}

// FIXME: did not find a way to automatically implement this. Runtime would need
// to implement PartialOrd.
impl<T, I> PartialOrd for DotName<T, I>
where
	T: Config<I>,
	I: 'static,
{
	fn partial_cmp(&self, other: &Self) -> Option<sp_std::cmp::Ordering> {
		Some(self.0.as_slice().cmp(other.0.as_slice()))
	}
}

// FIXME: did not find a way to automatically implement this. Runtime would need
// to implement Ord.
impl<T, I> Ord for DotName<T, I>
where
	T: Config<I>,
	I: 'static,
{
	fn cmp(&self, other: &Self) -> sp_std::cmp::Ordering {
		self.0.cmp(&other.0)
	}
}

// FIXME: did not find a way to automatically implement this. Runtime would need
// to implement Clone.
impl<T, I> Clone for DotName<T, I>
where
	T: Config<I>,
	I: 'static,
{
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

fn is_valid_dot_name(input: &[u8]) -> bool {
	let Some(dot_name_without_suffix) = input.strip_suffix(b".dot") else {
		return false;
	};
	// Char validation logic taken from https://github.com/paritytech/polkadot-sdk/blob/657b5503a04e97737696fa7344641019350fb521/substrate/frame/identity/src/lib.rs#L1435
	dot_name_without_suffix
		.iter()
		.all(|c| c.is_ascii_digit() || c.is_ascii_lowercase())
}

#[cfg(test)]
mod test {
	use super::*;

	const MIN_LENGTH: u32 = 4;
	const MAX_LENGTH: u32 =

	let valid_inputs = [
		b"ntn.dot",
		b"012.dot",
		b"n01.dot",
		b"
	];
}
