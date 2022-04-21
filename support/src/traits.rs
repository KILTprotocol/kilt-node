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

use frame_support::dispatch::Weight;
use sp_runtime::{traits::Zero, DispatchError};

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

pub trait IdentityIncrementer {
	fn increment(&mut self) -> Weight;
}

pub trait IdentityDecrementer {
	fn decrement(&mut self) -> Weight;
}

pub trait IdentityConsumer<Identity> {
	type IdentityIncrementer: IdentityIncrementer;
	type IdentityDecrementer: IdentityDecrementer;
	type Error;

	fn get_incrementer(id: &Identity) -> Result<Self::IdentityIncrementer, Self::Error>;
	fn get_incrementer_max_weight() -> Weight;
	fn get_decrementer(id: &Identity) -> Result<Self::IdentityDecrementer, Self::Error>;
	fn get_decrementer_max_weight() -> Weight;
	fn has_consumers(id: &Identity) -> bool {
		Self::get_decrementer(id).is_ok()
	}
}

impl IdentityIncrementer for () {
	fn increment(&mut self) -> Weight {
		Weight::zero()
	}
}

impl IdentityDecrementer for () {
	fn decrement(&mut self) -> Weight {
		Weight::zero()
	}
}

impl<T> IdentityConsumer<T> for () {
	type IdentityIncrementer = Self;
	type IdentityDecrementer = Self;
	type Error = DispatchError;

	fn get_incrementer(_id: &T) -> Result<Self::IdentityIncrementer, Self::Error> {
		Ok(())
	}

	fn get_incrementer_max_weight() -> Weight {
		Weight::zero()
	}

	fn get_decrementer(_id: &T) -> Result<Self::IdentityDecrementer, Self::Error> {
		Ok(())
	}

	fn get_decrementer_max_weight() -> Weight {
		Weight::zero()
	}
}
