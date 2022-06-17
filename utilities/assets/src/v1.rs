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

use crate::*;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct Asset {
	pub chain_id: ChainId,
	pub asset_id: AssetId,
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum AssetError {
	ChainId(ChainIdError),
	AssetId(AssetIdError),
	InvalidInput,
}

impl Asset {
	pub fn ether() -> Self {
		Self { chain_id: ChainId::Eip155(Eip155Reference::ethereum_mainnet()), asset_id: AssetId::Slip44(Slip44Reference::from_slice_unchecked(b"60")) }
	}

	pub fn bitcoin() -> Self {
		Self { chain_id: ChainId::Bip122(GenesisHexHashReference::from_slice_unchecked(b"000000000019d6689c085ae165831e93")), asset_id: AssetId::Slip44(Slip44Reference::from_slice_unchecked(b"0")) }
	}

	pub fn litecoin() -> Self {
		Self { chain_id: ChainId::Bip122(GenesisHexHashReference::from_slice_unchecked(b"12a765e31ffd4059bada1e25190f6e98")), asset_id: AssetId::Slip44(Slip44Reference::from_slice_unchecked(b"2")) }
	}

	pub fn dai() -> Self {
		Self { chain_id: ChainId::Eip155(Eip155Reference::ethereum_mainnet()), asset_id: AssetId::Erc20(EvmSmartContractFungibleReference::from_slice_unchecked(b"0x6b175474e89094c44da98b954eedeac495271d0f")) }
	}

	pub fn req() -> Self {
		Self { chain_id: ChainId::Eip155(Eip155Reference::ethereum_mainnet()), asset_id: AssetId::Erc20(EvmSmartContractFungibleReference::from_slice_unchecked(b"0x8f8221afbb33998d8584a2b05749ba73c37a938a")) }
	}

	pub fn cryptokitties_collection() -> Self {
		Self { chain_id: ChainId::Eip155(Eip155Reference::ethereum_mainnet()), asset_id: AssetId::Erc721(EvmSmartContractNonFungibleReference::from_raw_unchecked(b"0x06012c8cf97BEaD5deAe237070F9587f8E7A266d", None)) }
	}

	pub fn themanymatts_collection() -> Self {
		Self { chain_id: ChainId::Eip155(Eip155Reference::ethereum_mainnet()), asset_id: AssetId::Erc1155(EvmSmartContractNonFungibleReference::from_raw_unchecked(b"0x28959Cf125ccB051E70711D0924a62FB28EAF186", None)) }
	}
}

impl TryFrom<&[u8]> for Asset {
    type Error = AssetError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let mut components = value.split(|c| *c == b'.');

		let chain_id = components
			.next()
			.ok_or(AssetError::InvalidInput)
			.and_then(|input| ChainId::try_from(input).map_err(AssetError::ChainId))?;

		let asset_id = components
			.next()
			.ok_or(AssetError::InvalidInput)
			.and_then(|input| AssetId::try_from(input).map_err(AssetError::AssetId))?;

		Ok(Self{ chain_id, asset_id })
    }
}

#[cfg(test)]
mod test {
    use super::*;

	#[test]
	fn test_valid_ids() {
		let raw_ids = [
			// Test cases from https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-20.md
			"eip155:1.slip44:60",
			"bip122:000000000019d6689c085ae165831e93.slip44:0",
			"cosmos:cosmoshub-3.slip44:118",
			"bip122:12a765e31ffd4059bada1e25190f6e98.slip44:2",
			"cosmos:Binance-Chain-Tigris.slip44:714",
			"cosmos:iov-mainnet.slip44:234",
			// Test cases from https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-21.md
			"eip155:1.erc20:0x6b175474e89094c44da98b954eedeac495271d0f",
			"eip155:1.erc20:0x8f8221afbb33998d8584a2b05749ba73c37a938a",
			// Test cases from https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-22.md
			"eip155:1.erc721:0x06012c8cf97BEaD5deAe237070F9587f8E7A266d",
			"eip155:1.erc721:0x06012c8cf97BEaD5deAe237070F9587f8E7A266d:771769",
			// Test cases from https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-29.md
			"eip155:1.erc1155:0x28959Cf125ccB051E70711D0924a62FB28EAF186",
			"eip155:1.erc1155:0x28959Cf125ccB051E70711D0924a62FB28EAF186:0",
		];

		for id in raw_ids {
			assert!(Asset::try_from(id.as_bytes()).is_ok());
		}
	}
}
