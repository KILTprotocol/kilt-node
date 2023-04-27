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

use crate::{errors, Config};
use frame_support::{ensure, traits::Get, BoundedVec, RuntimeDebug};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::traits::SaturatedConversion;
use sp_std::str;
#[cfg(any(test, feature = "runtime-benchmarks"))]
use sp_std::{convert::TryInto, vec::Vec};

use crate::utils as crate_utils;

/// A bounded vector of bytes for a service endpoint ID.
pub type ServiceEndpointId<T> = BoundedVec<u8, <T as Config>::MaxServiceIdLength>;

/// A bounded vectors of bytes for a service endpoint type.
pub(crate) type ServiceEndpointType<T> = BoundedVec<u8, <T as Config>::MaxServiceTypeLength>;
/// A bounded vector of [ServiceEndpointType]s.
pub(crate) type ServiceEndpointTypeEntries<T> =
	BoundedVec<ServiceEndpointType<T>, <T as Config>::MaxNumberOfTypesPerService>;

/// A bounded vectors of bytes for a service endpoint URL.
pub(crate) type ServiceEndpointUrl<T> = BoundedVec<u8, <T as Config>::MaxServiceUrlLength>;
/// A bounded vector of [ServiceEndpointUrl]s.
pub(crate) type ServiceEndpointUrlEntries<T> =
	BoundedVec<ServiceEndpointUrl<T>, <T as Config>::MaxNumberOfUrlsPerService>;

/// A single service endpoint description.
#[derive(Clone, Decode, RuntimeDebug, Encode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct DidEndpoint<T: Config> {
	/// The ID of the service endpoint. Allows the endpoint to be queried and
	/// resolved directly.
	pub id: ServiceEndpointId<T>,
	/// A vector of types description for the service.
	pub service_types: ServiceEndpointTypeEntries<T>,
	/// A vector of URLs the service points to.
	pub urls: ServiceEndpointUrlEntries<T>,
}

impl<T: Config> DidEndpoint<T> {
	/// Validates a given [DidEndpoint] instance against the constraint
	/// set in the pallet's [Config].
	pub(crate) fn validate_against_constraints(&self) -> Result<(), errors::InputError> {
		// Check that the maximum number of service types is provided.
		ensure!(
			self.service_types.len() <= T::MaxNumberOfTypesPerService::get().saturated_into(),
			errors::InputError::MaxTypeCountExceeded
		);
		// Check that the maximum number of URLs is provided.
		ensure!(
			self.urls.len() <= T::MaxNumberOfUrlsPerService::get().saturated_into(),
			errors::InputError::MaxUrlCountExceeded
		);
		// Check that the ID is the maximum allowed length and only contain ASCII
		// characters.
		ensure!(
			self.id.len() <= T::MaxServiceIdLength::get().saturated_into(),
			errors::InputError::MaxIdLengthExceeded
		);
		let str_id = str::from_utf8(&self.id).map_err(|_| errors::InputError::InvalidEncoding)?;
		ensure!(
			crate_utils::is_valid_ascii_string(str_id),
			errors::InputError::InvalidEncoding
		);
		// Check that all types are the maximum allowed length and only contain ASCII
		// characters.
		self.service_types.iter().try_for_each(|s_type| {
			ensure!(
				s_type.len() <= T::MaxServiceTypeLength::get().saturated_into(),
				errors::InputError::MaxTypeLengthExceeded
			);
			let str_type = str::from_utf8(s_type).map_err(|_| errors::InputError::InvalidEncoding)?;
			ensure!(
				crate_utils::is_valid_ascii_string(str_type),
				errors::InputError::InvalidEncoding
			);
			Ok(())
		})?;
		// Check that all URLs are the maximum allowed length AND only contain ASCII
		// characters.
		for s_url in self.urls.iter() {
			ensure!(
				s_url.len() <= T::MaxServiceUrlLength::get().saturated_into(),
				errors::InputError::MaxUrlLengthExceeded
			);
			let str_url = str::from_utf8(s_url).map_err(|_| errors::InputError::InvalidEncoding)?;
			ensure!(
				crate_utils::is_valid_ascii_string(str_url),
				errors::InputError::InvalidEncoding
			);
		}
		Ok(())
	}
}

#[cfg(any(test, feature = "runtime-benchmarks"))]
impl<T: Config> DidEndpoint<T> {
	pub(crate) fn new(id: Vec<u8>, types: Vec<Vec<u8>>, urls: Vec<Vec<u8>>) -> Self {
		let bounded_id = id.try_into().expect("Service ID too long.");
		let bounded_types = types
			.iter()
			.map(|el| el.to_vec().try_into().expect("Service type too long."))
			.collect::<Vec<ServiceEndpointType<T>>>()
			.try_into()
			.expect("Too many types for the given service.");
		let bounded_urls = urls
			.iter()
			.map(|el| el.to_vec().try_into().expect("Service URL too long."))
			.collect::<Vec<ServiceEndpointUrl<T>>>()
			.try_into()
			.expect("Too many URLs for the given service.");

		Self {
			id: bounded_id,
			service_types: bounded_types,
			urls: bounded_urls,
		}
	}
}

pub mod utils {
	use super::*;

	pub(crate) fn validate_new_service_endpoints<T: Config>(
		endpoints: &[DidEndpoint<T>],
	) -> Result<(), errors::InputError> {
		// Check if up the maximum number of endpoints is provided.
		ensure!(
			endpoints.len() <= T::MaxNumberOfServicesPerDid::get().saturated_into(),
			errors::InputError::MaxServicesCountExceeded
		);

		// Then validate each service.
		endpoints
			.iter()
			.try_for_each(DidEndpoint::<T>::validate_against_constraints)?;

		Ok(())
	}
}
