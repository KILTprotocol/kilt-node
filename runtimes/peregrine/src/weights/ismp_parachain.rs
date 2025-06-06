// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

//! Autogenerated weights for `ismp_parachain`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 47.0.0
//! DATE: 2025-05-15, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `rust-2`, CPU: `12th Gen Intel(R) Core(TM) i9-12900K`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// frame-omni-bencher
// v1
// benchmark
// pallet
// --pallet=ismp-parachain
// --extrinsic=*
// --genesis-builder=runtime
// --runtime=./target/release/wbuild/peregrine-runtime/peregrine_runtime.compact.compressed.wasm
// --header=HEADER-GPL
// --template=.maintain/runtime-weight-template.hbs
// --output=./runtimes/peregrine/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_imports)]
#![allow(clippy::as_conversions)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `ismp_parachain`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> ismp_parachain::WeightInfo for WeightInfo<T> {
	/// Storage: `IsmpParachain::Parachains` (r:0 w:100)
	/// Proof: `IsmpParachain::Parachains` (`max_values`: None, `max_size`: Some(12), added: 2487, mode: `MaxEncodedLen`)
	/// Storage: `Ismp::ChallengePeriod` (r:0 w:100)
	/// Proof: `Ismp::ChallengePeriod` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `n` is `[1, 100]`.
	fn add_parachain(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 6_388_000 picoseconds.
		Weight::from_parts(6_725_135, 0)
			.saturating_add(Weight::from_parts(0, 0))
			// Standard Error: 4_114
			.saturating_add(Weight::from_parts(1_995_999, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(n.into())))
	}
	/// Storage: `IsmpParachain::Parachains` (r:0 w:5)
	/// Proof: `IsmpParachain::Parachains` (`max_values`: None, `max_size`: Some(12), added: 2487, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[1, 100]`.
	fn remove_parachain(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_479_000 picoseconds.
		Weight::from_parts(8_755_956, 0)
			.saturating_add(Weight::from_parts(0, 0))
			// Standard Error: 475
			.saturating_add(Weight::from_parts(14_864, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: `IsmpParachain::ConsensusUpdated` (r:1 w:1)
	/// Proof: `IsmpParachain::ConsensusUpdated` (`max_values`: Some(1), `max_size`: Some(1), added: 496, mode: `MaxEncodedLen`)
	/// Storage: `Ismp::ConsensusStateClient` (r:1 w:0)
	/// Proof: `Ismp::ConsensusStateClient` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Ismp::ConsensusStates` (r:1 w:1)
	/// Proof: `Ismp::ConsensusStates` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Ismp::FrozenConsensusClients` (r:1 w:0)
	/// Proof: `Ismp::FrozenConsensusClients` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Timestamp::Now` (r:1 w:0)
	/// Proof: `Timestamp::Now` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `Ismp::UnbondingPeriod` (r:1 w:0)
	/// Proof: `Ismp::UnbondingPeriod` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Ismp::ConsensusClientUpdateTime` (r:1 w:1)
	/// Proof: `Ismp::ConsensusClientUpdateTime` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `IsmpParachain::RelayChainStateCommitments` (r:1 w:0)
	/// Proof: `IsmpParachain::RelayChainStateCommitments` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
	/// Storage: `ParachainSystem::ValidationData` (r:1 w:0)
	/// Proof: `ParachainSystem::ValidationData` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `IsmpParachain::Parachains` (r:2 w:0)
	/// Proof: `IsmpParachain::Parachains` (`max_values`: None, `max_size`: Some(12), added: 2487, mode: `MaxEncodedLen`)
	/// Storage: `Ismp::LatestStateMachineHeight` (r:1 w:1)
	/// Proof: `Ismp::LatestStateMachineHeight` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Ismp::StateCommitments` (r:1 w:1)
	/// Proof: `Ismp::StateCommitments` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Ismp::StateMachineUpdateTime` (r:0 w:1)
	/// Proof: `Ismp::StateMachineUpdateTime` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: UNKNOWN KEY `0x7374617465859b5c7d03c68da7d492f1cc906e886ce9b49cc592d063993bdd8c` (r:1 w:1)
	/// Proof: UNKNOWN KEY `0x7374617465859b5c7d03c68da7d492f1cc906e886ce9b49cc592d063993bdd8c` (r:1 w:1)
	fn update_parachain_consensus() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `753`
		//  Estimated: `5964`
		// Minimum execution time: 75_559_000 picoseconds.
		Weight::from_parts(79_529_000, 0)
			.saturating_add(Weight::from_parts(0, 5964))
			.saturating_add(T::DbWeight::get().reads(14))
			.saturating_add(T::DbWeight::get().writes(7))
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn test_update_parachain_consensus() {
		assert!(
			<crate::Runtime as frame_system::Config>::BlockWeights::get()
				.per_class
				.get(frame_support::dispatch::DispatchClass::Normal)
				.max_extrinsic
				.unwrap_or_else(<sp_weights::Weight as sp_runtime::traits::Bounded>::max_value)
				.proof_size()
				> 5964
		);
	}
}
