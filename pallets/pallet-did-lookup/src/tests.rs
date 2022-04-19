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

use codec::Encode;
use frame_support::{assert_noop, assert_ok};
use kilt_support::{deposit::Deposit, mock::mock_origin};
use sp_runtime::{
	app_crypto::{sr25519, Pair},
	traits::IdentifyAccount,
	MultiSignature, MultiSigner,
};

use crate::{mock::*, ConnectedAccounts, ConnectedDids, ConnectionRecord, Error};

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
			assert!(ConnectedAccounts::<Test>::get(DID_00, ACCOUNT_00).is_some());
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as crate::Config>::Deposit::get()
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
			assert!(ConnectedAccounts::<Test>::get(DID_00, ACCOUNT_00).is_none());
			assert!(ConnectedAccounts::<Test>::get(DID_01, ACCOUNT_00).is_some());
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as crate::Config>::Deposit::get()
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
			let expire_at: BlockNumber = 500;
			let account_hash_alice = MultiSigner::from(pair_alice.public()).into_account();
			let sig_alice_0 = MultiSignature::from(
				pair_alice.sign(&[b"<Bytes>", &Encode::encode(&(&DID_00, expire_at))[..], b"</Bytes>"].concat()[..]),
			);
			let sig_alice_1 = MultiSignature::from(
				pair_alice.sign(&[b"<Bytes>", &Encode::encode(&(&DID_01, expire_at))[..], b"</Bytes>"].concat()[..]),
			);

			// new association. No overwrite
			assert!(DidLookup::associate_account(
				mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(),
				account_hash_alice.clone(),
				expire_at,
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
			assert!(ConnectedAccounts::<Test>::get(DID_00, &account_hash_alice).is_some());
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as crate::Config>::Deposit::get()
			);

			// overwrite existing association
			assert!(DidLookup::associate_account(
				mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into(),
				account_hash_alice.clone(),
				expire_at,
				sig_alice_1.clone()
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
			assert!(ConnectedAccounts::<Test>::get(DID_00, &account_hash_alice).is_none());
			assert!(ConnectedAccounts::<Test>::get(DID_01, &account_hash_alice).is_some());
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as crate::Config>::Deposit::get()
			);

			// overwrite existing deposit
			assert!(DidLookup::associate_account(
				mock_origin::DoubleOrigin(ACCOUNT_01, DID_01).into(),
				account_hash_alice.clone(),
				expire_at,
				sig_alice_1
			)
			.is_ok());
			assert_eq!(
				ConnectedDids::<Test>::get(&account_hash_alice),
				Some(ConnectionRecord {
					did: DID_01,
					deposit: Deposit {
						owner: ACCOUNT_01,
						amount: 10,
					}
				})
			);
			assert!(ConnectedAccounts::<Test>::get(DID_00, &account_hash_alice).is_none());
			assert!(ConnectedAccounts::<Test>::get(DID_01, &account_hash_alice).is_some());
			assert_eq!(Balances::reserved_balance(ACCOUNT_00), 0);
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_01),
				<Test as crate::Config>::Deposit::get()
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
			let expire_at: BlockNumber = 500;
			// Try signing only the encoded tuple without the <Bytes>...</Bytes> wrapper
			let sig_alice_0 = MultiSignature::from(pair_alice.sign(&Encode::encode(&(&DID_01, expire_at))[..]));

			assert_noop!(
				DidLookup::associate_account(
					mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into(),
					account_hash_alice,
					expire_at,
					sig_alice_0
				),
				Error::<Test>::NotAuthorized
			);
		});
}

#[test]
fn test_add_association_account_expired() {
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, 100), (ACCOUNT_01, 100)])
		.build()
		.execute_with(|| {
			let pair_alice = sr25519::Pair::from_seed(&*b"Alice                           ");
			let account_hash_alice = MultiSigner::from(pair_alice.public()).into_account();
			let expire_at: BlockNumber = 2;
			let sig_alice_0 = MultiSignature::from(
				pair_alice.sign(&[b"<Bytes>", &Encode::encode(&(&DID_01, expire_at))[..], b"</Bytes>"].concat()[..]),
			);
			System::set_block_number(3);

			assert_noop!(
				DidLookup::associate_account(
					mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into(),
					account_hash_alice,
					expire_at,
					sig_alice_0
				),
				Error::<Test>::OutdatedProof
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
			assert!(ConnectedAccounts::<Test>::get(DID_01, ACCOUNT_00).is_none());
			assert_eq!(Balances::reserved_balance(ACCOUNT_00), 0);
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
		.with_connections(vec![(ACCOUNT_01, DID_01, ACCOUNT_00)])
		.build()
		.execute_with(|| {
			assert!(DidLookup::remove_account_association(
				mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into(),
				ACCOUNT_00
			)
			.is_ok());
			assert_eq!(ConnectedDids::<Test>::get(ACCOUNT_00), None);
			assert!(ConnectedAccounts::<Test>::get(DID_01, ACCOUNT_00).is_none());
			assert_eq!(Balances::reserved_balance(ACCOUNT_01), 0);
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
		.with_connections(vec![(ACCOUNT_01, DID_01, ACCOUNT_00)])
		.build()
		.execute_with(|| {
			assert_noop!(
				DidLookup::remove_account_association(mock_origin::DoubleOrigin(ACCOUNT_01, DID_00).into(), ACCOUNT_00),
				Error::<Test>::NotAuthorized
			);
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_01),
				<Test as crate::Config>::Deposit::get()
			);
		});
}

#[test]
fn test_reclaim_deposit() {
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, 100), (ACCOUNT_01, 100)])
		.with_connections(vec![(ACCOUNT_01, DID_01, ACCOUNT_00)])
		.build()
		.execute_with(|| {
			assert_ok!(DidLookup::reclaim_deposit(Origin::signed(ACCOUNT_01), ACCOUNT_00));
			assert_eq!(Balances::reserved_balance(ACCOUNT_01), 0);
		});
}

#[test]
fn test_reclaim_deposit_not_authorized() {
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, 100), (ACCOUNT_01, 100)])
		.with_connections(vec![(ACCOUNT_01, DID_01, ACCOUNT_00)])
		.build()
		.execute_with(|| {
			assert_noop!(
				DidLookup::reclaim_deposit(Origin::signed(ACCOUNT_00), ACCOUNT_00),
				Error::<Test>::NotAuthorized
			);
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_01),
				<Test as crate::Config>::Deposit::get()
			);
		});
}
