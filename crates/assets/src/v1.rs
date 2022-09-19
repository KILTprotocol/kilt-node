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
use sp_std::{fmt::Display, vec::Vec};

use core::str;

use crate::*;

/// The minimum length, including separator symbols, an asset DID can have
/// according to the Asset DID specification.
/// The minimum length is given by the length of the "did:asset:" prefix,
/// plus the minimum length of a valid CAIP-2 chain ID, the minimum length of
/// a valid CAIP-19 asset ID, and the separator symbol between the two.
pub const MINIMUM_ASSET_DID_LENGTH: usize =
	DID_ASSET_PREFIX.len() + MINIMUM_CHAIN_ID_LENGTH + 1 + MINIMUM_ASSET_ID_LENGTH;
/// The maximum length, including separator symbols, an asset DID can have
/// according to the Asset DID specification.
/// The maximum length is given by the length of the "did:asset:" prefix,
/// plus the maximum length of a valid CAIP-2 chain ID, the maximum length of
/// a valid CAIP-19 asset ID, and the separator symbol between the two.
pub const MAXIMUM_ASSET_DID_LENGTH: usize =
	DID_ASSET_PREFIX.len() + MAXIMUM_CHAIN_ID_LENGTH + 1 + MAXIMUM_ASSET_ID_LENGTH;

const DID_ASSET_PREFIX: &[u8] = b"did:asset:";
const CHAIN_ASSET_SEPARATOR: u8 = b'.';

/// An Asset DID as specified in the Asset DID method specification.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct AssetDid {
	pub chain_id: ChainId,
	pub asset_id: AssetId,
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
		let input = input.as_ref();
		let input_length = input.len();
		if !(MINIMUM_ASSET_DID_LENGTH..=MAXIMUM_ASSET_DID_LENGTH).contains(&input_length) {
			log::trace!(
				"Length of provided input {} is not included in the inclusive range [{},{}]",
				input_length,
				MINIMUM_ASSET_DID_LENGTH,
				MAXIMUM_ASSET_DID_LENGTH
			);
			return Err(AssetDidError::InvalidFormat);
		}

		let asset_id = input
			.as_ref()
			.strip_prefix(DID_ASSET_PREFIX)
			.ok_or(AssetDidError::InvalidFormat)?;

		let mut split = asset_id.splitn(2, |c| *c == CHAIN_ASSET_SEPARATOR);
		let (chain, asset) = (split.next(), split.next());

		if let (Some(chain), Some(asset)) = (chain, asset) {
			let chain_id = ChainId::from_utf8_encoded(chain).map_err(AssetDidError::ChainId)?;
			let asset_id = AssetId::from_utf8_encoded(asset).map_err(AssetDidError::AssetId)?;
			Ok(Self { chain_id, asset_id })
		} else {
			Err(AssetDidError::InvalidFormat)?
		}
	}
}

impl Display for AssetDid {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(
			f,
			"{}{}{}{}",
			str::from_utf8(DID_ASSET_PREFIX).expect("Conversion of Asset DID prefix to string should never fail."),
			self.chain_id,
			char::from(CHAIN_ASSET_SEPARATOR),
			self.asset_id
		)
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
			let asset_did = AssetDid::from_utf8_encoded(id.as_bytes())
				.unwrap_or_else(|_| panic!("Test for valid IDs failed for {:?}", id));
			// Verify that the ToString implementation prints exactly the original input
			assert_eq!(asset_did.to_string().to_lowercase(), id.to_lowercase());
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
