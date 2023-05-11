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

use crate::{Banned, Config, Names, Owner, Web3NameOf, Web3NameOwnerOf, Web3OwnershipOf};

pub fn do_try_state<T: Config>() -> Result<(), &'static str> {
	// check if for each owner there is a name stored.
	Owner::<T>::iter().try_for_each(
		|(w3n, ownership): (Web3NameOf<T>, Web3OwnershipOf<T>)| -> Result<(), &'static str> {
			ensure!(Names::<T>::contains_key(ownership.owner.clone()), "W3n unknown");
			ensure!(Names::<T>::get(ownership.owner) == Some(w3n), "Unequal w3n");
			Ok(())
		},
	)?;

	// check for each name there is an owner.
	Names::<T>::iter().try_for_each(
		|(w3n_owner, w3n): (Web3NameOwnerOf<T>, Web3NameOf<T>)| -> Result<(), &'static str> {
			ensure!(Owner::<T>::contains_key(&w3n), "Owner unknown");
			ensure!(Owner::<T>::get(w3n).unwrap().owner == w3n_owner, "Unequal owner");
			Ok(())
		},
	)?;
	// a banned name should have no owner.
	Banned::<T>::iter_keys().try_for_each(|banned_w3n| -> Result<(), &'static str> {
		ensure!(!Owner::<T>::contains_key(banned_w3n), "W3n: Banned name is owned.");
		Ok(())
	})
}
