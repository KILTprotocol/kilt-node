// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

// The KILT Blockchain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The KILT Blockchain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// If you feel like getting in touch with us, you can do so at info@botlabs.org

use frame_support::ensure;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::TryRuntimeError;

use crate::{Config, LocalCurrencyBalanceOf, Pallet, SwitchPair};

pub(crate) fn try_state<T, I>(_n: BlockNumberFor<T>) -> Result<(), TryRuntimeError>
where
	T: Config<I>,
	I: 'static,
	LocalCurrencyBalanceOf<T, I>: Into<u128>,
{
	let Some(switch_pair) = SwitchPair::<T, I>::get() else {
		return Ok(());
	};
	ensure!(
		switch_pair.remote_asset_circulating_supply
			<= Pallet::<T, I>::get_pool_usable_balance(&switch_pair.pool_account).into(),
		TryRuntimeError::Other("Circulating supply less than the switch pool account.")
	);
	ensure!(
		switch_pair.reducible_remote_balance() + switch_pair.remote_asset_ed
			== switch_pair.remote_asset_total_supply - switch_pair.remote_asset_circulating_supply,
		TryRuntimeError::Other("Locked supply does not match the tracked balance.")
	);
	Ok(())
}
