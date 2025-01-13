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

use frame_support::traits::{
	fungible::{Balanced, Dust, Inspect, InspectHold, Mutate, MutateHold, Unbalanced, UnbalancedHold},
	tokens::{Balance as BalanceT, DepositConsequence, Fortitude, Preservation, Provenance, WithdrawConsequence},
};
use parity_scale_codec::Encode;
use scale_info::{prelude::string::String, TypeInfo};
use sp_runtime::{DispatchError, DispatchResult, TryRuntimeError};
use sp_std::marker::PhantomData;

/// Logs the error message and returns "Sanity test error"
pub fn log_and_return_error_message(error_message: String) -> TryRuntimeError {
	log::error!("{}", error_message);
	TryRuntimeError::Other("Test")
}

// Mock currency that implements all required traits, allowing test runtimes to
// not include the actual `pallet_balances` pallet. This mock currency is useful
// for mocks in which a `Currency` is required but not relevant for the goal of
// the tests.
pub struct MockCurrency<Balance, RuntimeHoldReason>(PhantomData<(Balance, RuntimeHoldReason)>);

impl<AccountId, Balance, RuntimeHoldReason> MutateHold<AccountId> for MockCurrency<Balance, RuntimeHoldReason>
where
	Balance: BalanceT,
	RuntimeHoldReason: Encode + TypeInfo + 'static,
{
}

impl<AccountId, Balance, RuntimeHoldReason> UnbalancedHold<AccountId> for MockCurrency<Balance, RuntimeHoldReason>
where
	Balance: BalanceT,
	RuntimeHoldReason: Encode + TypeInfo + 'static,
{
	fn set_balance_on_hold(_reason: &Self::Reason, _who: &AccountId, _amount: Self::Balance) -> DispatchResult {
		Ok(())
	}
}

impl<AccountId, Balance, RuntimeHoldReason> InspectHold<AccountId> for MockCurrency<Balance, RuntimeHoldReason>
where
	Balance: BalanceT,
	RuntimeHoldReason: Encode + TypeInfo + 'static,
{
	type Reason = RuntimeHoldReason;

	fn total_balance_on_hold(_who: &AccountId) -> Self::Balance {
		Self::Balance::default()
	}

	fn balance_on_hold(_reason: &Self::Reason, _who: &AccountId) -> Self::Balance {
		Self::Balance::default()
	}
}

impl<AccountId, Balance, RuntimeHoldReason> Mutate<AccountId> for MockCurrency<Balance, RuntimeHoldReason>
where
	AccountId: Eq,
	Balance: BalanceT,
{
}

impl<AccountId, Balance, RuntimeHoldReason> Inspect<AccountId> for MockCurrency<Balance, RuntimeHoldReason>
where
	Balance: BalanceT,
{
	type Balance = Balance;

	fn active_issuance() -> Self::Balance {
		Self::Balance::default()
	}

	fn balance(_who: &AccountId) -> Self::Balance {
		Self::Balance::default()
	}

	fn can_deposit(_who: &AccountId, _amount: Self::Balance, _provenance: Provenance) -> DepositConsequence {
		DepositConsequence::Success
	}

	fn can_withdraw(_who: &AccountId, _amount: Self::Balance) -> WithdrawConsequence<Self::Balance> {
		WithdrawConsequence::Success
	}

	fn minimum_balance() -> Self::Balance {
		Self::Balance::default()
	}

	fn reducible_balance(_who: &AccountId, _preservation: Preservation, _force: Fortitude) -> Self::Balance {
		Self::Balance::default()
	}

	fn total_balance(_who: &AccountId) -> Self::Balance {
		Self::Balance::default()
	}

	fn total_issuance() -> Self::Balance {
		Self::Balance::default()
	}
}

impl<AccountId, Balance, RuntimeHoldReason> Unbalanced<AccountId> for MockCurrency<Balance, RuntimeHoldReason>
where
	Balance: BalanceT,
{
	fn handle_dust(_dust: Dust<AccountId, Self>) {}

	fn write_balance(_who: &AccountId, _amount: Self::Balance) -> Result<Option<Self::Balance>, DispatchError> {
		Ok(Some(Self::Balance::default()))
	}

	fn set_total_issuance(_amount: Self::Balance) {}
}

impl<AccountId, Balance, RuntimeHoldReason> Balanced<AccountId> for MockCurrency<Balance, RuntimeHoldReason>
where
	Balance: BalanceT,
{
	type OnDropDebt = ();
	type OnDropCredit = ();
}
