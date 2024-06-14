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

use emulated_integration_tests_common::{
	impl_accounts_helpers_for_relay_chain, impl_assert_events_helpers_for_relay_chain,
	impl_hrmp_channels_helpers_for_relay_chain, impl_send_transact_helpers_for_relay_chain,
	xcm_emulator::decl_test_relay_chains,
};
use polkadot_emulated_chain::genesis;

// Polkadot declaration
decl_test_relay_chains! {
	#[api_version(10)]
	pub struct Polkadot {
		genesis = genesis::genesis(),
		on_init = (),
		runtime = polkadot_runtime,
		core = {
			SovereignAccountOf: polkadot_runtime::xcm_config::SovereignAccountOf,
		},
		pallets = {
			XcmPallet: polkadot_runtime::XcmPallet,
			Balances: polkadot_runtime::Balances,
			Treasury: polkadot_runtime::Treasury,
			AssetRate: polkadot_runtime::AssetRate,
			Hrmp: polkadot_runtime::Hrmp,
			Identity: polkadot_runtime::Identity,
			IdentityMigrator: polkadot_runtime::IdentityMigrator,
		}
	},
}

// Polkadot implementation
impl_accounts_helpers_for_relay_chain!(Polkadot);
impl_assert_events_helpers_for_relay_chain!(Polkadot);
impl_hrmp_channels_helpers_for_relay_chain!(Polkadot);
impl_send_transact_helpers_for_relay_chain!(Polkadot);
