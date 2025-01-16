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

use crate::{
	kilt::did::{
		DidDeletionHookBenchmarkWeightInfo, WORST_CASE_DOT_NAME_LINKING_STORAGE_READ, WORST_CASE_DOT_NAME_STORAGE_READ,
		WORST_CASE_WEB3_NAME_LINKING_STORAGE_READ, WORST_CASE_WEB3_NAME_STORAGE_READ,
	},
	weights::did_deletion_hooks,
	Runtime,
};

#[test]
fn test_worst_case_web3_name_storage_read() {
	assert_eq!(
		WORST_CASE_WEB3_NAME_STORAGE_READ,
		did_deletion_hooks::WeightInfo::<Runtime>::read_web3_name()
	);
}

#[test]
fn test_worst_case_dot_name_storage_read() {
	assert_eq!(
		WORST_CASE_DOT_NAME_STORAGE_READ,
		did_deletion_hooks::WeightInfo::<Runtime>::read_dot_name()
	);
}

#[test]
fn test_worst_case_web3_name_linked_account_storage_read() {
	assert_eq!(
		WORST_CASE_WEB3_NAME_LINKING_STORAGE_READ,
		did_deletion_hooks::WeightInfo::<Runtime>::read_web3_account()
	);
}

#[test]
fn test_worst_case_dot_name_linked_account_storage_read() {
	assert_eq!(
		WORST_CASE_DOT_NAME_LINKING_STORAGE_READ,
		did_deletion_hooks::WeightInfo::<Runtime>::read_dot_account()
	);
}
