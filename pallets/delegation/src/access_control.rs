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
use frame_support::{dispatch::Weight, ensure};
use scale_info::TypeInfo;
use sp_runtime::DispatchError;

use attestation::ClaimHashOf;
use ctype::CtypeHashOf;

use crate::{default_weights::WeightInfo, Config, DelegationNodeIdOf, DelegatorIdOf, Error, Pallet};

/// Controls the access to attestations.
///
/// Can attest if all conditions are fulfilled
///     * delegation node of sender is not revoked
///     * delegation node of sender has ATTEST permission
///     * the CType of the delegation root matches the CType of the attestation
///
/// Can revoke if all conditions are fulfilled
///    * sender delegation node is not revoked
///    * sender delegation node is equal to OR parent of the delegation node
///      stored in the attestation
///
/// Can revoke if all conditions are fulfilled
///    * sender delegation node is not revoked
///    * sender delegation node is equal to OR parent of the delegation node
///      stored in the attestation
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub struct DelegationAc<T: Config>(pub(crate) DelegationNodeIdOf<T>, pub(crate) u32);

impl<T: Config>
	attestation::AttestationAccessControl<DelegatorIdOf<T>, DelegationNodeIdOf<T>, CtypeHashOf<T>, ClaimHashOf<T>>
	for DelegationAc<T>
{
	fn can_attest(
		&self,
		who: &DelegatorIdOf<T>,
		ctype: &CtypeHashOf<T>,
		claim: &ClaimHashOf<T>,
	) -> Result<Weight, DispatchError> {
		match Pallet::<T>::is_delegating(who, &self.0, self.1)? {
			(true, checks) => Ok(<T as Config>::WeightInfo::can_attest(checks)),
			_ => Err(Error::<T>::AccessDenied.into()),
		}
	}

	fn can_revoke(
		&self,
		who: &DelegatorIdOf<T>,
		ctype: &CtypeHashOf<T>,
		claim: &ClaimHashOf<T>,
		auth_id: &DelegationNodeIdOf<T>,
	) -> Result<Weight, DispatchError> {
		ensure!(auth_id == &self.0, Error::<T>::AccessDenied);

		match Pallet::<T>::is_delegating(who, &self.0, self.1)? {
			(true, checks) => Ok(<T as Config>::WeightInfo::can_revoke(checks)),
			_ => Err(Error::<T>::AccessDenied.into()),
		}
	}

	fn can_remove(
		&self,
		who: &DelegatorIdOf<T>,
		ctype: &CtypeHashOf<T>,
		claim: &ClaimHashOf<T>,
		auth_id: &DelegationNodeIdOf<T>,
	) -> Result<Weight, DispatchError> {
		ensure!(auth_id == &self.0, Error::<T>::AccessDenied);

		match Pallet::<T>::is_delegating(who, &self.0, self.1)? {
			(true, checks) => Ok(<T as Config>::WeightInfo::can_remove(checks)),
			_ => Err(Error::<T>::AccessDenied.into()),
		}
	}

	fn authorization_id(&self) -> DelegationNodeIdOf<T> {
		self.0
	}

	fn weight(&self) -> Weight {
		<T as Config>::WeightInfo::can_attest(self.1)
			.max(<T as Config>::WeightInfo::can_revoke(self.1))
			.max(<T as Config>::WeightInfo::can_remove(self.1))
	}
}

#[cfg(test)]
mod tests {

	#[test]
	fn test_can_attest() {
		todo!()
	}

	#[test]
	fn test_can_attest_missing_permission() {
		todo!()
	}

	#[test]
	fn test_can_attest_missing_node() {
		todo!()
	}

	#[test]
	fn test_can_attest_wrong_ctype() {
		todo!()
	}

	#[test]
	fn test_can_revoke_same_node() {
		todo!()
	}

	#[test]
	fn test_can_revoke_parent() {
		todo!()
	}

	#[test]
	fn test_can_revoke_same_node_revoked() {
		todo!()
	}

	#[test]
	fn test_can_revoke_parent_revoked() {
		todo!()
	}
}
