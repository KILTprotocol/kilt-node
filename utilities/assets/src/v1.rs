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
use scale_info::TypeInfo;

use frame_support::sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

use crate::*;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct AssetDid {
	pub chain_id: ChainId,
	pub asset_id: AssetId,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug)]
pub enum AssetDidError {
	ChainId(ChainIdError),
	AssetId(AssetIdError),
	InvalidInput,
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
			asset_id: Slip44Reference::from_slice_unchecked(b"60").into(),
		}
	}

	pub fn bitcoin_currency() -> Self {
		Self {
			chain_id: ChainId::Bip122(GenesisHexHashReference::from_slice_unchecked(
				b"000000000019d6689c085ae165831e93",
			)),
			asset_id: Slip44Reference::from_slice_unchecked(b"0").into(),
		}
	}

	pub fn litecoin_currency() -> Self {
		Self {
			chain_id: ChainId::Bip122(GenesisHexHashReference::from_slice_unchecked(
				b"12a765e31ffd4059bada1e25190f6e98",
			)),
			asset_id: Slip44Reference::from_slice_unchecked(b"2").into(),
		}
	}

	pub fn dai_currency() -> Self {
		Self {
			chain_id: Eip155Reference::ethereum_mainnet().into(),
			asset_id: EvmSmartContractFungibleReference::from_slice_unchecked(
				b"6b175474e89094c44da98b954eedeac495271d0f",
			)
			.into(),
		}
	}

	pub fn req_currency() -> Self {
		Self {
			chain_id: Eip155Reference::ethereum_mainnet().into(),
			asset_id: EvmSmartContractFungibleReference::from_slice_unchecked(
				b"8f8221afbb33998d8584a2b05749ba73c37a938a",
			)
			.into(),
		}
	}

	pub fn cryptokitties_collection() -> Self {
		Self {
			chain_id: Eip155Reference::ethereum_mainnet().into(),
			asset_id: AssetId::Erc721(EvmSmartContractNonFungibleReference::from_raw_unchecked(
				b"06012c8cf97BEaD5deAe237070F9587f8E7A266d",
				None,
			)),
		}
	}

	pub fn themanymatts_collection() -> Self {
		Self {
			chain_id: Eip155Reference::ethereum_mainnet().into(),
			asset_id: AssetId::Erc1155(EvmSmartContractNonFungibleReference::from_raw_unchecked(
				b"28959Cf125ccB051E70711D0924a62FB28EAF186",
				None,
			)),
		}
	}
}

impl TryFrom<&[u8]> for AssetDid {
	type Error = AssetDidError;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		match value {
			// Asset DIDs must start with "did:asset:" to be considered further.
			[b'd', b'i', b'd', b':', b'a', b's', b's', b'e', b't', b':', components @ ..] => {
				let mut components = components.split(|c| *c == b'.');

				let chain_id = components
					.next()
					.ok_or(AssetDidError::InvalidInput)
					.and_then(|input| ChainId::try_from(input).map_err(AssetDidError::ChainId))?;

				let asset_id = components
					.next()
					.ok_or(AssetDidError::InvalidInput)
					.and_then(|input| AssetId::try_from(input).map_err(AssetDidError::AssetId))?;

				Ok(Self { chain_id, asset_id })
			},
			_ => Err(AssetDidError::InvalidInput)
		}
	}
}

impl TryFrom<Vec<u8>> for AssetDid {
	type Error = AssetDidError;

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(&value[..])
	}
}

impl TryFrom<&'static str> for AssetDid {
	type Error = AssetDidError;

	fn try_from(value: &'static str) -> Result<Self, Self::Error> {
		Self::try_from(value.as_bytes())
	}
}

#[cfg(feature = "std")]
impl TryFrom<String> for AssetDid {
	type Error = AssetDidError;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::try_from(value.as_bytes())
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn valid_ids() {
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

		// FIXME: Better test logic
		for id in raw_ids {
			assert!(AssetDid::try_from(id.as_bytes()).is_ok());
		}
	}

	#[test]
	fn helpers() {
		// These functions should never crash. We just check that here.
		AssetDid::ether_currency();
		AssetDid::bitcoin_currency();
		AssetDid::litecoin_currency();
		AssetDid::dai_currency();
		AssetDid::req_currency();
		AssetDid::cryptokitties_collection();
		AssetDid::themanymatts_collection();
	}
}
