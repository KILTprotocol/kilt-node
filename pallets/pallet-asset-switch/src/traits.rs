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

use xcm::{v4::QueryId, VersionedLocation};

use crate::{Config, LocalCurrencyBalanceOf};

/// Runtime-injected logic into the switch pallet around the time a switch takes
/// place.
///
/// The order in which the hooks are called, for any given local -> remote
/// switch is the following:
/// 1. `pre_local_to_remote_switch` (fallible): Called from within the switch
///    pallet before any other logic is executed. E.g., it can be used to verify
///    some requirements about the transfer.
/// 2. `post_local_to_remote_switch_dispatch` (fallible): Called from within the
///    switch pallet after all other logic is executed and after the XCM message
///    for the transfer is prepared to be sent to the remote location. E.g., it
///    can be used to update storage elements of other pallets once the transfer
///    operation is guaranteed to be sent to destination. This function can
///    still fail, in which case the message won't be sent. But otherwise, at
///    this stage the XCM message is guaranteed to at least be deliverable to
///    the destination. It can still fail at destination, which will in turn
///    call the `post_local_to_remote_transfer_revert` hook.
/// 3. `post_local_to_remote_confirmed` (infallible): Called when the remote
///    destination confirms the transfer was successful.
/// 4. `post_local_to_remote_transfer_revert` (infallible): Called when the
///    remote destination signals that a previously local -> remote switch has
///    instead failed.
///
/// The order in which the hooks are called, for any given remote -> local
/// switch is the following:
/// 1. `pre_remote_to_local_switch` (fallible): Called from the XCM components
///    handling incoming transfers representing remote -> local switches. If
///    this hook fails, the assets will most likely trap and need to be unlocked
///    manually.
/// 2. `post_remote_to_local_switch` (fallible): Same as
///    `pre_remote_to_local_switch`, but called after all the XCM logic has been
///    executed and guaranteed to be correct.
pub trait SwitchHooks<T, I>
where
	T: Config<I>,
	I: 'static,
{
	type Error: Into<u8>;

	fn pre_local_to_remote_switch(
		from: &T::AccountId,
		to: &VersionedLocation,
		amount: LocalCurrencyBalanceOf<T, I>,
	) -> Result<(), Self::Error>;

	fn post_local_to_remote_switch_dispatch(
		from: &T::AccountId,
		to: &VersionedLocation,
		amount: LocalCurrencyBalanceOf<T, I>,
	) -> Result<(), Self::Error>;

	fn post_local_to_remote_finalized(
		from: &T::AccountId,
		to: &VersionedLocation,
		amount: LocalCurrencyBalanceOf<T, I>,
	);

	fn post_local_to_remote_transfer_revert(
		from: &T::AccountId,
		to: &VersionedLocation,
		amount: LocalCurrencyBalanceOf<T, I>,
	);

	fn pre_remote_to_local_switch(to: &T::AccountId, amount: u128) -> Result<(), Self::Error>;

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

	fn post_local_to_remote_switch_dispatch(
		_from: &<T>::AccountId,
		_to: &VersionedLocation,
		_amount: LocalCurrencyBalanceOf<T, I>,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn post_local_to_remote_finalized(
		_from: &T::AccountId,
		_to: &VersionedLocation,
		_amount: LocalCurrencyBalanceOf<T, I>,
	) {
	}

	fn post_local_to_remote_transfer_revert(
		_from: &<T>::AccountId,
		_to: &VersionedLocation,
		_amount: LocalCurrencyBalanceOf<T, I>,
	) {
	}

	fn pre_remote_to_local_switch(_to: &<T>::AccountId, _amount: u128) -> Result<(), Self::Error> {
		Ok(())
	}

	fn post_remote_to_local_switch(_to: &<T>::AccountId, _amount: u128) -> Result<(), Self::Error> {
		Ok(())
	}
}

pub trait QueryIdProvider {
	fn next_id() -> QueryId;
}

impl QueryIdProvider for () {
	fn next_id() -> QueryId {
		QueryId::default()
	}
}
