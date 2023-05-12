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

use frame_support::ensure;
use kilt_support::test::convert_error_message;
use scale_info::prelude::format;
use sp_core::Get;
use sp_runtime::SaturatedConversion;

use crate::{did_details::DidDetails, Config, Did, DidBlacklist, DidEndpointsCount, DidIdentifierOf, ServiceEndpoints};

pub(crate) fn do_try_state<T: Config>() -> Result<(), &'static str> {
	Did::<T>::iter().try_for_each(
		|(did_subject, did_details): (DidIdentifierOf<T>, DidDetails<T>)| -> Result<(), &'static str> {
			let service_endpoints_count = ServiceEndpoints::<T>::iter_prefix(&did_subject).count();

			ensure!(
				service_endpoints_count == DidEndpointsCount::<T>::get(&did_subject).saturated_into::<usize>(),
				convert_error_message(format!("DID {:?} has not matching service endpoints.", did_subject))
			);

			ensure!(
				did_details.key_agreement_keys.len()
					<= (<T as Config>::MaxTotalKeyAgreementKeys::get()).saturated_into::<usize>(),
				convert_error_message(format!("DID {:?} has to many key agreement keys.", did_subject,))
			);

			ensure!(
				service_endpoints_count <= <T as Config>::MaxNumberOfServicesPerDid::get().saturated_into::<usize>(),
				convert_error_message(format!("DID {:?} has to many service endpoints.", did_subject))
			);

			ensure!(
				!DidBlacklist::<T>::contains_key(did_subject),
				convert_error_message(format!("DID {:?} is blacklisted.", did_subject))
			);

			Ok(())
		},
	)?;

	DidBlacklist::<T>::iter_keys().try_for_each(|deleted_did_subject| -> Result<(), &'static str> {
		let service_endpoints_count = ServiceEndpoints::<T>::iter_prefix(&deleted_did_subject).count();
		ensure!(
			service_endpoints_count == 0,
			convert_error_message(format!(
				"Blacklisted DID {:?} has service endpoints.",
				deleted_did_subject,
			))
		);
		Ok(())
	})
}
