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

use frame_support::traits::{
	fungible::hold::Mutate,
	tokens::fungible::{Inspect, MutateHold},
};
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

use crate::deposit::{free_deposit, reserve_deposit, Deposit};

/// The sources of a call struct.
///
/// This trait allows to differentiate between the sender of a call and the
/// subject of the call. The sender account submitted the call to the chain and
/// might pay all fees and deposits that are required by the call.
pub trait CallSources<S, P> {
	/// The sender of the call who will pay for all deposits and fees.
	fn sender(&self) -> S;

	/// The subject of the call.
	fn subject(&self) -> P;
}

impl<S: Clone> CallSources<S, S> for S {
	fn sender(&self) -> S {
		self.clone()
	}

	fn subject(&self) -> S {
		self.clone()
	}
}

impl<S: Clone, P: Clone> CallSources<S, P> for (S, P) {
	fn sender(&self) -> S {
		self.0.clone()
	}

	fn subject(&self) -> P {
		self.1.clone()
	}
}

/// A trait that allows version migrators to access the underlying pallet's
/// context, e.g., its Config trait.
///
/// In this way, the migrator can access the pallet's storage and the pallet's
/// types directly.
pub trait VersionMigratorTrait<T>: Sized {
	#[cfg(feature = "try-runtime")]
	fn pre_migrate(&self) -> Result<(), &'static str>;
	fn migrate(&self) -> frame_support::weights::Weight;
	#[cfg(feature = "try-runtime")]
	fn post_migrate(&self) -> Result<(), &'static str>;
}

/// Trait to simulate an origin with different sender and subject.
/// This origin is only used on benchmarks and testing.
#[cfg(feature = "runtime-benchmarks")]
pub trait GenerateBenchmarkOrigin<OuterOrigin, AccountId, SubjectId> {
	fn generate_origin(sender: AccountId, subject: SubjectId) -> OuterOrigin;
}

/// Trait that allows types to implement a worst case value for a type,
/// only when running benchmarks.
#[cfg(feature = "runtime-benchmarks")]
pub trait GetWorstCase<Context = ()> {
	type Output;
	fn worst_case(context: Context) -> Self::Output;
}

#[cfg(feature = "runtime-benchmarks")]
impl<T> GetWorstCase<T> for u32 {
	type Output = Self;
	fn worst_case(_context: T) -> Self::Output {
		u32::MAX
	}
}

#[cfg(feature = "runtime-benchmarks")]
impl<T> GetWorstCase<T> for () {
	type Output = Self;
	fn worst_case(_context: T) -> Self::Output {}
}

#[cfg(feature = "runtime-benchmarks")]
impl<T> GetWorstCase<T> for bool {
	type Output = Self;
	fn worst_case(_context: T) -> Self::Output {
		true
	}
}

/// Trait that allows instanciating multiple instances of a type.
#[cfg(feature = "runtime-benchmarks")]
pub trait Instanciate {
	fn new(instance: u32) -> Self;
}

#[cfg(feature = "runtime-benchmarks")]
impl Instanciate for sp_runtime::AccountId32 {
	fn new(instance: u32) -> Self {
		use sp_runtime::traits::Hash;
		sp_runtime::AccountId32::from(<[u8; 32]>::from(sp_runtime::traits::BlakeTwo256::hash(
			&instance.to_be_bytes(),
		)))
	}
}

/// Generic filter.
pub trait ItemFilter<Item> {
	fn should_include(&self, credential: &Item) -> bool;
}

pub trait BalanceMigrationManager<AccountId, Balance> {
	fn release_reserved_deposit(user: &AccountId, balance: &Balance);

	fn exclude_key_from_migration(key: &[u8]);

	fn is_key_migrated(key: &[u8]) -> bool;
}

impl<AccountId, Balance> BalanceMigrationManager<AccountId, Balance> for () {
	fn exclude_key_from_migration(_key: &[u8]) {}

	fn is_key_migrated(_key: &[u8]) -> bool {
		true
	}

	fn release_reserved_deposit(_user: &AccountId, _balance: &Balance) {}
}

pub trait StorageDepositCollector<AccountId, Key, RuntimeHoldReason> {
	type Currency: MutateHold<AccountId, Reason = RuntimeHoldReason>;

	type Reason: Into<RuntimeHoldReason>;

	/// Returns the hold reason for deposits taken by the deposit collector;
	fn reason() -> Self::Reason;

	/// Returns the deposit of the storage entry that is stored behind the key.
	fn deposit(key: &Key)
		-> Result<Deposit<AccountId, <Self::Currency as Inspect<AccountId>>::Balance>, DispatchError>;

