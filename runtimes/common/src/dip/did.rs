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

use did::did_details::DidDetails;
use pallet_dip_provider::traits::IdentityProvider;
use sp_std::marker::PhantomData;

pub struct DidIdentityProvider<T>(PhantomData<T>);

impl<T> IdentityProvider<T::DidIdentifier, DidDetails<T>, ()> for DidIdentityProvider<T>
where
	T: did::Config,
{
	// TODO: Proper error handling
	type Error = ();

	fn retrieve(identifier: &T::DidIdentifier) -> Result<Option<(DidDetails<T>, ())>, Self::Error> {
		match (
			did::Pallet::<T>::get_did(identifier),
			did::Pallet::<T>::get_deleted_did(identifier),
		) {
			(Some(details), _) => Ok(Some((details, ()))),
			(_, Some(_)) => Ok(None),
			_ => Err(()),
		}
	}
}
