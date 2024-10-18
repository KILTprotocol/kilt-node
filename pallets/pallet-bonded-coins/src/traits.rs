use frame_support::dispatch::DispatchResult;

use crate::Config;

pub trait Freeze<T: Config> {
	fn freeze_asset(who: T::AccountId, asset_id: T::AssetId) -> DispatchResult;
}
