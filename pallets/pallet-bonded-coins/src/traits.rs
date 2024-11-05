use frame_support::{dispatch::DispatchResult, traits::fungibles::Inspect};
use sp_runtime::DispatchError;

pub trait FreezeAccounts<AccountId, AssetId> {
	type Error: Into<DispatchError>;

	fn freeze(asset_id: &AssetId, who: &AccountId) -> Result<(), Self::Error>;

	fn thaw(asset_id: &AssetId, who: &AccountId) -> Result<(), Self::Error>;
}

/// Copy from the Polkadot SDK. once we are at version 1.13.0, we can remove
/// this.
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
