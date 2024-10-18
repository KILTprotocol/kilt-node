use frame_support::{dispatch::DispatchResult, traits::fungibles::Inspect};

use crate::Config;

pub trait Freeze<T: Config> {
	fn freeze_asset(who: T::AccountId, asset_id: T::AssetId) -> DispatchResult;
}

/// Copied from the Polkadot SDK. For version 1.7.0, the trait is not yet implemented.
/// /// This should be removed once the pallet_asset includes the implementation.
/// Trait for resetting the team configuration of an existing fungible asset.
pub trait ResetTeam<AccountId>: Inspect<AccountId> {
	/// Reset the team for the asset with the given `id`.
	///
	/// ### Parameters
	/// - `id`: The identifier of the asset for which the team is being reset.
	/// - `owner`: The new `owner` account for the asset.
	/// - `admin`: The new `admin` account for the asset.
	/// - `issuer`: The new `issuer` account for the asset.
	/// - `freezer`: The new `freezer` account for the asset.
	fn reset_team(
		id: Self::AssetId,
		owner: AccountId,
		admin: AccountId,
		issuer: AccountId,
		freezer: AccountId,
	) -> DispatchResult;
}
