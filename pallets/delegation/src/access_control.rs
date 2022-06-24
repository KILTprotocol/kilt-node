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
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct DelegationAc<T: Config> {
	pub(crate) subject_node_id: DelegationNodeIdOf<T>,
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
		let delegation_node =
			DelegationNodes::<T>::get(self.authorization_id()).ok_or(Error::<T>::DelegationNotFound)?;
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
		// NOTE: The node IDs of the sender (provided by the user through `who`) and
		// attester (provided by the attestation pallet through on-chain storage) can be
		// different!
		match Pallet::<T>::is_delegating(who, attester_node_id, self.max_checks)? {
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
		self.subject_node_id
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
	use frame_support::{assert_noop, assert_ok};

	use attestation::{mock::generate_base_attestation, AttestationAccessControl};
	use ctype::mock::get_ctype_hash;
	use kilt_support::{deposit::Deposit, mock::mock_origin::DoubleOrigin};

	use super::*;
	use crate::{mock::*, DelegationDetails, DelegationNode};

	#[test]
	fn test_can_attest() {
		let root_owner: DelegatorIdOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
		let delegate = sr25519_did_from_seed(&BOB_SEED);

		let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
		let hierarchy_details = generate_base_delegation_hierarchy_details();
		let ctype_hash = hierarchy_details.ctype_hash;
		let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
		let parent_node = DelegationNode {
			details: DelegationDetails {
				owner: delegate.clone(),
				permissions: Permissions::DELEGATE | Permissions::ATTEST,
				revoked: false,
			},
			children: Default::default(),
			hierarchy_root_id,
			parent: Some(hierarchy_root_id),
			deposit: Deposit {
				owner: ACCOUNT_00,
				amount: <Test as Config>::Deposit::get(),
			},
		};
		let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
		let ac_info = Some(DelegationAc {
			subject_node_id: parent_id,
			max_checks: 1,
		});

		ExtBuilder::default()
			.with_ctypes(vec![(ctype_hash, root_owner.clone())])
			.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, root_owner, ACCOUNT_00)])
			.with_delegations(vec![(parent_id, parent_node)])
			.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
			.build()
			.execute_with(|| {
				assert_ok!(Attestation::add(
					DoubleOrigin(ACCOUNT_00, delegate.clone()).into(),
					claim_hash,
					ctype_hash,
					ac_info.clone()
				));
				let stored_attestation =
					Attestation::attestations(&claim_hash).expect("Attestation should be present on chain.");

				assert_eq!(stored_attestation.ctype_hash, ctype_hash);
				assert_eq!(stored_attestation.attester, delegate);
				assert_eq!(
					stored_attestation.authorization_id,
					ac_info.map(|ac| ac.authorization_id())
				);
				assert!(!stored_attestation.revoked);
			});
	}

	#[test]
	fn test_cannot_attest_missing_permission() {
		let root_owner: DelegatorIdOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
		let delegate = sr25519_did_from_seed(&BOB_SEED);

		let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
		let hierarchy_details = generate_base_delegation_hierarchy_details();
		let ctype_hash = hierarchy_details.ctype_hash;
		let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
		let parent_node = DelegationNode {
			details: DelegationDetails {
				owner: delegate.clone(),
				permissions: Permissions::DELEGATE,
				revoked: false,
			},
			children: Default::default(),
			hierarchy_root_id,
			parent: Some(hierarchy_root_id),
			deposit: Deposit {
				owner: ACCOUNT_00,
				amount: <Test as Config>::Deposit::get(),
			},
		};
		let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
		let ac_info = Some(DelegationAc {
			subject_node_id: parent_id,
			max_checks: 1,
		});

		ExtBuilder::default()
			.with_ctypes(vec![(ctype_hash, root_owner.clone())])
			.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, root_owner, ACCOUNT_00)])
			.with_delegations(vec![(parent_id, parent_node)])
			.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
			.build()
			.execute_with(|| {
				assert_noop!(
					Attestation::add(
						DoubleOrigin(ACCOUNT_00, delegate.clone()).into(),
						claim_hash,
						ctype_hash,
						ac_info.clone()
					),
					Error::<Test>::AccessDenied
				);
			});
	}

	#[test]
	fn test_can_attest_revoked() {
		let root_owner: DelegatorIdOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
		let delegate = sr25519_did_from_seed(&BOB_SEED);

		let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
		let hierarchy_details = generate_base_delegation_hierarchy_details();
		let ctype_hash = hierarchy_details.ctype_hash;
		let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
		let parent_node = DelegationNode {
			details: DelegationDetails {
				owner: delegate.clone(),
				permissions: Permissions::DELEGATE | Permissions::ATTEST,
				revoked: true,
			},
			children: Default::default(),
			hierarchy_root_id,
			parent: Some(hierarchy_root_id),
			deposit: Deposit {
				owner: ACCOUNT_00,
				amount: <Test as Config>::Deposit::get(),
			},
		};
		let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
		let ac_info = Some(DelegationAc {
			subject_node_id: parent_id,
			max_checks: 1,
		});

		ExtBuilder::default()
			.with_ctypes(vec![(ctype_hash, root_owner.clone())])
			.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, root_owner, ACCOUNT_00)])
			.with_delegations(vec![(parent_id, parent_node)])
			.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
			.build()
			.execute_with(|| {
				assert_noop!(
					Attestation::add(
						DoubleOrigin(ACCOUNT_00, delegate.clone()).into(),
						claim_hash,
						ctype_hash,
						ac_info.clone()
					),
					Error::<Test>::AccessDenied
				);
			});
	}

	#[test]
	fn test_cannot_attest_missing_node() {
		let root_owner: DelegatorIdOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
		let delegate = sr25519_did_from_seed(&BOB_SEED);

		let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
		let hierarchy_details = generate_base_delegation_hierarchy_details();
		let ctype_hash = hierarchy_details.ctype_hash;
		let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
		let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
		let ac_info = Some(DelegationAc {
			subject_node_id: parent_id,
			max_checks: 1,
		});

		ExtBuilder::default()
			.with_ctypes(vec![(ctype_hash, root_owner.clone())])
			.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, root_owner, ACCOUNT_00)])
			.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
			.build()
			.execute_with(|| {
				assert_noop!(
					Attestation::add(
						DoubleOrigin(ACCOUNT_00, delegate.clone()).into(),
						claim_hash,
						ctype_hash,
						ac_info.clone()
					),
					Error::<Test>::DelegationNotFound
				);
			});
	}

	#[test]
	fn test_cannot_attest_wrong_ctype() {
		let root_owner: DelegatorIdOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
		let delegate = sr25519_did_from_seed(&BOB_SEED);

		let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
		let hierarchy_details = generate_base_delegation_hierarchy_details();
		let ctype_hash = get_ctype_hash::<Test>(false);
		let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
		let parent_node = DelegationNode {
			details: DelegationDetails {
				owner: delegate.clone(),
				permissions: Permissions::DELEGATE | Permissions::ATTEST,
				revoked: false,
			},
			children: Default::default(),
			hierarchy_root_id,
			parent: Some(hierarchy_root_id),
			deposit: Deposit {
				owner: ACCOUNT_00,
				amount: <Test as Config>::Deposit::get(),
			},
		};
		let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
		let ac_info = Some(DelegationAc {
			subject_node_id: parent_id,
			max_checks: 1,
		});

		ExtBuilder::default()
			.with_ctypes(vec![(ctype_hash, root_owner.clone())])
			.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, root_owner, ACCOUNT_00)])
			.with_delegations(vec![(parent_id, parent_node)])
			.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
			.build()
			.execute_with(|| {
				assert_noop!(
					Attestation::add(
						DoubleOrigin(ACCOUNT_00, delegate.clone()).into(),
						claim_hash,
						ctype_hash,
						ac_info.clone()
					),
					Error::<Test>::AccessDenied
				);
			});
	}

	#[test]
	fn test_can_revoke_same_node() {
		let root_owner: DelegatorIdOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
		let delegate = sr25519_did_from_seed(&BOB_SEED);

		let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
		let hierarchy_details = generate_base_delegation_hierarchy_details();
		let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
		let parent_node = DelegationNode {
			details: DelegationDetails {
				owner: delegate.clone(),
				permissions: Permissions::DELEGATE | Permissions::ATTEST,
				revoked: false,
			},
			children: Default::default(),
			hierarchy_root_id,
			parent: Some(hierarchy_root_id),
			deposit: Deposit {
				owner: ACCOUNT_00,
				amount: <Test as Config>::Deposit::get(),
			},
		};
		let ac_info = Some(DelegationAc {
			subject_node_id: parent_id,
			max_checks: 1,
		});

		let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
		let mut attestation = generate_base_attestation::<Test>(delegate.clone(), ACCOUNT_00);
		attestation.authorization_id = Some(parent_id);

		ExtBuilder::default()
			.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
			.with_ctypes(vec![(attestation.ctype_hash, delegate.clone())])
			.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, root_owner, ACCOUNT_00)])
			.with_delegations(vec![(parent_id, parent_node)])
			.with_attestations(vec![(claim_hash, attestation)])
			.build()
			.execute_with(|| {
				assert_ok!(Attestation::revoke(
					DoubleOrigin(ACCOUNT_00, delegate.clone()).into(),
					claim_hash,
					ac_info
				));
			});
	}

	#[test]
	fn test_can_revoke_parent() {
		let root_owner: DelegatorIdOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
		let delegate = sr25519_did_from_seed(&BOB_SEED);

		let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
		let hierarchy_details = generate_base_delegation_hierarchy_details();
		let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
		let parent_node = DelegationNode {
			details: DelegationDetails {
				owner: delegate.clone(),
				permissions: Permissions::DELEGATE | Permissions::ATTEST,
				revoked: false,
			},
			children: Default::default(),
			hierarchy_root_id,
			parent: Some(hierarchy_root_id),
			deposit: Deposit {
				owner: ACCOUNT_00,
				amount: <Test as Config>::Deposit::get(),
			},
		};
		let ac_info = Some(DelegationAc {
			subject_node_id: parent_id,
			max_checks: 1,
		});

		let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
		let mut attestation = generate_base_attestation::<Test>(delegate.clone(), ACCOUNT_00);
		attestation.authorization_id = Some(parent_id);

		ExtBuilder::default()
			.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
			.with_ctypes(vec![(attestation.ctype_hash, delegate)])
			.with_delegation_hierarchies(vec![(
				hierarchy_root_id,
				hierarchy_details,
				root_owner.clone(),
				ACCOUNT_00,
			)])
			.with_delegations(vec![(parent_id, parent_node)])
			.with_attestations(vec![(claim_hash, attestation)])
			.build()
			.execute_with(|| {
				assert_ok!(Attestation::revoke(
					DoubleOrigin(ACCOUNT_00, root_owner.clone()).into(),
					claim_hash,
					ac_info
				));
			});
	}

	#[test]
	fn test_can_revoke_same_node_revoked() {
		let root_owner: DelegatorIdOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
		let delegate = sr25519_did_from_seed(&BOB_SEED);

		let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
		let hierarchy_details = generate_base_delegation_hierarchy_details();
		let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
		let parent_node = DelegationNode {
			details: DelegationDetails {
				owner: delegate.clone(),
				permissions: Permissions::DELEGATE | Permissions::ATTEST,
				revoked: true,
			},
			children: Default::default(),
			hierarchy_root_id,
			parent: Some(hierarchy_root_id),
			deposit: Deposit {
				owner: ACCOUNT_00,
				amount: <Test as Config>::Deposit::get(),
			},
		};
		let ac_info = Some(DelegationAc {
			subject_node_id: parent_id,
			max_checks: 1,
		});

		let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
		let mut attestation = generate_base_attestation::<Test>(delegate.clone(), ACCOUNT_00);
		attestation.authorization_id = Some(parent_id);

		ExtBuilder::default()
			.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
			.with_ctypes(vec![(attestation.ctype_hash, delegate.clone())])
			.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, root_owner, ACCOUNT_00)])
			.with_delegations(vec![(parent_id, parent_node)])
			.with_attestations(vec![(claim_hash, attestation)])
			.build()
			.execute_with(|| {
				assert_ok!(Attestation::revoke(
					DoubleOrigin(ACCOUNT_00, delegate.clone()).into(),
					claim_hash,
					ac_info
				));
			});
	}
}
