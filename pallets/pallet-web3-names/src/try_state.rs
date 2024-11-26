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
use kilt_support::test_utils::log_and_return_error_message;
use scale_info::prelude::format;
use sp_runtime::TryRuntimeError;

use crate::{Banned, Config, Names, Owner, Web3NameOf, Web3NameOwnerOf, Web3OwnershipOf};

pub fn do_try_state<T: Config<I>, I: 'static>() -> Result<(), TryRuntimeError> {
	// check if for each owner there is a name stored.
	Owner::<T, I>::iter().try_for_each(
		|(w3n, ownership): (Web3NameOf<T, I>, Web3OwnershipOf<T, I>)| -> Result<(), TryRuntimeError> {
			ensure!(
				Names::<T, I>::get(&ownership.owner) == Some(w3n.clone()),
				log_and_return_error_message(format!(
					"Owned w3n from owner {:?} does not match with saved w3n {:?}",
					ownership.owner, w3n
				))
			);
			Ok(())
		},
	)?;

	// check for each name there is an owner.
	Names::<T, I>::iter().try_for_each(
		|(w3n_owner, w3n): (Web3NameOwnerOf<T, I>, Web3NameOf<T, I>)| -> Result<(), TryRuntimeError> {
			ensure!(
				Owner::<T, I>::get(&w3n).expect("Unknown w3n").owner == w3n_owner,
				log_and_return_error_message(format!("Owner {:?} with w3n {:?} not found", w3n_owner, w3n))
			);
			Ok(())
		},
	)?;
	// a banned name should have no owner.
	Banned::<T, I>::iter_keys().try_for_each(|banned_w3n| -> Result<(), TryRuntimeError> {
		ensure!(
			!Owner::<T, I>::contains_key(&banned_w3n),
			log_and_return_error_message(format!("Owner contains banned name {:?}", banned_w3n))
		);
		Ok(())
	})
}
