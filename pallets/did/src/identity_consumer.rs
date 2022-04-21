// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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

use kilt_support::traits::{IdentityIncrementer, IdentityDecrementer, IdentityConsumer};

use crate::{Config, Pallet, Error, DidIdentifierOf, WeightInfo};

pub struct DidIncrementer<T>(T);

impl<T: Config, Identity> IdentityIncrementer for DidIncrementer<(Option<T>, Identity)>
	where Identity: AsRef<DidIdentifierOf<T>> {
    fn increment(&self) -> frame_support::dispatch::Weight {
		Pallet::<T>::increment_consumers_unsafe(self.0.1.as_ref());
        T::WeightInfo::increment_consumers()
    }
}

pub struct DidDecrementer<T>(T);

impl<T: Config, Identity> IdentityDecrementer for DidDecrementer<(Option<T>, Identity)>
	where Identity: AsRef<DidIdentifierOf<T>> {
    fn decrement(&self) -> frame_support::dispatch::Weight {
        Pallet::<T>::decrement_consumers_unsafe(self.0.1.as_ref());
		T::WeightInfo::decrement_consumers()
    }
}

impl<T: Config, Identity> IdentityConsumer<Identity> for Pallet<T>
	where Identity: AsRef<DidIdentifierOf<T>> + Clone {
    type IdentityIncrementer = DidIncrementer<(Option<T>, Identity)>;
    type IdentityDecrementer = DidDecrementer<(Option<T>, Identity)>;
    type Error = Error<T>;

    fn get_incrementer(id: &Identity) -> Result<Self::IdentityIncrementer, Self::Error> {
		if Self::can_increment_consumers(id.as_ref()) {
			Ok(DidIncrementer((None, id.clone())))
		} else {
			Err(Self::Error::MaxConsumersExceeded)
		}
    }

    fn get_incrementer_max_weight() -> frame_support::dispatch::Weight {
        T::WeightInfo::increment_consumers()
    }

    fn get_decrementer(id: &Identity) -> Result<Self::IdentityDecrementer, Self::Error> {
        if Self::can_decrement_consumers(id.as_ref()) {
			Ok(DidDecrementer((None, id.clone())))
		} else {
			Err(Self::Error::NoOutstandingConsumers)
		}
    }

    fn get_decrementer_max_weight() -> frame_support::dispatch::Weight {
        T::WeightInfo::decrement_consumers()
    }
}
