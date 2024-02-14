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

use did::did_details::DidVerificationKey;

use crate::{
	constants::dip_provider::MAX_LINKED_ACCOUNTS,
	dip::{
		merkle::v0::generate_commitment,
		mock::{create_linked_info, TestRuntime, ACCOUNT},
	},
};

#[test]
fn generate_commitment_for_complete_info() {
	let linked_info = create_linked_info(DidVerificationKey::Account(ACCOUNT), true, MAX_LINKED_ACCOUNTS);
	let commitment_result = generate_commitment::<TestRuntime, MAX_LINKED_ACCOUNTS>(&linked_info);
	assert!(commitment_result.is_ok());
}

#[test]
fn generate_commitment_for_did_details() {
	let linked_info = create_linked_info(DidVerificationKey::Account(ACCOUNT), false, 0);
	let commitment_result = generate_commitment::<TestRuntime, MAX_LINKED_ACCOUNTS>(&linked_info);
	assert!(commitment_result.is_ok());
}

#[test]
fn generate_commitment_for_did_details_and_web3name() {
	let linked_info = create_linked_info(DidVerificationKey::Account(ACCOUNT), true, 0);
	let commitment_result = generate_commitment::<TestRuntime, MAX_LINKED_ACCOUNTS>(&linked_info);
	assert!(commitment_result.is_ok());
}

#[test]
fn generate_commitment_for_did_details_and_max_linked_accounts() {
	let linked_info = create_linked_info(DidVerificationKey::Account(ACCOUNT), false, MAX_LINKED_ACCOUNTS);
	let commitment_result = generate_commitment::<TestRuntime, MAX_LINKED_ACCOUNTS>(&linked_info);
	assert!(commitment_result.is_ok());
}
