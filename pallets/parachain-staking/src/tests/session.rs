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

use frame_support::traits::EstimateNextSessionRotation;
use pallet_session::ShouldEndSession;
use sp_runtime::Permill;

use crate::mock::{ExtBuilder, StakePallet};

#[test]
fn should_estimate_current_session_progress() {
	ExtBuilder::default()
		.set_blocks_per_round(100)
		.with_balances(vec![
			(1, 10),
			(2, 20),
			(3, 10),
			(4, 10),
			(5, 10),
			(6, 10),
			(7, 10),
			(8, 10),
			(9, 10),
			(10, 10),
			(11, 10),
		])
		.with_collators(vec![
			(1, 10),
			(2, 20),
			(3, 10),
			(4, 10),
			(5, 10),
			(6, 10),
			(7, 10),
			(8, 10),
			(9, 10),
			(10, 10),
		])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				StakePallet::estimate_current_session_progress(10).0.unwrap(),
				Permill::from_percent(10)
			);
			assert_eq!(
				StakePallet::estimate_current_session_progress(20).0.unwrap(),
				Permill::from_percent(20)
			);
			assert_eq!(
				StakePallet::estimate_current_session_progress(30).0.unwrap(),
				Permill::from_percent(30)
			);
			assert_eq!(
				StakePallet::estimate_current_session_progress(60).0.unwrap(),
				Permill::from_percent(60)
			);
			assert_eq!(
				StakePallet::estimate_current_session_progress(100).0.unwrap(),
				Permill::from_percent(100)
			);
		});
}

#[test]
fn should_estimate_next_session_rotation() {
	ExtBuilder::default()
		.set_blocks_per_round(100)
		.with_balances(vec![
			(1, 10),
			(2, 20),
			(3, 10),
			(4, 10),
			(5, 10),
			(6, 10),
			(7, 10),
			(8, 10),
			(9, 10),
			(10, 10),
			(11, 10),
		])
		.with_collators(vec![
			(1, 10),
			(2, 20),
			(3, 10),
			(4, 10),
			(5, 10),
			(6, 10),
			(7, 10),
			(8, 10),
			(9, 10),
			(10, 10),
		])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(StakePallet::estimate_next_session_rotation(10).0.unwrap(), 100);
			assert_eq!(StakePallet::estimate_next_session_rotation(20).0.unwrap(), 100);
			assert_eq!(StakePallet::estimate_next_session_rotation(30).0.unwrap(), 100);
			assert_eq!(StakePallet::estimate_next_session_rotation(60).0.unwrap(), 100);
			assert_eq!(StakePallet::estimate_next_session_rotation(100).0.unwrap(), 100);
		});
}

#[test]
fn should_end_session_when_appropriate() {
	ExtBuilder::default()
		.set_blocks_per_round(100)
		.with_balances(vec![
			(1, 10),
			(2, 20),
			(3, 10),
			(4, 10),
			(5, 10),
			(6, 10),
			(7, 10),
			(8, 10),
			(9, 10),
			(10, 10),
			(11, 10),
		])
		.with_collators(vec![
			(1, 10),
			(2, 20),
			(3, 10),
			(4, 10),
			(5, 10),
			(6, 10),
			(7, 10),
			(8, 10),
			(9, 10),
			(10, 10),
		])
		.build_and_execute_with_sanity_tests(|| {
			assert!(!StakePallet::should_end_session(10));
			assert!(!StakePallet::should_end_session(20));
			assert!(!StakePallet::should_end_session(30));
			assert!(!StakePallet::should_end_session(60));
			assert!(StakePallet::should_end_session(100));
		});
}
