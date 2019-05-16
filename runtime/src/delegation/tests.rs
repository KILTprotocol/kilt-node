// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019  BOTLabs GmbH

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


use super::*;
use system;
use runtime_io::with_externalities;
use primitives::{H256, H512, Blake2Hasher, ed25519 as x25519};
use primitives::*;
use support::{impl_outer_origin, assert_ok, assert_err};
use parity_codec::Encode;

use runtime_primitives::{
    BuildStorage, traits::{BlakeTwo256, IdentityLookup}, testing::{Digest, DigestItem, Header}
};

impl_outer_origin! {
    pub enum Origin for Test {}
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
impl system::Trait for Test {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Digest = Digest;
    type AccountId = <x25519::Signature as Verify>::Signer;
    type Header = Header;
    type Event = ();
    type Log = DigestItem;
    type Lookup = IdentityLookup<Self::AccountId>;
}

impl ctype::Trait for Test {
    type Event = ();
}

impl error::Trait for Test {
    type Event = ();
    type ErrorCode = u16;
}

impl Trait for Test {
    type Event = ();
    type Signature = x25519::Signature;
    type Signer = <x25519::Signature as Verify>::Signer;
    type DelegationNodeId = H256;
}

type CType = ctype::Module<Test>;
type Delegation = Module<Test>;

fn hash_to_u8<T : Encode> (hash : T) -> Vec<u8>{
    return hash.encode();
}

fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
    system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
}

