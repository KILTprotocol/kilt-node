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

use frame_system::pallet_prelude::BlockNumberFor;
use pallet_dip_provider::{IdentityCommitmentOf, IdentityCommitmentVersion};
use sp_core::storage::StorageKey;
use sp_runtime::traits::{CheckedAdd, One, Zero};
use sp_std::marker::PhantomData;

use crate::utils::OutputOf;

// TODO: Switch to the `Incrementable` trait once it's added to the root of
// `frame_support`.
/// A trait for "incrementable" types, i.e., types that have some notion of
/// order of its members.
pub trait Incrementable {
	/// Increment the type instance to its next value. Overflows are assumed to
	/// be taken care of by the type internal logic.
	fn increment(&mut self);
}

impl<T> Incrementable for T
where
	T: CheckedAdd + Zero + One,
{
	fn increment(&mut self) {
		*self = self.checked_add(&Self::one()).unwrap_or_else(Self::zero);
	}
}

/// A trait for types that implement access control logic where the call is the
/// controlled resource and access is granted based on the provided info.
/// The generic types are the following:
/// * `Call`: The type of the call being checked.
pub trait DipCallOriginFilter<Call> {
	/// The error type for cases where the checks fail.
	type Error;
	/// The type of additional information required by the type to perform the
	/// checks on the `Call` input.
	type OriginInfo;
	/// The success type for cases where the checks succeed.
	type Success;

	/// Check whether the provided call can be dispatch with the given origin
	/// information.
	fn check_call_origin_info(call: &Call, info: &Self::OriginInfo) -> Result<Self::Success, Self::Error>;
}

/// A trait that provides context (e.g., runtime type definitions, storage keys)
/// about the relaychain that is relevant for cross-chain state proofs.
pub trait RelayChainStorageInfo {
	/// The type of relaychain block numbers.
	type BlockNumber;
	/// The type of the relaychain hashing algorithm.
	type Hasher: sp_runtime::traits::Hash;
	/// The type of the relaychain storage key.
	type Key;
	/// The type of parachain IDs.
	type ParaId;

	/// Return the storage key pointing to the head of the parachain
	/// identified by the provided ID.
	fn parachain_head_storage_key(para_id: &Self::ParaId) -> Self::Key;
}

/// A trait that provides state information about specific relaychain blocks.
pub trait RelayChainStateInfo: RelayChainStorageInfo {
	/// Return the relaychain state root at a given block height.
	fn state_root_for_block(block_height: &Self::BlockNumber) -> Option<OutputOf<Self::Hasher>>;
}

/// A trait that provides context (e.g., runtime type definitions, storage keys)
/// about the DIP provider parachain that is relevant for cross-chain state
/// proofs.
pub trait ProviderParachainStorageInfo {
	/// The type of the provider chain's block numbers.
	type BlockNumber;
	/// The type of the provider chain's identity commitments.
	type Commitment;
	/// The type of the provider chain's storage keys.
	type Key;
	/// The type of the provider chain's hashing algorithm.
	type Hasher: sp_runtime::traits::Hash;
	/// The type of the provider chain's identity subject identifiers.
	type Identifier;

	/// Return the storage key pointing to the identity commitment for the given
	/// identifier and version.
	fn dip_subject_storage_key(identifier: &Self::Identifier, version: IdentityCommitmentVersion) -> Self::Key;
}

/// Implementation of the [`ProviderParachainStorageInfo`] trait that builds on
/// the definitions of a runtime that includes the DIP provider pallet (e.g.,
/// KILT runtimes).
/// The generic types are the following:
/// * `T`: The runtime including the [`pallet_dip_provider::Pallet`] pallet.
pub struct ProviderParachainStateInfoViaProviderPallet<T>(PhantomData<T>);

impl<T> ProviderParachainStorageInfo for ProviderParachainStateInfoViaProviderPallet<T>
where
	T: pallet_dip_provider::Config,
{
	type BlockNumber = BlockNumberFor<T>;
	type Commitment = IdentityCommitmentOf<T>;
	type Hasher = T::Hashing;
	type Identifier = T::Identifier;
	type Key = StorageKey;

	fn dip_subject_storage_key(identifier: &Self::Identifier, version: IdentityCommitmentVersion) -> Self::Key {
		StorageKey(pallet_dip_provider::IdentityCommitments::<T>::hashed_key_for(
			identifier, version,
		))
	}
}

/// A trait that provides the consumer parachain runtime additional context to
/// verify cross-chain DID signatures by subjects of the provider parachain.
pub trait DidSignatureVerifierContext {
	/// Max number of blocks a cross-chain DID signature can have to be
	/// considered fresh.
	const SIGNATURE_VALIDITY: u16;

	/// The type of consumer parachain's block numbers.
	type BlockNumber;
	/// The type of consumer parachain's hashes.
	type Hash;
	/// Additional information that must be included in the payload being
	/// DID-signed by the subject.
	type SignedExtra;

	/// Returns the block number of the consumer's chain in which the DID
	/// signature is being evaluated.
	fn current_block_number() -> Self::BlockNumber;
	/// Returns the genesis hash of the consumer's chain.
	fn genesis_hash() -> Self::Hash;
	/// Returns any additional info that must be appended to the payload before
	/// verifying a cross-chain DID signature.
	fn signed_extra() -> Self::SignedExtra;
}

/// Implementation of the [`DidSignatureVerifierContext`] trait that draws
/// information dynamically from the consumer's runtime using its system pallet.
/// The generic types are the following:
/// * `T`: The runtime including the [`frame_system::Pallet`] pallet.
/// * `SIGNATURE_VALIDITY`: The max number of blocks DID signatures can have to
///   be considered valid.
pub struct FrameSystemDidSignatureContext<T, const SIGNATURE_VALIDITY: u16>(PhantomData<T>);

impl<T, const SIGNATURE_VALIDITY: u16> DidSignatureVerifierContext
	for FrameSystemDidSignatureContext<T, SIGNATURE_VALIDITY>
where
	T: frame_system::Config,
{
	const SIGNATURE_VALIDITY: u16 = SIGNATURE_VALIDITY;

	type BlockNumber = BlockNumberFor<T>;
	type Hash = T::Hash;
	type SignedExtra = ();

	fn current_block_number() -> Self::BlockNumber {
		frame_system::Pallet::<T>::block_number()
	}

	fn genesis_hash() -> Self::Hash {
		frame_system::Pallet::<T>::block_hash(Self::BlockNumber::zero())
	}

	fn signed_extra() -> Self::SignedExtra {}
}

/// A trait that provides access to information on historical blocks.
pub trait HistoricalBlockRegistry {
	/// The runtime definition of block numbers.
	type BlockNumber;
	/// The runtime hashing algorithm.
	type Hasher: sp_runtime::traits::Hash;

	/// Retrieve a block hash given its number.
	fn block_hash_for(block: &Self::BlockNumber) -> Option<OutputOf<Self::Hasher>>;
}

impl<T> HistoricalBlockRegistry for T
where
	T: frame_system::Config,
{
	type BlockNumber = BlockNumberFor<T>;
	type Hasher = T::Hashing;

	fn block_hash_for(block: &Self::BlockNumber) -> Option<OutputOf<Self::Hasher>> {
		let retrieved_block = frame_system::Pallet::<T>::block_hash(block);
		let default_block_hash_value = <T::Hash as Default>::default();

		if retrieved_block == default_block_hash_value {
			None
		} else {
			Some(retrieved_block)
		}
	}
}
