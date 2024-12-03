use frame_support::traits::{
	fungible::InspectHold,
	fungibles::{metadata::Inspect as InspectMetadata, roles::Inspect as InspectRoles, Inspect},
};
use sp_runtime::{traits::Zero, TryRuntimeError};

use crate::{types::PoolDetails, Config, FungiblesAssetIdOf, HoldReason, Pools};

pub(crate) fn do_try_state<T: Config>() -> Result<(), TryRuntimeError> {
	// checked currency ids. Each Currency should only be associated with one pool.
	let mut checked_currency_ids = Vec::<FungiblesAssetIdOf<T>>::new();

	Pools::<T>::iter().try_for_each(|(pool_id, pool_details)| {
		let PoolDetails {
			collateral_id,
			deposit,
			owner,
			bonded_currencies,
			state,
			denomination,
			..
		} = pool_details;

		let pool_account = pool_id.into();

		// Collateral checks
		let collateral_exists = T::Collaterals::asset_exists(collateral_id.clone());
		assert!(collateral_exists);
		let collateral_issuance = T::Collaterals::total_issuance(collateral_id);

		// Deposit checks
		let balance_on_hold_user =
			T::DepositCurrency::balance_on_hold(&T::RuntimeHoldReason::from(HoldReason::Deposit), &owner);
		assert!(balance_on_hold_user >= deposit);

		// Bonded currencies checks
		bonded_currencies
			.iter()
			.try_for_each(|currency_id| -> Result<(), TryRuntimeError> {
				// check if currency is already associated with another pool
				if checked_currency_ids.contains(currency_id) {
					return Err(TryRuntimeError::Other(
						"Currency is already associated with another pool",
					));
				}

				checked_currency_ids.push(currency_id.clone());

				// if Pool is live, all underlying assets should be live too
				// Other states are not checked because there is no trait to gather the
				// information.
				if state.is_live() {
					let asset_exists = T::Fungibles::asset_exists(currency_id.clone());
					assert!(asset_exists);

					// the owner and issuer should always be the pool account. Admins and Freezer
					// can be changed.
					let owner = T::Fungibles::owner(currency_id.clone()).unwrap();
					assert_eq!(&owner, &pool_account);

					let issuer = T::Fungibles::issuer(currency_id.clone()).unwrap();
					assert_eq!(&issuer, &pool_account);

					// The Currency in the fungibles pallet should always match with the
					// denomination stored in the pool.
					let currency_denomination = T::Fungibles::decimals(currency_id.clone());
					assert_eq!(currency_denomination, denomination);

					// if currency has issuance -> collateral issuance must exists.
					let bonded_issuance = T::Fungibles::total_issuance(currency_id.clone());
					if bonded_issuance > Zero::zero() && collateral_issuance == Zero::zero() {
						return Err(TryRuntimeError::Other(
							"Collateral issuance must exists if bonded currency has issuance",
						));
					}
				}

				Ok(())
			})?;

		Ok(())
	})
}
