// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

use codec::Encode;
use frame_support::assert_noop;
use kilt_support::deposit::Deposit;
use sp_runtime::{
	app_crypto::{sr25519, Pair},
	traits::IdentifyAccount,
	MultiSignature, MultiSigner,
};

use crate::{mock::*, ConnectedDids, ConnectionRecord, Error};

#[test]
fn test_add_association_sender() {
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, 100), (ACCOUNT_01, 100)])
		.build()
		.execute_with(|| {
			// new association. No overwrite
			assert!(DidLookup::associate_sender(mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into()).is_ok());
			assert_eq!(
				ConnectedDids::<Test>::get(ACCOUNT_00),
				Some(ConnectionRecord {
					did: DID_00,
					deposit: Deposit {
						owner: ACCOUNT_00,
						amount: 10,
					}
				})
			);

			// overwrite existing association
			assert!(DidLookup::associate_sender(mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into()).is_ok());
			assert_eq!(
				ConnectedDids::<Test>::get(ACCOUNT_00),
				Some(ConnectionRecord {
					did: DID_01,
					deposit: Deposit {
						owner: ACCOUNT_00,
						amount: 10,
					}
				})
			);
		});
}

#[test]
fn test_add_association_account() {
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, 100), (ACCOUNT_01, 100)])
		.build()
		.execute_with(|| {
			let pair_alice = sr25519::Pair::from_seed(&*b"Alice                           ");
			let account_hash_alice = MultiSigner::from(pair_alice.public()).into_account();
			let sig_alice_0 = MultiSignature::from(pair_alice.sign(&Encode::encode(&DID_00)[..]));
			let sig_alice_1 = MultiSignature::from(pair_alice.sign(&Encode::encode(&DID_01)[..]));

			// new association. No overwrite
			assert!(DidLookup::associate_account(
				mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(),
				account_hash_alice.clone(),
				sig_alice_0
			)
			.is_ok());
			assert_eq!(
				ConnectedDids::<Test>::get(&account_hash_alice),
				Some(ConnectionRecord {
					did: DID_00,
					deposit: Deposit {
						owner: ACCOUNT_00,
						amount: 10,
					}
				})
			);

			// overwrite existing association
			assert!(DidLookup::associate_account(
				mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into(),
				account_hash_alice.clone(),
				sig_alice_1
			)
			.is_ok());
			assert_eq!(
				ConnectedDids::<Test>::get(&account_hash_alice),
				Some(ConnectionRecord {
					did: DID_01,
					deposit: Deposit {
						owner: ACCOUNT_00,
						amount: 10,
					}
				})
			);
		});
}

#[test]
fn test_add_association_account_invalid_signature() {
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, 100), (ACCOUNT_01, 100)])
		.build()
		.execute_with(|| {
			let pair_alice = sr25519::Pair::from_seed(&*b"Alice                           ");
			let account_hash_alice = MultiSigner::from(pair_alice.public()).into_account();
			let sig_alice_0 = MultiSignature::from(pair_alice.sign(&Encode::encode(&0_u64)[..]));

			assert_noop!(
				DidLookup::associate_account(
					mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into(),
					account_hash_alice,
					sig_alice_0
				),
				Error::<Test>::NotAuthorized
			);
		});
}

#[test]
fn test_remove_association_sender() {
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, 100), (ACCOUNT_01, 100)])
		.with_connections(vec![(ACCOUNT_00, DID_01, ACCOUNT_00)])
		.build()
		.execute_with(|| {
			// remove association
			assert!(DidLookup::remove_sender_association(Origin::signed(ACCOUNT_00)).is_ok());
			assert_eq!(ConnectedDids::<Test>::get(ACCOUNT_00), None);
		});
}

#[test]
fn test_remove_association_sender_not_found() {
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, 100), (ACCOUNT_01, 100)])
		.build()
		.execute_with(|| {
			assert_noop!(
				DidLookup::remove_sender_association(Origin::signed(ACCOUNT_00)),
				Error::<Test>::AssociationNotFound
			);
		});
}

#[test]
fn test_remove_association_account() {
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, 100), (ACCOUNT_01, 100)])
		.with_connections(vec![(ACCOUNT_00, DID_01, ACCOUNT_00)])
		.build()
		.execute_with(|| {
			assert!(DidLookup::remove_account_association(
				mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into(),
				ACCOUNT_00
			)
			.is_ok());
			assert_eq!(ConnectedDids::<Test>::get(ACCOUNT_00), None);
		});
}

#[test]
fn test_remove_association_account_not_found() {
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, 100), (ACCOUNT_01, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(ConnectedDids::<Test>::get(ACCOUNT_00), None);

			assert_noop!(
				DidLookup::remove_account_association(mock_origin::DoubleOrigin(ACCOUNT_01, DID_01).into(), ACCOUNT_00),
				Error::<Test>::AssociationNotFound
			);
		});
}

#[test]
fn test_remove_association_account_not_authorized() {
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, 100), (ACCOUNT_01, 100)])
		.build()
		.execute_with(|| {
			// create association for DID 1
			ConnectedDids::<Test>::insert(
				ACCOUNT_00,
				ConnectionRecord {
					did: DID_01,
					deposit: Deposit {
						owner: ACCOUNT_00,
						amount: 10,
					},
				},
			);
			assert_eq!(
				ConnectedDids::<Test>::get(ACCOUNT_00),
				Some(ConnectionRecord {
					did: DID_01,
					deposit: Deposit {
						owner: ACCOUNT_00,
						amount: 10,
					}
				})
			);

			// DID 0 tries to remove association
			assert_noop!(
				DidLookup::remove_account_association(mock_origin::DoubleOrigin(ACCOUNT_01, DID_00).into(), ACCOUNT_00),
				Error::<Test>::NotAuthorized
			);
		});
}
