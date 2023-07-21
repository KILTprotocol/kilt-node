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
use sp_std::marker::PhantomData;

use crate::utils::OutputOf;

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
pub trait DipCallOriginFilter<Call> {
	/// The error type for cases where the checks fail.
	type Error;
	/// The type of additional information required by the type to perform the
	/// checks on the `Call` input.
	type OriginInfo;
	/// The success type for cases where the checks succeed.
	type Success;

	fn check_call_origin_info(call: &Call, info: &Self::OriginInfo) -> Result<Self::Success, Self::Error>;
}

pub trait RelayChainStateInfo {
	type BlockNumber;
	type Key;
	type Hasher: sp_runtime::traits::Hash;
	type ParaId;

	fn state_root_for_block(block_height: &Self::BlockNumber) -> Option<OutputOf<Self::Hasher>>;
	fn parachain_head_storage_key(para_id: &Self::ParaId) -> Self::Key;
}

pub trait ProviderParachainStateInfo {
	type BlockNumber;
	type Commitment;
	type Key;
	type Hasher: sp_runtime::traits::Hash;
	type Identifier;

	fn dip_subject_storage_key(identifier: &Self::Identifier) -> Self::Key;
}

pub trait DidSignatureVerifierContext {
	const SIGNATURE_VALIDITY: u16;

	type BlockNumber;
	type Hash;
	type SignedExtra;

	fn block_number() -> Self::BlockNumber;
	fn genesis_hash() -> Self::Hash;
	fn signed_extra() -> Self::SignedExtra;
}

pub struct FrameSystemDidSignatureContext<T, const SIGNATURE_VALIDITY: u16>(PhantomData<T>);

impl<T, const SIGNATURE_VALIDITY: u16> DidSignatureVerifierContext
	for FrameSystemDidSignatureContext<T, SIGNATURE_VALIDITY>
where
	T: frame_system::Config,
{
	const SIGNATURE_VALIDITY: u16 = SIGNATURE_VALIDITY;

	type BlockNumber = T::BlockNumber;
	type Hash = T::Hash;
	type SignedExtra = ();

	fn block_number() -> Self::BlockNumber {
		frame_system::Pallet::<T>::block_number()
	}

	fn genesis_hash() -> Self::Hash {
		frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero())
	}

	fn signed_extra() -> Self::SignedExtra {}
}
