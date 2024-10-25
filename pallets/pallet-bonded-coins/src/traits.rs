pub trait FreezeAccounts<AccountId, AssetId> {
	type Error: Into<u8>;

	fn freeze(caller: &AccountId, who: &AccountId, asset_id: &AssetId) -> Result<(), Self::Error>;

	fn thaw(caller: &AccountId, who: &AccountId, asset_id: &AssetId) -> Result<(), Self::Error>;
}
