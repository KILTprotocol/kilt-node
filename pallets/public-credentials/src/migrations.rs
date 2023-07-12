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

use frame_support::traits::ReservableCurrency;
use kilt_support::{migration::switch_reserved_to_hold, Deposit};
use sp_runtime::SaturatedConversion;

use crate::{AccountIdOf, Config, CredentialEntry, Credentials, CurrencyOf, HoldReason};

pub fn do_migration<T: Config>(who: T::AccountId)
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	Credentials::<T>::iter()
		.filter(|(_, _, details)| details.deposit.owner == who && details.deposit.version.is_none())
		.for_each(|(key1, key2, delegation_details)| {
			// switch reserves to hold.
			let deposit = delegation_details.deposit;
			let result = switch_reserved_to_hold::<AccountIdOf<T>, CurrencyOf<T>>(
				deposit.owner,
				&HoldReason::Deposit.into(),
				deposit.amount.saturated_into(),
			);

			// update the deposit
			Credentials::<T>::mutate(key1.clone(), key2.clone(), |details| {
				if let Some(d) = details {
					*d = CredentialEntry {
						deposit: Deposit {
							version: Some(1),
							owner: d.deposit.owner.clone(),
							amount: d.deposit.amount,
						},
						..delegation_details
					}
				}
			});

			debug_assert!(
				result.is_ok(),
				"Delegation: Could not convert reserves to hold from Delegation: {:?}, {:?} error: {:?}",
				key1,
				key2,
				result
			);
		});
}
