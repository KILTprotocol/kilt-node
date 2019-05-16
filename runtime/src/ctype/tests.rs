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

use primitives::{Blake2Hasher, H256};
use runtime_io::with_externalities;
use system;
use support::{impl_outer_origin, assert_ok, assert_err};
use runtime_primitives::{
    testing::{Digest, DigestItem, Header},
    traits::{BlakeTwo256,IdentityLookup},
    BuildStorage,
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
    type AccountId = H256;
    type Header = Header;
    type Event = ();
    type Log = DigestItem;
    type Lookup = IdentityLookup<H256>;
}

impl error::Trait for Test {
    type Event = ();
    type ErrorCode = u16;
}

impl Trait for Test {
    type Event = ();
}

type CType = Module<Test>;

fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
    system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
}

#[test]
fn it_works_for_default_value() {
    with_externalities(&mut new_test_ext(), || {
        let account = H256::from_low_u64_be(1);
        let ctype_hash = H256::from_low_u64_be(2);
        assert_ok!(
            CType::add(
                Origin::signed(account.clone()),
                ctype_hash.clone()
            )
        );
        assert_eq!(<CTYPEs<Test>>::exists(ctype_hash), true);
        assert_eq!(CType::ctypes(ctype_hash.clone()), account.clone());
        assert_err!(
            CType::add(
                Origin::signed(account.clone()),
                ctype_hash.clone()
            ),
            CType::ERROR_CTYPE_ALREADY_EXISTS.1
        );
    });
}