use frame_support::dispatch::Weight;
use sp_runtime::DispatchError;

use attestation::AttestationAccessControl;

pub enum AuthorizationId<DelegationId> {
	Delegation(DelegationId),
}

pub enum PalletAuthorize<DelegationAc> {
	Delegation(DelegationAc),
}

impl<T, DelegationAc, DelegationId> AttestationAccessControl<T::AttesterId, AuthorizationId<DelegationId>, T>
	for PalletAuthorize<DelegationAc>
where
	T: attestation::Config<AuthorizationId = AuthorizationId<DelegationId>>,
	DelegationAc: AttestationAccessControl<T::AttesterId, DelegationId, T>,
{
	fn can_attest(&self, who: &T::AttesterId) -> Result<frame_support::dispatch::Weight, DispatchError> {
		match self {
			PalletAuthorize::Delegation(ac) => ac.can_attest(who),
		}
	}

	fn can_revoke(
		&self,
		who: &T::AttesterId,
		attestation: &attestation::AttestationDetails<T>,
	) -> Result<frame_support::dispatch::Weight, DispatchError> {
		match self {
			PalletAuthorize::Delegation(ac) => ac.can_revoke(who, attestation),
		}
	}

	fn can_remove(
		&self,
		who: &T::AttesterId,
		attestation: &attestation::AttestationDetails<T>,
	) -> Result<frame_support::dispatch::Weight, DispatchError> {
		match self {
			PalletAuthorize::Delegation(ac) => ac.can_revoke(who, attestation),
		}
	}

	fn authorization_id(&self) -> T::AuthorizationId {
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
