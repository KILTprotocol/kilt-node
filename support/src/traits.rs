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

use codec::{EncodeLike, FullCodec};
use cumulus_primitives_core::ParaId;
use frame_support::weights::Weight;
use scale_info::TypeInfo;
use sp_std::vec::Vec;
use xcm::latest::Xcm;

/// The sources of a call struct.
///
/// This trait allows to differentiate between the sender of a call and the
/// subject of the call. The sender account submitted the call to the chain and
/// might pay all fees and deposits that are required by the call.
pub trait CallSources<S, P> {
	/// The sender of the call who will pay for all deposits and fees.
	fn sender(&self) -> S;

	/// The subject of the call.
	fn subject(&self) -> P;
}

impl<S: Clone> CallSources<S, S> for S {
	fn sender(&self) -> S {
		self.clone()
	}

	fn subject(&self) -> S {
		self.clone()
	}
}

impl<S: Clone, P: Clone> CallSources<S, P> for (S, P) {
	fn sender(&self) -> S {
		self.0.clone()
	}

	fn subject(&self) -> P {
		self.1.clone()
	}
}

/// A trait that allows version migrators to access the underlying pallet's
/// context, e.g., its Config trait.
///
/// In this way, the migrator can access the pallet's storage and the pallet's
/// types directly.
pub trait VersionMigratorTrait<T>: Sized {
	#[cfg(feature = "try-runtime")]
	fn pre_migrate(&self) -> Result<(), &'static str>;
	fn migrate(&self) -> frame_support::weights::Weight;
	#[cfg(feature = "try-runtime")]
	fn post_migrate(&self) -> Result<(), &'static str>;
}

/// Trait to simulate an origin with different sender and subject.
/// This origin is only used on benchmarks and testing.
#[cfg(feature = "runtime-benchmarks")]
pub trait GenerateBenchmarkOrigin<OuterOrigin, AccountId, SubjectId> {
	fn generate_origin(sender: AccountId, subject: SubjectId) -> OuterOrigin;
}

/// Trait to reflect calls to the relaychain which we support on the pallet
/// level.
pub trait RelayCallBuilder {
	type AccountId: FullCodec;
	type Balance: FullCodec;
	type RelayChainCall: FullCodec + EncodeLike + sp_std::fmt::Debug + Clone + PartialEq + Eq + TypeInfo;

	/// Execute multiple calls in a batch.
	///
	/// * calls: The list of calls to be executed.
	fn utility_batch_call(calls: Vec<Self::RelayChainCall>) -> Self::RelayChainCall;

	/// Execute a call, replacing the `Origin` with a sub-account.
	///
	/// * call: The call to be executed. Can be nested with
	///   `utility_batch_call`.
	/// * index: The index of the sub-account to be used as the new origin.
	fn utility_as_derivative_call(call: Self::RelayChainCall, index: u16) -> Self::RelayChainCall;

	/// Execute a parachain lease swap call.
	///
	/// * id: One of the two para ids. Typically, this should be the one of the
	///   parachain that executes this XCM call, e.g. the source.
	/// * other: The target para id with which the lease swap should be
	///   executed.
	fn swap_call(id: ParaId, other: ParaId) -> Self::RelayChainCall;

	/// Wrap the final calls into the latest Xcm format.
	///
	/// * call: The relaychain call to be executed
	/// * extra_fee: The extra fee (in relaychain currency) used for buying the
	///   `weight` and `debt`.
	/// * weight: The weight limit used for XCM.
	/// * debt: The weight limit used to process the call.
	fn finalize_call_into_xcm_message(call: Vec<u8>, extra_fee: Self::Balance, weight: Weight) -> Xcm<()>;
}
