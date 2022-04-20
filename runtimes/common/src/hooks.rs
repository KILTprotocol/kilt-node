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

use frame_support::dispatch::Weight;
use sp_runtime::{traits::Zero, DispatchError};

use did::WeightInfo;

pub struct AttestationLifecycleHandler<T>(sp_std::marker::PhantomData<T>);

//TODO: Add logging if needed
impl<T: attestation::Config + did::Config + ctype::Config> attestation::OnAttestationLifecycle<T>
	for AttestationLifecycleHandler<T>
where
	<T as attestation::Config>::AttesterId: Into<<T as did::Config>::DidIdentifier>,
{
	fn attestation_created(
		attester: &attestation::AttesterOf<T>,
		_claim_hash: &attestation::ClaimHashOf<T>,
		_ctype_hash: &ctype::CtypeHashOf<T>,
		_authorization_id: &Option<attestation::AuthorizationIdOf<T>>,
	) -> Result<Weight, DispatchError> {
		// FIXME: I tried with where <T as attestation::Config>::AttesterId: AsRef<<T as
		// did::Config>::DidIdentifier> but apparently AccountId32 does not implement
		// AsRef<AccountId32>
		did::Pallet::<T>::increment_consumers(&attester.clone().into()).map_err(did::Error::<T>::from)?;
		Ok(<T as did::Config>::WeightInfo::increment_consumers())
	}

	fn attestation_created_max_weight() -> Weight {
		<T as did::Config>::WeightInfo::increment_consumers()
	}

	// No action taken when an attestation is revoked.
	fn attestation_revoked(
		_attester: &attestation::AttesterOf<T>,
		_claim_hash: &attestation::ClaimHashOf<T>,
	) -> Result<Weight, DispatchError> {
		Ok(Weight::zero())
	}

	fn attestation_revoked_max_weight() -> Weight {
		Weight::zero()
	}

	fn attestation_removed(
		attester: &attestation::AttesterOf<T>,
		_claim_hash: &attestation::ClaimHashOf<T>,
	) -> Result<Weight, DispatchError> {
		// FIXME: same as in `attestation_created`
		did::Pallet::<T>::decrement_consumers(&attester.clone().into()).map_err(did::Error::<T>::from)?;
		Ok(<T as did::Config>::WeightInfo::decrement_consumers())
	}

	fn attestation_removed_max_weight() -> Weight {
		<T as did::Config>::WeightInfo::decrement_consumers()
	}

	fn deposit_reclaimed(
		attester: &attestation::AttesterOf<T>,
		claim_hash: &attestation::ClaimHashOf<T>,
	) -> Result<Weight, DispatchError> {
		Self::attestation_removed(attester, claim_hash)
	}

	fn deposit_reclaimed_max_weight() -> Weight {
		Self::attestation_removed_max_weight()
	}
}
