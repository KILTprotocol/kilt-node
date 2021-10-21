// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

use crate::Config;
use codec::{Decode, Encode};
use frame_support::BoundedVec;
use sp_std::str;
#[cfg(any(test, feature = "runtime-benchmarks"))]
use sp_std::{convert::TryInto, vec::Vec};

use crate::utils as crate_utils;

pub type ServiceEndpointId<T> = BoundedVec<u8, <T as Config>::MaxServiceIdLength>;

pub type ServiceEndpointType<T> = BoundedVec<u8, <T as Config>::MaxServiceTypeLength>;
pub type ServiceEndpointTypeEntries<T> = BoundedVec<ServiceEndpointType<T>, <T as Config>::MaxTypeCountPerService>;

pub type ServiceEndpointUrl<T> = BoundedVec<u8, <T as Config>::MaxServiceUrlLength>;
pub type ServiceEndpointUrlEntries<T> = BoundedVec<ServiceEndpointUrl<T>, <T as Config>::MaxUrlCountPerService>;

#[derive(Clone, Decode, Encode, PartialEq, Eq)]
pub struct DidEndpointDetails<T: Config> {
	pub(crate) id: ServiceEndpointId<T>,
	pub(crate) service_type: ServiceEndpointTypeEntries<T>,
	pub(crate) url: ServiceEndpointUrlEntries<T>,
}

impl<T: Config> sp_std::fmt::Debug for DidEndpointDetails<T> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		f.debug_struct("DidEndpointDetails")
			.field("id", &self.id.clone().into_inner())
			.field("service_type", &self.service_type.encode())
			.field("url", &self.url.encode())
			.finish()
	}
}

#[cfg(any(test, feature = "runtime-benchmarks"))]
impl<T: Config> DidEndpointDetails<T> {
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
			service_type: bounded_types,
			url: bounded_urls,
		}
	}
}

pub mod utils {
	use super::*;
	use crate::InputError;
	use frame_support::{ensure, traits::Get};
	use sp_runtime::traits::SaturatedConversion;

	pub(crate) fn validate_new_service_endpoints<T: Config>(
		endpoints: &[DidEndpointDetails<T>],
	) -> Result<(), InputError> {
		// Check if the maximum number of endpoints is provided
		ensure!(
			endpoints.len() <= T::MaxDidServicesCount::get().saturated_into(),
			InputError::MaxServicesCountExceeded
		);

		// For each service...
		endpoints
			.iter()
			.try_for_each(|endpoint| validate_single_service_endpoint_entry(endpoint))?;

		Ok(())
	}

	pub(crate) fn validate_single_service_endpoint_entry<T: Config>(
		endpoint: &DidEndpointDetails<T>,
	) -> Result<(), InputError> {
		// Check that the maximum number of service types is provided.
		ensure!(
			endpoint.service_type.len() <= T::MaxTypeCountPerService::get().saturated_into(),
			InputError::MaxTypeCountExceeded
		);
		// Check that the maximum number of URLs is provided.
		ensure!(
			endpoint.url.len() <= T::MaxUrlCountPerService::get().saturated_into(),
			InputError::MaxUrlCountExceeded
		);
		// Check that the ID is the maximum allowed length and only contain ASCII
		// characters.
		ensure!(
			endpoint.id.len() <= T::MaxServiceIdLength::get().saturated_into(),
			InputError::MaxIdLengthExceeded
		);
		let str_id = str::from_utf8(&endpoint.id).map_err(|_| InputError::InvalidUrlEncoding)?;
		ensure!(
			crate_utils::is_valid_ascii_string(str_id),
			InputError::InvalidUrlEncoding
		);
		// Check that all types are the maximum allowed length and only contain ASCII
		// characters.
		endpoint.service_type.iter().try_for_each(|s_type| {
			ensure!(
				s_type.len() <= T::MaxServiceTypeLength::get().saturated_into(),
				InputError::MaxTypeLengthExceeded
			);
			let str_type = str::from_utf8(s_type).map_err(|_| InputError::InvalidUrlEncoding)?;
			ensure!(
				crate_utils::is_valid_ascii_string(str_type),
				InputError::InvalidUrlEncoding
			);
			Ok(())
		})?;
		// Check that all URLs are the maximum allowed length AND only contain ASCII
		// characters.
		endpoint.url.iter().try_for_each(|s_url| {
			ensure!(
				s_url.len() <= T::MaxServiceUrlLength::get().saturated_into(),
				InputError::MaxUrlLengthExceeded
			);
			let str_url = str::from_utf8(s_url).map_err(|_| InputError::InvalidUrlEncoding)?;
			ensure!(
				crate_utils::is_valid_ascii_string(str_url),
				InputError::InvalidUrlEncoding
			);
			Ok(())
		})?;
		Ok(())
	}
}
