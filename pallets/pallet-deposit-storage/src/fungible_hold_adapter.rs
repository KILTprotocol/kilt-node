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

use frame_support::traits::{
	fungible::{Dust, Inspect, InspectHold, MutateHold, Unbalanced, UnbalancedHold},
	tokens::{DepositConsequence, Fortitude, Preservation, Provenance},
};
use sp_runtime::{DispatchError, DispatchResult};

use crate::{Config, Pallet};

impl<T> Inspect<T::AccountId> for Pallet<T>
where
	T: Config,
{
	type Balance = <T::Currency as Inspect<T::AccountId>>::Balance;

	fn total_issuance() -> Self::Balance {
		<T::Currency as Inspect<T::AccountId>>::total_issuance()
	}

	fn total_balance(who: &T::AccountId) -> Self::Balance {
		<T::Currency as Inspect<T::AccountId>>::total_balance(who)
	}

	fn minimum_balance() -> Self::Balance {
		<T::Currency as Inspect<T::AccountId>>::minimum_balance()
	}

	fn balance(who: &T::AccountId) -> Self::Balance {
		<T::Currency as Inspect<T::AccountId>>::balance(who)
	}

	fn reducible_balance(who: &T::AccountId, preservation: Preservation, force: Fortitude) -> Self::Balance {
		<T::Currency as Inspect<T::AccountId>>::reducible_balance(who, preservation, force)
	}

	fn can_deposit(who: &T::AccountId, amount: Self::Balance, provenance: Provenance) -> DepositConsequence {
		<T::Currency as Inspect<T::AccountId>>::can_deposit(who, amount, provenance)
	}

	fn can_withdraw(
		who: &T::AccountId,
		amount: Self::Balance,
	) -> frame_support::traits::tokens::WithdrawConsequence<Self::Balance> {
		<T::Currency as Inspect<T::AccountId>>::can_withdraw(who, amount)
	}
}

impl<T> InspectHold<T::AccountId> for Pallet<T>
where
	T: Config,
{
	type Reason = T::Namespace;

	fn total_balance_on_hold(who: &T::AccountId) -> Self::Balance {
		<T::Currency as InspectHold<T::AccountId>>::total_balance_on_hold(who)
	}

	fn balance_on_hold(reason: &Self::Reason, who: &T::AccountId) -> Self::Balance {
		<T::Currency as InspectHold<T::AccountId>>::balance_on_hold(reason, who)
	}
}

impl<T> Unbalanced<T::AccountId> for Pallet<T>
where
	T: Config,
{
	fn handle_dust(dust: Dust<T::AccountId, Self>) {
		<T::Currency as Unbalanced<T::AccountId>>::handle_dust(Dust(dust.0));
	}

	fn write_balance(who: &T::AccountId, amount: Self::Balance) -> Result<Option<Self::Balance>, DispatchError> {
		<T::Currency as Unbalanced<T::AccountId>>::write_balance(who, amount)
	}

	fn set_total_issuance(amount: Self::Balance) {
		<T::Currency as Unbalanced<T::AccountId>>::set_total_issuance(amount);
	}
}

impl<T> UnbalancedHold<T::AccountId> for Pallet<T>
where
	T: Config,
{
	fn set_balance_on_hold(reason: &Self::Reason, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<T::Currency as UnbalancedHold<T::AccountId>>::set_balance_on_hold(reason, who, amount)
	}
}

impl<T> MutateHold<T::AccountId> for Pallet<T> where T: Config {}
