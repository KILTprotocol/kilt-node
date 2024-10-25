use frame_support::dispatch::DispatchResult;

pub trait FreezeAccounts<AccountId, AssetId> {
	fn freeze(caller: AccountId, who: AccountId, asset_id: AssetId) -> DispatchResult;

	fn thaw(caller: AccountId, who: AccountId, asset_id: AssetId) -> DispatchResult;
}
