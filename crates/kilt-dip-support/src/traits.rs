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

use sp_core::Get;
use sp_runtime::traits::{CheckedAdd, One, Zero};
use sp_std::marker::PhantomData;

// TODO: Switch to the `Incrementable` trait once it's added to the root of
// `frame_support`.
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
	fn bump(&mut self) {
		*self = self.checked_add(&Self::one()).unwrap_or_else(Self::zero);
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

pub struct GenesisProvider<T>(PhantomData<T>);

impl<T> Get<T::Hash> for GenesisProvider<T>
where
	T: frame_system::Config,
	T::BlockNumber: Zero,
{
	fn get() -> T::Hash {
		frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero())
	}
}

pub struct BlockNumberProvider<T>(PhantomData<T>);

impl<T> Get<T::BlockNumber> for BlockNumberProvider<T>
where
	T: frame_system::Config,
{
	fn get() -> T::BlockNumber {
		frame_system::Pallet::<T>::block_number()
	}
}
