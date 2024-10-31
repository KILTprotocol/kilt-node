use sp_runtime::DispatchError;

pub trait FreezeAccounts<AccountId, AssetId> {
	type Error: Into<DispatchError>;

	fn freeze(asset_id: &AssetId, who: &AccountId) -> Result<(), Self::Error>;

	fn thaw(asset_id: &AssetId, who: &AccountId) -> Result<(), Self::Error>;
}
