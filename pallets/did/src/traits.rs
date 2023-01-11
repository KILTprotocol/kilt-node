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

use codec::{Decode, Encode};
use frame_support::{weights::Weight, RuntimeDebug};
use scale_info::TypeInfo;
use sp_runtime::DispatchError;
use xcm::v2::MultiLocation;

use crate::{did_details::DidDetails, Config, DidIdentifierOf};

pub trait DidDocumentHasher<DidIdentifier, DidDetails, Output> {
	const MAX_WEIGHT: Weight;

	fn calculate_root(did: &DidIdentifier, details: &DidDetails) -> Result<(Output, Weight), DispatchError>;
}

impl<T: Config> DidDocumentHasher<DidIdentifierOf<T>, DidDetails<T>, T::Hash> for () {
	const MAX_WEIGHT: Weight = Weight::zero();

	fn calculate_root(
		_did: &DidIdentifierOf<T>,
		_details: &DidDetails<T>,
	) -> Result<(T::Hash, Weight), sp_runtime::DispatchError> {
		Ok((<T as frame_system::Config>::Hash::default(), Weight::zero()))
	}
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum DidRootStateAction<DidIdentifier, Root> {
	Updated(DidIdentifier, Root),
	Deleted(DidIdentifier),
}

pub trait DidRootDispatcher<DidIdentifier, Root, Location> {
	const MAX_WEIGHT: Weight;

	fn dispatch(action: DidRootStateAction<DidIdentifier, Root>, location: Location) -> Result<Weight, DispatchError>;
}

pub struct NullDispatcher<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> DidRootDispatcher<DidIdentifierOf<T>, T::Hash, MultiLocation> for NullDispatcher<T> {
	const MAX_WEIGHT: frame_support::weights::Weight = Weight::zero();

	fn dispatch(
		_action: DidRootStateAction<DidIdentifierOf<T>, T::Hash>,
		_location: MultiLocation,
	) -> Result<Weight, DispatchError> {
		Ok(Weight::zero())
	}
}
