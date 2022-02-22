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
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::Weight;
use scale_info::TypeInfo;
use sp_runtime::DispatchError;

use attestation::AttestationAccessControl;

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum AuthorizationId<DelegationId> {
	Delegation(DelegationId),
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum PalletAuthorize<DelegationAc> {
	Delegation(DelegationAc),
}

impl<AttesterId, DelegationAc, DelegationId, Ctype, ClaimHash>
	AttestationAccessControl<AttesterId, AuthorizationId<DelegationId>, Ctype, ClaimHash> for PalletAuthorize<DelegationAc>
where
	DelegationAc: AttestationAccessControl<AttesterId, DelegationId, Ctype, ClaimHash>,
{
	fn can_attest(
		&self,
		who: &AttesterId,
		ctype: &Ctype,
		claim: &ClaimHash,
	) -> Result<frame_support::dispatch::Weight, DispatchError> {
		match self {
			PalletAuthorize::Delegation(ac) => ac.can_attest(who, ctype, claim),
		}
	}

	fn can_revoke(
		&self,
		who: &AttesterId,
		ctype: &Ctype,
		claim: &ClaimHash,
		auth_id: &AuthorizationId<DelegationId>,
	) -> Result<frame_support::dispatch::Weight, DispatchError> {
		match (self, auth_id) {
			(PalletAuthorize::Delegation(ac), AuthorizationId::Delegation(auth_id)) => {
				ac.can_revoke(who, ctype, claim, auth_id)
			} // _ => Err(DispatchError::Other("unauthorized")),
		}
	}

	fn can_remove(
		&self,
		who: &AttesterId,
		ctype: &Ctype,
		claim: &ClaimHash,
		auth_id: &AuthorizationId<DelegationId>,
	) -> Result<frame_support::dispatch::Weight, DispatchError> {
		match (self, auth_id) {
			(PalletAuthorize::Delegation(ac), AuthorizationId::Delegation(auth_id)) => {
				ac.can_remove(who, ctype, claim, auth_id)
			} // _ => Err(DispatchError::Other("unauthorized")),
		}
	}

	fn authorization_id(&self) -> AuthorizationId<DelegationId> {
		match self {
			PalletAuthorize::Delegation(ac) => AuthorizationId::Delegation(ac.authorization_id()),
		}
	}

	fn can_attest_weight(&self) -> Weight {
		match self {
			PalletAuthorize::Delegation(ac) => ac.can_attest_weight(),
		}
	}
	fn can_revoke_weight(&self) -> Weight {
		match self {
			PalletAuthorize::Delegation(ac) => ac.can_revoke_weight(),
		}
	}
	fn can_remove_weight(&self) -> Weight {
		match self {
			PalletAuthorize::Delegation(ac) => ac.can_remove_weight(),
		}
	}
}
