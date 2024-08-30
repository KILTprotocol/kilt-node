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

use frame_support::traits::fungible::Mutate;
use pallet_treasury::ArgumentsFactory;
use sp_std::marker::PhantomData;

use crate::constants::KILT;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

/// Benchmark helper for the treasury pallet. Implements the `ArgumentsFactory`
/// trait. Used to create accounts and assets for the treasury pallet
/// benchmarks.
pub struct BenchmarkHelper<T>(PhantomData<T>);

impl<T> ArgumentsFactory<(), AccountIdOf<T>> for BenchmarkHelper<T>
where
	T: pallet_balances::Config,
	<T as pallet_balances::Config>::Balance: From<u128>,
	AccountIdOf<T>: From<sp_runtime::AccountId32>,
{
	fn create_asset_kind(_seed: u32) {}

	fn create_beneficiary(seed: [u8; 32]) -> AccountIdOf<T> {
		let who = AccountIdOf::<T>::from(seed.into());

		// endow account with some funds. If creation is failing, we panic.
		<pallet_balances::Pallet<T> as Mutate<AccountIdOf<T>>>::mint_into(&who, KILT.into()).unwrap();

		who
	}
}
