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

use frame_support::{
	sp_runtime::DispatchError,
	traits::{
		fungible::{hold::Mutate, Inspect},
		tokens::Precision,
	},
};
use kilt_support::Deposit;
use pallet_dip_provider::{traits::ProviderHooks as DipProviderHooks, IdentityCommitmentOf, IdentityCommitmentVersion};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::traits::Get;
use sp_std::marker::PhantomData;

use crate::{BalanceOf, Config, Error, HoldReason, Pallet};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Details associated to an on-chain deposit.
#[derive(Clone, Debug, Encode, Decode, Eq, PartialEq, Ord, PartialOrd, TypeInfo, MaxEncodedLen)]
pub struct DepositEntry<AccountId, Balance, Reason> {
	/// The [`Deposit`] entry.
	pub deposit: Deposit<AccountId, Balance>,
	/// The `Reason` for the deposit.
	pub reason: Reason,
}

const LOG_TARGET: &str = "pallet_deposit_storage::FixedDepositCollectorViaDepositsPallet";

/// Type implementing the [`DipProviderHooks`] hooks trait by taking a deposit
/// whenever an identity commitment is stored, and releasing the deposit
/// whenever an identity commitment is removed.
pub struct FixedDepositCollectorViaDepositsPallet<DepositsNamespace, FixedDepositAmount, DepositKey>(
	PhantomData<(DepositsNamespace, FixedDepositAmount, DepositKey)>,
);

pub enum FixedDepositCollectorViaDepositsPalletError {
	DepositAlreadyTaken,
	DepositNotFound,
	FailedToHold,
	FailedToRelease,
	Internal,
}

impl From<FixedDepositCollectorViaDepositsPalletError> for u16 {
	fn from(value: FixedDepositCollectorViaDepositsPalletError) -> Self {
		match value {
			// DO NOT USE 0
			// Errors of different sub-parts are separated by a `u8::MAX`.
			// A value of 0 would make it confusing whether it's the previous sub-part error (u8::MAX)
			// or the new sub-part error (u8::MAX + 0).
			FixedDepositCollectorViaDepositsPalletError::DepositAlreadyTaken => 1,
			FixedDepositCollectorViaDepositsPalletError::DepositNotFound => 2,
			FixedDepositCollectorViaDepositsPalletError::FailedToHold => 3,
			FixedDepositCollectorViaDepositsPalletError::FailedToRelease => 4,
			FixedDepositCollectorViaDepositsPalletError::Internal => u16::MAX,
		}
	}
}

impl<Runtime, DepositsNamespace, FixedDepositAmount, DepositKey> DipProviderHooks<Runtime>
	for FixedDepositCollectorViaDepositsPallet<DepositsNamespace, FixedDepositAmount, DepositKey>
