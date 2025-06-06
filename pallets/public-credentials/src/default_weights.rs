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

//! Autogenerated weights for public_credentials
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-05-18
//! STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `rust-2`, CPU: `12th Gen Intel(R) Core(TM) i9-12900K`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/kilt-parachain
// benchmark
// pallet
// --template=.maintain/weight-template.hbs
// --header=HEADER-GPL
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --steps=50
// --repeat=20
// --chain=dev
// --pallet=public-credentials
// --extrinsic=*
// --output=./pallets/public-credentials/src/default_weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_imports)]
#![allow(clippy::as_conversions)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for public_credentials.
pub trait WeightInfo {
	fn add(c: u32, ) -> Weight;
	fn revoke() -> Weight;
	fn unrevoke() -> Weight;
	fn remove() -> Weight;
	fn reclaim_deposit() -> Weight;
	fn change_deposit_owner() -> Weight;
	fn update_deposit() -> Weight;
}

/// Weights for public_credentials using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: Ctype Ctypes (r:1 w:0)
	/// Proof: Ctype Ctypes (max_values: None, max_size: Some(88), added: 2563, mode: MaxEncodedLen)
	/// Storage: PublicCredentials Credentials (r:1 w:1)
	/// Proof: PublicCredentials Credentials (max_values: None, max_size: Some(475), added: 2950, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
	/// Storage: PublicCredentials CredentialSubjects (r:0 w:1)
	/// Proof: PublicCredentials CredentialSubjects (max_values: None, max_size: Some(312), added: 2787, mode: MaxEncodedLen)
	/// The range of component `c` is `[1, 100000]`.
	fn add(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `829`
		//  Estimated: `8120`
		// Minimum execution time: 27_323 nanoseconds.
		Weight::from_parts(27_065_888, 8120)
			// Standard Error: 15
			.saturating_add(Weight::from_parts(1_595, 0 ).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: PublicCredentials CredentialSubjects (r:1 w:0)
	/// Proof: PublicCredentials CredentialSubjects (max_values: None, max_size: Some(312), added: 2787, mode: MaxEncodedLen)
	/// Storage: PublicCredentials Credentials (r:1 w:1)
	/// Proof: PublicCredentials Credentials (max_values: None, max_size: Some(475), added: 2950, mode: MaxEncodedLen)
	fn revoke() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `914`
		//  Estimated: `5737`
		// Minimum execution time: 15_690 nanoseconds.
		Weight::from_parts(16_193_000, 5737)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: PublicCredentials CredentialSubjects (r:1 w:0)
	/// Proof: PublicCredentials CredentialSubjects (max_values: None, max_size: Some(312), added: 2787, mode: MaxEncodedLen)
	/// Storage: PublicCredentials Credentials (r:1 w:1)
	/// Proof: PublicCredentials Credentials (max_values: None, max_size: Some(475), added: 2950, mode: MaxEncodedLen)
	fn unrevoke() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `914`
		//  Estimated: `5737`
		// Minimum execution time: 17_962 nanoseconds.
		Weight::from_parts(29_462_000, 5737)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: PublicCredentials CredentialSubjects (r:1 w:1)
	/// Proof: PublicCredentials CredentialSubjects (max_values: None, max_size: Some(312), added: 2787, mode: MaxEncodedLen)
	/// Storage: PublicCredentials Credentials (r:1 w:1)
	/// Proof: PublicCredentials Credentials (max_values: None, max_size: Some(475), added: 2950, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
	fn remove() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1592`
		//  Estimated: `8344`
		// Minimum execution time: 27_101 nanoseconds.
		Weight::from_parts(29_244_000, 8344)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: PublicCredentials CredentialSubjects (r:1 w:1)
	/// Proof: PublicCredentials CredentialSubjects (max_values: None, max_size: Some(312), added: 2787, mode: MaxEncodedLen)
	/// Storage: PublicCredentials Credentials (r:1 w:1)
	/// Proof: PublicCredentials Credentials (max_values: None, max_size: Some(475), added: 2950, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
	fn reclaim_deposit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1592`
		//  Estimated: `8344`
		// Minimum execution time: 27_519 nanoseconds.
		Weight::from_parts(28_728_000, 8344)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: PublicCredentials CredentialSubjects (r:1 w:0)
	/// Proof: PublicCredentials CredentialSubjects (max_values: None, max_size: Some(312), added: 2787, mode: MaxEncodedLen)
	/// Storage: PublicCredentials Credentials (r:1 w:1)
	/// Proof: PublicCredentials Credentials (max_values: None, max_size: Some(475), added: 2950, mode: MaxEncodedLen)
	/// Storage: System Account (r:2 w:2)
	/// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
	fn change_deposit_owner() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1465`
		//  Estimated: `10951`
		// Minimum execution time: 36_253 nanoseconds.
		Weight::from_parts(38_070_000, 10951)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: PublicCredentials CredentialSubjects (r:1 w:0)
	/// Proof: PublicCredentials CredentialSubjects (max_values: None, max_size: Some(312), added: 2787, mode: MaxEncodedLen)
	/// Storage: PublicCredentials Credentials (r:1 w:1)
	/// Proof: PublicCredentials Credentials (max_values: None, max_size: Some(475), added: 2950, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
	fn update_deposit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1052`
		//  Estimated: `8344`
		// Minimum execution time: 33_132 nanoseconds.
		Weight::from_parts(34_010_000, 8344)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: Ctype Ctypes (r:1 w:0)
	/// Proof: Ctype Ctypes (max_values: None, max_size: Some(88), added: 2563, mode: MaxEncodedLen)
	/// Storage: PublicCredentials Credentials (r:1 w:1)
	/// Proof: PublicCredentials Credentials (max_values: None, max_size: Some(475), added: 2950, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
	/// Storage: PublicCredentials CredentialSubjects (r:0 w:1)
	/// Proof: PublicCredentials CredentialSubjects (max_values: None, max_size: Some(312), added: 2787, mode: MaxEncodedLen)
	/// The range of component `c` is `[1, 100000]`.
	fn add(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `829`
		//  Estimated: `8120`
		// Minimum execution time: 27_323 nanoseconds.
		Weight::from_parts(27_065_888, 8120)
			// Standard Error: 15
			.saturating_add(Weight::from_parts(1_595, 0).saturating_mul(c.into()))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: PublicCredentials CredentialSubjects (r:1 w:0)
	/// Proof: PublicCredentials CredentialSubjects (max_values: None, max_size: Some(312), added: 2787, mode: MaxEncodedLen)
	/// Storage: PublicCredentials Credentials (r:1 w:1)
	/// Proof: PublicCredentials Credentials (max_values: None, max_size: Some(475), added: 2950, mode: MaxEncodedLen)
	fn revoke() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `914`
		//  Estimated: `5737`
		// Minimum execution time: 15_690 nanoseconds.
		Weight::from_parts(16_193_000, 5737)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: PublicCredentials CredentialSubjects (r:1 w:0)
	/// Proof: PublicCredentials CredentialSubjects (max_values: None, max_size: Some(312), added: 2787, mode: MaxEncodedLen)
	/// Storage: PublicCredentials Credentials (r:1 w:1)
	/// Proof: PublicCredentials Credentials (max_values: None, max_size: Some(475), added: 2950, mode: MaxEncodedLen)
	fn unrevoke() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `914`
		//  Estimated: `5737`
		// Minimum execution time: 17_962 nanoseconds.
		Weight::from_parts(29_462_000, 5737)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: PublicCredentials CredentialSubjects (r:1 w:1)
	/// Proof: PublicCredentials CredentialSubjects (max_values: None, max_size: Some(312), added: 2787, mode: MaxEncodedLen)
	/// Storage: PublicCredentials Credentials (r:1 w:1)
	/// Proof: PublicCredentials Credentials (max_values: None, max_size: Some(475), added: 2950, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
	fn remove() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1592`
		//  Estimated: `8344`
		// Minimum execution time: 27_101 nanoseconds.
		Weight::from_parts(29_244_000, 8344)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: PublicCredentials CredentialSubjects (r:1 w:1)
	/// Proof: PublicCredentials CredentialSubjects (max_values: None, max_size: Some(312), added: 2787, mode: MaxEncodedLen)
	/// Storage: PublicCredentials Credentials (r:1 w:1)
	/// Proof: PublicCredentials Credentials (max_values: None, max_size: Some(475), added: 2950, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
	fn reclaim_deposit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1592`
		//  Estimated: `8344`
		// Minimum execution time: 27_519 nanoseconds.
		Weight::from_parts(28_728_000, 8344)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: PublicCredentials CredentialSubjects (r:1 w:0)
	/// Proof: PublicCredentials CredentialSubjects (max_values: None, max_size: Some(312), added: 2787, mode: MaxEncodedLen)
	/// Storage: PublicCredentials Credentials (r:1 w:1)
	/// Proof: PublicCredentials Credentials (max_values: None, max_size: Some(475), added: 2950, mode: MaxEncodedLen)
	/// Storage: System Account (r:2 w:2)
	/// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
	fn change_deposit_owner() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1465`
		//  Estimated: `10951`
		// Minimum execution time: 36_253 nanoseconds.
		Weight::from_parts(38_070_000, 10951)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: PublicCredentials CredentialSubjects (r:1 w:0)
	/// Proof: PublicCredentials CredentialSubjects (max_values: None, max_size: Some(312), added: 2787, mode: MaxEncodedLen)
	/// Storage: PublicCredentials Credentials (r:1 w:1)
	/// Proof: PublicCredentials Credentials (max_values: None, max_size: Some(475), added: 2950, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
	fn update_deposit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1052`
		//  Estimated: `8344`
		// Minimum execution time: 33_132 nanoseconds.
		Weight::from_parts(34_010_000, 8344)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
}
