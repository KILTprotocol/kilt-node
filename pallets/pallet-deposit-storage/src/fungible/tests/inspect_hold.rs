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

use frame_support::traits::fungible::{InspectHold, MutateHold};
use sp_runtime::{traits::Zero, AccountId32};

use crate::{
	fungible::tests::mock::{DepositNamespace, ExtBuilder, TestRuntime, OTHER_ACCOUNT, OWNER},
	Pallet, PalletDepositStorageReason,
};

#[test]
fn balance_on_hold() {
	ExtBuilder::default()
		.with_balances(vec![(OWNER, 100_000)])
		.build()
		.execute_with(|| {
			let reason = PalletDepositStorageReason {
				namespace: DepositNamespace::ExampleNamespace,
				key: [0].to_vec().try_into().unwrap(),
			};
			<Pallet<TestRuntime> as MutateHold<AccountId32>>::hold(&reason, &OWNER, 10)
				.expect("Failed to hold amount for user.");

			let balance_on_hold_for_same_reason_same_user =
				<Pallet<TestRuntime> as InspectHold<AccountId32>>::balance_on_hold(&reason, &OWNER);
			assert_eq!(balance_on_hold_for_same_reason_same_user, 10);

			let balance_on_hold_for_same_reason_different_user =
				<Pallet<TestRuntime> as InspectHold<AccountId32>>::balance_on_hold(&reason, &OTHER_ACCOUNT);
			assert!(balance_on_hold_for_same_reason_different_user.is_zero());

			let other_reason = PalletDepositStorageReason {
				namespace: DepositNamespace::ExampleNamespace,
				key: [1].to_vec().try_into().unwrap(),
			};
			let balance_on_hold_for_different_reason_same_user =
				<Pallet<TestRuntime> as InspectHold<AccountId32>>::balance_on_hold(&other_reason, &OWNER);
			assert!(balance_on_hold_for_different_reason_same_user.is_zero());

			let balance_on_hold_for_different_reason_different_user =
				<Pallet<TestRuntime> as InspectHold<AccountId32>>::balance_on_hold(&other_reason, &OTHER_ACCOUNT);
			assert!(balance_on_hold_for_different_reason_different_user.is_zero());
		});
}
