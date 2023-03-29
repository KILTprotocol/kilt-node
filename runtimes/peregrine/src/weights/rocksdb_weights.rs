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

//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION
//! 4.0.0-dev DATE: 2023-03-29 (Y/M/D)
//! HOSTNAME: `eyrie-7`, CPU: `Intel(R) Core(TM) i7-7700 CPU @ 3.60GHz`
//!
//! DATABASE: `RocksDb`, RUNTIME: `KILT Spiritnet`
//! BLOCK-NUM: `BlockId::Number(3460187)`
//! SKIP-WRITE: `false`, SKIP-READ: `false`, WARMUPS: `1`
//! STATE-VERSION: `V1`, STATE-CACHE-SIZE: ``
//! WEIGHT-PATH: `runtimes/spiritnet/src/weights/rocksdb_weights.rs`
//! METRIC: `Average`, WEIGHT-MUL: `1.1`, WEIGHT-ADD: `0`

// Executed Command:
//   target/release/kilt-parachain
//   benchmark
//   storage
//   --chain=spiritnet
//   --base-path=/home/bird/spiritnet-db/
//   --template-path=.maintain/weight-db-template.hbs
//   --state-version
//   1
//   --weight-path=runtimes/spiritnet/src/weights/rocksdb_weights.rs
//   --mul=1.1

/// Storage DB weights for the `KILT Spiritnet` runtime and `RocksDb`.
pub mod constants {
	use frame_support::weights::constants;
	use sp_core::parameter_types;
	use sp_weights::RuntimeDbWeight;

	parameter_types! {
		/// By default, Substrate uses `RocksDB`, so this will be the weight used throughout
		/// the runtime.
		pub const RocksDbWeight: RuntimeDbWeight = RuntimeDbWeight {
			/// Time to read one storage item.
			/// Calculated by multiplying the *Average* of all values with `1.1` and adding `0`.
			///
			/// Stats nanoseconds:
			///   Min, Max: 5_380, 11_860_691
			///   Average:  32_804
			///   Median:   23_288
			///   Std-Dev:  55265.31
			///
			/// Percentiles nanoseconds:
			///   99th: 278_383
			///   95th: 57_425
			///   75th: 30_265
			read: 36_085 * constants::WEIGHT_REF_TIME_PER_NANOS,

			/// Time to write one storage item.
			/// Calculated by multiplying the *Average* of all values with `1.1` and adding `0`.
			///
			/// Stats nanoseconds:
			///   Min, Max: 19_775, 13_936_221
			///   Average:  74_821
			///   Median:   72_516
			///   Std-Dev:  66500.91
			///
			/// Percentiles nanoseconds:
			///   99th: 113_045
			///   95th: 95_002
			///   75th: 81_339
			write: 82_304 * constants::WEIGHT_REF_TIME_PER_NANOS,
		};
	}

	#[cfg(test)]
	mod test_db_weights {
		use super::constants::RocksDbWeight as W;
		use sp_weights::constants;

		/// Checks that all weights exist and have sane values.
		// NOTE: If this test fails but you are sure that the generated values are fine,
		// you can delete it.
		#[test]
		fn bound() {
			// At least 1 µs.
			assert!(
				W::get().reads(1).ref_time() >= constants::WEIGHT_REF_TIME_PER_MICROS,
				"Read weight should be at least 1 µs."
			);
			assert!(
				W::get().writes(1).ref_time() >= constants::WEIGHT_REF_TIME_PER_MICROS,
				"Write weight should be at least 1 µs."
			);
			// At most 1 ms.
			assert!(
				W::get().reads(1).ref_time() <= constants::WEIGHT_REF_TIME_PER_MILLIS,
				"Read weight should be at most 1 ms."
			);
			assert!(
				W::get().writes(1).ref_time() <= constants::WEIGHT_REF_TIME_PER_MILLIS,
				"Write weight should be at most 1 ms."
			);
		}
	}
}