where
	Runtime: pallet_dip_provider::Config + Config,
	DepositsNamespace: Get<Runtime::Namespace>,
	FixedDepositAmount: Get<BalanceOf<Runtime>>,
	DepositKey: From<(Runtime::Identifier, Runtime::AccountId, IdentityCommitmentVersion)> + Encode,
{
	type Error = u16;

	fn on_identity_committed(
		identifier: &Runtime::Identifier,
		submitter: &Runtime::AccountId,
		_commitment: &IdentityCommitmentOf<Runtime>,
		version: IdentityCommitmentVersion,
	) -> Result<(), Self::Error> {
		let namespace = DepositsNamespace::get();
		let key = DepositKey::from((identifier.clone(), submitter.clone(), version))
			.encode()
			.try_into()
			.map_err(|_| {
				log::error!(
					target: LOG_TARGET,
					"Failed to convert tuple ({:#?}, {version}) to BoundedVec<u8, {:#?}>",
					identifier,
					Runtime::MaxKeyLength::get()
				);
				FixedDepositCollectorViaDepositsPalletError::Internal
			})?;
		let deposit_entry = DepositEntry {
			deposit: Deposit {
				amount: FixedDepositAmount::get(),
				owner: submitter.clone(),
			},
			reason: HoldReason::Deposit.into(),
		};
		Pallet::<Runtime>::add_deposit(namespace, key, deposit_entry).map_err(|e| {
			if e == DispatchError::from(Error::<Runtime>::DepositExisting) {
				FixedDepositCollectorViaDepositsPalletError::DepositAlreadyTaken
			} else if e == DispatchError::from(Error::<Runtime>::FailedToHold) {
				FixedDepositCollectorViaDepositsPalletError::FailedToHold
			} else {
				log::error!(
					target: LOG_TARGET,
					"Error {:#?} generated inside `on_identity_committed` hook.",
					e
				);
				FixedDepositCollectorViaDepositsPalletError::Internal
			}
		})?;
		Ok(())
	}

	fn on_commitment_removed(
		identifier: &Runtime::Identifier,
		submitter: &Runtime::AccountId,
		_commitment: &IdentityCommitmentOf<Runtime>,
		version: pallet_dip_provider::IdentityCommitmentVersion,
	) -> Result<(), Self::Error> {
		let namespace = DepositsNamespace::get();
		let key = DepositKey::from((identifier.clone(), submitter.clone(), version))
			.encode()
			.try_into()
			.map_err(|_| {
				log::error!(
					target: LOG_TARGET,
					"Failed to convert tuple ({:#?}, {version}) to BoundedVec<u8, {:#?}>",
					identifier,
					Runtime::MaxKeyLength::get()
				);
				FixedDepositCollectorViaDepositsPalletError::Internal
			})?;
		// We don't set any expected owner for the deposit on purpose, since this hook
		// assumes the dip-provider pallet has performed all the access control logic
		// necessary.
		Pallet::<Runtime>::remove_deposit(&namespace, &key, None).map_err(|e| {
			if e == DispatchError::from(Error::<Runtime>::DepositNotFound) {
				FixedDepositCollectorViaDepositsPalletError::DepositNotFound
			} else if e == DispatchError::from(Error::<Runtime>::FailedToRelease) {
				FixedDepositCollectorViaDepositsPalletError::FailedToRelease
			} else {
				log::error!(
					target: LOG_TARGET,
					"Error {:#?} generated inside `on_commitment_removed` hook.",
					e
				);
				FixedDepositCollectorViaDepositsPalletError::Internal
			}
		})?;
		Ok(())
	}
}

// Taken from dip_support logic, not to make that pub
pub(crate) fn reserve_deposit<Account, Currency: Mutate<Account>>(
	account: Account,
	deposit_amount: Currency::Balance,
	reason: &Currency::Reason,
) -> Result<Deposit<Account, Currency::Balance>, DispatchError> {
	Currency::hold(reason, &account, deposit_amount)?;
	Ok(Deposit {
		owner: account,
		amount: deposit_amount,
	})
}

// Taken from dip_support logic, not to make that pub
pub(crate) fn free_deposit<Account, Currency: Mutate<Account>>(
	deposit: &Deposit<Account, Currency::Balance>,
	reason: &Currency::Reason,
) -> Result<<Currency as Inspect<Account>>::Balance, DispatchError> {
	let result = Currency::release(reason, &deposit.owner, deposit.amount, Precision::BestEffort);
	debug_assert!(
		result == Ok(deposit.amount),
		"Released deposit amount does not match with expected amount. Expected: {:#?}, Released amount: {:#?}  Error: {:#?}",
		deposit.amount,
		result.ok(),
		result.err(),
	);
	// Same as the `debug_assert` above, but also run in release mode.
	if result != Ok(deposit.amount) {
		log::error!(target: LOG_TARGET, "Released deposit amount does not match with expected amount. Expected: {:#?}, Released amount: {:#?}  Error: {:#?}", deposit.amount, result.ok(), result.err());
	}
	result
}
