// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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

use frame_support::{assert_noop, assert_ok, crypto::ecdsa::ECDSAExt};
use kilt_support::{deposit::Deposit, mock::mock_origin};
use parity_scale_codec::Encode;
use sha3::{Digest, Keccak256};
use sp_runtime::{
	app_crypto::{ecdsa, sr25519, Pair},
	traits::{IdentifyAccount, Zero},
	MultiSignature, MultiSigner,
};

use crate::{
	account::{AccountId20, EthereumSignature},
	associate_account_request::{get_challenge, AssociateAccountRequest},
	linkable_account::LinkableAccountId,
	mock::*,
	signature::get_wrapped_payload,
	ConnectedAccounts, ConnectedDids, ConnectionRecord, Error,
};

#[test]
fn test_add_association_sender() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.build_and_execute_with_sanity_tests(|| {
			// new association. No overwrite
			assert!(DidLookup::associate_sender(mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into()).is_ok());
			assert_eq!(
				ConnectedDids::<Test>::get(LINKABLE_ACCOUNT_00),
				Some(ConnectionRecord {
					did: DID_00,
					deposit: Deposit {
						owner: ACCOUNT_00,
						amount: 10,
					}
				})
			);
			assert!(ConnectedAccounts::<Test>::get(DID_00, LINKABLE_ACCOUNT_00).is_some());
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as crate::Config>::Deposit::get()
			);

			// overwrite existing association
			assert!(DidLookup::associate_sender(mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into()).is_ok());
			assert_eq!(
				ConnectedDids::<Test>::get(LINKABLE_ACCOUNT_00),
				Some(ConnectionRecord {
					did: DID_01,
					deposit: Deposit {
						owner: ACCOUNT_00,
						amount: 10,
					}
				})
			);
			assert!(ConnectedAccounts::<Test>::get(DID_00, LINKABLE_ACCOUNT_00).is_none());
			assert!(ConnectedAccounts::<Test>::get(DID_01, LINKABLE_ACCOUNT_00).is_some());
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as crate::Config>::Deposit::get()
			);
		});
}

#[test]
fn test_add_association_account() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.build_and_execute_with_sanity_tests(|| {
			let pair_alice = sr25519::Pair::from_seed(b"Alice                           ");
			let expire_at: BlockNumber = 500;
			let account_hash_alice = MultiSigner::from(pair_alice.public()).into_account();
			let sig_alice_0 = MultiSignature::from(
				pair_alice.sign(&[b"<Bytes>", get_challenge(&DID_00, expire_at).as_bytes(), b"</Bytes>"].concat()[..]),
			);
			let sig_alice_1 = MultiSignature::from(
				pair_alice.sign(&[b"<Bytes>", get_challenge(&DID_01, expire_at).as_bytes(), b"</Bytes>"].concat()[..]),
			);

			// new association. No overwrite
			assert!(DidLookup::associate_account(
				mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(),
				AssociateAccountRequest::Polkadot(account_hash_alice.clone(), sig_alice_0),
				expire_at,
			)
			.is_ok());
			assert_eq!(
				ConnectedDids::<Test>::get(LinkableAccountId::from(account_hash_alice.clone())),
				Some(ConnectionRecord {
					did: DID_00,
					deposit: Deposit {
						owner: ACCOUNT_00,
						amount: 10,
					}
				})
			);
			assert!(
				ConnectedAccounts::<Test>::get(DID_00, LinkableAccountId::from(account_hash_alice.clone())).is_some()
			);
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as crate::Config>::Deposit::get()
			);

			// overwrite existing association
			let res = DidLookup::associate_account(
				mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into(),
				AssociateAccountRequest::Polkadot(account_hash_alice.clone(), sig_alice_1.clone()),
				expire_at,
			);
			if let Err(err) = res {
				println!("Error overwriting association: {:?}", err);
			}
			assert!(res.is_ok());
			assert_eq!(
				ConnectedDids::<Test>::get(LinkableAccountId::from(account_hash_alice.clone())),
				Some(ConnectionRecord {
					did: DID_01,
					deposit: Deposit {
						owner: ACCOUNT_00,
						amount: 10,
					}
				})
			);
			assert!(
				ConnectedAccounts::<Test>::get(DID_00, LinkableAccountId::from(account_hash_alice.clone())).is_none()
			);
			assert!(
				ConnectedAccounts::<Test>::get(DID_01, LinkableAccountId::from(account_hash_alice.clone())).is_some()
			);
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as crate::Config>::Deposit::get()
			);

			// overwrite existing deposit
			assert!(DidLookup::associate_account(
				mock_origin::DoubleOrigin(ACCOUNT_01, DID_01).into(),
				AssociateAccountRequest::Polkadot(account_hash_alice.clone(), sig_alice_1),
				expire_at,
			)
			.is_ok());
			assert_eq!(
				ConnectedDids::<Test>::get(LinkableAccountId::from(account_hash_alice.clone())),
				Some(ConnectionRecord {
					did: DID_01,
					deposit: Deposit {
						owner: ACCOUNT_01,
						amount: 10,
					}
				})
			);
			assert!(
				ConnectedAccounts::<Test>::get(DID_00, LinkableAccountId::from(account_hash_alice.clone())).is_none()
			);
			assert!(ConnectedAccounts::<Test>::get(DID_01, LinkableAccountId::from(account_hash_alice)).is_some());
			assert_eq!(Balances::reserved_balance(ACCOUNT_00), 0);
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_01),
				<Test as crate::Config>::Deposit::get()
			);
		});
}

