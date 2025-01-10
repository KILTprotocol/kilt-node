//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION
//! 33.0.0 DATE: 2025-01-10 (Y/M/D)
//! HOSTNAME: `rust-2`, CPU: `12th Gen Intel(R) Core(TM) i9-12900K`
//!
//! DATABASE: `RocksDb`, RUNTIME: `KILT Peregrine Develop`
//! BLOCK-NUM: `BlockId::Number(0)`
//! SKIP-WRITE: `false`, SKIP-READ: `false`, WARMUPS: `1`
//! STATE-VERSION: `V1`, STATE-CACHE-SIZE: ``
//! WEIGHT-PATH: `runtimes/peregrine/src/weights/rocksdb_weights.rs`
//! METRIC: `Average`, WEIGHT-MUL: `1.1`, WEIGHT-ADD: `0`

// Executed Command:
//   target/debug/kilt-parachain
//   benchmark
//   storage
//   --chain
//   dev
//   --template-path
//   .maintain/weight-db-template.hbs
//   --state-version
//   1
//   --weight-path
//   runtimes/peregrine/src/weights/rocksdb_weights.rs
//   --mul=1.1

/// Storage DB weights for the `KILT Peregrine Develop` runtime and `RocksDb`.
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
			///   Min, Max: 15_180, 14_278_535
			///   Average:  169_414
			///   Median:   20_103
			///   Std-Dev:  1447573.56
			///
			/// Percentiles nanoseconds:
			///   99th: 14_278_535
			///   95th: 27_260
			///   75th: 23_057
			read: 186_356 * constants::WEIGHT_REF_TIME_PER_NANOS,

			/// Time to write one storage item.
			/// Calculated by multiplying the *Average* of all values with `1.1` and adding `0`.
			///
			/// Stats nanoseconds:
			///   Min, Max: 41_658, 708_109_449
			///   Average:  7_511_596
			///   Median:   129_472
			///   Std-Dev:  71879832.74
			///
			/// Percentiles nanoseconds:
			///   99th: 708_109_449
			///   95th: 190_014
			///   75th: 155_799
			write: 8_262_756 * constants::WEIGHT_REF_TIME_PER_NANOS,
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
