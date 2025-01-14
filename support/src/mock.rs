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

//! This module contains utilities for testing.

use frame_support::traits::{
	fungible::{Balanced, Dust, Inspect, InspectHold, Mutate, MutateHold, Unbalanced, UnbalancedHold},
	tokens::{Balance as BalanceT, DepositConsequence, Fortitude, Preservation, Provenance, WithdrawConsequence},
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::sr25519;
use sp_runtime::{AccountId32, DispatchError, DispatchResult};
use sp_std::marker::PhantomData;

/// This pallet only contains an origin which supports separated sender and
/// subject.
///
/// WARNING: This is only used for testing!
#[frame_support::pallet]
#[allow(dead_code)]
pub mod mock_origin {
	use sp_std::marker::PhantomData;

	use frame_support::{
		traits::{EnsureOrigin, EnsureOriginWithArg},
		Parameter,
	};
	use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
	use scale_info::TypeInfo;
	use sp_runtime::AccountId32;

	use crate::traits::CallSources;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeOrigin: From<Origin<Self>>;
		type AccountId: Parameter;
		type SubjectId: Parameter;
	}

	/// A dummy pallet for adding an origin to the runtime that contains
	/// separate sender and subject accounts.
	///
	/// WARNING: This is only used for testing!
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// An origin that is split into sender and subject.
	///
	/// WARNING: This is only used for testing!
	#[pallet::origin]
	pub type Origin<T> = DoubleOrigin<<T as Config>::AccountId, <T as Config>::SubjectId>;

	/// An origin that is split into sender and subject.
	///
	/// WARNING: This is only used for testing!
	#[derive(Debug, Clone, PartialEq, Eq, TypeInfo, Encode, Decode, MaxEncodedLen)]
	pub struct DoubleOrigin<AccountId, SubjectId>(pub AccountId, pub SubjectId);

	impl<AccountId: Clone, SubjectId: Clone> CallSources<AccountId, SubjectId> for DoubleOrigin<AccountId, SubjectId> {
		fn sender(&self) -> AccountId {
			self.0.clone()
		}

		fn subject(&self) -> SubjectId {
			self.1.clone()
		}
	}

	/// Ensure that the call was made using the split origin.
	///
	/// WARNING: This is only used for testing!
	pub struct EnsureDoubleOrigin<AccountId, SubjectId>(PhantomData<(AccountId, SubjectId)>);

	impl<OuterOrigin, AccountId, SubjectId> EnsureOrigin<OuterOrigin> for EnsureDoubleOrigin<AccountId, SubjectId>
	where
		OuterOrigin:
			Into<Result<DoubleOrigin<AccountId, SubjectId>, OuterOrigin>> + From<DoubleOrigin<AccountId, SubjectId>>,
		AccountId: From<AccountId32>,
		SubjectId: From<AccountId32>,
	{
		type Success = DoubleOrigin<AccountId, SubjectId>;

		fn try_origin(o: OuterOrigin) -> Result<Self::Success, OuterOrigin> {
			o.into()
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn try_successful_origin() -> Result<OuterOrigin, ()> {
			const TEST_ACC: AccountId32 = AccountId32::new([0u8; 32]);

			Ok(OuterOrigin::from(DoubleOrigin(
				TEST_ACC.clone().into(),
				TEST_ACC.into(),
			)))
		}
	}

	impl<OuterOrigin, AccountId, SubjectId> EnsureOriginWithArg<OuterOrigin, SubjectId>
		for EnsureDoubleOrigin<AccountId, SubjectId>
	where
		OuterOrigin: Into<Result<DoubleOrigin<AccountId, SubjectId>, OuterOrigin>>
			+ From<DoubleOrigin<AccountId, SubjectId>>
			+ Clone,
		SubjectId: PartialEq<SubjectId> + Clone,
		AccountId: Clone + Decode,
	{
		type Success = DoubleOrigin<AccountId, SubjectId>;

		fn try_origin(o: OuterOrigin, a: &SubjectId) -> Result<Self::Success, OuterOrigin> {
			let did_origin: DoubleOrigin<AccountId, SubjectId> = o.clone().into()?;
			if did_origin.1 == *a {
				Ok(did_origin)
			} else {
				Err(o)
			}
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn try_successful_origin(a: &SubjectId) -> Result<OuterOrigin, ()> {
			let zero_account_id = AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
				.expect("infinite length input; no invalid inputs for type; qed");

			Ok(OuterOrigin::from(DoubleOrigin(zero_account_id, a.clone())))
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<OuterOrigin, AccountId, SubjectId> crate::traits::GenerateBenchmarkOrigin<OuterOrigin, AccountId, SubjectId>
		for EnsureDoubleOrigin<AccountId, SubjectId>
	where
		OuterOrigin:
			Into<Result<DoubleOrigin<AccountId, SubjectId>, OuterOrigin>> + From<DoubleOrigin<AccountId, SubjectId>>,
	{
		fn generate_origin(sender: AccountId, subject: SubjectId) -> OuterOrigin {
			OuterOrigin::from(DoubleOrigin(sender, subject))
		}
	}
}

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct SubjectId(pub AccountId32);

impl From<AccountId32> for SubjectId {
	fn from(acc: AccountId32) -> Self {
		SubjectId(acc)
	}
}

impl From<sr25519::Public> for SubjectId {
	fn from(acc: sr25519::Public) -> Self {
		SubjectId(acc.into())
	}
}

impl AsRef<[u8]> for SubjectId {
	fn as_ref(&self) -> &[u8] {
		self.0.as_ref()
	}
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
