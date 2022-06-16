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

pub mod chain_id {

	use base58::FromBase58;

	use frame_support::{ensure, traits::ConstU32, BoundedVec};
	use sp_runtime::traits::CheckedConversion;
	use sp_std::str;

	use crate::{Config, Error};

	const MINIMUM_NAMESPACE_LENGTH: u32 = 3;
	const MAXIMUM_NAMESPACE_LENGTH: u32 = 8;
	const MINIMUM_REFERENCE_LENGTH: u32 = 1;
	const MAXIMUM_REFERENCE_LENGTH: u32 = 32;

	#[derive(std::fmt::Debug, PartialEq, Eq, PartialOrd, Ord)]
	pub enum ChainId<C> {
		Eip155(Eip155Reference<C>),
		Bip122(GenesisHexHashReference<C, 32>),
		Dotsama(GenesisHexHashReference<C, 32>),
		Solana(GenesisBase58HashReference<C>),
		Generic(GenericChainId<C>),
	}

	impl<C: Config> TryFrom<Vec<u8>> for ChainId<C> {
		type Error = Error<C>;

		fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
			match value.as_slice() {
				// "eip155:" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-3.md
				[b'e', b'i', b'p', b'1', b'5', b'5', b':', chain_id @ ..] => {
					Eip155Reference::<C>::try_from(chain_id).map(|reference| Self::Eip155(reference))
				}
				// "bip122:" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-4.md
				[b'b', b'i', b'p', b'1', b'2', b'2', b':', chain_id @ ..] => {
					GenesisHexHashReference::<C, 32>::try_from(chain_id).map(|reference| Self::Bip122(reference))
				}
				// "polkadot" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-13.md
				[b'p', b'o', b'l', b'k', b'a', b'd', b'o', b't', b':', chain_id @ ..] => {
					GenesisHexHashReference::<C, 32>::try_from(chain_id).map(|reference| Self::Dotsama(reference))
				}
				// "solana" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-30.md
				[b's', b'o', b'l', b'a', b'n', b'a', b':', chain_id @ ..] => {
					GenesisBase58HashReference::<C>::try_from(chain_id).map(|reference| Self::Solana(reference))
				}
				chain_id => GenericChainId::<C>::try_from(chain_id).map(|id| Self::Generic(id)),
			}
		}
	}

	impl<C: Config> ChainId<C> {
		pub fn ethereum_mainnet() -> Self {
			Self::Eip155(Eip155Reference::<C>::from_slice_unsafe(b"1"))
		}

		pub fn moonriver_eth() -> Self {
			// Info taken from https://chainlist.org/
			Self::Eip155(Eip155Reference::<C>::from_slice_unsafe(b"1285"))
		}

		pub fn moonbeam_eth() -> Self {
			// Info taken from https://chainlist.org/
			Self::Eip155(Eip155Reference::<C>::from_slice_unsafe(b"1284"))
		}

		pub fn bitcoin_mainnet() -> Self {
			Self::Bip122(GenesisHexHashReference::<C, 32>::from_slice_unsafe(
				b"000000000019d6689c085ae165831e93",
			))
		}

		pub fn polkadot() -> Self {
			Self::Dotsama(GenesisHexHashReference::<C, 32>::from_slice_unsafe(
				b"91b171bb158e2d3848fa23a9f1c25182",
			))
		}

		pub fn kusama() -> Self {
			Self::Dotsama(GenesisHexHashReference::<C, 32>::from_slice_unsafe(
				b"b0a8d493285c2df73290dfb7e61f870f",
			))
		}

		pub fn kilt_spiritnet() -> Self {
			Self::Dotsama(GenesisHexHashReference::<C, 32>::from_slice_unsafe(
				b"411f057b9107718c9624d6aa4a3f23c1",
			))
		}

		pub fn solana_mainnet() -> Self {
			Self::Solana(GenesisBase58HashReference::<C>::from_slice_unsafe(
				b"4sGjMW1sUnHzSxGspuhpqLDx6wiyjNtZ",
			))
		}
	}

	#[derive(sp_runtime::RuntimeDebug, PartialEq, Eq, PartialOrd, Ord)]
	pub struct Eip155Reference<C>(
		pub BoundedVec<u8, ConstU32<MAXIMUM_REFERENCE_LENGTH>>,
		Option<sp_std::marker::PhantomData<C>>,
	);

	impl<C: Config> Eip155Reference<C> {
		#[allow(dead_code)]
		pub(crate) fn from_slice_unsafe(slice: &[u8]) -> Self {
			Self(slice.to_vec().try_into().unwrap(), None)
		}
	}

	impl<C: Config> TryFrom<&[u8]> for Eip155Reference<C> {
		type Error = Error<C>;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			let input_len = value.len().checked_into::<u32>().ok_or(Error::<C>::InvalidInput)?;
			ensure!(
				(MINIMUM_REFERENCE_LENGTH..=MAXIMUM_REFERENCE_LENGTH).contains(&input_len),
				Error::<C>::InvalidInput
			);
			value.iter().try_for_each(|c| {
				if !(b'0'..=b'9').contains(c) {
					Err(Error::<C>::InvalidInput)
				} else {
					Ok(())
				}
			})?;
			// Unwrapping since we just checked for length
			Ok(Self(value.to_vec().try_into().unwrap(), None))
		}
	}

	#[derive(sp_runtime::RuntimeDebug, PartialEq, Eq, PartialOrd, Ord)]
	pub struct GenesisHexHashReference<C, const L: usize = 32>(pub [u8; L], Option<sp_std::marker::PhantomData<C>>);

	impl<C: Config, const L: usize> GenesisHexHashReference<C, L> {
		#[allow(dead_code)]
		pub(crate) fn from_slice_unsafe(slice: &[u8]) -> Self {
			Self(slice.try_into().unwrap(), None)
		}
	}

	impl<C: Config, const L: usize> TryFrom<&[u8]> for GenesisHexHashReference<C, L> {
		type Error = Error<C>;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			// Verify it's a valid HEX string
			hex::decode(value).map_err(|_| Error::<C>::InvalidInput)?;
			let inner: [u8; L] = value.try_into().map_err(|_| Error::<C>::InvalidInput)?;
			Ok(Self(inner, None))
		}
	}

	#[derive(sp_runtime::RuntimeDebug, PartialEq, Eq, PartialOrd, Ord)]
	pub struct GenesisBase58HashReference<C>(
		pub BoundedVec<u8, ConstU32<MAXIMUM_REFERENCE_LENGTH>>,
		Option<sp_std::marker::PhantomData<C>>,
	);

	impl<C: Config> GenesisBase58HashReference<C> {
		#[allow(dead_code)]
		pub(crate) fn from_slice_unsafe(slice: &[u8]) -> Self {
			Self(slice.to_vec().try_into().unwrap(), None)
		}
	}

	impl<C: Config> TryFrom<&[u8]> for GenesisBase58HashReference<C> {
		type Error = Error<C>;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			let input_len = value.len().checked_into::<u32>().ok_or(Error::<C>::InvalidInput)?;
			ensure!(
				(MINIMUM_REFERENCE_LENGTH..=MAXIMUM_REFERENCE_LENGTH).contains(&input_len),
				Error::<C>::InvalidInput
			);
			let decoded_string = str::from_utf8(value).map_err(|_| Error::<C>::InvalidInput)?;
			// Check that the string is valid base58
			decoded_string.from_base58().map_err(|_| Error::<C>::InvalidInput)?;
			// Unwrapping since we just checked for length
			Ok(Self(value.to_vec().try_into().unwrap(), None))
		}
	}

	#[derive(sp_runtime::RuntimeDebug, PartialEq, Eq, PartialOrd, Ord)]
	pub struct GenericChainId<C> {
		namespace: BoundedVec<u8, ConstU32<MAXIMUM_NAMESPACE_LENGTH>>,
		reference: BoundedVec<u8, ConstU32<MAXIMUM_REFERENCE_LENGTH>>,
		_phantom: Option<C>,
	}

	impl<C> GenericChainId<C> {
		#[allow(dead_code)]
		fn from_components_unsafe(namespace: &[u8], reference: &[u8]) -> Self {
			Self {
				namespace: namespace.to_vec().try_into().unwrap(),
				reference: reference.to_vec().try_into().unwrap(),
				_phantom: None,
			}
		}
		fn from_components(
			namespace: BoundedVec<u8, ConstU32<MAXIMUM_NAMESPACE_LENGTH>>,
			reference: BoundedVec<u8, ConstU32<MAXIMUM_REFERENCE_LENGTH>>,
		) -> Self {
			Self {
				namespace,
				reference,
				_phantom: None,
			}
		}
	}

	impl<C: Config> TryFrom<&[u8]> for GenericChainId<C> {
		type Error = Error<C>;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			ensure!(
				value.len() <= (MAXIMUM_REFERENCE_LENGTH + MAXIMUM_NAMESPACE_LENGTH + 1) as usize,
				Error::<C>::InvalidInput
			);
			let mut components = value.split(|c| *c == b':');

			if let (Some(namespace), Some(reference)) = (components.next(), components.next()) {
				let namespace_length = namespace.iter().try_fold(0u32, |length, c| {
					let new_length = length + 1;
					if new_length > MAXIMUM_NAMESPACE_LENGTH {
						return Err(Error::<C>::InvalidInput);
					}
					if !matches!(c, b'-' | b'a'..=b'z' | b'0'..=b'9') {
						return Err(Error::<C>::InvalidInput);
					}
					Ok(new_length)
				})?;
				if namespace_length < MINIMUM_NAMESPACE_LENGTH {
					return Err(Error::<C>::InvalidInput);
				}

				let reference_length = reference.iter().try_fold(0u32, |length, c| {
					let new_length = length + 1;
					if new_length > MAXIMUM_REFERENCE_LENGTH {
						return Err(Error::<C>::InvalidInput);
					}
					if !matches!(c, b'-' | b'a'..=b'z' | b'A'..=b'Z' |b'0'..=b'9') {
						return Err(Error::<C>::InvalidInput);
					}
					Ok(new_length)
				})?;
				if reference_length < MINIMUM_REFERENCE_LENGTH {
					return Err(Error::<C>::InvalidInput);
				}
				Ok(Self::from_components(
					namespace.to_vec().try_into().unwrap(),
					reference.to_vec().try_into().unwrap(),
				))
			} else {
				Err(Error::<C>::InvalidInput)
			}
		}
	}

	#[cfg(test)]
	mod test {
		use super::*;

		use crate::mock::Test;

		#[test]
		fn test_eip155_chains() {
			let valid_chains = [
				"eip155:1",
				"eip155:5",
				"eip155:99999999999999999999999999999999",
				"eip155:0",
			];
			for chain in valid_chains {
				assert!(
					ChainId::<Test>::try_from(chain.as_bytes().to_vec()).is_ok(),
					"Chain ID {:?} should not fail to parse for eip155 chains",
					chain
				);
			}

			let invalid_chains = [
				// Too short
				"e",
				"ei",
				"eip",
				"eip1",
				"eip15",
				"eip155",
				"eip155:",
				// Not a number
				"eip155:a",
				"eip155::",
				"eip155:›",
				"eip155:😁",
				// Max chars + 1
				"eip155:999999999999999999999999999999999",
			];
			for chain in invalid_chains {
				assert!(
					ChainId::<Test>::try_from(chain.as_bytes().to_vec()).is_err(),
					"Chain ID {:?} should fail to parse for eip155 chains",
					chain
				);
			}
		}

		#[test]
		fn test_bip122_chains() {
			let valid_chains = [
				"bip122:000000000019d6689c085ae165831e93",
				"bip122:000000000019D6689C085AE165831E93",
				"bip122:12a765e31ffd4059bada1e25190f6e98",
				"bip122:fdbe99b90c90bae7505796461471d89a",
				"bip122:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
			];
			for chain in valid_chains {
				assert!(
					ChainId::<Test>::try_from(chain.as_bytes().to_vec()).is_ok(),
					"Chain ID {:?} should not fail to parse for polkadot chains",
					chain
				);
			}

			let invalid_chains = [
				// Too short
				"b",
				"bi",
				"bip",
				"bip1",
				"bip12",
				"bip122",
				"bip122:",
				// Not an HEX string
				"bip122:gg",
				"bip122::",
				"bip122:›",
				"bip122:😁",
				// Not the expected length
				"bip122:a",
				"bip122:aa",
				"bip122:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
			];
			for chain in invalid_chains {
				assert!(
					ChainId::<Test>::try_from(chain.as_bytes().to_vec()).is_err(),
					"Chain ID {:?} should fail to parse for polkadot chains",
					chain
				);
			}
		}

		#[test]
		fn test_dotsama_chains() {
			let valid_chains = [
				"polkadot:b0a8d493285c2df73290dfb7e61f870f",
				"polkadot:B0A8D493285C2DF73290DFB7E61F870F",
				"polkadot:742a2ca70c2fda6cee4f8df98d64c4c6",
				"polkadot:37e1f8125397a98630013a4dff89b54c",
				"polkadot:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
			];
			for chain in valid_chains {
				assert!(
					ChainId::<Test>::try_from(chain.as_bytes().to_vec()).is_ok(),
					"Chain ID {:?} should not fail to parse for polkadot chains",
					chain
				);
			}

			let invalid_chains = [
				// Too short
				"p",
				"po",
				"pol",
				"polk",
				"polka",
				"polkad",
				"polkado",
				"polkadot",
				"polkadot:",
				// Not an HEX string
				"polkadot:gg",
				"polkadot::",
				"polkadot:›",
				"polkadot:😁",
				// Not the expected length
				"polkadot:a",
				"polkadot:aa",
				"polkadot:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
			];
			for chain in invalid_chains {
				assert!(
					ChainId::<Test>::try_from(chain.as_bytes().to_vec()).is_err(),
					"Chain ID {:?} should fail to parse for polkadot chains",
					chain
				);
			}
		}

		#[test]
		fn test_solana_chains() {
			let valid_chains = [
				"solana:a",
				"solana:4sGjMW1sUnHzSxGspuhpqLDx6wiyjNtZ",
				"solana:8E9rvCKLFQia2Y35HXjjpWzj8weVo44K",
			];
			for chain in valid_chains {
				assert!(
					ChainId::<Test>::try_from(chain.as_bytes().to_vec()).is_ok(),
					"Chain ID {:?} should not fail to parse for solana chains",
					chain
				);
			}

			let invalid_chains = [
				// Too short
				"s",
				"so",
				"sol",
				"sola",
				"solan",
				"solana",
				"solana:",
				// Not a Base58 string
				"solana::",
				"solana:›",
				"solana:😁",
				"solana:random-string",
				// Valid base58 text, too long (34 chars)
				"solana:TJ24pxm996UCBScuQRwjYo4wvPjUa8pzKo",
			];
			for chain in invalid_chains {
				assert!(
					ChainId::<Test>::try_from(chain.as_bytes().to_vec()).is_err(),
					"Chain ID {:?} should fail to parse for generic chains",
					chain
				);
			}
		}

		#[test]
		fn test_generic_chains() {
			let valid_chains = [
				// Edge cases
				"abc:-",
				"-as01-aa:A",
				"12345678:abcdefghjklmnopqrstuvwxyzABCD012",
				// Filecoin examples -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-23.md
				"fil:t",
				"fil:f",
				// Tezos examples -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-26.md
				"tezos:NetXdQprcVkpaWU",
				"tezos:NetXm8tYqnMWky1",
				// Cosmos examples -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-5.md
				"cosmos:cosmoshub-2",
				"cosmos:cosmoshub-3",
				"cosmos:Binance-Chain-Tigris",
				"cosmos:iov-mainnet",
				"cosmos:x",
				"cosmos:hash-",
				"cosmos:hashed",
				// Lisk examples -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-6.md
				"lip9:9ee11e9df416b18b",
				"lip9:e48feb88db5b5cf5",
				// EOSIO examples -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-7.md
				"eosio:aca376f206b8fc25a6ed44dbdc66547c",
				"eosio:e70aaab8997e1dfce58fbfac80cbbb8f",
				"eosio:4667b205c6838ef70ff7988f6e8257e8",
				"eosio:1eaa0824707c8c16bd25145493bf062a",
				// Stellar examples -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-28.md
				"stellar:testnet",
				"stellar:pubnet",
			];
			for chain in valid_chains {
				println!("Testing right chain {:?}", chain);
				assert!(
					ChainId::<Test>::try_from(chain.as_bytes().to_vec()).is_ok(),
					"Chain ID {:?} should not fail to parse for generic chains",
					chain
				);
			}

			let invalid_chains = [
				// Too short
				"a",
				"ab",
				"01:",
				"ab-:",
				// Too long
				"123456789:1",
				"12345678:123456789123456789123456789123456",
				"123456789:123456789123456789123456789123456",
				// Unallowed characters
				"::",
				"c?1:›",
				"de:😁",
			];
			for chain in invalid_chains {
				println!("Testing wrong chain {:?}", chain);
				assert!(
					ChainId::<Test>::try_from(chain.as_bytes().to_vec()).is_err(),
					"Chain ID {:?} should fail to parse for solana chains",
					chain
				);
			}
		}

		#[test]
		fn test_utility_functions() {
			// These functions should never crash. We just check that here.
			ChainId::<Test>::ethereum_mainnet();
			ChainId::<Test>::moonbeam_eth();
			ChainId::<Test>::bitcoin_mainnet();
			ChainId::<Test>::polkadot();
			ChainId::<Test>::kusama();
			ChainId::<Test>::kilt_spiritnet();
			ChainId::<Test>::solana_mainnet();
		}
	}
}
