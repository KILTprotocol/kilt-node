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

//! Autogenerated weights for `pallet_migration`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-07-19, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// --steps=50
// --repeat=20
// --chain=dev
// --pallet=pallet-migration
// --extrinsic=*
// --output=./runtimes/peregrine/src/weights/pallet_migration.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_migration`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_migration::WeightInfo for WeightInfo<T> {
	/// Storage: Did Did (r:1 w:1)
	/// Proof: Did Did (max_values: None, max_size: Some(2314), added: 4789, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
	/// Storage: Balances Holds (r:1 w:1)
	/// Proof: Balances Holds (max_values: None, max_size: Some(949), added: 3424, mode: MaxEncodedLen)
	fn general_weight() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `459`
		//  Estimated: `5779`
		// Minimum execution time: 35_980_000 picoseconds.
		Weight::from_parts(36_698_000, 0)
			.saturating_add(Weight::from_parts(0, 5779))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: Balances Locks (r:1 w:1)
	/// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
	/// Storage: Balances Freezes (r:1 w:1)
	/// Proof: Balances Freezes (max_values: None, max_size: Some(949), added: 3424, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
	fn staking_weight() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `536`
		//  Estimated: `4764`
		// Minimum execution time: 31_183_000 picoseconds.
		Weight::from_parts(31_889_000, 0)
			.saturating_add(Weight::from_parts(0, 4764))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn test_general_weight() {
		assert!(
			<crate::Runtime as frame_system::Config>::BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<sp_weights::Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 5779
		);
	}
	#[test]
	fn test_staking_weight() {
		assert!(
			<crate::Runtime as frame_system::Config>::BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<sp_weights::Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 4764
		);
	}
}