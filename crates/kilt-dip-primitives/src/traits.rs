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

use sp_core::H256;
use sp_runtime::traits::{CheckedAdd, One, Zero};
use sp_std::marker::PhantomData;

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

/// A trait similar in functionality to the [`frame_support::traits::Get`], but
/// with an input argument and an associated return type.
pub trait GetWithArg<Arg> {
	type Result;

	fn get(arg: &Arg) -> Self::Result;
}

/// Implementer of the [`GetWithArg`] trait that return the state
/// root of a relaychain block with a given number by retrieving it from the
/// [`pallet_relay_store::Pallet`] pallet storage. It hardcodes the
/// relaychain `BlockNumber`, `Hasher`, `StorageKey`, and `ParaId` to the
/// ones used by Polkadot-based relaychains. This type cannot be used with
/// relaychains that adopt a different definition for any on those types.
pub struct RelayStateRootsViaRelayStorePallet<Runtime>(PhantomData<Runtime>);

impl<Runtime> GetWithArg<u32> for RelayStateRootsViaRelayStorePallet<Runtime>
where
	Runtime: pallet_relay_store::Config,
{
	type Result = Option<H256>;

	fn get(arg: &u32) -> Self::Result {
		pallet_relay_store::Pallet::<Runtime>::latest_relay_head_for_block(arg)
			.map(|relay_header| relay_header.relay_parent_storage_root)
	}
}

/// A trait similar in functionality to the [`frame_support::traits::Get`], but
/// with an associated return type.
pub trait GetWithoutArg {
	type Result;

	fn get() -> Self::Result;
}

impl GetWithoutArg for () {
	type Result = ();

	fn get() -> Self::Result {}
}

// Marker trait that requires a type to implement `Default` only for benchmarks.
// Avoids code duplication.
#[cfg(not(feature = "runtime-benchmarks"))]
pub trait BenchmarkDefault {}
#[cfg(not(feature = "runtime-benchmarks"))]
impl<T> BenchmarkDefault for T {}

#[cfg(feature = "runtime-benchmarks")]
pub trait BenchmarkDefault: Default {}
#[cfg(feature = "runtime-benchmarks")]
impl<T: Default> BenchmarkDefault for T {}
