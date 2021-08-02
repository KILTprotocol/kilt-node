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

/// Deprecated types used in versions 1 to 4 (Vec instead of BoundedVec).
use crate::Config;

pub(crate) mod v1_v4 {
	use crate::types::{AccountIdOf, BalanceOf, CollatorStatus, Stake};
	use frame_support::dispatch::fmt::Debug;
	#[cfg(feature = "std")]
	use serde::{Deserialize, Serialize};
	use sp_runtime::{
		codec::{Decode, Encode},
		RuntimeDebug,
	};
	use sp_std::prelude::*;

	use super::*;

	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(RuntimeDebug, PartialEq, Eq, Encode, Decode, Default, Clone)]
	pub struct OrderedSet<T>(Vec<T>);
	impl<T: Ord> OrderedSet<T> {
		pub(crate) fn sort_greatest_to_lowest(&mut self) {
			self.0.sort_by(|a, b| b.cmp(a));
		}
	}

	#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
	pub struct Collator<AccountId, Balance>
	where
		AccountId: Eq + Ord + Debug,
		Balance: Eq + Ord + Debug,
	{
		pub(crate) id: AccountId,
		pub(crate) stake: Balance,
		pub(crate) delegators: OrderedSet<Stake<AccountId, Balance>>,
		pub(crate) total: Balance,
		pub(crate) state: CollatorStatus,
	}
	pub(crate) type CollatorOf<T> = Collator<AccountIdOf<T>, BalanceOf<T>>;

	#[derive(Encode, Decode, RuntimeDebug, PartialEq)]
	pub struct Delegator<AccountId: Eq + Ord, Balance: Eq + Ord> {
		pub(crate) delegations: OrderedSet<Stake<AccountId, Balance>>,
		pub(crate) total: Balance,
	}

	pub(crate) mod storage {
		use frame_support::{decl_module, decl_storage};
		use sp_std::prelude::*;

		use super::*;

		decl_module! {
			pub struct OldPallet<T: Config> for enum Call where origin: T::Origin {}
		}

		decl_storage! {
			pub(crate) trait Store for OldPallet<T: Config> as ParachainStaking {
				pub(crate) DelegatorState get(fn delegator_state): map hasher(twox_64_concat) T::AccountId => Option<Delegator<T::AccountId, BalanceOf<T>>>;
				pub(crate) CollatorState get(fn collator_state): map hasher(twox_64_concat) T::AccountId => Option<Collator<T::AccountId, BalanceOf<T>>>;
				pub(crate) SelectedCandidates get(fn selected_candidates): Vec<T::AccountId>;
				pub(crate) CandidatePool get(fn candidate_pool): OrderedSet<Stake<T::AccountId, BalanceOf<T>>>;
			}
		}
	}
}
