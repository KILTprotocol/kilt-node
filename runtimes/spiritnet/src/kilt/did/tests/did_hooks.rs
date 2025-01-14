// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

use pallet_did_lookup::linkable_account::LinkableAccountId;
use parity_scale_codec::MaxEncodedLen;

use crate::{
	kilt::did::{
		WORST_CASE_DOT_NAME_STORAGE_READ_SIZE, WORST_CASE_LINKING_STORAGE_READ_SIZE,
		WORST_CASE_WEB3_NAME_STORAGE_READ_SIZE,
	},
	DotName, Web3Name,
};

#[test]
fn test_worst_case_web3_name_storage_read() {
	assert_eq!(
		Web3Name::max_encoded_len() as u64,
		WORST_CASE_WEB3_NAME_STORAGE_READ_SIZE
	);
}

#[test]
fn test_worst_case_dot_name_storage_read() {
	assert_eq!(DotName::max_encoded_len() as u64, WORST_CASE_DOT_NAME_STORAGE_READ_SIZE);
}

#[test]
fn test_worst_case_web3_name_linked_account_storage_read() {
	assert_eq!(
		LinkableAccountId::max_encoded_len() as u64,
		WORST_CASE_LINKING_STORAGE_READ_SIZE
	);
}
