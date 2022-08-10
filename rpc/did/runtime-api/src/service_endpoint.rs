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
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

#[derive(Encode, Decode, TypeInfo, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct ServiceEndpoint<Id, Type, Url> {
	pub id: Id,
	pub service_types: Vec<Type>,
	pub urls: Vec<Url>,
}

impl<T: did::Config> From<did::service_endpoints::DidEndpoint<T>> for ServiceEndpoint<Vec<u8>, Vec<u8>, Vec<u8>> {
	fn from(runtime_endpoint: did::service_endpoints::DidEndpoint<T>) -> Self {
		ServiceEndpoint {
			id: runtime_endpoint.id.into_inner(),
			service_types: runtime_endpoint
				.service_types
				.into_inner()
				.into_iter()
				.map(|v| v.into_inner())
				.collect(),
			urls: runtime_endpoint
				.urls
				.into_inner()
				.into_iter()
				.map(|v| v.into_inner())
				.collect(),
		}
	}
}
