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
		fungible::{Inspect, MutateHold},
		ConstU32,
	},
	BoundedVec,
};
use kilt_support::{traits::StorageDepositCollector, Deposit};
use pallet_dip_provider::{traits::ProviderHooks, IdentityCommitmentVersion};
use parity_scale_codec::Encode;
use sp_runtime::traits::{Get, Hash};
use sp_std::marker::PhantomData;

use crate::{AccountIdOf, BalanceOf, Config, Deposits, Error, HoldReason, MAX_NAMESPACE_LENGTH};

type HasherOf<Runtime> = <Runtime as frame_system::Config>::Hashing;

pub struct StorageDepositCollectorViaDepositsPallet<Runtime, Namespace, DepositAmount, Key, RuntimeHoldReason>(
	PhantomData<(Runtime, Namespace, DepositAmount, Key, RuntimeHoldReason)>,
);

impl<Runtime, Namespace, DepositAmount, Key, RuntimeHoldReason>
	StorageDepositCollector<AccountIdOf<Runtime>, Key, RuntimeHoldReason>
	for StorageDepositCollectorViaDepositsPallet<Runtime, Namespace, DepositAmount, Key, RuntimeHoldReason>
where
	Runtime: Config,
	Runtime::Currency: MutateHold<AccountIdOf<Runtime>, Reason = RuntimeHoldReason>,
	Namespace: Get<BoundedVec<u8, ConstU32<MAX_NAMESPACE_LENGTH>>>,
	DepositAmount: Get<BalanceOf<Runtime>>,
	Key: Encode,
	RuntimeHoldReason: From<HoldReason>,
{
	type Currency = Runtime::Currency;
	type Reason = HoldReason;

	fn reason() -> Self::Reason {
		HoldReason::Deposit
	}

	fn deposit(
		key: &Key,
	) -> Result<Deposit<AccountIdOf<Runtime>, <Self::Currency as Inspect<AccountIdOf<Runtime>>>::Balance>, DispatchError>
	{
		let namespace = Namespace::get();
		let key_hash = HasherOf::<Runtime>::hash(key.encode().as_ref());
		let deposit = Deposits::<Runtime>::get(namespace, key_hash)
			.ok_or(DispatchError::from(Error::<Runtime>::DepositNotFound))?;
		Ok(deposit)
	}

	fn deposit_amount(_key: &Key) -> <Self::Currency as Inspect<AccountIdOf<Runtime>>>::Balance {
		DepositAmount::get()
	}

	fn get_hashed_key(key: &Key) -> Result<Vec<u8>, DispatchError> {
		let namespace = Namespace::get();
		let key_hash = HasherOf::<Runtime>::hash(key.encode().as_ref());
		Ok(Deposits::<Runtime>::hashed_key_for(namespace, key_hash))
	}

	fn store_deposit(
		key: &Key,
		deposit: Deposit<AccountIdOf<Runtime>, <Self::Currency as Inspect<AccountIdOf<Runtime>>>::Balance>,
	) -> Result<(), DispatchError> {
		let namespace = Namespace::get();
		let key_hash = HasherOf::<Runtime>::hash(key.encode().as_ref());
		Deposits::<Runtime>::try_mutate(namespace, key_hash, |deposit_entry| match deposit_entry {
			Some(_) => Err(Error::<Runtime>::DepositExisting),
			None => {
				*deposit_entry = Some(deposit);
				Ok(())
			}
		})
		.map_err(DispatchError::from)?;
		Ok(())
	}
}

type DepositKey<Runtime> = (AccountIdOf<Runtime>, IdentityCommitmentVersion);

impl<Runtime, Namespace, DepositAmount, RuntimeHoldReason> ProviderHooks
	for StorageDepositCollectorViaDepositsPallet<Runtime, Namespace, DepositAmount, DepositKey<Runtime>, RuntimeHoldReason>
where
	Runtime: pallet_dip_provider::Config + Config,
	Runtime::Currency: MutateHold<AccountIdOf<Runtime>, Reason = RuntimeHoldReason>,
	Namespace: Get<BoundedVec<u8, ConstU32<MAX_NAMESPACE_LENGTH>>>,
	DepositAmount: Get<BalanceOf<Runtime>>,
	RuntimeHoldReason: From<HoldReason>,
{
	type Error = u16;
	type Identifier = Runtime::Identifier;
	type IdentityCommitment = Runtime::IdentityCommitment;
	type Submitter = AccountIdOf<Runtime>;
	type Success = ();

	fn on_identity_committed(
		_identifier: &Self::Identifier,
		submitter: &Self::Submitter,
		_commitment: &Self::IdentityCommitment,
		version: IdentityCommitmentVersion,
	) -> Result<Self::Success, Self::Error> {
		let deposit = Self::create_deposit(submitter.clone(), DepositAmount::get()).map_err(|_| 1u16)?;
		Self::store_deposit(&(submitter.clone(), version), deposit).map_err(|_| 2u16)?;
		Ok(())
	}

	fn on_commitment_removed(
		_identifier: &Self::Identifier,
		submitter: &Self::Submitter,
		_commitment: &Self::IdentityCommitment,
		version: pallet_dip_provider::IdentityCommitmentVersion,
	) -> Result<Self::Success, Self::Error> {
		let deposit = Self::deposit(&(submitter.clone(), version)).map_err(|_| 3u16)?;
		Self::free_deposit(deposit).map_err(|_| 4u16)?;
		Ok(())
	}
}
