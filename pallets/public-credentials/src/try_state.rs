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

use crate::{Config, CredentialSubjects, Credentials};

pub(crate) fn do_try_state<T: Config>() -> Result<(), &'static str> {
	Credentials::<T>::iter().try_for_each(|(subject_id, credential_id, entry)| -> Result<(), &'static str> {
		ensure!(
			CredentialSubjects::<T>::contains_key(&credential_id),
			"Unknown credential subject"
		);

		ensure!(
			CredentialSubjects::<T>::get(&credential_id) == Some(subject_id),
			"Unequal credential subject"
		);

		ensure!(ctype::Ctypes::<T>::contains_key(entry.ctype_hash), "Unknown ctype");

		Ok(())
	})?;

	CredentialSubjects::<T>::iter().try_for_each(|(credential_id, subject_id)| -> Result<(), &'static str> {
		ensure!(
			Credentials::<T>::contains_key(subject_id, credential_id),
			"Unknown credential"
		);
		Ok(())
	})
}
