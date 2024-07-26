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

use cumulus_primitives_core::ParaId;
use frame_support::parameter_types;
use polkadot_runtime_common::xcm_sender::{NoPriceForMessageDelivery, ToParachainDeliveryHelper};
use xcm::v4::prelude::*;

use crate::constants::EXISTENTIAL_DEPOSIT;

parameter_types! {
	pub const RandomParaId: ParaId = ParaId::new(42424242);
	pub ExistentialDepositAsset: Option<Asset> = Some((
		Here,
		EXISTENTIAL_DEPOSIT
	).into());

	pub ParachainLocation: Location = ParentThen(Parachain(RandomParaId::get().into()).into()).into();
	pub NativeAsset: Asset = Asset {
					fun: Fungible(EXISTENTIAL_DEPOSIT),
					id: AssetId(Here.into())
				};
}

/// Implementation of the `EnsureDelivery` for the benchmarks.
/// Needed type for the `pallet_xcm::benchmarking::Config`
pub type ParachainDeliveryHelper<ParachainSystem, XcmConfig> = ToParachainDeliveryHelper<
	XcmConfig,
	ExistentialDepositAsset,
	NoPriceForMessageDelivery<ParaId>,
	RandomParaId,
	ParachainSystem,
>;
