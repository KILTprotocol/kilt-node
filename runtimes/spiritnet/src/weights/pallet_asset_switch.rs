
//! Autogenerated weights for `pallet_asset_switch`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 33.0.0
//! DATE: 2024-08-26, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `rust-2`, CPU: `12th Gen Intel(R) Core(TM) i9-12900K`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: Some("spiritnet-dev"), DB CACHE: 1024

// Executed Command:
// ./target/debug/kilt-parachain
// benchmark
// pallet
// --chain=spiritnet-dev
// --steps=50
// --repeat=20
// --pallet=pallet-asset-switch
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./runtimes/spiritnet/src/weights/
// --template=.maintain/runtime-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_asset_switch`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_asset_switch::WeightInfo for WeightInfo<T> {
	/// Storage: `AssetSwitchPool1::SwitchPair` (r:1 w:1)
	/// Proof: `AssetSwitchPool1::SwitchPair` (`max_values`: Some(1), `max_size`: Some(1939), added: 2434, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:0)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
	fn set_switch_pair() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `220`
		//  Estimated: `3597`
		// Minimum execution time: 201_873_000 picoseconds.
		Weight::from_parts(203_847_000, 0)
			.saturating_add(Weight::from_parts(0, 3597))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `AssetSwitchPool1::SwitchPair` (r:0 w:1)
	/// Proof: `AssetSwitchPool1::SwitchPair` (`max_values`: Some(1), `max_size`: Some(1939), added: 2434, mode: `MaxEncodedLen`)
	fn force_set_switch_pair() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 99_925_000 picoseconds.
		Weight::from_parts(101_714_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `AssetSwitchPool1::SwitchPair` (r:1 w:1)
	/// Proof: `AssetSwitchPool1::SwitchPair` (`max_values`: Some(1), `max_size`: Some(1939), added: 2434, mode: `MaxEncodedLen`)
	fn force_unset_switch_pair() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `218`
		//  Estimated: `3424`
		// Minimum execution time: 113_309_000 picoseconds.
		Weight::from_parts(116_506_000, 0)
			.saturating_add(Weight::from_parts(0, 3424))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `AssetSwitchPool1::SwitchPair` (r:1 w:1)
	/// Proof: `AssetSwitchPool1::SwitchPair` (`max_values`: Some(1), `max_size`: Some(1939), added: 2434, mode: `MaxEncodedLen`)
	fn pause_switch_pair() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `218`
		//  Estimated: `3424`
		// Minimum execution time: 88_871_000 picoseconds.
		Weight::from_parts(89_525_000, 0)
			.saturating_add(Weight::from_parts(0, 3424))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `AssetSwitchPool1::SwitchPair` (r:1 w:1)
	/// Proof: `AssetSwitchPool1::SwitchPair` (`max_values`: Some(1), `max_size`: Some(1939), added: 2434, mode: `MaxEncodedLen`)
	fn resume_switch_pair() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `218`
		//  Estimated: `3424`
		// Minimum execution time: 115_902_000 picoseconds.
		Weight::from_parts(118_293_000, 0)
			.saturating_add(Weight::from_parts(0, 3424))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `AssetSwitchPool1::SwitchPair` (r:1 w:1)
	/// Proof: `AssetSwitchPool1::SwitchPair` (`max_values`: Some(1), `max_size`: Some(1939), added: 2434, mode: `MaxEncodedLen`)
	fn update_remote_xcm_fee() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `218`
		//  Estimated: `3424`
		// Minimum execution time: 91_074_000 picoseconds.
		Weight::from_parts(92_105_000, 0)
			.saturating_add(Weight::from_parts(0, 3424))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `AssetSwitchPool1::SwitchPair` (r:1 w:1)
	/// Proof: `AssetSwitchPool1::SwitchPair` (`max_values`: Some(1), `max_size`: Some(1939), added: 2434, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
	/// Storage: `PolkadotXcm::SupportedVersion` (r:1 w:0)
	/// Proof: `PolkadotXcm::SupportedVersion` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::VersionDiscoveryQueue` (r:1 w:1)
	/// Proof: `PolkadotXcm::VersionDiscoveryQueue` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::SafeXcmVersion` (r:1 w:0)
	/// Proof: `PolkadotXcm::SafeXcmVersion` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Fungibles::Asset` (r:1 w:1)
	/// Proof: `Fungibles::Asset` (`max_values`: None, `max_size`: Some(808), added: 3283, mode: `MaxEncodedLen`)
	/// Storage: `Fungibles::Account` (r:1 w:1)
	/// Proof: `Fungibles::Account` (`max_values`: None, `max_size`: Some(732), added: 3207, mode: `MaxEncodedLen`)
	/// Storage: `ParachainSystem::RelevantMessagingState` (r:1 w:0)
	/// Proof: `ParachainSystem::RelevantMessagingState` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `XcmpQueue::OutboundXcmpStatus` (r:1 w:1)
	/// Proof: `XcmpQueue::OutboundXcmpStatus` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `XcmpQueue::OutboundXcmpMessages` (r:0 w:1)
	/// Proof: `XcmpQueue::OutboundXcmpMessages` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn switch() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1091`
		//  Estimated: `6204`
		// Minimum execution time: 1_543_529_000 picoseconds.
		Weight::from_parts(1_578_149_000, 0)
			.saturating_add(Weight::from_parts(0, 6204))
			.saturating_add(T::DbWeight::get().reads(10))
			.saturating_add(T::DbWeight::get().writes(8))
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn test_set_switch_pair() {
		assert!(
			<crate::Runtime as frame_system::Config>::BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<sp_weights::Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 3597
		);
	}
	#[test]
	fn test_force_unset_switch_pair() {
		assert!(
			<crate::Runtime as frame_system::Config>::BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<sp_weights::Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 3424
		);
	}
	#[test]
	fn test_pause_switch_pair() {
		assert!(
			<crate::Runtime as frame_system::Config>::BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<sp_weights::Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 3424
		);
	}
	#[test]
	fn test_resume_switch_pair() {
		assert!(
			<crate::Runtime as frame_system::Config>::BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<sp_weights::Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 3424
		);
	}
	#[test]
	fn test_update_remote_xcm_fee() {
		assert!(
			<crate::Runtime as frame_system::Config>::BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<sp_weights::Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 3424
		);
	}
	#[test]
	fn test_switch() {
		assert!(
			<crate::Runtime as frame_system::Config>::BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<sp_weights::Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 6204
		);
	}
}