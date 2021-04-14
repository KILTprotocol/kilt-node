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

use crate::{self as ctype, mock::*};
use did::mock as did_mock;

#[test]
fn it_works_for_default_value() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_enc_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_ed25519_authentication_key(true);
}

// new_test_ext().execute_with(|| {
// 	let pair = ed25519::Pair::from_seed(&*b"Alice                           ");
// 	let ctype_hash = H256::from_low_u64_be(1);
// 	let account = MultiSigner::from(pair.public()).into_account();
// 	assert_ok!(CType::add(Origin::signed(account.clone()), ctype_hash));
// 	assert_eq!(<CTYPEs<Test>>::contains_key(ctype_hash), true);
// 	assert_eq!(CType::ctypes(ctype_hash), Some(account.clone()));
// 	assert_noop!(
// 		CType::add(Origin::signed(account), ctype_hash),
// 		Error::<Test>::AlreadyExists
// 	);
// });
