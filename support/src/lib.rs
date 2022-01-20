// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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
#![cfg_attr(not(feature = "std"), no_std)]

use deposit::Deposit;
use frame_support::traits::{Currency, ReservableCurrency};
use sp_runtime::traits::Zero;

pub mod deposit;
#[cfg(any(feature = "runtime-benchmarks", feature = "mock"))]
pub mod mock;
pub mod signature;
pub mod traits;

pub fn free_deposit<A, C>(deposit: &Deposit<A, C::Balance>)
where
	C: Currency<A> + ReservableCurrency<A>,
{
	let err_amount = C::unreserve(&deposit.owner, deposit.amount);
	debug_assert!(err_amount.is_zero());
}
