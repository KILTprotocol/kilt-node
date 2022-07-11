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

use frame_support::sp_runtime::RuntimeDebug;

/// An error in the chain ID parsing logic.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug)]
pub enum ChainIdError {
	/// An error in the chain namespace parsing logic.
	Namespace(NamespaceError),
	/// An error in the chain reference parsing logic.
	Reference(ReferenceError),
	/// A generic error not belonging to any of the other categories.
	InvalidFormat,
}

/// An error in the chain namespace parsing logic.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug)]
pub enum NamespaceError {
	/// Namespace too long.
	TooLong,
	/// Namespace too short.
	TooShort,
	/// A generic error not belonging to any of the other categories.
	InvalidFormat,
}

/// An error in the chain reference parsing logic.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug)]
pub enum ReferenceError {
	/// Reference too long.
	TooLong,
	/// Reference too short.
	TooShort,
	/// A generic error not belonging to any of the other categories.
	InvalidFormat,
}

impl From<NamespaceError> for ChainIdError {
	fn from(err: NamespaceError) -> Self {
		Self::Namespace(err)
	}
}

impl From<ReferenceError> for ChainIdError {
	fn from(err: ReferenceError) -> Self {
		Self::Reference(err)
	}
}

// Exported types. This will always only re-export the latest version by
// default.
pub use v1::*;

mod v1 {
	use super::{ChainIdError, NamespaceError, ReferenceError};

	use base58::FromBase58;

	use core::str;

	use codec::{Decode, Encode, MaxEncodedLen};
	use scale_info::TypeInfo;

	use frame_support::{sp_runtime::RuntimeDebug, traits::ConstU32, BoundedVec};
	use sp_std::vec::Vec;

	/// The minimum length, including separator symbols, a chain ID can have
	/// according to the minimum values defined by the CAIP-2 definition.
	pub const MINIMUM_CHAIN_ID_LENGTH: usize = MINIMUM_NAMESPACE_LENGTH + b":".len() + MINIMUM_REFERENCE_LENGTH;
	/// The maximum length, including separator symbols, a chain ID can have
	/// according to the minimum values defined by the CAIP-2 definition.
	pub const MAXIMUM_CHAIN_ID_LENGTH: usize = MAXIMUM_NAMESPACE_LENGTH + b":".len() + MAXIMUM_REFERENCE_LENGTH;

	/// The minimum length of a valid chain ID namespace.
	pub const MINIMUM_NAMESPACE_LENGTH: usize = 3;
	/// The maximum length of a valid chain ID namespace.
	pub const MAXIMUM_NAMESPACE_LENGTH: usize = 8;
	const MAXIMUM_NAMESPACE_LENGTH_U32: u32 = MAXIMUM_NAMESPACE_LENGTH as u32;
	/// The minimum length of a valid chain ID reference.
	pub const MINIMUM_REFERENCE_LENGTH: usize = 1;
	/// The maximum length of a valid chain ID reference.
	pub const MAXIMUM_REFERENCE_LENGTH: usize = 32;
	const MAXIMUM_REFERENCE_LENGTH_U32: u32 = MAXIMUM_REFERENCE_LENGTH as u32;

	// TODO: Add link to the Asset DID spec once merged.

