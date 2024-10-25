use sp_runtime::DispatchError;

pub trait FreezeAccounts<AccountId, AssetId> {
	type Error: Into<DispatchError>;

	fn freeze(caller: &AccountId, who: &AccountId, asset_id: &AssetId) -> Result<(), Self::Error>;

	fn thaw(caller: &AccountId, who: &AccountId, asset_id: &AssetId) -> Result<(), Self::Error>;
}
