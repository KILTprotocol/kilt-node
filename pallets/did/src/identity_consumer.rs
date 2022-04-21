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

use kilt_support::traits::{IdentityConsumer, IdentityCounter, IdentityDecrementer, IdentityIncrementer};
use sp_runtime::DispatchError;

use crate::{Config, DidConsumers, DidIdentifierOf, Error, Pallet, WeightInfo};

#[derive(Debug, PartialEq)]
pub struct DidIncrementer<T>(T);

impl<T: Config, Identity> IdentityCounter<u32> for DidIncrementer<(Option<T>, Identity)>
where
	// FIXME: remove dependency
	// on Clone, and change Into
	// to AsRef, if possible
	Identity: Into<DidIdentifierOf<T>> + Clone,
{
	fn current_value(&self) -> u32 {
		DidConsumers::<T>::get(&self.0 .1.clone().into())
	}
}

impl<T: Config, Identity> IdentityIncrementer<u32> for DidIncrementer<(Option<T>, Identity)>
where
	// FIXME: remove dependency
	// on Clone, and change Into
	// to AsRef, if possible
	Identity: Into<DidIdentifierOf<T>> + Clone,
{
	fn increment(&mut self) -> frame_support::dispatch::Weight {
		Pallet::<T>::increment_consumers_unsafe(&self.0 .1.clone().into());
		T::WeightInfo::increment_consumers()
	}
}

#[derive(Debug, PartialEq)]
pub struct DidDecrementer<T>(T);

impl<T: Config, Identity> IdentityCounter<u32> for DidDecrementer<(Option<T>, Identity)>
where
	// FIXME: remove dependency
	// on Clone, and change Into
	// to AsRef, if possible
	Identity: Into<DidIdentifierOf<T>> + Clone,
{
	fn current_value(&self) -> u32 {
		DidConsumers::<T>::get(&self.0 .1.clone().into())
	}
}

impl<T: Config, Identity> IdentityDecrementer<u32> for DidDecrementer<(Option<T>, Identity)>
where
	// FIXME: remove dependency
	// on Clone, and change Into
	// to AsRef, if possible
	Identity: Into<DidIdentifierOf<T>> + Clone,
{
	fn decrement(&mut self) -> frame_support::dispatch::Weight {
		Pallet::<T>::decrement_consumers_unsafe(&self.0 .1.clone().into());
		T::WeightInfo::decrement_consumers()
	}
}

impl<T: Config, Identity> IdentityConsumer<Identity, u32> for Pallet<T>
where
	// FIXME: remove dependency on
	// Clone, and change Into to
	// AsRef, if possible
	Identity: Into<DidIdentifierOf<T>> + Clone,
{
	type IdentityIncrementer = DidIncrementer<(Option<T>, Identity)>;
	type IdentityDecrementer = DidDecrementer<(Option<T>, Identity)>;
	type Error = DispatchError;

	fn get_incrementer(id: &Identity) -> Result<Self::IdentityIncrementer, Self::Error> {
		if Self::can_increment_consumers(&id.clone().into()) {
			Ok(DidIncrementer((None, id.clone())))
		} else {
			Err(Error::<T>::MaxConsumersExceeded.into())
		}
	}

	fn get_incrementer_max_weight() -> frame_support::dispatch::Weight {
		T::WeightInfo::increment_consumers()
	}

	fn get_decrementer(id: &Identity) -> Result<Self::IdentityDecrementer, Self::Error> {
		if Self::can_decrement_consumers(&id.clone().into()) {
			Ok(DidDecrementer((None, id.clone())))
		} else {
			Err(Error::<T>::NoOutstandingConsumers.into())
		}
	}

	fn get_decrementer_max_weight() -> frame_support::dispatch::Weight {
		T::WeightInfo::decrement_consumers()
	}
}

#[cfg(test)]
mod test {

	use frame_support::assert_noop;
	use sp_core::Pair;

	use kilt_support::traits::{IdentityConsumer, IdentityDecrementer, IdentityIncrementer};
	use sp_runtime::DispatchError;

	use crate::{
		did_details::DidVerificationKey,
		mock::{get_did_identifier_from_ed25519_key, get_ed25519_authentication_key, ExtBuilder, Test},
		mock_utils::generate_base_did_details,
		DidConsumers, Error, Pallet,
	};

	#[test]
	fn incrementer_ok() {
		let auth_key = get_ed25519_authentication_key(true);
		let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
		let did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()));

		ExtBuilder::default()
			.with_dids(vec![(alice_did.clone(), did_details)])
			.build(None)
			.execute_with(|| {
				assert_eq!(DidConsumers::<Test>::get(&alice_did), 0);
				let mut incrementer =
					Pallet::<Test>::get_incrementer(&alice_did).expect("get_incrementer should not fail.");
				incrementer.increment();
				assert_eq!(DidConsumers::<Test>::get(&alice_did), 1);
			});
	}

	#[test]
	fn incrementer_max_limit() {
		let auth_key = get_ed25519_authentication_key(true);
		let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
		let did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()));

		ExtBuilder::default()
			.with_dids(vec![(alice_did.clone(), did_details)])
			.with_consumers(vec![(alice_did.clone(), u32::MAX)])
			.build(None)
			.execute_with(|| {
				assert_noop!(
					Pallet::<Test>::get_incrementer(&alice_did),
					DispatchError::from(Error::<Test>::MaxConsumersExceeded),
				);
			});
	}

	#[test]
	fn decrementer_ok() {
		let auth_key = get_ed25519_authentication_key(true);
		let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
		let did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()));

		ExtBuilder::default()
			.with_dids(vec![(alice_did.clone(), did_details)])
			.with_consumers(vec![(alice_did.clone(), u32::MAX)])
			.build(None)
			.execute_with(|| {
				assert_eq!(DidConsumers::<Test>::get(&alice_did), u32::MAX);
				let mut decrementer =
					Pallet::<Test>::get_decrementer(&alice_did).expect("get_decrementer should not fail.");
				decrementer.decrement();
				assert_eq!(DidConsumers::<Test>::get(&alice_did), u32::MAX - 1);
			});
	}

	#[test]
	fn decrementer_min_limit() {
		let auth_key = get_ed25519_authentication_key(true);
		let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
		let did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()));

		ExtBuilder::default()
			.with_dids(vec![(alice_did.clone(), did_details)])
			.with_consumers(vec![(alice_did.clone(), 0)])
			.build(None)
			.execute_with(|| {
				assert_noop!(
					Pallet::<Test>::get_decrementer(&alice_did),
					DispatchError::from(Error::<Test>::NoOutstandingConsumers),
				);
			});
	}
}
