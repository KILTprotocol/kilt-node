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
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, OnUnbalanced, StorageVersion},
	};
	use frame_system::pallet_prelude::*;

	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	pub(crate) type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::NegativeImbalance;

	pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Currency type.
		type Currency: Currency<AccountIdOf<Self>>;

		/// The length of the initial period in which the constant reward is
		/// minted. Once the current block exceeds this, rewards are no further
		/// issued.
		#[pallet::constant]
		type InitialPeriodLength: Get<<Self as frame_system::Config>::BlockNumber>;

		/// The amount of newly issued tokens per block during the initial
		/// period.
		#[pallet::constant]
		type InitialPeriodReward: Get<BalanceOf<Self>>;

		/// The beneficiary to receive the rewards.
		type Beneficiary: OnUnbalanced<NegativeImbalanceOf<Self>>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: T::BlockNumber) -> Weight {
			// The complement of this is handled in ParachainStaking.
			if now <= T::InitialPeriodLength::get() {
				let reward = T::Currency::issue(T::InitialPeriodReward::get());
				T::Beneficiary::on_unbalanced(reward);
				<T as Config>::WeightInfo::on_initialize_mint_to_treasury()
			} else {
				<T as Config>::WeightInfo::on_initialize_no_action()
			}
		}
	}
}
