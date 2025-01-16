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

#![allow(unused_imports)]

use frame_benchmarking::v2::benchmarks;
use runtime_common::DidIdentifier;
use sp_std::marker::PhantomData;

use crate::{
	kilt::{did::DotNamesDeployment, UniqueLinkingDeployment},
	DotName, Web3Name,
};

pub(crate) trait Config {}

pub struct Pallet<T: Config>(PhantomData<T>);

#[benchmarks]
mod benchmarks {
	use did::traits::DidDeletionHook;
	use pallet_did_lookup::linkable_account::LinkableAccountId;
	use runtime_common::{DidIdentifier, EnsureNoLinkedAccountDeletionHook, EnsureNoLinkedWeb3NameDeletionHook};
	use sp_core::Hasher;
	use sp_runtime::{traits::BlakeTwo256, AccountId32};

	use crate::Runtime;

	use super::*;

	const MAX_NUMBER_0F_WEB3_NAMES: usize = 100_000;
	const MAX_NUMBER_0F_DOT_NAMES: usize = 100_000;

	const LINKED_DID: DidIdentifier = DidIdentifier::new([100u8; 32]);
	const MAX_NUMBER_0F_WEB3_ACCOUNTS: usize = 100_000;

	fn fill_web3_names() {
		(0..MAX_NUMBER_0F_WEB3_NAMES).for_each(|i| {
			let owner = DidIdentifier::new(BlakeTwo256::hash(i.to_le_bytes().as_ref()).into());
			pallet_web3_names::Names::<Runtime>::insert(owner, Web3Name::try_from(b"test".to_vec()).unwrap());
		});
	}

	fn fill_dot_names() {
		(0..MAX_NUMBER_0F_DOT_NAMES).for_each(|i| {
			let owner = DidIdentifier::new(BlakeTwo256::hash(i.to_le_bytes().as_ref()).into());
			pallet_web3_names::Names::<Runtime, DotNamesDeployment>::insert(
				owner,
				DotName::try_from(b"test.dot".to_vec()).unwrap(),
			);
		});
	}

	fn fill_web3_accounts() {
		(0..MAX_NUMBER_0F_WEB3_ACCOUNTS).for_each(|i| {
			let linked_account = AccountId32::new(BlakeTwo256::hash(i.to_le_bytes().as_ref()).into());
			pallet_did_lookup::ConnectedAccounts::<Runtime>::insert(
				LINKED_DID,
				LinkableAccountId::from(linked_account),
				(),
			);
		});
	}

	// For Dotnames, we can only insert a single account per DID.
	fn fill_dot_accounts() {
		let linked_account = AccountId32::new([0u8; 32]);
		pallet_did_lookup::ConnectedAccounts::<Runtime, UniqueLinkingDeployment>::insert(
			LINKED_DID,
			LinkableAccountId::from(linked_account),
			(),
		);
	}

	#[benchmark]
	fn read_web3_name() {
		fill_web3_names();
		let did = DidIdentifier::new(BlakeTwo256::hash(1u32.to_le_bytes().as_ref()).into());
		#[block]
		{
			let _ = <EnsureNoLinkedWeb3NameDeletionHook<0, 0, ()> as DidDeletionHook<Runtime>>::can_delete(&did);
		}
		// We make sure it actually returns what we expect.
		assert_eq!(
			<EnsureNoLinkedWeb3NameDeletionHook<0, 0, ()> as DidDeletionHook<Runtime>>::can_delete(&did),
			Ok(())
		);
	}

	#[benchmark]
	fn read_dot_name() {
		fill_dot_names();
		let did = DidIdentifier::new(BlakeTwo256::hash(1u32.to_le_bytes().as_ref()).into());
		#[block]
		{
			let _ =
				<EnsureNoLinkedWeb3NameDeletionHook<0, 0, DotNamesDeployment> as DidDeletionHook<Runtime>>::can_delete(
					&did,
				);
		}
		// We make sure it actually returns what we expect.
		assert_eq!(
			<EnsureNoLinkedWeb3NameDeletionHook<0, 0, DotNamesDeployment> as DidDeletionHook<Runtime>>::can_delete(
				&did
			),
			Ok(())
		);
	}

	#[benchmark]
	fn read_web3_account() {
		fill_web3_accounts();
		#[block]
		{
			let _ = <EnsureNoLinkedAccountDeletionHook<0, 0, ()> as DidDeletionHook<Runtime>>::can_delete(&LINKED_DID);
		}
		// We make sure it actually returns what we expect.
		assert_eq!(
			<EnsureNoLinkedAccountDeletionHook<0, 0, ()> as DidDeletionHook<Runtime>>::can_delete(&LINKED_DID),
			Ok(())
		);
	}

	#[benchmark]
	fn read_dot_account() {
		fill_dot_accounts();
		#[block]
		{
			let _ = <EnsureNoLinkedAccountDeletionHook<0, 0, UniqueLinkingDeployment> as DidDeletionHook<Runtime>>::can_delete(
				&LINKED_DID,
			);
		}
		// We make sure it actually returns what we expect.
		assert_eq!(
			<EnsureNoLinkedAccountDeletionHook<0, 0, UniqueLinkingDeployment> as DidDeletionHook<Runtime>>::can_delete(
				&LINKED_DID
			),
			Ok(())
		);
	}
}
