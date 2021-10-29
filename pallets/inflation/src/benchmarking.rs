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
#![cfg(feature = "runtime-benchmarks")]

//! Benchmarking

use super::*;

#[allow(unused)]
use crate::Pallet as KiltTreasury;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_support::traits::{Currency, Get, OnInitialize};
use sp_runtime::traits::{One, Saturating, Zero};

benchmarks! {
	on_initialize_mint_to_treasury {
		assert!(T::Currency::free_balance(&KiltTreasury::<T>::account_id()).is_zero());
	}: { KiltTreasury::<T>::on_initialize(T::BlockNumber::one()) }
	verify {
		assert_eq!(T::Currency::free_balance(&KiltTreasury::<T>::account_id()), T::InitialPeriodReward::get());
	}

	on_initialize_no_action {
		assert!(T::Currency::free_balance(&KiltTreasury::<T>::account_id()).is_zero());
	}: { KiltTreasury::<T>::on_initialize(T::InitialPeriodLength::get().saturating_add(<T as frame_system::Config>::BlockNumber::one())) }
	verify {
		assert!(T::Currency::free_balance(&KiltTreasury::<T>::account_id()).is_zero());
	}
}

impl_benchmark_test_suite!(KiltTreasury, crate::mock::new_test_ext(), crate::mock::Test);
