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
use sp_runtime::DispatchError;

/// Allow for more complex schemes on who can attest, revoke and remove.
pub trait AttestationAccessControl<AttesterId, AuthorizationId, Ctype, ClaimHash> {
	/// Decides whether the account is allowed to attest with the given
	/// information provided by the sender (&self).
	fn can_attest(&self, who: &AttesterId, ctype: &Ctype, claim: &ClaimHash) -> Result<Weight, DispatchError>;

	/// Decides whether the account is allowed to revoke the attestation with
	/// the `authorization_id` and the access information provided by the sender
	/// (&self).
	fn can_revoke(
		&self,
		who: &AttesterId,
		ctype: &Ctype,
		claim: &ClaimHash,
		authorization_id: &AuthorizationId,
	) -> Result<Weight, DispatchError>;

	/// Decides whether the account is allowed to remove the attestation with
	/// the `authorization_id` and the access information provided by the sender
	/// (&self).
	fn can_remove(
		&self,
		who: &AttesterId,
		ctype: &Ctype,
		claim: &ClaimHash,
		authorization_id: &AuthorizationId,
	) -> Result<Weight, DispatchError>;

	/// The authorization ID that the sender provided. This will be used for new
	/// attestations.
	///
	/// NOTE: This method must not read storage or do any heavy computation
	/// since it's not covered by the weight returned by `self.weight()`.
	fn authorization_id(&self) -> AuthorizationId;

	/// The worst-case weight of `can_attest`.
	fn can_attest_weight(&self) -> Weight;

	/// The worst-case weight of `can_revoke`.
	fn can_revoke_weight(&self) -> Weight;

	/// The worst-case weight of `can_remove`.
	fn can_remove_weight(&self) -> Weight;
}

impl<AttesterId, AuthorizationId, Ctype, ClaimHash>
	AttestationAccessControl<AttesterId, AuthorizationId, Ctype, ClaimHash> for ()
where
	AuthorizationId: Default,
{
	fn can_attest(&self, _who: &AttesterId, _ctype: &Ctype, _claim: &ClaimHash) -> Result<Weight, DispatchError> {
		Err(DispatchError::Other("Unimplemented"))
	}
	fn can_revoke(
		&self,
		_who: &AttesterId,
		_ctype: &Ctype,
		_claim: &ClaimHash,
		_authorization_id: &AuthorizationId,
	) -> Result<Weight, DispatchError> {
		Err(DispatchError::Other("Unimplemented"))
	}
	fn can_remove(
		&self,
		_who: &AttesterId,
		_ctype: &Ctype,
		_claim: &ClaimHash,
		_authorization_id: &AuthorizationId,
	) -> Result<Weight, DispatchError> {
		Err(DispatchError::Other("Unimplemented"))
	}
	fn authorization_id(&self) -> AuthorizationId {
		Default::default()
	}
	fn can_attest_weight(&self) -> Weight {
		0
	}
	fn can_revoke_weight(&self) -> Weight {
		0
	}
	fn can_remove_weight(&self) -> Weight {
		0
	}
}
