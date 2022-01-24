use codec::{Decode, Encode};
use frame_support::dispatch::Weight;
use scale_info::TypeInfo;
use sp_runtime::DispatchError;

use attestation::AttestationAccessControl;

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub enum AuthorizationId<DelegationId> {
	Delegation(DelegationId),
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub enum PalletAuthorize<DelegationAc> {
	Delegation(DelegationAc),
}

impl<AttesterId, DelegationAc, DelegationId> AttestationAccessControl<AttesterId, AuthorizationId<DelegationId>>
	for PalletAuthorize<DelegationAc>
where
	DelegationAc: AttestationAccessControl<AttesterId, DelegationId>,
{
	fn can_attest(&self, who: &AttesterId) -> Result<frame_support::dispatch::Weight, DispatchError> {
		match self {
			PalletAuthorize::Delegation(ac) => ac.can_attest(who),
		}
	}

	fn can_revoke(
		&self,
		who: &AttesterId,
		auth_id: &AuthorizationId<DelegationId>,
	) -> Result<frame_support::dispatch::Weight, DispatchError> {
		match (self, auth_id) {
			(PalletAuthorize::Delegation(ac), AuthorizationId::Delegation(auth_id)) => ac.can_revoke(who, auth_id),
			// _ => Err(DispatchError::Other("unauthorized")),
		}
	}

	fn can_remove(
		&self,
		who: &AttesterId,
		auth_id: &AuthorizationId<DelegationId>,
	) -> Result<frame_support::dispatch::Weight, DispatchError> {
		match (self, auth_id) {
			(PalletAuthorize::Delegation(ac), AuthorizationId::Delegation(auth_id)) => ac.can_remove(who, auth_id),
			// _ => Err(DispatchError::Other("unauthorized")),
		}
	}

	fn authorization_id(&self) -> AuthorizationId<DelegationId> {
		match self {
			PalletAuthorize::Delegation(ac) => AuthorizationId::Delegation(ac.authorization_id()),
		}
	}

	fn weight(&self) -> Weight {
		match self {
			PalletAuthorize::Delegation(ac) => ac.weight(),
		}
	}
}
