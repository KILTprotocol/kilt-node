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

use sp_runtime::AccountId32;

use crate::mock::Balances;

mod force_set_swap_pair;
mod force_unset_swap_pair;
mod pause_swap_pair;
mod resume_swap_pair;
mod set_swap_pair;
mod swap;
mod update_remote_fee;

fn assert_total_supply_invariant(
	total_supply: impl Into<u128>,
	remote_balance: impl Into<u128>,
	pool_address: &AccountId32,
) {
	assert!(total_supply.into() - remote_balance.into() <= Balances::usable_balance(pool_address) as u128);
}