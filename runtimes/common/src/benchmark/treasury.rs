// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

use core::marker::PhantomData;
use pallet_treasury::ArgumentsFactory;

use crate::constants::KILT;
pub struct BenchmarkHelper<T>(PhantomData<T>);

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

impl<T> ArgumentsFactory<(), AccountIdOf<T>> for BenchmarkHelper<T>
where
	T: pallet_balances::Config,
	<T as pallet_balances::Config>::Balance: From<u128>,
	<T as frame_system::Config>::AccountId: From<sp_runtime::AccountId32>,
{
	fn create_asset_kind(_seed: u32) {}

	fn create_beneficiary(seed: [u8; 32]) -> AccountIdOf<T> {
		let who = AccountIdOf::<T>::from(seed.into());

		// endow account with some funds
		let result = <pallet_balances::Pallet<T> as frame_support::traits::fungible::Mutate<AccountIdOf<T>>>::mint_into(
			&who,
			KILT.into(),
		);

		debug_assert!(
			result.is_ok(),
			"Could not create account for benchmarking treasury pallet"
		);

		who
	}
}
