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

use crate::{Config, CredentialSubjects, Credentials};
use frame_support::ensure;
use kilt_support::test_utils::log_and_return_error_message;
use scale_info::prelude::format;
use sp_runtime::TryRuntimeError;

pub(crate) fn do_try_state<T: Config>() -> Result<(), TryRuntimeError> {
	Credentials::<T>::iter().try_for_each(|(subject_id, credential_id, entry)| -> Result<(), TryRuntimeError> {
		ensure!(
			CredentialSubjects::<T>::get(&credential_id) == Some(subject_id.clone()),
			log_and_return_error_message(format!(
				"Credential subject does not match. Credential id: {:?}. Subject id: {:?}",
				credential_id, subject_id
			))
		);

		ensure!(
			ctype::Ctypes::<T>::contains_key(entry.ctype_hash),
			log_and_return_error_message(format!("Unknown Ctype: {:?}", entry.ctype_hash))
		);

		Ok(())
	})?;

	CredentialSubjects::<T>::iter().try_for_each(|(credential_id, subject_id)| -> Result<(), TryRuntimeError> {
		ensure!(
			Credentials::<T>::contains_key(subject_id, &credential_id),
			log_and_return_error_message(format!("Unknown credential {:?}", credential_id))
		);
		Ok(())
	})
}