	/// The Chain ID component as specified in the Asset DID specification.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub enum ChainId {
		// An EIP155 chain reference.
		Eip155(Eip155Reference),
		// A BIP122 chain reference.
		Bip122(GenesisHexHash32Reference),
		// A Dotsama chain reference.
		Dotsama(GenesisHexHash32Reference),
		// A Solana chain reference.
		Solana(GenesisBase58Hash32Reference),
		// A generic chain.
		Generic(GenericChainId),
	}

	impl From<Eip155Reference> for ChainId {
		fn from(reference: Eip155Reference) -> Self {
			Self::Eip155(reference)
		}
	}

	impl From<GenesisBase58Hash32Reference> for ChainId {
		fn from(reference: GenesisBase58Hash32Reference) -> Self {
			Self::Solana(reference)
		}
	}

	impl From<GenericChainId> for ChainId {
		fn from(chain_id: GenericChainId) -> Self {
			Self::Generic(chain_id)
		}
	}

	impl ChainId {
		/// The chain ID for the Ethereum mainnet.
		pub fn ethereum_mainnet() -> Self {
			Eip155Reference::ethereum_mainnet().into()
		}

		/// The chain ID for the Moonriver EVM parachain.
		pub fn moonriver_eth() -> Self {
			// Info taken from https://chainlist.org/
			Eip155Reference::moonriver_eth().into()
		}

		/// The chain ID for the Moonbeam EVM parachain.
		pub fn moonbeam_eth() -> Self {
			// Info taken from https://chainlist.org/
			Eip155Reference::moonbeam_eth().into()
		}

		/// The chain ID for the Bitcoin mainnet.
		pub fn bitcoin_mainnet() -> Self {
			Self::Bip122(GenesisHexHash32Reference::bitcoin_mainnet())
		}

		/// The chain ID for the Litecoin mainnet.
		pub fn litecoin_mainnet() -> Self {
			Self::Bip122(GenesisHexHash32Reference::litecoin_mainnet())
		}

		/// The chain ID for the Polkadot relaychain.
		pub fn polkadot() -> Self {
			Self::Dotsama(GenesisHexHash32Reference::polkadot())
		}

		/// The chain ID for the Kusama relaychain.
		pub fn kusama() -> Self {
			Self::Dotsama(GenesisHexHash32Reference::kusama())
		}

		/// The chain ID for the KILT Spiritnet parachain.
		pub fn kilt_spiritnet() -> Self {
			Self::Dotsama(GenesisHexHash32Reference::kilt_spiritnet())
		}

		/// The chain ID for the Solana mainnet.
		pub fn solana_mainnet() -> Self {
			GenesisBase58Hash32Reference::solana_mainnet().into()
		}
	}

	impl ChainId {
		/// Try to parse a `ChainId` instance from the provided UTF8-encoded
		/// input.
		pub fn from_utf8_encoded<I>(input: I) -> Result<Self, ChainIdError>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			match input.as_ref() {
				// "eip155:" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-3.md
				[b'e', b'i', b'p', b'1', b'5', b'5', b':', chain_reference @ ..] => {
					Eip155Reference::from_utf8_encoded(chain_reference).map(Self::Eip155)
				}
				// "bip122:" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-4.md
				[b'b', b'i', b'p', b'1', b'2', b'2', b':', chain_reference @ ..] => {
					GenesisHexHash32Reference::from_utf8_encoded(chain_reference).map(Self::Bip122)
				}
				// "polkadot:" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-13.md
				[b'p', b'o', b'l', b'k', b'a', b'd', b'o', b't', b':', chain_reference @ ..] => {
					GenesisHexHash32Reference::from_utf8_encoded(chain_reference).map(Self::Dotsama)
				}
				// "solana:" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-30.md
				[b's', b'o', b'l', b'a', b'n', b'a', b':', chain_reference @ ..] => {
					GenesisBase58Hash32Reference::from_utf8_encoded(chain_reference).map(Self::Solana)
				}
				// Other chains that are still compatible with the CAIP-2 spec -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md
				chain_id => GenericChainId::from_utf8_encoded(chain_id).map(Self::Generic),
			}
		}
	}

	/// An EIP155 chain reference.
	/// It is a modification of the [CAIP-3 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-3.md)
	/// according to the rules defined in the Asset DID method specification.
	#[non_exhaustive]
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct Eip155Reference(pub u128);

	impl Eip155Reference {
		/// The EIP155 reference for the Ethereum mainnet.
		pub const fn ethereum_mainnet() -> Self {
			Self(1)
		}

		/// The EIP155 reference for the Moonriver parachain.
		pub const fn moonriver_eth() -> Self {
			// Info taken from https://chainlist.org/
			Self(1285)
		}

		/// The EIP155 reference for the Moonbeam parachain.
		pub const fn moonbeam_eth() -> Self {
			// Info taken from https://chainlist.org/
			Self(1284)
		}
	}

	impl Eip155Reference {
		/// Parse a UTF8-encoded decimal chain reference, failing if the input
		/// string is not valid.
		pub fn from_utf8_encoded<I>(input: I) -> Result<Self, ChainIdError>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let input = input.as_ref();
			let input_length = input.len();
			if input_length < MINIMUM_REFERENCE_LENGTH {
				Err(ReferenceError::TooShort.into())
			} else if input_length > MAXIMUM_REFERENCE_LENGTH {
				Err(ReferenceError::TooLong.into())
			} else {
				let decoded = str::from_utf8(input).map_err(|_| ReferenceError::InvalidFormat)?;
				let parsed = decoded.parse::<u128>().map_err(|_| ReferenceError::InvalidFormat)?;
				// Unchecked since we already checked for maximum length and hence maximum value
				Ok(Self(parsed))
			}
		}
	}

	impl TryFrom<u128> for Eip155Reference {
		type Error = ChainIdError;

		fn try_from(value: u128) -> Result<Self, Self::Error> {
			// Max value for 32-digit decimal values (used for EIP chains so far).
			// TODO: This could be enforced at compilation time once constraints on generics
			// will be available.
			(value <= 99999999999999999999999999999999)
				.then(|| Self(value))
				.ok_or_else(|| ReferenceError::TooLong.into())
		}
	}

	impl From<u64> for Eip155Reference {
		fn from(value: u64) -> Self {
			Self(value.into())
		}
	}

	/// A chain reference for CAIP-2 chains that are identified by a HEX genesis
	/// hash of 32 characters.
	#[non_exhaustive]
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct GenesisHexHash32Reference(pub [u8; 16]);

	impl GenesisHexHash32Reference {
		/// The CAIP-2 reference for the Bitcoin mainnet.
		pub const fn bitcoin_mainnet() -> Self {
			// HEX decoding of bitcoin genesis hash 0x000000000019d6689c085ae165831e93
			Self([18, 167, 101, 227, 31, 253, 64, 89, 186, 218, 30, 37, 25, 15, 110, 152])
		}

		/// The CAIP-2 reference for the Litecoin mainnet.
		pub const fn litecoin_mainnet() -> Self {
			// HEX decoding of litecoin genesis hash 0x12a765e31ffd4059bada1e25190f6e98
			Self([0, 0, 0, 0, 0, 25, 214, 104, 156, 8, 90, 225, 101, 131, 30, 147])
		}

		/// The CAIP-2 reference for the Polkadot relaychain.
		pub const fn polkadot() -> Self {
			// HEX decoding of Polkadot genesis hash 0x91b171bb158e2d3848fa23a9f1c25182
			Self([145, 177, 113, 187, 21, 142, 45, 56, 72, 250, 35, 169, 241, 194, 81, 130])
		}

		/// The CAIP-2 reference for the Kusama relaychain.
		pub const fn kusama() -> Self {
			// HEX decoding of Kusama genesis hash 0xb0a8d493285c2df73290dfb7e61f870f
			Self([176, 168, 212, 147, 40, 92, 45, 247, 50, 144, 223, 183, 230, 31, 135, 15])
		}

		/// The CAIP-2 reference for the KILT Spiritnet parachain.
		pub const fn kilt_spiritnet() -> Self {
			// HEX decoding of Kusama genesis hash 0x411f057b9107718c9624d6aa4a3f23c1
			Self([65, 31, 5, 123, 145, 7, 113, 140, 150, 36, 214, 170, 74, 63, 35, 193])
		}
	}

	impl GenesisHexHash32Reference {
		/// Parse a UTF8-encoded HEX chain reference, failing if the input
		/// string is not valid.
		pub fn from_utf8_encoded<I>(input: I) -> Result<Self, ChainIdError>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let input = input.as_ref();
			let input_length = input.len();
			if input_length < MINIMUM_REFERENCE_LENGTH {
				Err(ReferenceError::TooShort.into())
			} else if input_length > MAXIMUM_REFERENCE_LENGTH {
				Err(ReferenceError::TooLong.into())
			} else {
				let decoded = hex::decode(input).map_err(|_| ReferenceError::InvalidFormat)?;
				// Unwrap since we already checked for length
				Ok(Self(decoded.try_into().expect(
					"Creation of a generic HEX chain reference should not fail at this point.",
				)))
			}
		}
	}

	/// A chain reference for CAIP-2 chains that are identified by a
	/// Base58-encoded genesis hash of 32 characters.
	#[non_exhaustive]
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct GenesisBase58Hash32Reference(pub BoundedVec<u8, ConstU32<32>>);

	impl GenesisBase58Hash32Reference {
		/// The CAIP-2 reference for the Solana mainnet.
		pub fn solana_mainnet() -> Self {
			// Base58 decoding of Solana genesis hash 4sGjMW1sUnHzSxGspuhpqLDx6wiyjNtZ
			Self(
				vec![
					187, 54, 81, 91, 131, 4, 217, 218, 81, 6, 169, 34, 88, 214, 125, 109, 223, 209, 236, 21, 49, 109,
					82,
				]
				.try_into()
				.expect("Well-known chain ID for solana mainnet should never fail."),
			)
		}
	}

	impl GenesisBase58Hash32Reference {
		/// Parse a UTF8-encoded Base58 chain reference, failing if the input
		/// string is not valid.
		pub fn from_utf8_encoded<I>(input: I) -> Result<Self, ChainIdError>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let input = input.as_ref();
			let input_length = input.len();
			if input_length < MINIMUM_REFERENCE_LENGTH {
				Err(ReferenceError::TooShort.into())
			} else if input_length > MAXIMUM_REFERENCE_LENGTH {
				Err(ReferenceError::TooLong.into())
			} else {
				let decoded_string = str::from_utf8(input).map_err(|_| ReferenceError::InvalidFormat)?;
				let decoded = decoded_string
					.from_base58()
					.map_err(|_| ReferenceError::InvalidFormat)?;
				// Max length in bytes of a 32-character Base58 string is 32 -> it is the string
				// formed by all "1". Otherwise, it is always between 23 and 24 characters.
				// Unwrap since we already checked for length.
				Ok(Self(decoded.try_into().expect(
					"Creation of a generic Base58 chain reference should not fail at this point.",
				)))
			}
		}
	}

	/// A generic chain ID compliant with the [CAIP-2 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md) that cannot be boxed in any of the supported variants.
	#[non_exhaustive]
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct GenericChainId {
		pub namespace: GenericChainNamespace,
		pub reference: GenericChainReference,
	}

	impl GenericChainId {
		/// Parse a generic UTF8-encoded chain ID, failing if the input does not
		/// respect the CAIP-2 requirements.
		pub fn from_utf8_encoded<I>(input: I) -> Result<Self, ChainIdError>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let input = input.as_ref();
			let input_length = input.len();
			if !(MINIMUM_CHAIN_ID_LENGTH..=MAXIMUM_CHAIN_ID_LENGTH).contains(&input_length) {
				return Err(ChainIdError::InvalidFormat);
			}

			let mut components = input.split(|c| *c == b':');

			let namespace = components
				.next()
				.ok_or(ChainIdError::InvalidFormat)
				.and_then(GenericChainNamespace::from_utf8_encoded)?;
			let reference = components
				.next()
				.ok_or(ChainIdError::InvalidFormat)
				.and_then(GenericChainReference::from_utf8_encoded)?;

			Ok(Self { namespace, reference })
		}
	}

	/// A generic chain namespace as defined in the [CAIP-2 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md).
	/// It stores the provided UTF8-encoded namespace without trying to apply
	/// any parsing/decoding logic.
	#[non_exhaustive]
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct GenericChainNamespace(pub BoundedVec<u8, ConstU32<MAXIMUM_NAMESPACE_LENGTH_U32>>);

	impl GenericChainNamespace {
		/// Parse a generic UTF8-encoded chain namespace, failing if the input
		/// does not respect the CAIP-2 requirements.
		pub fn from_utf8_encoded<I>(input: I) -> Result<Self, ChainIdError>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let input = input.as_ref();
			let input_length = input.len();
			if input_length < MINIMUM_NAMESPACE_LENGTH {
				Err(NamespaceError::TooShort.into())
			} else if input_length > MAXIMUM_NAMESPACE_LENGTH {
				Err(NamespaceError::TooLong.into())
			} else {
				input.iter().try_for_each(|c| {
					if !matches!(c, b'-' | b'a'..=b'z' | b'0'..=b'9') {
						Err(NamespaceError::InvalidFormat)
					} else {
						Ok(())
					}
				})?;
				// Unwrap since we already checked for length
				Ok(Self(Vec::<u8>::from(input).try_into().expect(
					"Creation of a generic chain namespace should not fail at this point.",
				)))
			}
		}
	}

	/// A generic chain reference as defined in the [CAIP-2 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md).
	#[non_exhaustive]
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct GenericChainReference(pub BoundedVec<u8, ConstU32<MAXIMUM_REFERENCE_LENGTH_U32>>);

	impl GenericChainReference {
		/// Parse a generic UTF8-encoded chain reference, failing if the input
		/// does not respect the CAIP-2 requirements.
		pub fn from_utf8_encoded<I>(input: I) -> Result<Self, ChainIdError>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let input = input.as_ref();
			let input_length = input.len();
			if input_length < MINIMUM_REFERENCE_LENGTH {
				Err(ReferenceError::TooShort.into())
			} else if input_length > MAXIMUM_REFERENCE_LENGTH {
				Err(ReferenceError::TooLong.into())
			} else {
				input.iter().try_for_each(|c| {
					if !matches!(c, b'-' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9') {
						Err(ReferenceError::InvalidFormat)
					} else {
						Ok(())
					}
				})?;
				// Unchecked since we already checked for length
				Ok(Self(Vec::<u8>::from(input).try_into().expect(
					"Creation of a generic chain reference should not fail at this point.",
				)))
			}
		}
	}

	#[cfg(test)]
	mod test {
		use super::*;

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
					ChainId::from_utf8_encoded(chain.as_bytes()).is_ok(),
					"Chain ID {:?} should not fail to parse for eip155 chains",
					chain
				);
			}

			let invalid_chains = [
				// Too short
				"",
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
					ChainId::from_utf8_encoded(chain.as_bytes()).is_err(),
					"Chain ID {:?} should fail to parse for eip155 chains",
					chain
				);
			}
		}

		#[test]
		fn test_bip122_chains() {
			let valid_chains = [
				"bip122:000000000019d6689c085ae165831e93",
				"bip122:12a765e31ffd4059bada1e25190f6e98",
				"bip122:fdbe99b90c90bae7505796461471d89a",
				"bip122:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
			];
			for chain in valid_chains {
				assert!(
					ChainId::from_utf8_encoded(chain.as_bytes()).is_ok(),
					"Chain ID {:?} should not fail to parse for bip122 chains",
					chain
				);
			}

			let invalid_chains = [
				// Too short
				"",
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
					ChainId::from_utf8_encoded(chain.as_bytes()).is_err(),
					"Chain ID {:?} should fail to parse for bip122 chains",
					chain
				);
			}
		}

		#[test]
		fn test_dotsama_chains() {
			let valid_chains = [
				"polkadot:b0a8d493285c2df73290dfb7e61f870f",
				"polkadot:742a2ca70c2fda6cee4f8df98d64c4c6",
				"polkadot:37e1f8125397a98630013a4dff89b54c",
				"polkadot:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
			];
			for chain in valid_chains {
				assert!(
					ChainId::from_utf8_encoded(chain.as_bytes()).is_ok(),
					"Chain ID {:?} should not fail to parse for polkadot chains",
					chain
				);
			}

			let invalid_chains = [
				// Too short
				"",
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
					ChainId::from_utf8_encoded(chain.as_bytes()).is_err(),
					"Chain ID {:?} should fail to parse for polkadot chains",
					chain
				);
			}
		}

		#[test]
		fn test_solana_chains() {
			let valid_chains = [
				"solana:4sGjMW1sUnHzSxGspuhpqLDx6wiyjNtZ",
				"solana:8E9rvCKLFQia2Y35HXjjpWzj8weVo44K",
			];
			for chain in valid_chains {
				assert!(
					ChainId::from_utf8_encoded(chain.as_bytes()).is_ok(),
					"Chain ID {:?} should not fail to parse for solana chains",
					chain
				);
			}

			let invalid_chains = [
				// Too short
				"",
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
					ChainId::from_utf8_encoded(chain.as_bytes()).is_err(),
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
				assert!(
					ChainId::from_utf8_encoded(chain.as_bytes()).is_ok(),
					"Chain ID {:?} should not fail to parse for generic chains",
					chain
				);
			}

			let invalid_chains = [
				// Too short
				"",
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
				assert!(
					ChainId::from_utf8_encoded(chain.as_bytes()).is_err(),
					"Chain ID {:?} should fail to parse for solana chains",
					chain
				);
			}
		}

		#[test]
		fn test_helpers() {
			// These functions should never panic. We just check that here.
			ChainId::ethereum_mainnet();
			ChainId::moonbeam_eth();
			ChainId::bitcoin_mainnet();
			ChainId::litecoin_mainnet();
			ChainId::polkadot();
			ChainId::kusama();
			ChainId::kilt_spiritnet();
			ChainId::solana_mainnet();
		}
	}
}
