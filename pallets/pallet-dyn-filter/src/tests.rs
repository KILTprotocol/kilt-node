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

use frame_support::{assert_ok, traits::Contains};
use itertools::Itertools;

use crate::{
	mock::{ExtBuilder, Origin, Test, CALL_FEATURE, CALL_SYSTEM, CALL_TRANSFER, CALL_XCM},
	setting::FilterSettings,
	Pallet,
};

#[test]
fn check_filter() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(Pallet::<Test>::contains(&CALL_TRANSFER));
		assert!(Pallet::<Test>::contains(&CALL_FEATURE));
		assert!(Pallet::<Test>::contains(&CALL_XCM));
		assert!(Pallet::<Test>::contains(&CALL_SYSTEM));

		for items in [true, false].iter().combinations_with_replacement(3) {
			assert_ok!(Pallet::<Test>::set_filter(
				Origin::root(),
				FilterSettings {
					transfer_disabled: *items[0],
					feature_disabled: *items[1],
					xcm_disabled: *items[2],
				},
			));

			assert!(Pallet::<Test>::contains(&CALL_SYSTEM));
			assert_ne!(
				*items[0],
				Pallet::<Test>::contains(&CALL_TRANSFER),
				"Didn't filter transfer, Setting: {:?}",
				items
			);
			assert_ne!(
				*items[1],
				Pallet::<Test>::contains(&CALL_FEATURE),
				"Didn't filter feature, Setting: {:?}",
				items
			);
			assert_ne!(
				*items[2],
				Pallet::<Test>::contains(&CALL_XCM),
				"Didn't filter xcm, Setting: {:?}",
				items
			);
		}
	});
}