#[test]
fn test_add_eth_association() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.build_and_execute_with_sanity_tests(|| {
			let expire_at: BlockNumber = 500;
			let eth_pair = ecdsa::Pair::generate().0;
			let eth_account = AccountId20(eth_pair.public().to_eth_address().unwrap());

			let wrapped_payload = get_wrapped_payload(
				get_challenge(&DID_00, expire_at).as_bytes(),
				crate::signature::WrapType::Ethereum,
			);

			let sig = eth_pair.sign_prehashed(&Keccak256::digest(wrapped_payload).try_into().unwrap());

			// new association. No overwrite
			let res = DidLookup::associate_account(
				mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(),
				AssociateAccountRequest::Ethereum(eth_account, EthereumSignature::from(sig)),
				expire_at,
			);
			assert_ok!(res);
			assert_eq!(
				ConnectedDids::<Test>::get(LinkableAccountId::from(eth_account)),
				Some(ConnectionRecord {
					did: DID_00,
					deposit: Deposit {
						owner: ACCOUNT_00,
						amount: 10,
					}
				})
			);
			assert!(ConnectedAccounts::<Test>::get(DID_00, LinkableAccountId::from(eth_account)).is_some());
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as crate::Config>::Deposit::get()
			);
		});
}

#[test]
fn test_add_association_account_invalid_signature() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.build_and_execute_with_sanity_tests(|| {
			let pair_alice = sr25519::Pair::from_seed(b"Alice                           ");
			let account_hash_alice = MultiSigner::from(pair_alice.public()).into_account();
			let expire_at: BlockNumber = 500;
			// Try signing only the encoded tuple without the <Bytes>...</Bytes> wrapper
			let sig_alice_0 = MultiSignature::from(pair_alice.sign(&Encode::encode(&(&DID_01, expire_at))[..]));

			assert_noop!(
				DidLookup::associate_account(
					mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into(),
					AssociateAccountRequest::Polkadot(account_hash_alice, sig_alice_0),
					expire_at,
				),
				Error::<Test>::NotAuthorized
			);
		});
}

#[test]
fn test_add_association_account_expired() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.build_and_execute_with_sanity_tests(|| {
			let pair_alice = sr25519::Pair::from_seed(b"Alice                           ");
			let account_hash_alice = MultiSigner::from(pair_alice.public()).into_account();
			let expire_at: BlockNumber = 2;
			let sig_alice_0 = MultiSignature::from(
				pair_alice.sign(&[b"<Bytes>", &Encode::encode(&(&DID_01, expire_at))[..], b"</Bytes>"].concat()[..]),
			);
			System::set_block_number(3);

			assert_noop!(
				DidLookup::associate_account(
					mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into(),
					AssociateAccountRequest::Polkadot(account_hash_alice, sig_alice_0),
					expire_at,
				),
				Error::<Test>::OutdatedProof
			);
		});
}

#[test]
fn test_remove_association_sender() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.with_connections(vec![(ACCOUNT_00, DID_01, LINKABLE_ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			// remove association
			assert!(DidLookup::remove_sender_association(RuntimeOrigin::signed(ACCOUNT_00)).is_ok());
			assert_eq!(ConnectedDids::<Test>::get(LinkableAccountId::from(ACCOUNT_00)), None);
			assert!(ConnectedAccounts::<Test>::get(DID_01, LinkableAccountId::from(ACCOUNT_00)).is_none());
			assert_eq!(Balances::reserved_balance(ACCOUNT_00), 0);
		});
}

#[test]
fn test_remove_association_sender_not_found() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				DidLookup::remove_sender_association(RuntimeOrigin::signed(ACCOUNT_00)),
				Error::<Test>::NotFound
			);
		});
}

