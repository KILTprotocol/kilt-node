use ctype::ctype_entry::CtypeEntry;
use ctype::pallet::Ctypes;
use did::{did_details::DidVerificationKey, pallet::Did};
use runtime_common::AccountId;
use sp_core::H256;
use sp_runtime::AccountId32;
use xcm::VersionedMultiLocation;
use xcm_emulator::{Junction, Junctions, ParentThen};

use crate::mock::para_chains::{peregrine, AssetHubRococo, Peregrine};

pub fn create_mock_ctype(ctype_hash: H256) {
	let ctype_entry = CtypeEntry {
		creator: get_asset_hub_sovereign_account(),
		created_at: 0,
	};

	Ctypes::<peregrine_runtime::Runtime>::insert(ctype_hash, ctype_entry);
}

pub fn get_asset_hub_sovereign_account() -> AccountId {
	Peregrine::sovereign_account_id_of(Peregrine::sibling_location_of(AssetHubRococo::para_id()))
}

pub fn get_peregrine_destination() -> VersionedMultiLocation {
	ParentThen(Junctions::X1(Junction::Parachain(peregrine::PARA_ID))).into()
}

pub fn create_mock_did() {
	let did_key = DidVerificationKey::Account(get_asset_hub_sovereign_account());
	let mut details = did::did_details::DidDetails::<peregrine_runtime::Runtime>::new(
		did_key.clone(),
		0,
		AccountId32::new([0u8; 32]).into(),
	)
	.expect("Failed to generate new DidDetails");

	details.update_attestation_key(did_key, 0).unwrap();

	Did::<peregrine_runtime::Runtime>::insert(get_asset_hub_sovereign_account(), details);
}
