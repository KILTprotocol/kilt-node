// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.org>

use frame_support::ensure;
use kilt_support::test_utils::log_and_return_error_message;
use scale_info::prelude::format;
use sp_runtime::{
	traits::{Get, One},
	TryRuntimeError,
};

use crate::{Config, ConnectedAccounts, ConnectedDids, ConnectionRecord};

pub(crate) fn do_try_state<T: Config<I>, I: 'static>() -> Result<(), TryRuntimeError> {
	// Verify DID -> account link integrity.
	ConnectedDids::<T, I>::iter().try_for_each(|(account, record)| -> Result<(), TryRuntimeError> {
		ensure!(
			ConnectedAccounts::<T, I>::contains_key(&record.did, &account),
			log_and_return_error_message(format!("Account {:?} with did {:?} not found", record.did, account))
		);
		Ok(())
	})?;

	// Verify account -> DID link integrity.
	ConnectedAccounts::<T, I>::iter().try_for_each(
		|(did_identifier, linked_account_id, _)| -> Result<(), TryRuntimeError> {
			ensure!(
				ConnectedDids::<T, I>::get(&linked_account_id).expect("Unknown did").did == did_identifier,
				log_and_return_error_message(format!(
					"Linked Account {:?} for did {:?} not match",
					linked_account_id, did_identifier
				))
			);
			Ok(())
		},
	)?;

	// Verify account <-> DID link unicity.
	if <T as Config<I>>::UniqueLinkingEnabled::get() {
		let mut did_linked_to_accounts = ConnectedDids::<T, I>::iter_values().map(|ConnectionRecord { did, .. }| did);

		did_linked_to_accounts.try_for_each(|did_identifier| -> Result<(), TryRuntimeError> {
			let linked_accounts = ConnectedAccounts::<T, I>::iter_key_prefix(&did_identifier).count();
			ensure!(
				linked_accounts.is_one(),
				log_and_return_error_message(format!(
					"DID {:?} has more than a single account linked: {:?}.",
					did_identifier, linked_accounts
				))
			);
			Ok(())
		})?;
	}

	Ok(())
}
