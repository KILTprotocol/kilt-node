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

use xcm::VersionedMultiLocation;

use crate::{Config, LocalCurrencyBalanceOf};

pub trait SwapHooks<T>
where
	T: Config,
{
	type Error: Into<u8>;

	fn pre_local_to_remote_swap(
		from: &T::AccountId,
		to: &VersionedMultiLocation,
		amount: LocalCurrencyBalanceOf<T>,
	) -> Result<(), Self::Error>;

	fn post_local_to_remote_swap(
		from: &T::AccountId,
		to: &VersionedMultiLocation,
		amount: LocalCurrencyBalanceOf<T>,
	) -> Result<(), Self::Error>;

	fn pre_remote_to_local_swap(to: &T::AccountId, amount: u128) -> Result<(), Self::Error>;

	fn post_remote_to_local_swap(to: &T::AccountId, amount: u128) -> Result<(), Self::Error>;
}

impl<T> SwapHooks<T> for ()
where
	T: Config,
{
	type Error = u8;

	fn pre_local_to_remote_swap(
		_from: &T::AccountId,
		_to: &VersionedMultiLocation,
		_amount: LocalCurrencyBalanceOf<T>,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn post_local_to_remote_swap(
		_from: &T::AccountId,
		_to: &VersionedMultiLocation,
		_amount: LocalCurrencyBalanceOf<T>,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn pre_remote_to_local_swap(_to: &<T>::AccountId, _amount: u128) -> Result<(), Self::Error> {
		Ok(())
	}

	fn post_remote_to_local_swap(_to: &<T>::AccountId, _amount: u128) -> Result<(), Self::Error> {
		Ok(())
	}
}
