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

//! Pallet to store namespaced deposits for the configured `Currency`. It allows
//! the original payer of a deposit to claim it back, triggering a hook to
//! optionally perform related actions somewhere else in the runtime.
//! Each deposit is identified by a namespace and a key. There cannot be two
//! equal keys under the same namespace, but the same key can be present under
//! different namespaces.

use frame_support::{ensure, traits::fungible::InspectHold};
use frame_system::pallet_prelude::BlockNumberFor;
use kilt_support::Deposit;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::{traits::CheckedAdd, TryRuntimeError};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

use crate::{deposit::DepositEntry, AccountIdOf, BalanceOf, Config, Deposits, HoldReason};

pub(crate) fn try_state<T>(n: BlockNumberFor<T>) -> Result<(), TryRuntimeError>
where
	T: Config,
{
	crate::fungible::try_state::check_fungible_consistency::<T>(n)?;
	check_regular_deposits_consistency::<T>(n)?;

	Ok(())
}

// Verify the state outside of the `MutateHold` implementation does not
// interfere with the `MutateHold` state.
fn check_regular_deposits_consistency<T>(_n: BlockNumberFor<T>) -> Result<(), TryRuntimeError>
where
	T: Config,
{
	// Sum together all the deposits stored not part of the `MutateHold`
	// implementation, and fail if any of them has the unexpected
	// `crate::HoldReason::FungibleImpl` reason.
	let sum_of_deposits = Deposits::<T>::iter_values().try_fold(
		// We can't use `T::RuntimeHoldReason` as a key because it does not implement `Ord`, so we `.encode()` it here.
		BTreeMap::<(AccountIdOf<T>, Vec<u8>), BalanceOf<T>>::new(),
		|mut sum,
		 DepositEntry {
		     reason,
		     deposit: Deposit { amount, owner },
		 }|
		 -> Result<_, TryRuntimeError> {
			// Regular deposits should not interfere with the `MutateHold` implementation
			// state.
			ensure!(
				reason != HoldReason::FungibleImpl.into(),
				TryRuntimeError::Other("Found a deposit reason `HoldReason::FungibleImpl`, which is unexpected.")
			);

			// Fold the deposit amount for the current user.
			let entry = sum.entry((owner, reason.encode())).or_default();
			*entry = entry.checked_add(&amount).expect("Failed to fold deposits for user.");

			Ok(sum)
		},
	)?;
	// We verify that the total balance on hold for each hold reason matches the
	// amount of deposits stored in this pallet.
	sum_of_deposits.into_iter().try_for_each(
		|((owner, encoded_runtime_hold_reason), deposit_sum)| -> Result<_, TryRuntimeError> {
			let runtime_hold_reason =
				T::RuntimeHoldReason::decode(&mut encoded_runtime_hold_reason.as_slice()).unwrap();
			ensure!(
				<T::Currency as InspectHold<AccountIdOf<T>>>::balance_on_hold(&runtime_hold_reason, &owner)
					== deposit_sum,
				TryRuntimeError::Other("Deposit sum for user different than the expected amount")
			);
			Ok(())
		},
	)?;
	Ok(())
}
