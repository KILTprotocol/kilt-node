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

use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::BadOrigin;

use crate::{mock::runtime::*, Configuration, ConfigurationStore, Pallet};

// submit_ctype_creation_operation

#[test]
fn test_set_config() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(ConfigurationStore::<Test>::try_get(), Err(()));

		assert_ok!(Pallet::<Test>::set_configuration(
			RuntimeOrigin::signed(ACCOUNT_00),
			Configuration {
				relay_block_strictly_increasing: true,
			},
		));
		assert_eq!(
			ConfigurationStore::<Test>::get(),
			Configuration {
				relay_block_strictly_increasing: true,
			}
		);
	});
}

#[test]
fn test_set_config_unauthorized() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Pallet::<Test>::set_configuration(
				RuntimeOrigin::signed(ACCOUNT_01),
				Configuration {
					relay_block_strictly_increasing: true
				}
			),
			BadOrigin
		);
	});
}
