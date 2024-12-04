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

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::Codec;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	/// Runtime API to compute the collateral for a given amount and pool ID
	/// and to query all pool IDs where the given account is the manager or owner.
	#[api_version(1)]
	pub trait BondedCurrency<Balance, PoolId, Operation, AccountId, Error> where
		Balance: Codec,
		PoolId: Codec,
		Operation: Codec,
		AccountId: Codec,
		Error: Codec
		{
			/// Calculates the collateral for the given amount.
			/// The operation is determining whether the amount is minted or burned.
			/// The calculated collateral amount is based on the current state of the pool.
			fn calculate_collateral_for_amount(
				amount: Balance,
				pool_id: PoolId,
				operation: Operation,
				currency_idx: u8,
			) -> Result<Balance, Error>;

			/// Calculates the collateral for the given lower and upper bounds.
			/// This function computes the collateral amount based on the provided lower and upper bounds,
			/// regardless of the current state.
			fn calculate_collateral_for_low_and_high(
				low: Balance,
				high: Balance,
				pool_id: PoolId,
				currency_idx: u8,
			) -> Result<Balance, Error>;

			/// Query all pool IDs where the given account is the manager.
			fn query_pools_by_manager(account : AccountId) -> Result<Vec<PoolId>, Error>;

			/// Query all pool IDs where the given account is the owner.
			fn query_pools_by_owner(account : AccountId) -> Result<Vec<PoolId>, Error>;
		}
}