	/// Returns the deposit amount that should be reserved for the storage entry
	/// behind the key.
	///
	/// This value can differ from the actual deposit that is reserved at the
	/// time, since the deposit can be changed.
	fn deposit_amount(key: &Key) -> <Self::Currency as Inspect<AccountId>>::Balance;

	/// Get the storage key used to fetch a value corresponding to a specific
	/// key.
	fn get_hashed_key(key: &Key) -> Result<Vec<u8>, DispatchError>;

	/// Store the new deposit information in the storage entry behind the key.
	fn store_deposit(
		key: &Key,
		deposit: Deposit<AccountId, <Self::Currency as Inspect<AccountId>>::Balance>,
	) -> Result<(), DispatchError>;

	/// Release the deposit.
	fn free_deposit(
		deposit: Deposit<AccountId, <Self::Currency as Inspect<AccountId>>::Balance>,
	) -> Result<<Self::Currency as Inspect<AccountId>>::Balance, DispatchError> {
		free_deposit::<AccountId, Self::Currency>(&deposit, &Self::reason().into())
	}

	/// Creates a new deposit for user.
	///
	/// # Errors
	/// Can fail if the user has not enough balance.
	fn create_deposit(
		who: AccountId,
		amount: <Self::Currency as Inspect<AccountId>>::Balance,
	) -> Result<Deposit<AccountId, <Self::Currency as Inspect<AccountId>>::Balance>, DispatchError> {
		let reason = Self::reason();
		reserve_deposit::<AccountId, Self::Currency>(who, amount, &reason.into())
	}

	/// Change the deposit owner and returns the old owner.
	///
	/// The deposit balance of the current owner will be freed, while the
	/// deposit balance of the new owner will get reserved. The deposit amount
	/// will not change even if the required byte and item fees were updated.
	fn change_deposit_owner<DepositBalanceMigrationManager>(
		key: &Key,
		new_owner: AccountId,
	) -> Result<AccountId, DispatchError>
	where
		DepositBalanceMigrationManager:
			BalanceMigrationManager<AccountId, <Self::Currency as Inspect<AccountId>>::Balance>,
	{
		let hashed_key = Self::get_hashed_key(key)?;
		let is_key_migrated = DepositBalanceMigrationManager::is_key_migrated(&hashed_key);
		let deposit = Self::deposit(key)?;
		let reason = Self::reason();

		if is_key_migrated {
			free_deposit::<AccountId, Self::Currency>(&deposit, &Self::reason().into())?;
		} else {
			DepositBalanceMigrationManager::release_reserved_deposit(&deposit.owner, &deposit.amount);
			DepositBalanceMigrationManager::exclude_key_from_migration(&hashed_key);
		}

		let old_deposit_owner = deposit.owner;

		let deposit_with_new_owner = Deposit {
			owner: new_owner,
			..deposit
		};

		Self::Currency::hold(
			&reason.into(),
			&deposit_with_new_owner.owner,
			deposit_with_new_owner.amount,
		)?;

		Self::store_deposit(key, deposit_with_new_owner)?;

		Ok(old_deposit_owner)
	}

	/// Update the deposit amount.
	///
	/// In case the required deposit per item and byte changed, this function
	/// updates the deposit amount. It either frees parts of the reserved
	/// balance in case the deposit was lowered or reserves more balance when
	/// the deposit was raised.
	fn update_deposit<DepositBalanceMigrationManager>(key: &Key) -> Result<(), DispatchError>
	where
		DepositBalanceMigrationManager:
			BalanceMigrationManager<AccountId, <Self::Currency as Inspect<AccountId>>::Balance>,
	{
		let deposit = Self::deposit(key)?;
		let reason = Self::reason();
		let hashed_key = Self::get_hashed_key(key)?;
		let is_key_migrated = DepositBalanceMigrationManager::is_key_migrated(&hashed_key);

		if is_key_migrated {
			free_deposit::<AccountId, Self::Currency>(&deposit, &Self::reason().into())?;
		} else {
			DepositBalanceMigrationManager::release_reserved_deposit(&deposit.owner, &deposit.amount);
			DepositBalanceMigrationManager::exclude_key_from_migration(&hashed_key);
		}

		let deposit_with_new_amount = Deposit {
			amount: Self::deposit_amount(key),
			..deposit
		};
		Self::Currency::hold(
			&reason.into(),
			&deposit_with_new_amount.owner,
			deposit_with_new_amount.amount,
		)?;

		Self::store_deposit(key, deposit_with_new_amount)?;

		Ok(())
	}
}

/// Metadata trait for native asset.
pub trait InspectMetadata {
	// Get name for native asset.
	fn name() -> Vec<u8>;
	// Get symbol for native asset.
	fn symbol() -> Vec<u8>;
	// Get decimals for native asset.
	fn decimals() -> u8;
}
