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

use ctype::CtypeHashOf;

use crate::{AttesterOf, AuthorizationIdOf, ClaimHashOf, Config};

/// Trait called by the attestation pallet for each of the lifecycle stages of
/// an attestation.
pub trait OnAttestationLifecycle<T: Config> {
	/// A new attestation with the provided claim hash against the provided
	/// CType hash has been created by the provided attester using the
	/// optionally provided authorization information.
	fn attestation_created(
		attester: &AttesterOf<T>,
		claim_hash: &ClaimHashOf<T>,
		ctype_hash: &CtypeHashOf<T>,
		authorization_id: &Option<AuthorizationIdOf<T>>,
	) -> Result<Weight, DispatchError>;
	/// The maximum weight that the handler for the attestation creation can
	/// consume.
	///
	/// NOTE: the returned value must be equal or larger than the value returned
	/// by `attestation_created`.
	fn attestation_created_max_weight() -> Weight;
	/// The attestation with the provided claim hash and attester has been
	/// revoked.
	fn attestation_revoked(attester: &AttesterOf<T>, claim_hash: &ClaimHashOf<T>) -> Result<Weight, DispatchError>;
	/// The maximum weight that the handler for the attestation revocation can
	/// consume.
	///
	/// NOTE: the returned value must be equal or larger than the value returned
	/// by `attestation_revoked`.
	fn attestation_revoked_max_weight() -> Weight;
	/// The attestation with the provided claim hash and attester has been
	/// removed.
	fn attestation_removed(attester: &AttesterOf<T>, claim_hash: &ClaimHashOf<T>) -> Result<Weight, DispatchError>;
	/// The maximum weight that the handler for the attestation removal can
	/// consume.
	///
	/// NOTE: the returned value must be equal or larger than the value returned
	/// by `attestation_removed`.
	fn attestation_removed_max_weight() -> Weight;
	/// The deposit for the attestation with the provided claim hash and
	/// attester has been claimed back.
	fn deposit_reclaimed(attester: &AttesterOf<T>, claim_hash: &ClaimHashOf<T>) -> Result<Weight, DispatchError>;
	/// The maximum weight that the handler for the deposit reclaiming can
	/// consume.
	///
	/// NOTE: the returned value must be equal or larger than the value returned
	/// by `deposit_reclaimed`.
	fn deposit_reclaimed_max_weight() -> Weight;
}

impl<T: Config> OnAttestationLifecycle<T> for () {
	fn attestation_created(
		_attester: &AttesterOf<T>,
		_claim_hash: &ClaimHashOf<T>,
		_ctype_hash: &CtypeHashOf<T>,
		_authorization_id: &Option<AuthorizationIdOf<T>>,
	) -> Result<Weight, DispatchError> {
		Ok(Weight::zero())
	}

	fn attestation_created_max_weight() -> Weight {
		Weight::zero()
	}

	fn attestation_revoked(_attester: &AttesterOf<T>, _claim_hash: &ClaimHashOf<T>) -> Result<Weight, DispatchError> {
		Ok(Weight::zero())
	}

	fn attestation_revoked_max_weight() -> Weight {
		Weight::zero()
	}

	fn attestation_removed(_attester: &AttesterOf<T>, _claim_hash: &ClaimHashOf<T>) -> Result<Weight, DispatchError> {
		Ok(Weight::zero())
	}

	fn attestation_removed_max_weight() -> Weight {
		Weight::zero()
	}

	fn deposit_reclaimed(_attester: &AttesterOf<T>, _claim_hash: &ClaimHashOf<T>) -> Result<Weight, DispatchError> {
		Ok(Weight::zero())
	}

	fn deposit_reclaimed_max_weight() -> Weight {
		Weight::zero()
	}
}
