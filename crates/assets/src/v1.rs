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
use codec::{Decode, Encode, MaxEncodedLen};
use hex_literal::hex;
use scale_info::TypeInfo;

use frame_support::sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

use crate::*;

pub const MINIMUM_ASSET_DID_LENGTH: usize =
	b"did:asset:".len() + MINIMUM_CHAIN_ID_LENGTH + b".".len() + MINIMUM_ASSET_ID_LENGTH;
pub const MAXIMUM_ASSET_DID_LENGTH: usize =
	b"did:asset:".len() + MAXIMUM_CHAIN_ID_LENGTH + b".".len() + MAXIMUM_ASSET_ID_LENGTH;

/// An Asset DID as specified in the Asset DID method specification.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct AssetDid {
	pub chain_id: ChainId,
	pub asset_id: AssetId,
}

/// An error in the Asset DID parsing logic.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug)]
pub enum AssetDidError {
	/// An error in the chain ID parsing logic.
	ChainId(ChainIdError),
	/// An error in the asset ID parsing logic.
	AssetId(AssetIdError),
	/// A generic error not belonging to any of the other categories.
	InvalidFormat,
}

impl From<ChainIdError> for AssetDidError {
	fn from(err: ChainIdError) -> Self {
		Self::ChainId(err)
	}
}

impl From<AssetIdError> for AssetDidError {
	fn from(err: AssetIdError) -> Self {
		Self::AssetId(err)
	}
}

impl AssetDid {
	pub fn ether_currency() -> Self {
		Self {
			chain_id: Eip155Reference::ethereum_mainnet().into(),
			asset_id: Slip44Reference(60.into()).into(),
		}
	}

	pub fn bitcoin_currency() -> Self {
		Self {
			chain_id: ChainId::Bip122(GenesisHexHash32Reference::bitcoin_mainnet()),
			asset_id: Slip44Reference(0.into()).into(),
		}
	}

	pub fn litecoin_currency() -> Self {
		Self {
			chain_id: ChainId::Bip122(GenesisHexHash32Reference::litecoin_mainnet()),
			asset_id: Slip44Reference(2.into()).into(),
		}
	}

	pub fn dai_currency() -> Self {
		Self {
			chain_id: Eip155Reference::ethereum_mainnet().into(),
			asset_id: EvmSmartContractFungibleReference(hex!("6b175474e89094c44da98b954eedeac495271d0f")).into(),
		}
	}

	pub fn req_currency() -> Self {
		Self {
			chain_id: Eip155Reference::ethereum_mainnet().into(),
			asset_id: EvmSmartContractFungibleReference(hex!("8f8221afbb33998d8584a2b05749ba73c37a938a")).into(),
		}
	}

	pub fn cryptokitties_collection() -> Self {
		Self {
			chain_id: Eip155Reference::ethereum_mainnet().into(),
			asset_id: AssetId::Erc721(EvmSmartContractNonFungibleReference(
				EvmSmartContractFungibleReference(hex!("06012c8cf97BEaD5deAe237070F9587f8E7A266d")),
				None,
			)),
		}
	}

	pub fn themanymatts_collection() -> Self {
		Self {
			chain_id: Eip155Reference::ethereum_mainnet().into(),
			asset_id: AssetId::Erc1155(EvmSmartContractNonFungibleReference(
				EvmSmartContractFungibleReference(hex!("28959Cf125ccB051E70711D0924a62FB28EAF186")),
				None,
			)),
		}
	}
}

impl AssetDid {
	/// Try to parse an `AssetDID` instance from the provided UTF8-encoded
	/// input.
	pub fn from_utf8_encoded<I>(input: I) -> Result<Self, AssetDidError>
	where
		I: AsRef<[u8]> + Into<Vec<u8>>,
	{
		match input.as_ref() {
			// Asset DIDs must start with "did:asset:" to be valid. The "did:asset:" prefix is then stripped off.
			[b'd', b'i', b'd', b':', b'a', b's', b's', b'e', b't', b':', components @ ..] => {
				let mut components = components.split(|c| *c == b'.');

				let chain_id = components
					.next()
					.ok_or(AssetDidError::InvalidFormat)
					.and_then(|input| ChainId::from_utf8_encoded(input).map_err(AssetDidError::ChainId))?;

				let asset_id = components
					.next()
					.ok_or(AssetDidError::InvalidFormat)
					.and_then(|input| AssetId::from_utf8_encoded(input).map_err(AssetDidError::AssetId))?;

				Ok(Self { chain_id, asset_id })
			}
			_ => Err(AssetDidError::InvalidFormat),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn valid_ids() {
		let raw_ids = [
			// Test cases from https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-20.md
			"did:asset:eip155:1.slip44:60",
			"did:asset:bip122:000000000019d6689c085ae165831e93.slip44:0",
			"did:asset:cosmos:cosmoshub-3.slip44:118",
			"did:asset:bip122:12a765e31ffd4059bada1e25190f6e98.slip44:2",
			"did:asset:cosmos:Binance-Chain-Tigris.slip44:714",
			"did:asset:cosmos:iov-mainnet.slip44:234",
			// Test cases from https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-21.md
			"did:asset:eip155:1.erc20:0x6b175474e89094c44da98b954eedeac495271d0f",
			"did:asset:eip155:1.erc20:0x8f8221afbb33998d8584a2b05749ba73c37a938a",
			// Test cases from https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-22.md
			"did:asset:eip155:1.erc721:0x06012c8cf97BEaD5deAe237070F9587f8E7A266d",
			"did:asset:eip155:1.erc721:0x06012c8cf97BEaD5deAe237070F9587f8E7A266d:771769",
			// Test cases from https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-29.md
			"did:asset:eip155:1.erc1155:0x28959Cf125ccB051E70711D0924a62FB28EAF186",
			"did:asset:eip155:1.erc1155:0x28959Cf125ccB051E70711D0924a62FB28EAF186:0",
		];

		for id in raw_ids {
			assert!(
				AssetDid::from_utf8_encoded(id.as_bytes()).is_ok(),
				"Test for valid IDs failed for {:?}",
				id
			);
		}
	}

	#[test]
	fn helpers() {
		// These functions should never panic. We just check that here.
		AssetDid::ether_currency();
		AssetDid::bitcoin_currency();
		AssetDid::litecoin_currency();
		AssetDid::dai_currency();
		AssetDid::req_currency();
		AssetDid::cryptokitties_collection();
		AssetDid::themanymatts_collection();
	}
}
