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
use sp_runtime::{
	app_crypto::{sr25519, Pair},
	traits::IdentifyAccount,
	MultiSignature, MultiSigner,
};

use crate::{mock::*, ConnectedDids, Error};

#[test]
fn test_add_association_sender() {
	new_test_ext().execute_with(|| {
		// new association. No overwrite
		assert!(DidLookup::associate_sender(mock_origin::DoubleOrigin(ACCOUNT_00, 0).into()).is_ok());
		assert_eq!(ConnectedDids::<Test>::get(ACCOUNT_00), Some(0));

		// overwrite existing association
		assert!(DidLookup::associate_sender(mock_origin::DoubleOrigin(ACCOUNT_00, 1).into()).is_ok());
		assert_eq!(ConnectedDids::<Test>::get(ACCOUNT_00), Some(1));
	});
}

#[test]
fn test_add_association_account() {
	new_test_ext().execute_with(|| {
		let pair_alice = sr25519::Pair::from_seed(&*b"Alice                           ");
		let account_hash_alice = MultiSigner::from(pair_alice.public()).into_account();
		let sig_alice_0 = MultiSignature::from(pair_alice.sign(&Encode::encode(&0_u64)[..]));
		let sig_alice_1 = MultiSignature::from(pair_alice.sign(&Encode::encode(&1_u64)[..]));

		// new association. No overwrite
		assert!(DidLookup::associate_account(
			mock_origin::DoubleOrigin(ACCOUNT_00, 0).into(),
			account_hash_alice.clone(),
			sig_alice_0
		)
		.is_ok());
		assert_eq!(ConnectedDids::<Test>::get(&account_hash_alice), Some(0));

		// overwrite existing association
		assert!(DidLookup::associate_account(
			mock_origin::DoubleOrigin(ACCOUNT_00, 1).into(),
			account_hash_alice.clone(),
			sig_alice_1
		)
		.is_ok());
		assert_eq!(ConnectedDids::<Test>::get(&account_hash_alice), Some(1));
	});
}

#[test]
fn test_add_association_account_invalid_signature() {
	new_test_ext().execute_with(|| {
		let pair_alice = sr25519::Pair::from_seed(&*b"Alice                           ");
		let account_hash_alice = MultiSigner::from(pair_alice.public()).into_account();
		let sig_alice_0 = MultiSignature::from(pair_alice.sign(&Encode::encode(&0_u64)[..]));

		assert_noop!(
			DidLookup::associate_account(
				mock_origin::DoubleOrigin(ACCOUNT_00, 1).into(),
				account_hash_alice,
				sig_alice_0
			),
			Error::<Test>::NotAuthorized
		);
	});
}

#[test]
fn test_remove_association_sender() {
	new_test_ext().execute_with(|| {
		// insert association
		ConnectedDids::<Test>::insert(ACCOUNT_00, 1);
		assert_eq!(ConnectedDids::<Test>::get(ACCOUNT_00), Some(1));

		// remove association
		assert!(DidLookup::remove_sender_association(Origin::signed(ACCOUNT_00)).is_ok());
		assert_eq!(ConnectedDids::<Test>::get(ACCOUNT_00), None);
	});
}

#[test]
fn test_remove_association_sender_not_found() {
	new_test_ext().execute_with(|| {
		assert_eq!(ConnectedDids::<Test>::get(ACCOUNT_00), None);

		assert_noop!(
			DidLookup::remove_sender_association(Origin::signed(ACCOUNT_00)),
			Error::<Test>::AssociationNotFound
		);
	});
}

#[test]
fn test_remove_association_account() {
	new_test_ext().execute_with(|| {
		new_test_ext().execute_with(|| {
			ConnectedDids::<Test>::insert(ACCOUNT_00, 1);
			assert_eq!(ConnectedDids::<Test>::get(ACCOUNT_00), Some(1));

			assert!(
				DidLookup::remove_account_association(mock_origin::DoubleOrigin(ACCOUNT_00, 1).into(), ACCOUNT_00)
					.is_ok()
			);
			assert_eq!(ConnectedDids::<Test>::get(ACCOUNT_00), None);
		});
	});
}

#[test]
fn test_remove_association_account_not_found() {
	new_test_ext().execute_with(|| {
		assert_eq!(ConnectedDids::<Test>::get(ACCOUNT_00), None);

		assert_noop!(
			DidLookup::remove_account_association(mock_origin::DoubleOrigin(ACCOUNT_01, 1).into(), ACCOUNT_00),
			Error::<Test>::AssociationNotFound
		);
	});
}

#[test]
fn test_remove_association_account_not_authorized() {
	new_test_ext().execute_with(|| {
		// create association for DID 1
		ConnectedDids::<Test>::insert(ACCOUNT_00, 1);
		assert_eq!(ConnectedDids::<Test>::get(ACCOUNT_00), Some(1));

		// DID 0 tries to remove association
		assert_noop!(
			DidLookup::remove_account_association(mock_origin::DoubleOrigin(ACCOUNT_01, 0).into(), ACCOUNT_00),
			Error::<Test>::NotAuthorized
		);
	});
}
