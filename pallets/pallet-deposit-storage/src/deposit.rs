// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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

/// Details associated to an on-chain deposit.
#[derive(Clone, Debug, Encode, Decode, Eq, PartialEq, Ord, PartialOrd, TypeInfo, MaxEncodedLen)]
pub struct DepositEntry<AccountId, Balance, Reason> {
	/// The [`Deposit`] entry.
	pub deposit: Deposit<AccountId, Balance>,
	/// The `Reason` for the deposit.
	pub reason: Reason,
}

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
			FixedDepositCollectorViaDepositsPalletError::DepositAlreadyTaken => 0,
			FixedDepositCollectorViaDepositsPalletError::DepositNotFound => 1,
			FixedDepositCollectorViaDepositsPalletError::FailedToHold => 2,
			FixedDepositCollectorViaDepositsPalletError::FailedToRelease => 3,
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
	DepositKey: From<(Runtime::Identifier, IdentityCommitmentVersion)> + Encode,
{
	type Error = u16;

	fn on_identity_committed(
		identifier: &Runtime::Identifier,
		submitter: &Runtime::AccountId,
		_commitment: &IdentityCommitmentOf<Runtime>,
		version: IdentityCommitmentVersion,
	) -> Result<(), Self::Error> {
		let namespace = DepositsNamespace::get();
		let key = DepositKey::from((identifier.clone(), version))
			.encode()
			.try_into()
			.map_err(|_| {
				log::error!(
					"Failed to convert tuple ({:#?}, {version}) to BoundedVec with max length {}",
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
		Pallet::<Runtime>::add_deposit(namespace, key, deposit_entry).map_err(|e| match e {
			pallet_error if pallet_error == DispatchError::from(Error::<Runtime>::DepositExisting) => {
				FixedDepositCollectorViaDepositsPalletError::DepositAlreadyTaken
			}
			_ => {
				log::error!(
					"Error {:#?} should not be generated inside `on_identity_committed` hook.",
					e
				);
				FixedDepositCollectorViaDepositsPalletError::Internal
			}
		})?;
		Ok(())
	}

	fn on_commitment_removed(
		identifier: &Runtime::Identifier,
		_submitter: &Runtime::AccountId,
		_commitment: &IdentityCommitmentOf<Runtime>,
		version: pallet_dip_provider::IdentityCommitmentVersion,
	) -> Result<(), Self::Error> {
		let namespace = DepositsNamespace::get();
		let key = (identifier, version).encode().try_into().map_err(|_| {
			log::error!(
				"Failed to convert tuple ({:#?}, {version}) to BoundedVec with max length {}",
				identifier,
				Runtime::MaxKeyLength::get()
			);
			FixedDepositCollectorViaDepositsPalletError::Internal
		})?;
		Pallet::<Runtime>::remove_deposit(&namespace, &key, None).map_err(|e| match e {
			pallet_error if pallet_error == DispatchError::from(Error::<Runtime>::DepositNotFound) => {
				FixedDepositCollectorViaDepositsPalletError::DepositNotFound
			}
			_ => {
				log::error!(
					"Error {:#?} should not be generated inside `on_commitment_removed` hook.",
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
		"Released deposit amount does not match with expected amount. Expected: {:?}, Released amount: {:?}  Error: {:?}",
		deposit.amount,
		result.ok(),
		result.err(),
	);
	result
}
