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

use ctype::{ctype_entry::CtypeEntry, pallet::Ctypes};
use did::{did_details::DidVerificationKey, pallet::Did};
use runtime_common::{AccountId, Balance};
use sp_core::H256;
use sp_runtime::AccountId32;
use xcm::{
	v3::prelude::{
		Instruction::{BuyExecution, Transact, WithdrawAsset},
		Junction,
		Junctions::{self, Here},
		OriginKind, ParentThen, Weight, WeightLimit, Xcm,
	},
	DoubleEncoded, VersionedMultiLocation, VersionedXcm,
};
use xcm_emulator::Parachain;

use crate::mock::para_chains::{spiritnet, AssetHubPolkadot, Spiritnet};

pub fn create_mock_ctype(ctype_hash: H256, creator: AccountId32) {
	let ctype_entry = CtypeEntry { creator, created_at: 0 };

	Ctypes::<spiritnet_runtime::Runtime>::insert(ctype_hash, ctype_entry);
}

pub fn get_asset_hub_sovereign_account() -> AccountId {
	Spiritnet::sovereign_account_id_of(Spiritnet::sibling_location_of(AssetHubPolkadot::para_id()))
}

pub fn get_sibling_destination_spiritnet() -> VersionedMultiLocation {
	ParentThen(Junctions::X1(Junction::Parachain(spiritnet::PARA_ID))).into()
}

pub fn create_mock_did_from_account(account: AccountId32) {
	let did_key = DidVerificationKey::Account(account);
	let mut details = did::did_details::DidDetails::<spiritnet_runtime::Runtime>::new(
		did_key.clone(),
		0,
		AccountId32::new([0u8; 32]),
	)
	.expect("Failed to generate new DidDetails");

	details.update_attestation_key(did_key, 0).unwrap();

	Did::<spiritnet_runtime::Runtime>::insert(get_asset_hub_sovereign_account(), details);
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
