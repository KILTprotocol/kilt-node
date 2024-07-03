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

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;
use xcm::{VersionedAssetId, VersionedMultiAsset, VersionedMultiLocation};

/// Information related to a switch pair.
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, RuntimeDebug, Clone)]
pub struct SwitchPairInfo<AccountId> {
	/// The address that will hold the local tokens locked in return for the
	/// remote asset.
	pub pool_account: AccountId,
	/// The balance of the remote (fungible) asset for the chain sovereign
	/// account on the configured `remote_reserve_location`.
	pub remote_asset_balance: u128,
	/// The ID of the remote asset to switch 1:1 with the local token.
	pub remote_asset_id: VersionedAssetId,
	/// The assets to take from the user's balance on this chain to pay for XCM
	/// execution fees on the reserve location.
	pub remote_fee: VersionedMultiAsset,
	/// The location of the location on which the remote asset lives.
	pub remote_reserve_location: VersionedMultiLocation,
	/// The status of the switch pair.
	pub status: SwitchPairStatus,
}

/// All statues a switch pool can be in at any given time.
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, RuntimeDebug, Clone, Default)]
pub enum SwitchPairStatus {
	/// Switches are not enabled.
	#[default]
	Paused,
	/// Switches are enabled.
	Running,
}

impl<AccountId> SwitchPairInfo<AccountId> {
	pub(crate) fn can_switch(&self) -> bool {
		matches!(self.status, SwitchPairStatus::Running)
	}
}
