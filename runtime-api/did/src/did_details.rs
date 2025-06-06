// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

use frame_system::pallet_prelude::BlockNumberFor;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};

use did::{did_details::DidPublicKeyDetails, AccountIdOf, BalanceOf, KeyIdOf};
use kilt_support::Deposit;

#[derive(Encode, Decode, TypeInfo, Clone, Debug, Eq, PartialEq, MaxEncodedLen)]
pub struct DidDetails<Key: Ord, BlockNumber, AccountId, Balance> {
	pub authentication_key: Key,
	pub key_agreement_keys: BTreeSet<Key>,
	pub delegation_key: Option<Key>,
	pub attestation_key: Option<Key>,
	pub public_keys: BTreeMap<Key, DidPublicKeyDetails<BlockNumber, AccountId>>,
	pub last_tx_counter: u64,
	pub deposit: Deposit<AccountId, Balance>,
}

impl<T: did::Config> From<did::did_details::DidDetails<T>>
	for DidDetails<KeyIdOf<T>, BlockNumberFor<T>, AccountIdOf<T>, BalanceOf<T>>
{
	fn from(did_details: did::did_details::DidDetails<T>) -> Self {
		Self {
			authentication_key: did_details.authentication_key,
			key_agreement_keys: did_details.key_agreement_keys.into(),
			delegation_key: did_details.delegation_key,
			attestation_key: did_details.attestation_key,
			public_keys: did_details.public_keys.into(),
			last_tx_counter: did_details.last_tx_counter,
			deposit: did_details.deposit,
		}
	}
}
