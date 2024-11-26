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

use frame_support::parameter_types;
use runtime_common::constants;

use crate::{weights, Balances, Runtime, RuntimeEvent};

parameter_types! {
	pub const DmpPalletName: &'static str = "DmpQueue";
}

pub type RuntimeMigrations =
	frame_support::migrations::RemovePallet<DmpPalletName, <Runtime as frame_system::Config>::DbWeight>;

impl pallet_migration::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MaxMigrationsPerPallet = constants::pallet_migration::MaxMigrationsPerPallet;
	type WeightInfo = weights::pallet_migration::WeightInfo<Runtime>;
}
