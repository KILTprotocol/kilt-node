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

use crate::*;

use codec::{Decode, Encode};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::{dispatch::UnfilteredDispatchable, traits::Currency, unsigned::ValidateUnsigned};
use frame_system::RawOrigin;
use sp_runtime::{
	traits::{One, StaticLookup},
	Permill,
};

const SEED_1: u32 = 1;
const SEED_2: u32 = 2;

benchmarks! {
	set_registrar_account {
		let registrar: AccountIdOf<T> = account("registrar", 0, SEED_1);
		let new_registrar: AccountIdOf<T> = account("new_registrar", 0, SEED_2);
		RegistrarAccount::<T>::set(registrar.clone());
	}: _(RawOrigin::Signed(registrar), T::Lookup::unlookup(new_registrar.clone()))
	verify {
		assert_eq!(
			RegistrarAccount::<T>::get(),
			new_registrar,
			"Registrar account different than expected"
		);
	}

	set_contribution {
		let registrar: AccountIdOf<T> = account("registrar", 0, SEED_1);
		let contributor: AccountIdOf<T> = account("contributor", 0, SEED_2);
		let contribution: BalanceOf<T> = BalanceOf::<T>::one();
		RegistrarAccount::<T>::set(registrar.clone());
	}: _(RawOrigin::Signed(registrar), contributor.clone(), contribution)
	verify {
		assert_eq!(
			Contributions::<T>::get(&contributor),
			Some(contribution),
			"Contribution different than the expected one."
		);
	}

	set_config {
		let registrar: AccountIdOf<T> = account("registrar", 0, SEED_1);
		RegistrarAccount::<T>::set(registrar.clone());

		let config = GratitudeConfig::<T::BlockNumber> {
			vested_share: Permill::from_percent(42),
			start_block: 1_u32.into(),
			vesting_length: 10_u32.into(),
		};
	}: _(RawOrigin::Signed(registrar), config.clone())
	verify {
		assert_eq!(
			Configuration::<T>::get(),
			config,
		);
	}

	set_reserve_accounts {
		let registrar: AccountIdOf<T> = account("registrar", 0, SEED_1);
		let reserve_free: AccountIdOf<T> = account("reserve_free", 0, SEED_1);
		let reserve_vested: AccountIdOf<T> = account("reserve_vested", 0, SEED_1);
		RegistrarAccount::<T>::set(registrar.clone());

		let unlookup_reserve_vested = T::Lookup::unlookup(reserve_vested.clone());
		let unlookup_reserve_free = T::Lookup::unlookup(reserve_free.clone());

	}: _(
		RawOrigin::Signed(registrar),
		unlookup_reserve_vested,
		unlookup_reserve_free
	)
	verify {
		assert_eq!(
			Reserve::<T>::get(),
			ReserveAccounts {
				vested: reserve_vested,
				free: reserve_free,
			}
		);
	}

	// receive_gratitude is benchmarked together with validate_unsigned to accommodate for the additional cost of validate_unsigned
	receive_gratitude {
		let registrar: AccountIdOf<T> = account("registrar", 0, SEED_1);
		let reserve_free: AccountIdOf<T> = account("reserve_free", 0, SEED_1);
		let reserve_vested: AccountIdOf<T> = account("reserve_vested", 0, SEED_1);
		let contributor: AccountIdOf<T> = account("contributor", 0, SEED_1);

		let contribution: BalanceOf<T> = CurrencyOf::<T>::minimum_balance() * 3_u32.into();

		RegistrarAccount::<T>::set(registrar);
		Reserve::<T>::set(ReserveAccounts {
			vested: reserve_vested.clone(),
			free: reserve_free.clone(),
		});
		Contributions::<T>::insert(&contributor, contribution);
		Configuration::<T>::set(GratitudeConfig {
			vested_share: Permill::from_percent(50),
			start_block: 1_u32.into(),
			vesting_length: 10_u32.into(),
		});
		CurrencyOf::<T>::make_free_balance_be(&reserve_vested, contribution);
		CurrencyOf::<T>::make_free_balance_be(&reserve_free, contribution);

		let source = sp_runtime::transaction_validity::TransactionSource::External;
		let call_enc = Call::<T>::receive_gratitude {
			receiver: contributor.clone(),
		}.encode();
	}: {
		let call = <Call<T> as Decode>::decode(&mut &*call_enc)
			.expect("call is encoded above, encoding must be correct");
		Pallet::<T>::validate_unsigned(source, &call).map_err(|e| -> &'static str { e.into() })?;
		call.dispatch_bypass_filter(RawOrigin::None.into())?;
	}
	verify {
		assert!(Contributions::<T>::get(contributor.clone()).is_none());
		assert_eq!(CurrencyOf::<T>::free_balance(&contributor), contribution);
	}

	remove_contribution {
		let registrar: AccountIdOf<T> = account("registrar", 0, SEED_1);
		let contributor: AccountIdOf<T> = account("contributor", 0, SEED_2);
		let contribution: BalanceOf<T> = BalanceOf::<T>::one();
		RegistrarAccount::<T>::set(registrar.clone());
		Contributions::<T>::insert(&contributor, contribution);
	}: _(RawOrigin::Signed(registrar), contributor.clone())
	verify {
		assert!(
			Contributions::<T>::get(&contributor).is_none(),
			"Contribution should have been removed."
		);
	}
}

impl_benchmark_test_suite! {
	Pallet,
	crate::mock::ExtBuilder::default().build_with_keystore(),
	crate::mock::Test
}
