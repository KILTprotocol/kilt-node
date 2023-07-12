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

use frame_support::traits::{
	fungible::freeze::Mutate as MutateFreeze, LockIdentifier, LockableCurrency, ReservableCurrency,
};
use pallet_balances::{BalanceLock, Freezes, IdAmount, Locks};
use sp_runtime::SaturatedConversion;

use crate::{
	types::{AccountIdOf, CurrencyOf},
	Config, FreezeReason,
};

const STAKING_ID: LockIdentifier = *b"kiltpstk";

pub fn do_migration<T: Config>(who: T::AccountId)
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
	<T as Config>::Currency: LockableCurrency<T::AccountId>,
{
	Locks::<T>::iter()
		.filter(|(user_id, _)| user_id == &who)
		.for_each(|(user_id, locks)| {
			locks
				.iter()
				.filter(|lock| lock.id == STAKING_ID)
				.for_each(|lock: &BalanceLock<_>| {
					update_or_create_freeze::<T>(user_id.clone(), lock);
				});
		});
}

fn update_or_create_freeze<T: Config>(
	user_id: T::AccountId,
	lock: &BalanceLock<<T as pallet_balances::Config>::Balance>,
) where
	CurrencyOf<T>: LockableCurrency<AccountIdOf<T>>,
{
	let freezes = Freezes::<T>::get(&user_id);

	let result = if let Some(IdAmount { amount, .. }) = freezes
		.iter()
		.find(|freeze| freeze.id == <T as Config>::FreezeIdentifier::from(FreezeReason::Staking).into())
	{
		let total_lock = lock
			.amount
			.saturated_into::<u128>()
			.saturating_add((*amount).saturated_into());

		<CurrencyOf<T> as MutateFreeze<AccountIdOf<T>>>::extend_freeze(
			&<T as crate::Config>::FreezeIdentifier::from(FreezeReason::Staking),
			&user_id,
			total_lock.saturated_into(),
		)
	} else {
		<CurrencyOf<T> as MutateFreeze<AccountIdOf<T>>>::set_freeze(
			&<T as crate::Config>::FreezeIdentifier::from(FreezeReason::Staking),
			&user_id,
			lock.amount.saturated_into(),
		)
	};

	debug_assert!(
		result.is_ok(),
		"Staking: Could not convert locks to freezes from user: {:?} error: {:?}",
		user_id,
		result
	);

	<CurrencyOf<T> as LockableCurrency<AccountIdOf<T>>>::remove_lock(STAKING_ID, &user_id);
}
