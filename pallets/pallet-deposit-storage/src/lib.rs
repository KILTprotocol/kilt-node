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

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

mod deposit;

pub use deposit::StorageDepositCollectorViaDepositsPallet;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::{
		pallet_prelude::*,
		traits::{fungible::Inspect, ConstU32},
	};
	use kilt_support::Deposit;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);
	pub const MAX_NAMESPACE_LENGTH: u32 = 16;

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type BalanceOf<T> = <<T as Config>::Currency as Inspect<AccountIdOf<T>>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Currency: Inspect<Self::AccountId>;
	}

	#[pallet::composite_enum]
	pub enum HoldReason {
		Deposit,
	}

	#[pallet::error]
	pub enum Error<T> {
		DepositNotFound,
		DepositExisting,
	}

	// Double map (namespace, key) -> deposit
	#[pallet::storage]
	pub type Deposits<T> = StorageDoubleMap<
		_,
		Twox64Concat,
		BoundedVec<u8, ConstU32<MAX_NAMESPACE_LENGTH>>,
		Twox64Concat,
		<T as frame_system::Config>::Hash,
		Deposit<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
	>;

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);
}
