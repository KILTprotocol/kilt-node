// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

use frame_support::parameter_types;
use runtime_common::constants;

use crate::{weights, Balances, Runtime, RuntimeEvent};

parameter_types! {
	pub const Inflation: &'static str = "Inflation";

}

pub type RuntimeMigrations = (
	pallet_xcm::migration::MigrateToLatestXcmVersion<Runtime>,
	frame_support::migrations::RemovePallet<Inflation, <Runtime as frame_system::Config>::DbWeight>,
);

impl pallet_migration::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MaxMigrationsPerPallet = constants::pallet_migration::MaxMigrationsPerPallet;
	type WeightInfo = weights::pallet_migration::WeightInfo<Runtime>;
}
