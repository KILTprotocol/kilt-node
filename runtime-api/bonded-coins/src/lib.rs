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

use parity_scale_codec::{alloc::string::String, Codec, Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

mod pool_details;
pub use pool_details::*;

/// Coefficient representation.
#[derive(Decode, Encode, TypeInfo)]
pub struct Coefficient<BitType> {
	/// The internal calculated coefficient, represented as a string.
	pub representation: String,
	/// The bit representation.
	pub bits: BitType,
}

pub trait OperationValue<Balance> {
	fn value(&self) -> Balance;
}

sp_api::decl_runtime_apis! {
	/// Runtime API to compute the collateral for a given amount and pool ID
	/// and to query all pool IDs where the given account is the manager or owner.
	#[api_version(1)]
	pub trait BondedCurrency<Balance, PoolId, Operation, AccountId, BondedAssetId, CollateralAssetId, BitType, Error> where
		Balance: Codec,
		PoolId: Codec,
		Operation: Codec,
		AccountId: Codec,
		BondedAssetId: Codec,
		CollateralAssetId: Codec,
		BitType: Codec,
		Error: Codec
		{
			/// Calculates the collateral for the given amount.
			/// The operation is determining whether the amount is minted or burned.
			/// The calculated collateral amount is based on the current state of the pool.
			fn get_collateral(
				pool_id: PoolId,
				currency_idx: u8,
				operation: Operation,
			) -> Result<Balance, Error>;

			/// Calculates the collateral for the given integral bounds lower and upper.
			/// This function computes the collateral amount based on the provided lower and upper bounds,
			/// regardless of the current state.
			fn calculate_collateral_for_low_and_high_bounds(
				pool_id: PoolId,
				currency_idx: u8,
				low: Balance,
				high: Balance,
			) -> Result<Balance, Error>;

			/// Query all pool IDs where the given account is the manager.
			fn query_pools_by_manager(account : AccountId) -> Vec<PoolDetailsOf<AccountId, Balance, BondedAssetId, CollateralAssetId>>;

			/// Query all pool IDs where the given account is the owner.
			fn query_pools_by_owner(account : AccountId) -> Vec<PoolDetailsOf<AccountId, Balance, BondedAssetId, CollateralAssetId>>;

			/// Calculates the bit representation for the coefficient.
			/// The coefficient is constructed by `coefficient_int.coefficient_frac`.
			/// The first value in the tuple is the internal calculated coefficient, represented as a string.
			/// The second value is the bit representation.
			fn encode_curve_coefficient(coefficient: String) -> Result<Coefficient<BitType>, Error>;

			/// Parses the bit representation for the coefficient to a human readable format.
			fn decode_curve_coefficient(bit_representation: BitType) -> Result<String, Error>;

			/// Query the pool status in a human readable format.
			fn pool_info(pool_id: PoolId) -> Result<PoolDetailsOf<AccountId, Balance, BondedAssetId, CollateralAssetId>, Error>;

			/// Query the pools status in a human readable format.
			fn pool_infos(pool_ids: Vec<PoolId>) -> Result<Vec<PoolDetailsOf<AccountId, Balance, BondedAssetId, CollateralAssetId>>, Error>;
		}
}
