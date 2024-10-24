
//! Autogenerated weights for pallet_migration
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-09-14
//! STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `eyrie-7`, CPU: `Intel(R) Core(TM) i7-7700 CPU @ 3.60GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/kilt-parachain
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet-migration
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./pallets/pallet-migration/src/default_weights.rs
// --template=.maintain/weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_imports)]
#![allow(clippy::as_conversions)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_migration.
pub trait WeightInfo {
	fn attestation_migration_weight() -> Weight;
	fn delegation_migration_weight() -> Weight;
	fn did_migration_weight() -> Weight;
	fn did_lookup_migration_weight() -> Weight;
	fn w3n_migration_weight() -> Weight;
	fn public_credentials_migration_weight() -> Weight;
}

/// Weights for pallet_migration using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: Migration MigratedKeys (r:1 w:1)
	/// Proof: Migration MigratedKeys (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Attestation Attestations (r:1 w:0)
	/// Proof: Attestation Attestations (max_values: None, max_size: Some(195), added: 2670, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
	/// Storage: Balances Holds (r:1 w:1)
	/// Proof: Balances Holds (max_values: None, max_size: Some(949), added: 3424, mode: MaxEncodedLen)
	fn attestation_migration_weight() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `846`
		//  Estimated: `4414`
		// Minimum execution time: 69_529 nanoseconds.
		Weight::from_parts(70_605_000, 4414)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: Migration MigratedKeys (r:1 w:0)
	/// Proof: Migration MigratedKeys (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	fn delegation_migration_weight() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `115`
		//  Estimated: `3513`
		// Minimum execution time: 19_698 nanoseconds.
		Weight::from_parts(19_972_000, 3513)
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	/// Storage: Migration MigratedKeys (r:1 w:1)
	/// Proof: Migration MigratedKeys (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Did Did (r:1 w:0)
	/// Proof: Did Did (max_values: None, max_size: Some(2312), added: 4787, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
	/// Storage: Balances Holds (r:1 w:1)
	/// Proof: Balances Holds (max_values: None, max_size: Some(949), added: 3424, mode: MaxEncodedLen)
	fn did_migration_weight() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1002`
		//  Estimated: `5777`
		// Minimum execution time: 71_362 nanoseconds.
		Weight::from_parts(72_504_000, 5777)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: Migration MigratedKeys (r:1 w:0)
	/// Proof: Migration MigratedKeys (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	fn did_lookup_migration_weight() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `78`
		//  Estimated: `3513`
		// Minimum execution time: 18_201 nanoseconds.
		Weight::from_parts(18_569_000, 3513)
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	/// Storage: Migration MigratedKeys (r:1 w:0)
	/// Proof: Migration MigratedKeys (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	fn w3n_migration_weight() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `78`
		//  Estimated: `3513`
		// Minimum execution time: 18_348 nanoseconds.
		Weight::from_parts(18_689_000, 3513)
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	/// Storage: Migration MigratedKeys (r:1 w:0)
	/// Proof: Migration MigratedKeys (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	fn public_credentials_migration_weight() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `78`
		//  Estimated: `3513`
		// Minimum execution time: 22_587 nanoseconds.
		Weight::from_parts(22_874_000, 3513)
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: Migration MigratedKeys (r:1 w:1)
	/// Proof: Migration MigratedKeys (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Attestation Attestations (r:1 w:0)
	/// Proof: Attestation Attestations (max_values: None, max_size: Some(195), added: 2670, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
	/// Storage: Balances Holds (r:1 w:1)
	/// Proof: Balances Holds (max_values: None, max_size: Some(949), added: 3424, mode: MaxEncodedLen)
	fn attestation_migration_weight() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `846`
		//  Estimated: `4414`
		// Minimum execution time: 69_529 nanoseconds.
		Weight::from_parts(70_605_000, 4414)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: Migration MigratedKeys (r:1 w:0)
	/// Proof: Migration MigratedKeys (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	fn delegation_migration_weight() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `115`
		//  Estimated: `3513`
		// Minimum execution time: 19_698 nanoseconds.
		Weight::from_parts(19_972_000, 3513)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
	}
	/// Storage: Migration MigratedKeys (r:1 w:1)
	/// Proof: Migration MigratedKeys (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Did Did (r:1 w:0)
	/// Proof: Did Did (max_values: None, max_size: Some(2312), added: 4787, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
	/// Storage: Balances Holds (r:1 w:1)
	/// Proof: Balances Holds (max_values: None, max_size: Some(949), added: 3424, mode: MaxEncodedLen)
	fn did_migration_weight() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1002`
		//  Estimated: `5777`
		// Minimum execution time: 71_362 nanoseconds.
		Weight::from_parts(72_504_000, 5777)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: Migration MigratedKeys (r:1 w:0)
	/// Proof: Migration MigratedKeys (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	fn did_lookup_migration_weight() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `78`
		//  Estimated: `3513`
		// Minimum execution time: 18_201 nanoseconds.
		Weight::from_parts(18_569_000, 3513)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
	}
	/// Storage: Migration MigratedKeys (r:1 w:0)
	/// Proof: Migration MigratedKeys (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	fn w3n_migration_weight() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `78`
		//  Estimated: `3513`
		// Minimum execution time: 18_348 nanoseconds.
		Weight::from_parts(18_689_000, 3513)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
	}
	/// Storage: Migration MigratedKeys (r:1 w:0)
	/// Proof: Migration MigratedKeys (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	fn public_credentials_migration_weight() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `78`
		//  Estimated: `3513`
		// Minimum execution time: 22_587 nanoseconds.
		Weight::from_parts(22_874_000, 3513)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
	}
}
