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

use frame_benchmarking::{account, benchmarks};
use frame_support::traits::fungible::{Inspect, Mutate};
use frame_system::RawOrigin;
use sp_std::{convert::TryInto, fmt::Debug, vec::Vec};

use crate::*;

const SEED: u32 = 0;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

benchmarks! {
	where_clause {
		where
		<<T as pallet_did_lookup::Config>::Currency as Inspect<AccountIdOf<T>>>::Balance: TryFrom<usize>,
		<T as pallet_did_lookup::Config>::Currency: Mutate<AccountIdOf<T>>,
	}

	update_user {
		let max_migrations = 1;
		let caller = account("caller", 0, SEED);
		let initial_balance =  <T as pallet_did_lookup>::Currency::minimum_balance() * 10;
		<T as Config>::Currency::set_balance(&caller, initial_balance);
		let origin = RawOrigin::Signed(submitter);

	}: _<T::RuntimeOrigin>(origin, caller, max_migrations)
}
