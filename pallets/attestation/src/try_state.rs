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
use kilt_support::test_utils::log_and_return_error_message;
use scale_info::prelude::format;
use sp_runtime::TryRuntimeError;

use crate::{Attestations, Config, ExternalAttestations};

pub(crate) fn do_try_state<T: Config>() -> Result<(), TryRuntimeError> {
	Attestations::<T>::iter().try_for_each(|(claim_hash, attestation_details)| -> Result<(), TryRuntimeError> {
		if let Some(authorization_id) = attestation_details.authorization_id {
			ensure!(
				ExternalAttestations::<T>::get(&authorization_id, claim_hash),
				log_and_return_error_message(format!(
					"External attestation with authorization_id: {:?} and claim_hash {:?} does not exist",
					authorization_id, claim_hash
				))
			);
		}
		Ok(())
	})
}
