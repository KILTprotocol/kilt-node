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

use pallet_asset_switch::traits::SwitchHooks;
use runtime_common::{AccountId, Balance};
use xcm::{
	v4::{Junction, Junctions, Location},
	VersionedLocation,
};

use crate::{KiltToEKiltSwitchPallet, Runtime};

const LOG_TARGET: &str = "runtime::peregrine::asset-switch::RestrictTransfersToSameUser";

/// Check requiring the beneficiary be a single `AccountId32` junction
/// containing the same account ID as the account on this chain initiating the
/// switch.
pub struct RestrictswitchDestinationToSelf;

impl SwitchHooks<Runtime, KiltToEKiltSwitchPallet> for RestrictswitchDestinationToSelf {
	type Error = Error;

	fn pre_local_to_remote_switch(
		from: &AccountId,
		to: &VersionedLocation,
		_amount: Balance,
	) -> Result<(), Self::Error> {
		let to_as: Location = to.clone().try_into().map_err(|e| {
			log::error!(
				target: LOG_TARGET,
				"Failed to convert beneficiary Location {:?} to v4 with error {:?}",
				to,
				e
			);
			Error::Internal
		})?;

		let junctions: Junctions = [Junction::AccountId32 {
			network: None,
			id: from.clone().into(),
		}]
		.into();
		let is_beneficiary_self = to_as.interior == junctions;
		cfg_if::cfg_if! {
			if #[cfg(feature = "runtime-benchmarks")] {
				// Clippy complaints the variable is not used with this feature on, otherwise.
				let _ = is_beneficiary_self;
				Ok(())
			} else {
				frame_support::ensure!(is_beneficiary_self, Error::NotToSelf);
				Ok(())
			}
		}
	}

	// We don't need to take any actions after the switch is executed
	fn post_local_to_remote_switch(
		_from: &AccountId,
		_to: &VersionedLocation,
		_amount: Balance,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn pre_remote_to_local_switch(_to: &AccountId, _amount: u128) -> Result<(), Self::Error> {
		Ok(())
	}

	fn post_remote_to_local_switch(_to: &AccountId, _amount: u128) -> Result<(), Self::Error> {
		Ok(())
	}
}

#[cfg_attr(test, derive(enum_iterator::Sequence))]
pub enum Error {
	NotToSelf,
	Internal,
}

impl From<Error> for u8 {
	fn from(value: Error) -> Self {
		match value {
			Error::NotToSelf => 0,
			Error::Internal => Self::MAX,
		}
	}
}

#[test]
fn error_value_not_duplicated() {
	enum_iterator::all::<Error>().fold(
		sp_std::collections::btree_set::BTreeSet::<u8>::new(),
		|mut values, new_value| {
			let new_encoded_value = u8::from(new_value);
			assert!(
				values.insert(new_encoded_value),
				"Failed to add unique value {:#?} for error variant",
				new_encoded_value
			);
			values
		},
	);
}
