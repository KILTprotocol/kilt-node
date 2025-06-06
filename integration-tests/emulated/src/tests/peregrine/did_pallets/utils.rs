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

use ctype::{ctype_entry::CtypeEntry, pallet::Ctypes};
use did::{did_details::DidVerificationKey, pallet::Did};
use runtime_common::Balance;
use sp_core::H256;
use sp_runtime::AccountId32;
use xcm::{
	lts::prelude::{
		Instruction::{BuyExecution, Transact, WithdrawAsset},
		Junction,
		Junctions::{self, Here},
		OriginKind, ParentThen, Weight, WeightLimit, Xcm,
	},
	DoubleEncoded, VersionedLocation, VersionedXcm,
};
use xcm_emulator::Parachain;

use crate::mock::network::{AssetHub, Peregrine};

pub fn create_mock_ctype(ctype_hash: H256, creator: AccountId32) {
	let ctype_entry = CtypeEntry { creator, created_at: 0 };

	Ctypes::<peregrine_runtime::Runtime>::insert(ctype_hash, ctype_entry);
}

pub fn get_asset_hub_sovereign_account() -> AccountId32 {
	Peregrine::sovereign_account_id_of(Peregrine::sibling_location_of(AssetHub::para_id()))
}

pub fn get_sibling_destination_peregrine() -> VersionedLocation {
	ParentThen(Junctions::X1([Junction::Parachain(Peregrine::para_id().into())].into())).into()
}

pub fn create_mock_did_from_account(account: AccountId32) {
	let did_key = DidVerificationKey::Account(account);
	let mut details = did::did_details::DidDetails::<peregrine_runtime::Runtime>::new(
		did_key.clone(),
		0,
		AccountId32::new([0u8; 32]),
	)
	.expect("Failed to generate new DidDetails");

	details.update_attestation_key(did_key, 0).unwrap();

	Did::<peregrine_runtime::Runtime>::insert(get_asset_hub_sovereign_account(), details);
}

pub fn construct_basic_transact_xcm_message(
	origin_kind: OriginKind,
	withdraw_balance: Balance,
	call: DoubleEncoded<()>,
) -> VersionedXcm<()> {
	let require_weight_at_most = Weight::from_parts(10_000_600_000_000, 200_000_000_000);

	VersionedXcm::from(Xcm(vec![
		WithdrawAsset((Here, withdraw_balance).into()),
		BuyExecution {
			fees: (Here, withdraw_balance).into(),
			weight_limit: WeightLimit::Unlimited,
		},
		Transact {
			origin_kind,
			require_weight_at_most,
			call,
		},
	]))
}
