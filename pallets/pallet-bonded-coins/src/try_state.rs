use frame_support::traits::{
	fungible::InspectHold,
	fungibles::{metadata::Inspect as InspectMetadata, roles::Inspect as InspectRoles, Inspect},
};
use sp_runtime::{traits::Zero, TryRuntimeError};
use sp_std::vec::Vec;

use crate::{types::PoolDetails, Config, FungiblesAssetIdOf, Pools};

pub(crate) fn do_try_state<T: Config>() -> Result<(), TryRuntimeError> {
	// checked currency ids. Each Currency should only be associated with one pool.
	let mut checked_currency_ids = Vec::<FungiblesAssetIdOf<T>>::new();

	Pools::<T>::iter().try_for_each(|(pool_id, pool_details)| {
		let PoolDetails {
			collateral,
			deposit,
			owner,
			bonded_currencies,
			state,
			denomination,
			..
		} = pool_details;

		let pool_account = pool_id.clone().into();

		// Deposit checks
		let Ok(hold_reason) = T::HoldReason::try_from(pool_id) else {
			panic!("Failed to generate `HoldReason` from pool ID.");
		};
		let balance_on_hold_user =
			T::DepositCurrency::balance_on_hold(&T::RuntimeHoldReason::from(hold_reason), &owner);
		assert!(balance_on_hold_user >= deposit);

		// Collateral checks
		assert!(T::Collaterals::asset_exists(collateral.clone()));
		let collateral_issuance_pool = T::Collaterals::total_balance(collateral, &pool_account);

		// Bonded currencies checks
		bonded_currencies
			.iter()
			.try_for_each(|currency_id| -> Result<(), TryRuntimeError> {
				// check if currency is already associated with another pool
				assert!(!checked_currency_ids.contains(currency_id));
				checked_currency_ids.push(currency_id.clone());

				// if Pool is live or refunding, all underlying assets should be live.
				// Other states are not checked because there is no trait to gather the
				// information.
				if state.is_live() || state.is_refunding() {
					assert!(T::Fungibles::asset_exists(currency_id.clone()));

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

					// if currency has on-zero supply -> collateral in pool account must be
					// non-zero.
					let bonded_issuance = T::Fungibles::total_issuance(currency_id.clone());
					if bonded_issuance > Zero::zero() && collateral_issuance_pool == Zero::zero() {
						return Err(TryRuntimeError::Other(
							"Pool account must hold collateral if bonded currency has issuance",
						));
					}
				}

				Ok(())
			})?;

		Ok(())
	})
}
