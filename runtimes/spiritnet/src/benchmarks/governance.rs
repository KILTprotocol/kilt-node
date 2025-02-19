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

use frame_support::{pallet_prelude::OptionQuery, storage_alias, traits::ChangeMembers};
use runtime_common::AccountId;

// Implementation of `MembershipChanged` equivalent to using `()` but that
// returns `Some(AccountId::new([0; 32]))` in `get_prime()` only when
// benchmarking. TODO: Remove once we upgrade with a version containing the fix: https://github.com/paritytech/polkadot-sdk/pull/6439
pub struct MockMembershipChangedForBenchmarks;

#[storage_alias]
type PrimeMember = StorageValue<TipsMembership, AccountId, OptionQuery>;

impl ChangeMembers<AccountId> for MockMembershipChangedForBenchmarks {
	fn change_members_sorted(incoming: &[AccountId], outgoing: &[AccountId], sorted_new: &[AccountId]) {
		<()>::change_members_sorted(incoming, outgoing, sorted_new)
	}

	fn get_prime() -> Option<AccountId> {
		PrimeMember::get()
	}

	fn set_prime(prime: Option<AccountId>) {
		PrimeMember::set(prime)
	}
}