#[test]
fn test_remove_association_account() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.with_connections(vec![(ACCOUNT_01, DID_01, LINKABLE_ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert!(DidLookup::remove_account_association(
				mock_origin::DoubleOrigin(ACCOUNT_00, DID_01).into(),
				LinkableAccountId::from(ACCOUNT_00.clone())
			)
			.is_ok());
			assert_eq!(ConnectedDids::<Test>::get(LinkableAccountId::from(ACCOUNT_00)), None);
			assert!(ConnectedAccounts::<Test>::get(DID_01, LinkableAccountId::from(ACCOUNT_00)).is_none());
			assert_eq!(Balances::reserved_balance(ACCOUNT_01), 0);
		});
}

#[test]
fn test_remove_association_account_not_found() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(ConnectedDids::<Test>::get(LinkableAccountId::from(ACCOUNT_00)), None);

			assert_noop!(
				DidLookup::remove_account_association(
					mock_origin::DoubleOrigin(ACCOUNT_01, DID_01).into(),
					LinkableAccountId::from(ACCOUNT_00)
				),
				Error::<Test>::NotFound
			);
		});
}

#[test]
fn test_remove_association_account_not_authorized() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.with_connections(vec![(ACCOUNT_01, DID_01, LINKABLE_ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				DidLookup::remove_account_association(
					mock_origin::DoubleOrigin(ACCOUNT_01, DID_00).into(),
					ACCOUNT_00.into()
				),
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
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.with_connections(vec![(ACCOUNT_01, DID_01, LINKABLE_ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(DidLookup::reclaim_deposit(
				RuntimeOrigin::signed(ACCOUNT_01),
				ACCOUNT_00.into()
			));
			assert_eq!(Balances::reserved_balance(ACCOUNT_01), 0);
		});
}

#[test]
fn test_reclaim_deposit_not_authorized() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.with_connections(vec![(ACCOUNT_01, DID_01, LINKABLE_ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				DidLookup::reclaim_deposit(RuntimeOrigin::signed(ACCOUNT_00), ACCOUNT_00.into()),
				Error::<Test>::NotAuthorized
			);
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_01),
				<Test as crate::Config>::Deposit::get()
			);
		});
}

// #############################################################################
// transfer deposit

#[test]
fn test_change_deposit_owner() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.with_connections(vec![(ACCOUNT_00, DID_00, LINKABLE_ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(DidLookup::change_deposit_owner(
				mock_origin::DoubleOrigin(ACCOUNT_01, DID_00).into(),
				ACCOUNT_00.into()
			));
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_01),
				<Test as crate::Config>::Deposit::get()
			);
		})
}

#[test]
fn test_change_deposit_owner_insufficient_balance() {
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50)])
		.with_connections(vec![(ACCOUNT_00, DID_00, LINKABLE_ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				DidLookup::change_deposit_owner(
					mock_origin::DoubleOrigin(ACCOUNT_01, DID_00).into(),
					ACCOUNT_00.into()
				),
				pallet_balances::Error::<Test>::InsufficientBalance
			);
		})
}

#[test]
fn test_change_deposit_owner_not_found() {
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				DidLookup::change_deposit_owner(
					mock_origin::DoubleOrigin(ACCOUNT_01, DID_00).into(),
					ACCOUNT_00.into()
				),
				Error::<Test>::NotFound
			);
		})
}

#[test]
fn test_change_deposit_owner_not_authorized() {
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50)])
		.with_connections(vec![(ACCOUNT_00, DID_00, LINKABLE_ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				DidLookup::change_deposit_owner(
					mock_origin::DoubleOrigin(ACCOUNT_01, DID_01).into(),
					ACCOUNT_00.into()
				),
				Error::<Test>::NotAuthorized
			);
		})
}

#[test]
fn test_update_deposit() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.build_and_execute_with_sanity_tests(|| {
			insert_raw_connection::<Test>(
				ACCOUNT_00,
				DID_00,
				ACCOUNT_00.into(),
				<Test as crate::Config>::Deposit::get() * 2,
			);
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as crate::Config>::Deposit::get() * 2
			);
			assert_ok!(DidLookup::update_deposit(
				RuntimeOrigin::signed(ACCOUNT_00),
				ACCOUNT_00.into()
			));
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as crate::Config>::Deposit::get()
			);
		})
}

#[test]
fn test_update_deposit_unauthorized() {
	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
			(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
		])
		.build_and_execute_with_sanity_tests(|| {
			insert_raw_connection::<Test>(
				ACCOUNT_00,
				DID_00,
				ACCOUNT_00.into(),
				<Test as crate::Config>::Deposit::get() * 2,
			);
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as crate::Config>::Deposit::get() * 2
			);
			assert_noop!(
				DidLookup::update_deposit(RuntimeOrigin::signed(ACCOUNT_01), ACCOUNT_00.into()),
				Error::<Test>::NotAuthorized
			);
		})
}
