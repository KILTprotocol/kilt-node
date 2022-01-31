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

use crate::{
	default_weights::WeightInfo, Config, DelegationHierarchies, DelegationNodeIdOf, DelegationNodes, DelegatorIdOf,
	Error, Pallet, Permissions,
};

/// Controls the access to attestations.
///
/// Can attest if
///     * delegation node of sender is not revoked
///     * delegation node of sender has ATTEST permission
///     * the CType of the delegation root matches the CType of the attestation
///
/// Can revoke attestations if
///    * delegation node of sender is not revoked
///    * sender delegation node is equal to OR parent of the delegation node
///      stored in the attestation
///
/// Can remove attestations if <the same as revoke>
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub struct DelegationAc<T: Config> {
	pub(crate) sender_node_id: DelegationNodeIdOf<T>,
	pub(crate) max_checks: u32,
}

impl<T: Config>
	attestation::AttestationAccessControl<DelegatorIdOf<T>, DelegationNodeIdOf<T>, CtypeHashOf<T>, ClaimHashOf<T>>
	for DelegationAc<T>
{
	fn can_attest(
		&self,
		who: &DelegatorIdOf<T>,
		ctype: &CtypeHashOf<T>,
		_claim: &ClaimHashOf<T>,
	) -> Result<Weight, DispatchError> {
		let delegation_node = DelegationNodes::<T>::get(self.sender_node_id).ok_or(Error::<T>::DelegationNotFound)?;
		let root =
			DelegationHierarchies::<T>::get(delegation_node.hierarchy_root_id).ok_or(Error::<T>::DelegationNotFound)?;
		ensure!(
			// has permission
			((delegation_node.details.permissions & Permissions::ATTEST) == Permissions::ATTEST)
				// not revoked
				&& !delegation_node.details.revoked
				// is owner of delegation
				&& &delegation_node.details.owner == who
				// delegation matches the ctype
				&& &root.ctype_hash == ctype,
			Error::<T>::AccessDenied
		);

		Ok(<T as Config>::WeightInfo::can_attest())
	}

	fn can_revoke(
		&self,
		who: &DelegatorIdOf<T>,
		_ctype: &CtypeHashOf<T>,
		_claim: &ClaimHashOf<T>,
		attester_node_id: &DelegationNodeIdOf<T>,
	) -> Result<Weight, DispatchError> {
		// `attester_node` was supplied by the attestation pallet and is stored in the
		// attestation. `self.sender_node` was supplied by the user. `attester_node` and
		// `self.sender_node` can be different!

		match Pallet::<T>::is_delegating(who, &attester_node_id, self.max_checks)? {
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
		self.can_revoke(who, ctype, claim, auth_id)
	}

	fn authorization_id(&self) -> DelegationNodeIdOf<T> {
		self.sender_node_id
	}

	fn can_attest_weight(&self) -> Weight {
		<T as Config>::WeightInfo::can_attest()
	}

	fn can_revoke_weight(&self) -> Weight {
		<T as Config>::WeightInfo::can_revoke(self.max_checks)
	}

	fn can_remove_weight(&self) -> Weight {
		<T as Config>::WeightInfo::can_remove(self.max_checks)
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
