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

use codec::Encode;
use sp_runtime::{traits::{Hash, SaturatedConversion}};
use sp_std::vec::Vec;

use frame_support::{ensure, traits::Get};

use crate::{Config, DidPublicKey, KeyIdOf, InputError, DidEndpointDetails};

pub fn calculate_key_id<T: Config>(key: &DidPublicKey) -> KeyIdOf<T> {
	let hashed_values: Vec<u8> = key.encode();
	T::Hashing::hash(&hashed_values)
}

pub(crate) fn validate_service_endpoints<T: Config>(endpoints: &[DidEndpointDetails<T>]) -> Result<(), InputError> {
	// Check if the maximum number of endpoints is provided
	ensure!(
		endpoints.len() <= T::MaxDidServicesCount::get().saturated_into(),
		InputError::MaxServicesCountExceeded
	);

	// For each service...
	endpoints.iter().try_for_each(|endpoint| {
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
		// Check that the ID is the maximum allowed length.
		ensure!(
			endpoint.id.len() <= T::MaxServiceIdLength::get().saturated_into(),
			InputError::MaxIdLengthExceeded
		);
		// Check that all types are the maximum allowed length.
		endpoint.service_type.iter().try_for_each(|s_type| {
			ensure!(
				s_type.len() <= T::MaxServiceTypeLength::get().saturated_into(),
				InputError::MaxTypeLengthExceeded
			);
			Ok(())
		})?;
		// Check that all URLs are the maximum allowed length.
		endpoint.url.iter().try_for_each(|s_url| {
			ensure!(
				s_url.len() <= T::MaxServiceUrlLength::get().saturated_into(),
				InputError::MaxUrlLengthExceeded
			);
			Ok(())
		})?;

		Ok(())
	})?;

	Ok(())
}
