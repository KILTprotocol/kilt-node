// TODO: UPDATE BEFORE RELEASE

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

//! Autogenerated weights for `pallet_indices`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-04-05, STEPS: `2`, REPEAT: `1`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `rust-2`, CPU: `12th Gen Intel(R) Core(TM) i9-12900K`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/kilt-parachain
// benchmark
// pallet
// --template=.maintain/runtime-weight-template.hbs
// --header=HEADER-GPL
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --steps=2
// --repeat=1
// --chain=dev
// --pallet=pallet-indices
// --extrinsic=*
// --output=./runtimes/peregrine/src/weights/pallet_indices.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_indices`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_indices::WeightInfo for WeightInfo<T> {
	/// Storage: Indices Accounts (r:1 w:1)
	/// Proof: Indices Accounts (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	fn claim() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `42`
		//  Estimated: `2548`
		// Minimum execution time: 21_287_000 picoseconds.
		Weight::from_parts(21_287_000, 0)
			.saturating_add(Weight::from_parts(0, 2548))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Indices Accounts (r:1 w:1)
	/// Proof: Indices Accounts (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
	fn transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `615`
		//  Estimated: `5155`
		// Minimum execution time: 22_558_000 picoseconds.
		Weight::from_parts(22_558_000, 0)
			.saturating_add(Weight::from_parts(0, 5155))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Indices Accounts (r:1 w:1)
	/// Proof: Indices Accounts (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	fn free() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `174`
		//  Estimated: `2548`
		// Minimum execution time: 17_823_000 picoseconds.
		Weight::from_parts(17_823_000, 0)
			.saturating_add(Weight::from_parts(0, 2548))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Indices Accounts (r:1 w:1)
	/// Proof: Indices Accounts (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
	fn force_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `516`
		//  Estimated: `5155`
		// Minimum execution time: 37_762_000 picoseconds.
		Weight::from_parts(37_762_000, 0)
			.saturating_add(Weight::from_parts(0, 5155))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Indices Accounts (r:1 w:1)
	/// Proof: Indices Accounts (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	fn freeze() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `174`
		//  Estimated: `2548`
		// Minimum execution time: 33_623_000 picoseconds.
		Weight::from_parts(33_623_000, 0)
			.saturating_add(Weight::from_parts(0, 2548))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn test_claim() {
		assert!(
			<crate::Runtime as frame_system::Config>::BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<sp_weights::Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 2548
		);
	}
	#[test]
	fn test_transfer() {
		assert!(
			<crate::Runtime as frame_system::Config>::BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<sp_weights::Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 5155
		);
	}
	#[test]
	fn test_free() {
		assert!(
			<crate::Runtime as frame_system::Config>::BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<sp_weights::Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 2548
		);
	}
	#[test]
	fn test_force_transfer() {
		assert!(
			<crate::Runtime as frame_system::Config>::BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<sp_weights::Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 5155
		);
	}
	#[test]
	fn test_freeze() {
		assert!(
			<crate::Runtime as frame_system::Config>::BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<sp_weights::Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 2548
		);
	}
}
