
//! Autogenerated weights for pallet_dip_consumer
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-11-23
//! STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `rust-2`, CPU: `12th Gen Intel(R) Core(TM) i9-12900K`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./target/debug/kilt-parachain
// benchmark
// pallet
// --pallet
// pallet-dip-consumer
// --extrinsic
// *
// --template
// ./.maintain/weight-template.hbs
// --output
// ./pallets/pallet-dip-consumer/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_imports)]
#![allow(clippy::as_conversions)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_dip_consumer.
pub trait WeightInfo {
	fn dispatch_as() -> Weight;
}

/// Weights for pallet_dip_consumer using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `PalletDipConsumer::IdentityEntries` (r:1 w:1)
	/// Proof: `PalletDipConsumer::IdentityEntries` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn dispatch_as() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `147`
		//  Estimated: `3612`
		// Minimum execution time: 127_413 nanoseconds.
		Weight::from_parts(129_497_000, 3612)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: `PalletDipConsumer::IdentityEntries` (r:1 w:1)
	/// Proof: `PalletDipConsumer::IdentityEntries` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn dispatch_as() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `147`
		//  Estimated: `3612`
		// Minimum execution time: 127_413 nanoseconds.
		Weight::from_parts(129_497_000, 3612)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}
