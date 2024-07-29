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

use xcm::VersionedLocation;

use crate::{Config, LocalCurrencyBalanceOf};

/// Runtime-injected logic into the switch pallet around the time a switch takes
/// place.
pub trait SwitchHooks<T, I>
where
	T: Config<I>,
	I: 'static,
{
	type Error: Into<u8>;

	/// Called before anything related to a switch happens.
	fn pre_local_to_remote_switch(
		from: &T::AccountId,
		to: &VersionedLocation,
		amount: LocalCurrencyBalanceOf<T, I>,
	) -> Result<(), Self::Error>;

	/// Called after the switch takes place and **after** the XCM message has
	/// been sent to the reserve location.
	fn post_local_to_remote_switch(
		from: &T::AccountId,
		to: &VersionedLocation,
		amount: LocalCurrencyBalanceOf<T, I>,
	) -> Result<(), Self::Error>;

	/// Called upon receiving an XCM message from the reserve location to
	/// deposit some of the remote assets into a specified account, but before
	/// the asset is actually deposited by the asset transactor.
	fn pre_remote_to_local_switch(to: &T::AccountId, amount: u128) -> Result<(), Self::Error>;

	/// Same as [Self::pre_remote_to_local_switch], but called after the
	/// transactor has deposited the incoming remote asset.
	fn post_remote_to_local_switch(to: &T::AccountId, amount: u128) -> Result<(), Self::Error>;
}

impl<T, I> SwitchHooks<T, I> for ()
where
	T: Config<I>,
	I: 'static,
{
	type Error = u8;

	fn pre_local_to_remote_switch(
		_from: &T::AccountId,
		_to: &VersionedLocation,
		_amount: LocalCurrencyBalanceOf<T, I>,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn post_local_to_remote_switch(
		_from: &T::AccountId,
		_to: &VersionedLocation,
		_amount: LocalCurrencyBalanceOf<T, I>,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn pre_remote_to_local_switch(_to: &<T>::AccountId, _amount: u128) -> Result<(), Self::Error> {
		Ok(())
	}

	fn post_remote_to_local_switch(_to: &<T>::AccountId, _amount: u128) -> Result<(), Self::Error> {
		Ok(())
	}
}
