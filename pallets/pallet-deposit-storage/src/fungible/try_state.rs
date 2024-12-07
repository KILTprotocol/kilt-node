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

use frame_support::{ensure, traits::fungible::InspectHold};
use frame_system::pallet_prelude::BlockNumberFor;
use kilt_support::Deposit;
use sp_runtime::{traits::CheckedAdd, TryRuntimeError};
use sp_std::collections::btree_map::BTreeMap;

use crate::{deposit::DepositEntry, AccountIdOf, BalanceOf, Config, HoldReason, SystemDeposits};

// Verify the state kept as part of the `MutateHold` implementation is
// consistent with the state of the `Currency` type.
pub(crate) fn check_fungible_consistency<T>(_n: BlockNumberFor<T>) -> Result<(), TryRuntimeError>
where
	T: Config,
{
	// Sum together all the deposits stored as part of the `MutateHold`
	// implementation, and fail if any of them does not have the expected
	// `crate::HoldReason::FungibleImpl` reason.
	let sum_of_deposits = SystemDeposits::<T>::iter_values().try_fold(
		BTreeMap::<AccountIdOf<T>, BalanceOf<T>>::new(),
		|mut sum,
		 DepositEntry {
		     reason,
		     deposit: Deposit { amount, owner },
		 }| {
			ensure!(
				reason == HoldReason::FungibleImpl.into(),
				TryRuntimeError::Other("Found a deposit reason different than the expected `HoldReason::FungibleImpl`")
			);

			// Fold the deposit amount for the current user.
			sum.entry(owner)
				.and_modify(|s| *s = s.checked_add(&amount).expect("Failed to fold deposits for user."));

			Ok::<_, TryRuntimeError>(sum)
		},
	)?;
	// We verify that the total balance on hold for the `HoldReason::FungibleImpl`
	// matches the amount of deposits stored in this pallet.
	sum_of_deposits.into_iter().try_for_each(|(owner, deposit_sum)| {
		ensure!(
			<T::Currency as InspectHold<AccountIdOf<T>>>::balance_on_hold(&HoldReason::FungibleImpl.into(), &owner)
				== deposit_sum,
			TryRuntimeError::Other("Deposit sum for user less than the expected amount")
		);
		Ok::<_, TryRuntimeError>(())
	})?;
	Ok(())
}
