// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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
use kilt_support::test_utils::log_and_return_error_message;
use scale_info::prelude::format;

use crate::{Config, ConnectedAccounts, ConnectedDids};

pub(crate) fn do_try_state<T: Config>() -> Result<(), &'static str> {
	ConnectedDids::<T>::iter().try_for_each(|(account, record)| -> Result<(), &'static str> {
		ensure!(
			ConnectedAccounts::<T>::contains_key(&record.did, &account),
			log_and_return_error_message(format!("Account {:?} with did {:?} not found", record.did, account))
		);
		Ok(())
	})?;

	ConnectedAccounts::<T>::iter().try_for_each(|(did_identifier, linked_account_id, _)| -> Result<(), &'static str> {
		ensure!(
			ConnectedDids::<T>::get(&linked_account_id).expect("Unknown did").did == did_identifier,
			log_and_return_error_message(format!(
				"Linked Account {:?} for did {:?} not match",
				linked_account_id, did_identifier
			))
		);
		Ok(())
	})
}
