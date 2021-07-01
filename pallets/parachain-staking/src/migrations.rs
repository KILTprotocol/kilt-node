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

use crate::{
	inflation::InflationInfo,
	pallet::*,
	types::{BalanceOf, Releases},
};
use frame_support::{dispatch::Weight, traits::Get};
use kilt_primitives::constants::MAX_COLLATOR_STAKE;
use sp_runtime::traits::Zero;

pub mod v2 {

	use super::*;

	pub fn pre_migrate<T: Config>() -> Result<(), &'static str> {
		assert!(
			MaxCollatorCandidateStake::<T>::get().is_zero(),
			"MaxCollatorCandidateStake already set."
		);
		// should use default value if it has not existed before
		assert_eq!(StorageVersion::<T>::get(), Releases::V1_0_0);
		Ok(())
	}

	pub fn migrate<T: Config>() -> Weight {
		log::info!("Migrating staking to Releases::V2_0_0");

		MaxCollatorCandidateStake::<T>::put(BalanceOf::<T>::from(MAX_COLLATOR_STAKE));

		// update rewards per block
		InflationConfig::<T>::mutate(|inflation| {
			*inflation = InflationInfo::new(
				inflation.collator.max_rate,
				inflation.collator.reward_rate.annual,
				inflation.delegator.max_rate,
				inflation.delegator.reward_rate.annual,
			);
		});

		StorageVersion::<T>::put(Releases::V2_0_0);
		log::info!("Completed staking migration to Releases::V2_0_0");

		T::DbWeight::get().reads_writes(1, 3)
	}

	pub fn post_migrate<T: Config>() -> Result<(), &'static str> {
		assert_eq!(
			MaxCollatorCandidateStake::<T>::get(),
			BalanceOf::<T>::from(MAX_COLLATOR_STAKE)
		);
		assert_eq!(StorageVersion::<T>::get(), Releases::V2_0_0);
		Ok(())
	}
}

pub mod v3 {
	use kilt_primitives::constants::INFLATION_CONFIG;

	use super::*;

	pub fn migrate<T: Config>() -> Weight {
		log::info!("Migrating staking to Releases::V3_0_0");

		// update rewards per block
		InflationConfig::<T>::mutate(|inflation| *inflation = InflationInfo::from(INFLATION_CONFIG));

		StorageVersion::<T>::put(Releases::V3_0_0);
		log::info!("Completed staking migration to Releases::V3_0_0");

		T::DbWeight::get().reads_writes(1, 2)
	}

	pub fn post_migrate<T: Config>() -> Result<(), &'static str> {
		assert_eq!(InflationConfig::<T>::get(), InflationInfo::from(INFLATION_CONFIG));
		assert_eq!(StorageVersion::<T>::get(), Releases::V3_0_0);
		Ok(())
	}
}
