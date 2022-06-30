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

//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-06-30 (Y/M/D)
//! HOSTNAME: ``, CPU: ``
//!
//! DATABASE: `RocksDb`, RUNTIME: `KILT Spiritnet`
//! BLOCK-NUM: `BlockId::Number(1684659)`
//! SKIP-WRITE: `false`, SKIP-READ: `false`, WARMUPS: `1`
//! STATE-VERSION: `V1`, STATE-CACHE-SIZE: `0`
//! WEIGHT-PATH: `runtimes/spiritnet/src/weights/rocksdb_weights.rs`
//! METRIC: `Average`, WEIGHT-MUL: `1.1`, WEIGHT-ADD: `0`

// Executed Command:
//   target/release/kilt-parachain
//   benchmark
//   storage
//   --chain=spiritnet
//   --base-path=/home/weich/spiritnet-db/parachain/
//   --template-path=.maintain/template-db-weight.hbs
//   --state-version
//   1
//   --weight-path=runtimes/spiritnet/src/weights/rocksdb_weights.rs
//   --mul=1.1

/// Storage DB weights for the `KILT Spiritnet` runtime and `RocksDb`.
pub mod constants {
	use frame_support::{
		parameter_types,
		weights::{constants, RuntimeDbWeight},
	};

	parameter_types! {
		/// By default, Substrate uses `RocksDB`, so this will be the weight used throughout
		/// the runtime.
		pub const RocksDbWeight: RuntimeDbWeight = RuntimeDbWeight {
			/// Time to read one storage item.
			/// Calculated by multiplying the *Average* of all values with `1.1` and adding `0`.
			///
			/// Stats nanoseconds:
			///   Min, Max: 7_330, 2_137_697
			///   Average:  48_624
			///   Median:   23_368
			///   Std-Dev:  113173.39
			///
			/// Percentiles nanoseconds:
			///   99th: 682_900
			///   95th: 179_679
			///   75th: 28_961
			read: 53_487 * constants::WEIGHT_PER_NANOS,

			/// Time to write one storage item.
			/// Calculated by multiplying the *Average* of all values with `1.1` and adding `0`.
			///
			/// Stats nanoseconds:
			///   Min, Max: 13_033, 2_461_814
			///   Average:  80_297
			///   Median:   54_905
			///   Std-Dev:  115358.17
			///
			/// Percentiles nanoseconds:
			///   99th: 730_094
			///   95th: 217_545
			///   75th: 63_643
			write: 88_327 * constants::WEIGHT_PER_NANOS,
		};
	}

	#[cfg(test)]
	mod test_db_weights {
		use super::constants::RocksDbWeight as W;
		use frame_support::weights::constants;

		/// Checks that all weights exist and have sane values.
		// NOTE: If this test fails but you are sure that the generated values are fine,
		// you can delete it.
		#[test]
		fn bound() {
			// At least 1 µs.
			assert!(
				W::get().reads(1) >= constants::WEIGHT_PER_MICROS,
				"Read weight should be at least 1 µs."
			);
			assert!(
				W::get().writes(1) >= constants::WEIGHT_PER_MICROS,
				"Write weight should be at least 1 µs."
			);
			// At most 1 ms.
			assert!(
				W::get().reads(1) <= constants::WEIGHT_PER_MILLIS,
				"Read weight should be at most 1 ms."
			);
			assert!(
				W::get().writes(1) <= constants::WEIGHT_PER_MILLIS,
				"Write weight should be at most 1 ms."
			);
		}
	}
}