#[test]
fn check_add_and_revoke_delegations() {
    with_externalities(&mut new_test_ext(), || {
        let pair_alice = x25519::Pair::from_seed(*b"Alice                           ");
        let account_hash_alice = pair_alice.public();
        let pair_bob = x25519::Pair::from_seed(*b"Bob                             ");
        let account_hash_bob = pair_bob.public();
        let pair_charlie = x25519::Pair::from_seed(*b"Charlie                         ");
        let account_hash_charlie = pair_charlie.public();

        let ctype_hash = H256::from_low_u64_be(1);
        let id_level_0 = H256::from_low_u64_be(1);
        let id_level_1 = H256::from_low_u64_be(2);
        let id_level_2_1 = H256::from_low_u64_be(21);
        let id_level_2_2 = H256::from_low_u64_be(22);
        let id_level_2_2_1 = H256::from_low_u64_be(221);

        assert_ok!(CType::add(Origin::signed(account_hash_alice.clone()), ctype_hash.clone()));

        assert_ok!(Delegation::create_root(Origin::signed(account_hash_alice.clone()), id_level_0.clone(), ctype_hash.clone()));
        assert_err!(Delegation::create_root(Origin::signed(account_hash_alice.clone()), id_level_0.clone(), ctype_hash.clone()),
            Delegation::ERROR_ROOT_ALREADY_EXISTS.1);
        assert_err!(Delegation::create_root(Origin::signed(account_hash_alice.clone()), id_level_1.clone(), H256::from_low_u64_be(2)),
            CType::ERROR_CTYPE_NOT_FOUND.1);

        assert_ok!(Delegation::add_delegation(Origin::signed(account_hash_alice.clone()), id_level_1.clone(), id_level_0.clone(), 
            None, account_hash_bob.clone(), Permissions::DELEGATE, 
            x25519::Signature::from(pair_bob.sign(&hash_to_u8(
                Delegation::calculate_hash(id_level_1.clone(), id_level_0.clone(), None, Permissions::DELEGATE))))));
        assert_err!(Delegation::add_delegation(Origin::signed(account_hash_alice.clone()), id_level_1.clone(), id_level_0.clone(), 
            None, account_hash_bob.clone(), Permissions::DELEGATE, 
            x25519::Signature::from(pair_bob.sign(&hash_to_u8(
                Delegation::calculate_hash(id_level_1.clone(), id_level_0.clone(), None, Permissions::DELEGATE))))),
            Delegation::ERROR_DELEGATION_ALREADY_EXISTS.1);
        assert_err!(Delegation::add_delegation(Origin::signed(account_hash_bob.clone()), id_level_2_1.clone(), id_level_0.clone(), 
            Some(id_level_1.clone()), account_hash_charlie.clone(), Permissions::ATTEST, x25519::Signature::from_h512(H512::from_low_u64_be(0))),
            Delegation::ERROR_BAD_DELEGATION_SIGNATURE.1);
        assert_err!(Delegation::add_delegation(Origin::signed(account_hash_charlie.clone()), id_level_2_1.clone(), id_level_0.clone(), 
            None, account_hash_bob.clone(), Permissions::DELEGATE, 
            x25519::Signature::from(pair_bob.sign(&hash_to_u8(
                Delegation::calculate_hash(id_level_2_1.clone(), id_level_0.clone(), None, Permissions::DELEGATE))))),
            Delegation::ERROR_NOT_OWNER_OF_ROOT.1);
        assert_err!(Delegation::add_delegation(Origin::signed(account_hash_alice.clone()), id_level_2_1.clone(), id_level_1.clone(), 
            None, account_hash_bob.clone(), Permissions::DELEGATE, 
            x25519::Signature::from(pair_bob.sign(&hash_to_u8(
                Delegation::calculate_hash(id_level_2_1.clone(), id_level_1.clone(), None, Permissions::DELEGATE))))),
            Delegation::ERROR_ROOT_NOT_FOUND.1);


        assert_ok!(Delegation::add_delegation(Origin::signed(account_hash_bob.clone()), id_level_2_1.clone(), id_level_0.clone(), 
            Some(id_level_1.clone()), account_hash_charlie.clone(), Permissions::ATTEST, 
            x25519::Signature::from(pair_charlie.sign(&hash_to_u8(
                Delegation::calculate_hash(id_level_2_1.clone(), id_level_0.clone(), Some(id_level_1.clone()), Permissions::ATTEST))))));
        assert_err!(Delegation::add_delegation(Origin::signed(account_hash_alice.clone()), id_level_2_2.clone(), id_level_0.clone(), 
            Some(id_level_1.clone()), account_hash_charlie.clone(), Permissions::ATTEST, 
            x25519::Signature::from(pair_charlie.sign(&hash_to_u8(
                Delegation::calculate_hash(id_level_2_2.clone(), id_level_0.clone(), Some(id_level_1.clone()), Permissions::ATTEST))))),
            Delegation::ERROR_NOT_OWNER_OF_PARENT.1);
        assert_err!(Delegation::add_delegation(Origin::signed(account_hash_charlie.clone()), id_level_2_2.clone(), id_level_0.clone(), 
            Some(id_level_2_1.clone()), account_hash_alice.clone(), Permissions::ATTEST, 
            x25519::Signature::from(pair_alice.sign(&hash_to_u8(
                Delegation::calculate_hash(id_level_2_2.clone(), id_level_0.clone(), Some(id_level_2_1.clone()), Permissions::ATTEST))))),
            Delegation::ERROR_NOT_AUTHORIZED_TO_DELEGATE.1);
        assert_err!(Delegation::add_delegation(Origin::signed(account_hash_bob.clone()), id_level_2_2.clone(), id_level_0.clone(), 
            Some(id_level_0.clone()), account_hash_charlie.clone(), Permissions::ATTEST, 
            x25519::Signature::from(pair_charlie.sign(&hash_to_u8(
                Delegation::calculate_hash(id_level_2_2.clone(), id_level_0.clone(), Some(id_level_0.clone()), Permissions::ATTEST))))),
            Delegation::ERROR_PARENT_NOT_FOUND.1);
        
        assert_ok!(Delegation::add_delegation(Origin::signed(account_hash_bob.clone()), id_level_2_2.clone(), id_level_0.clone(), 
            Some(id_level_1.clone()), account_hash_charlie.clone(), Permissions::ATTEST | Permissions::DELEGATE, 
            x25519::Signature::from(pair_charlie.sign(&hash_to_u8(
                Delegation::calculate_hash(id_level_2_2.clone(), id_level_0.clone(), Some(id_level_1.clone()), 
                Permissions::ATTEST | Permissions::DELEGATE))))));

        assert_ok!(Delegation::add_delegation(Origin::signed(account_hash_charlie.clone()), id_level_2_2_1.clone(), id_level_0.clone(), 
            Some(id_level_2_2.clone()), account_hash_alice.clone(), Permissions::ATTEST, 
            x25519::Signature::from(pair_alice.sign(&hash_to_u8(
                Delegation::calculate_hash(id_level_2_2_1.clone(), id_level_0.clone(), Some(id_level_2_2.clone()), Permissions::ATTEST))))));

        
        let root = Delegation::root(id_level_0.clone());
        assert_eq!(root.0, ctype_hash.clone());
        assert_eq!(root.1, account_hash_alice.clone());
        assert_eq!(root.2, false);

        let delegation_1 = Delegation::delegation(id_level_1.clone());
        assert_eq!(delegation_1.0, id_level_0.clone());
        assert_eq!(delegation_1.1, None);
        assert_eq!(delegation_1.2, account_hash_bob.clone());
        assert_eq!(delegation_1.3, Permissions::DELEGATE);
        assert_eq!(delegation_1.4, false);

        let delegation_2 = Delegation::delegation(id_level_2_2.clone());
        assert_eq!(delegation_2.0, id_level_0.clone());
        assert_eq!(delegation_2.1, Some(id_level_1.clone()));
        assert_eq!(delegation_2.2, account_hash_charlie.clone());
        assert_eq!(delegation_2.3, Permissions::ATTEST | Permissions::DELEGATE);
        assert_eq!(delegation_2.4, false);

        let children = Delegation::children(id_level_1.clone());
        assert_eq!(children.len(), 2);
        assert_eq!(children[0], id_level_2_1.clone());
        assert_eq!(children[1], id_level_2_2.clone());

        // check is_delgating
        assert_eq!(Delegation::is_delegating(&account_hash_alice, &id_level_1), Ok(true));
        assert_eq!(Delegation::is_delegating(&account_hash_alice, &id_level_2_1), Ok(true));
        assert_eq!(Delegation::is_delegating(&account_hash_bob, &id_level_2_1), Ok(true));
        assert_eq!(Delegation::is_delegating(&account_hash_charlie, &id_level_2_1), Ok(true));
        assert_eq!(Delegation::is_delegating(&account_hash_charlie, &id_level_1), Ok(false));
        assert_err!(Delegation::is_delegating(&account_hash_charlie, &id_level_0), Delegation::ERROR_DELEGATION_NOT_FOUND.1);

        assert_err!(Delegation::revoke_delegation(Origin::signed(account_hash_charlie.clone()), H256::from_low_u64_be(999)),
            Delegation::ERROR_DELEGATION_NOT_FOUND.1);
        assert_err!(Delegation::revoke_delegation(Origin::signed(account_hash_charlie.clone()), id_level_1.clone()),
            Delegation::ERROR_NOT_PERMITTED_TO_REVOKE.1);
        assert_ok!(Delegation::revoke_delegation(Origin::signed(account_hash_charlie.clone()), id_level_2_2.clone()));
        
        assert_eq!(Delegation::delegation(id_level_2_2.clone()).4, true);
        assert_eq!(Delegation::delegation(id_level_2_2_1.clone()).4, true);

        assert_err!(Delegation::revoke_root(Origin::signed(account_hash_bob.clone()), H256::from_low_u64_be(999)),
            Delegation::ERROR_ROOT_NOT_FOUND.1);
        assert_err!(Delegation::revoke_root(Origin::signed(account_hash_bob.clone()), id_level_0.clone()),
            Delegation::ERROR_NOT_PERMITTED_TO_REVOKE.1);
        assert_ok!(Delegation::revoke_root(Origin::signed(account_hash_alice.clone()), id_level_0.clone()));
        
        assert_eq!(Delegation::root(id_level_0.clone()).2, true);
        assert_eq!(Delegation::delegation(id_level_1.clone()).4, true);
        assert_eq!(Delegation::delegation(id_level_2_1.clone()).4, true);
    });
}
