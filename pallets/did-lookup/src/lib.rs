// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

//! # Treasury minting pallet
//!
//! Mints a pre-configured amount of tokens to the Treasury once every block.
//!
//! - [`Pallet`]
//!
//! ## Assumptions
//!
//! - The minting of rewards after [InitialPeriodLength] many blocks is handled
//!   by another pallet, e.g., ParachainStaking.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod default_weights;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use crate::{default_weights::WeightInfo, pallet::*};

#[frame_support::pallet]
pub mod pallet {
	use super::WeightInfo;
	use frame_support::{pallet_prelude::*, traits::StorageVersion};
	use frame_system::pallet_prelude::*;

	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO: doc
		// TODO: test
		// TODO: benchmark
		#[pallet::weight(10)]
		pub fn associate_address(
			origin: OriginFor<T>,
			account: AccountIdOf<T>,
			proof: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			
			Ok(().into())
		}

		// TODO: doc
		// TODO: test
		// TODO: benchmark
		#[pallet::weight(10)]
		pub fn invalidate_association(
			origin: OriginFor<T>,
			account: AccountIdOf<T>,
			proof: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			Ok(().into())
		}
	}
}
