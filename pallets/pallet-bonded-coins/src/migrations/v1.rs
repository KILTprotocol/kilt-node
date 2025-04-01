use frame_support::{
	pallet_prelude::*,
	traits::{Get, OnRuntimeUpgrade},
};
use sp_runtime::traits::Saturating;

use crate::{
	curves::Curve,
	types::{Locks, PoolStatus},
	BondedCurrenciesSettingsOf, BoundedCurrencyVec, CollateralAssetIdOf, Config, CurveParameterTypeOf,
	DepositBalanceOf, FungiblesBalanceOf,
};

#[cfg(feature = "try-runtime")]
const LOG_TARGET: &str = "migration::pallet-bonded-coins";

/// Collection of storage item formats from the previous storage version.
///
/// Required so we can read values in the v0 storage format during the
/// migration.
mod v0 {
	use super::*;
	use frame_support::{storage_alias, Twox64Concat};
	use parity_scale_codec::{Decode, Encode};
	use scale_info::TypeInfo;

	// V0 pool details
	#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen, Debug)]
	pub struct PoolDetails<AccountId, ParametrizedCurve, Currencies, BaseCurrencyId, DepositBalance, FungiblesBalance> {
		/// The owner of the pool.
		pub owner: AccountId,
		/// The manager of the pool. If a manager is set, the pool is
		/// permissioned.
		pub manager: Option<AccountId>,
		/// The curve of the pool.
		pub curve: ParametrizedCurve,
		/// The collateral currency of the pool.
		pub collateral: BaseCurrencyId,
		/// The bonded currencies of the pool.
		pub bonded_currencies: Currencies,
		/// The status of the pool.
		pub state: PoolStatus<Locks>,
		/// Whether the pool is transferable or not.
		pub transferable: bool,
		/// The denomination of the pool.
		pub denomination: u8,
		/// The minimum amount that can be minted/burnt.
		pub min_operation_balance: FungiblesBalance,
		/// The deposit to be returned upon destruction of this pool.
		pub deposit: DepositBalance,
	}

	pub type PoolDetailsOf<T> = PoolDetails<
		<T as frame_system::Config>::AccountId,
		Curve<CurveParameterTypeOf<T>>,
		BoundedCurrencyVec<T>,
		CollateralAssetIdOf<T>,
		DepositBalanceOf<T>,
		FungiblesBalanceOf<T>,
	>;

	/// V0 type for [`crate::Pools`].
	#[storage_alias]
	pub type Pools<T: crate::Config> =
		StorageMap<crate::Pallet<T>, Twox64Concat, <T as crate::Config>::PoolId, PoolDetailsOf<T>, OptionQuery>;
}

fn v0_to_v1<T: Config>(old_value: v0::PoolDetailsOf<T>) -> crate::PoolDetailsOf<T> {
	let v0::PoolDetailsOf::<T> {
		owner,
		curve,
		manager,
		collateral,
		bonded_currencies,
		state,
		transferable,
		denomination,
		min_operation_balance,
		deposit,
	} = old_value;

	crate::PoolDetailsOf::<T> {
		owner,
		curve,
		manager,
		collateral,
		bonded_currencies,
		state,
		deposit,
		currencies_settings: BondedCurrenciesSettingsOf::<T> {
			denomination,
			min_operation_balance,
			transferable,
			allow_reset_team: true,
		},
	}
}

pub struct InnerMigrateV0ToV1<T: crate::Config>(core::marker::PhantomData<T>);

impl<T: crate::Config> OnRuntimeUpgrade for InnerMigrateV0ToV1<T>
where
	T::PoolId: sp_std::fmt::Debug,
{
	/// Return a vector of existing [`crate::Pools`] values so we can check that
	/// they were correctly set in `InnerMigrateV0ToV1::post_upgrade`.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, sp_runtime::TryRuntimeError> {
		// Access the old value using the `storage_alias` type
		let old_value: sp_std::vec::Vec<(T::PoolId, v0::PoolDetailsOf<T>)> = v0::Pools::<T>::iter().collect();
		// Return it as an encoded `Vec<u8>`
		Ok(old_value.encode())
	}

	/// Migrate the storage from V0 to V1.
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let mut translated = 0u64;
		// Read values in-place
		crate::Pools::<T>::translate_values::<v0::PoolDetailsOf<T>, _>(|old_value| {
			translated.saturating_inc();
			Some(v0_to_v1::<T>(old_value))
		});

		// One read for taking the old value, and one write for setting the new value
		T::DbWeight::get().reads_writes(translated, translated)
	}

	/// Verifies the storage was migrated correctly.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: sp_std::vec::Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		use sp_runtime::traits::SaturatedConversion;

		let old_values = sp_std::vec::Vec::<(T::PoolId, v0::PoolDetailsOf<T>)>::decode(&mut &state[..])
			.map_err(|_| sp_runtime::TryRuntimeError::Other("Failed to decode old value from storage"))?;

		let prev_count: u32 = old_values.len().saturated_into();
		let post_count: u32 = crate::Pools::<T>::iter().count().saturated_into();

		ensure!(
			prev_count == post_count,
			"the pool count before and after the migration should be the same"
		);

		log::info!(target: LOG_TARGET, "Migrated {} pool entries", post_count);

		old_values.into_iter().try_for_each(|(pool_id, old_value)| {
			let expected_new_value = v0_to_v1::<T>(old_value);
			let actual_new_value = crate::Pools::<T>::get(&pool_id);

			ensure!(actual_new_value.is_some(), {
				log::error!(target: LOG_TARGET, "Expected pool with id {:?} but found none", &pool_id);
				sp_runtime::TryRuntimeError::Other("Pool not migrated")
			});
			ensure!(actual_new_value == Some(expected_new_value), {
				log::error!(target: LOG_TARGET, "Pool with id {:?} contains unexpected data", &pool_id);
				sp_runtime::TryRuntimeError::Other("Incorrect Pool Data")
			});

			ensure!(actual_new_value.unwrap().currencies_settings.allow_reset_team, {
				log::error!(target: LOG_TARGET, "Pool with id {:?} has allow_reset_team = false", &pool_id);
				sp_runtime::TryRuntimeError::Other(
					"all migrated pools should have the allow_reset_team flag set to true",
				)
			});

			Ok(())
		})
	}
}

pub type MigrateV0ToV1<T> = frame_support::migrations::VersionedMigration<
	0, // The migration will only execute when the on-chain storage version is 0
	1, // The on-chain storage version will be set to 1 after the migration is complete
	InnerMigrateV0ToV1<T>,
	crate::pallet::Pallet<T>,
	<T as frame_system::Config>::DbWeight,
>;
