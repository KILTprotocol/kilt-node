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

use parity_scale_codec::Encode;
use sp_core::{storage::StorageKey, Get};
use sp_runtime::traits::{BlakeTwo256, CheckedAdd, One, Zero};
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

pub trait RelayChainStateInfoProvider {
	type BlockNumber;
	type Key;
	type Hasher: sp_runtime::traits::Hash;
	type ParaId;

	fn parachain_head_storage_key(para_id: &Self::ParaId) -> Self::Key;
	fn valid_state_roots<I: FromIterator<<Self::Hasher as sp_runtime::traits::Hash>::Output>>() -> I;
}

pub struct RococoParachainRuntime<Runtime>(PhantomData<Runtime>);

impl<Runtime> RelayChainStateInfoProvider for RococoParachainRuntime<Runtime>
where
	Runtime: pallet_dip_consumer::Config,
{
	type BlockNumber = u64;
	// TODO: This is not exported
	type Hasher = BlakeTwo256;
	type Key = StorageKey;
	type ParaId = u32;

	fn valid_state_roots<I: FromIterator<<Self::Hasher as sp_runtime::traits::Hash>::Output>>() -> I {
		let Some((previous, last, _)) = pallet_dip_consumer::Pallet::<Runtime>::latest_relay_roots() else { return I::from_iter([].into_iter()) };
		I::from_iter([previous, last].into_iter())
	}

	fn parachain_head_storage_key(para_id: &Self::ParaId) -> Self::Key {
		// TODO: It's not possible to access the runtime definition from here.
		let encoded_para_id = para_id.encode();
		let storage_key = [
			frame_support::storage::storage_prefix(b"Paras", b"Heads").as_slice(),
			sp_io::hashing::twox_64(&encoded_para_id).as_slice(),
			encoded_para_id.as_slice(),
		]
		.concat();
		StorageKey(storage_key)
	}
}

pub trait ParachainStateInfoProvider {
	type Commitment;
	type Key;
	type Hasher: sp_runtime::traits::Hash;
	type Identifier;

	fn dip_subject_storage_key(identifier: &Self::Identifier) -> Self::Key;
}

pub struct DipProviderParachainRuntime<Runtime>(PhantomData<Runtime>);

impl<Runtime> ParachainStateInfoProvider for DipProviderParachainRuntime<Runtime>
where
	Runtime: pallet_dip_provider::Config,
{
	type Commitment = <Runtime as pallet_dip_provider::Config>::IdentityCommitment;
	type Hasher = <Runtime as frame_system::Config>::Hashing;
	type Identifier = <Runtime as pallet_dip_provider::Config>::Identifier;
	type Key = StorageKey;

	fn dip_subject_storage_key(identifier: &Self::Identifier) -> Self::Key {
		// TODO: Replace with actual runtime definition
		let encoded_identifier = identifier.encode();
		let storage_key = [
			frame_support::storage::storage_prefix(b"DipProvider", b"IdentityCommitments").as_slice(),
			sp_io::hashing::twox_64(&encoded_identifier).as_slice(),
			encoded_identifier.as_slice(),
		]
		.concat();
		StorageKey(storage_key)
	}
}
