use cumulus_primitives_core::Junction;
use hex_literal::hex;
use xcm::v4::{AssetId, Junctions, Location, NetworkId};

pub mod is_reserve;
pub mod matcher;

fn get_remote_asset_id() -> AssetId {
	let asset_address: [u8; 20] = hex!("5d3d01fd6d2ad1169b17918eb4f153c6616288eb");

	AssetId(Location {
		parents: 1,
		interior: Junctions::X2(
			[
				Junction::GlobalConsensus(NetworkId::Ethereum { chain_id: 11155111 }),
				Junction::AccountKey20 {
					network: None,
					key: asset_address,
				},
			]
			.into(),
		),
	})
}

fn get_remote_reserve_location() -> Location {
	Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(100u32)].into()),
	}
}
