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

//! Unit testing

use frame_support::traits::fungible::Inspect;
use pallet_session::SessionManager;
use std::convert::TryInto;

use crate::{
	mock::{AccountId, Balance, Balances, ExtBuilder, StakePallet, System, Test},
	set::OrderedSet,
	types::{Candidate, CandidateStatus, RoundInfo, StakeOf, TotalStake},
	CandidatePool, Config,
};

#[test]
fn should_select_collators_genesis_session() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 20),
			(2, 20),
			(3, 20),
			(4, 20),
			(5, 20),
			(6, 20),
			(7, 20),
			(8, 20),
			(9, 20),
			(10, 20),
			(11, 20),
		])
		.with_collators(vec![(1, 20), (2, 20)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				StakePallet::new_session(0)
					.expect("first session must return new collators")
					.len(),
				2
			);
			assert_eq!(
				StakePallet::new_session(1)
					.expect("second session must return new collators")
					.len(),
				2
			);
		});
}

#[test]
fn genesis() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 1000),
			(2, 300),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 9),
			(9, 4),
		])
		.with_collators(vec![(1, 500), (2, 200)])
		.with_delegators(vec![(3, 1, 100), (4, 1, 100), (5, 2, 100), (6, 2, 100)])
		.build_and_execute_with_sanity_tests(|| {
			assert!(System::events().is_empty());

			// Collators
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 700,
					delegators: 400
				}
			);
			assert_eq!(
				vec![
					StakeOf::<Test> { owner: 1, amount: 700 },
					StakeOf::<Test> { owner: 2, amount: 400 }
				]
				.try_into(),
				Ok(StakePallet::top_candidates().into_bounded_vec())
			);
			assert_eq!(CandidatePool::<Test>::count(), 2);

			// 1
			assert_eq!(Balances::usable_balance(1), 500);
			assert_eq!(Balances::balance(&1), 1000);
			assert!(StakePallet::is_active_candidate(&1).is_some());
			assert_eq!(
				StakePallet::candidate_pool(1),
				Some(
					Candidate::<AccountId, Balance, <Test as Config>::MaxDelegatorsPerCollator> {
						id: 1,
						stake: 500,
						delegators: OrderedSet::from_sorted_set(
							vec![
								StakeOf::<Test> { owner: 3, amount: 100 },
								StakeOf::<Test> { owner: 4, amount: 100 }
							]
							.try_into()
							.unwrap()
						),
						total: 700,
						status: CandidateStatus::Active,
					}
				)
			);
			// 2
			assert_eq!(Balances::usable_balance(2), 100);
			assert_eq!(Balances::balance(&2), 300);
			assert!(StakePallet::is_active_candidate(&2).is_some());
			assert_eq!(
				StakePallet::candidate_pool(2),
				Some(
					Candidate::<AccountId, Balance, <Test as Config>::MaxDelegatorsPerCollator> {
						id: 2,
						stake: 200,
						delegators: OrderedSet::from_sorted_set(
							vec![
								StakeOf::<Test> { owner: 5, amount: 100 },
								StakeOf::<Test> { owner: 6, amount: 100 }
							]
							.try_into()
							.unwrap()
						),
						total: 400,
						status: CandidateStatus::Active,
					}
				)
			);
			// Delegators
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 700,
					delegators: 400,
				}
			);
			for x in 3..7 {
				assert!(StakePallet::is_delegator(&x));
				assert_eq!(Balances::usable_balance(x), 0);
				assert_eq!(Balances::balance(&x), 100);
			}
			// Uninvolved
			for x in 7..10 {
				assert!(!StakePallet::is_delegator(&x));
			}
			assert_eq!(Balances::balance(&7), 100);
			assert_eq!(Balances::usable_balance(7), 100);
			assert_eq!(Balances::balance(&8), 9);
			assert_eq!(Balances::usable_balance(8), 9);
			assert_eq!(Balances::balance(&9), 4);
			assert_eq!(Balances::usable_balance(9), 4);

			// Safety first checks
			assert_eq!(
				StakePallet::max_selected_candidates(),
				<Test as Config>::MinCollators::get()
			);
			assert_eq!(
				StakePallet::round(),
				RoundInfo::new(0u32, 0u32.into(), <Test as Config>::DefaultBlocksPerRound::get())
			);
		});
	ExtBuilder::default()
		.with_balances(vec![
			(1, 100),
			(2, 100),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 100),
			(9, 100),
			(10, 100),
		])
		.with_collators(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 10)])
		.with_delegators(vec![(6, 1, 10), (7, 1, 10), (8, 2, 10), (9, 2, 10), (10, 1, 10)])
		.build_and_execute_with_sanity_tests(|| {
			assert!(System::events().is_empty());
			assert_eq!(CandidatePool::<Test>::count(), 5);

			// Collators
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 40,
					delegators: 50
				}
			);
			assert_eq!(
				Ok(StakePallet::top_candidates().into_bounded_vec()),
				vec![
					StakeOf::<Test> { owner: 1, amount: 50 },
					StakeOf::<Test> { owner: 2, amount: 40 },
					StakeOf::<Test> { owner: 3, amount: 20 },
					StakeOf::<Test> { owner: 4, amount: 20 },
					StakeOf::<Test> { owner: 5, amount: 10 }
				]
				.try_into()
			);
			for x in 1..5 {
				assert!(StakePallet::is_active_candidate(&x).is_some());
				assert_eq!(Balances::balance(&x), 100);
				assert_eq!(Balances::usable_balance(x), 80);
			}
			assert!(StakePallet::is_active_candidate(&5).is_some());
			assert_eq!(Balances::balance(&5), 100);
			assert_eq!(Balances::usable_balance(5), 90);
			// Delegators
			for x in 6..11 {
				assert!(StakePallet::is_delegator(&x));
				assert_eq!(Balances::balance(&x), 100);
				assert_eq!(Balances::usable_balance(x), 90);
			}

			// Safety first checks
			assert_eq!(
				StakePallet::max_selected_candidates(),
				<Test as Config>::MinCollators::get()
			);
			assert_eq!(
				StakePallet::round(),
				RoundInfo::new(0, 0, <Test as Config>::DefaultBlocksPerRound::get())
			);
		});
}
