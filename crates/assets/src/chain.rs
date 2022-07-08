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

	pub const MINIMUM_CHAIN_ID_LENGTH: usize = MINIMUM_NAMESPACE_LENGTH + b":".len() + MINIMUM_REFERENCE_LENGTH;
	pub const MAXIMUM_CHAIN_ID_LENGTH: usize = MAXIMUM_NAMESPACE_LENGTH + b":".len() + MAXIMUM_REFERENCE_LENGTH;

	pub const MINIMUM_NAMESPACE_LENGTH: usize = 3;
	pub const MAXIMUM_NAMESPACE_LENGTH: usize = 8;
	const MAXIMUM_NAMESPACE_LENGTH_U32: u32 = MAXIMUM_NAMESPACE_LENGTH as u32;
	pub const MINIMUM_REFERENCE_LENGTH: usize = 1;
	pub const MAXIMUM_REFERENCE_LENGTH: usize = 32;
	const MAXIMUM_REFERENCE_LENGTH_U32: u32 = MAXIMUM_REFERENCE_LENGTH as u32;

	// TODO: Add link to the Asset DID spec once merged.

	/// The Chain ID component as specified in the Asset DID specification.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub enum ChainId {
		// An EIP155 chain reference.
		Eip155(Eip155Reference),
		// A BIP122 chain reference.
		Bip122(GenesisHexHashReference<MAXIMUM_REFERENCE_LENGTH>),
		// A Dotsama chain reference.
		Dotsama(GenesisHexHashReference<MAXIMUM_REFERENCE_LENGTH>),
		// A Solana chain reference.
		Solana(GenesisBase58HashReference<MAXIMUM_REFERENCE_LENGTH>),
		// A generic chain.
		Generic(GenericChainId),
	}

	impl From<Eip155Reference> for ChainId {
		fn from(reference: Eip155Reference) -> Self {
			Self::Eip155(reference)
		}
	}

	impl From<GenericChainId> for ChainId {
		fn from(chain_id: GenericChainId) -> Self {
			Self::Generic(chain_id)
		}
	}

	impl TryFrom<&[u8]> for ChainId {
		type Error = ChainIdError;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			match value {
				// "eip155:" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-3.md
				[b'e', b'i', b'p', b'1', b'5', b'5', b':', chain_reference @ ..] => {
					Eip155Reference::try_from(chain_reference).map(Self::Eip155)
				}
				// "bip122:" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-4.md
				[b'b', b'i', b'p', b'1', b'2', b'2', b':', chain_reference @ ..] => {
					GenesisHexHashReference::try_from(chain_reference).map(Self::Bip122)
				}
				// "polkadot:" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-13.md
				[b'p', b'o', b'l', b'k', b'a', b'd', b'o', b't', b':', chain_reference @ ..] => {
					GenesisHexHashReference::try_from(chain_reference).map(Self::Dotsama)
				}
				// "solana:" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-30.md
				[b's', b'o', b'l', b'a', b'n', b'a', b':', chain_reference @ ..] => {
					GenesisBase58HashReference::try_from(chain_reference).map(Self::Solana)
				}
				// Other chains that are still compatible with the CAIP-2 spec -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md
				chain_id => GenericChainId::try_from(chain_id).map(Self::Generic),
			}
		}
	}

	impl TryFrom<Vec<u8>> for ChainId {
		type Error = ChainIdError;

		fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
			Self::try_from(&value[..])
		}
	}

	impl TryFrom<&'static str> for ChainId {
		type Error = ChainIdError;

		fn try_from(value: &'static str) -> Result<Self, Self::Error> {
			Self::try_from(value.as_bytes())
		}
	}

	#[cfg(feature = "std")]
	impl TryFrom<String> for ChainId {
		type Error = ChainIdError;

		fn try_from(value: String) -> Result<Self, Self::Error> {
			Self::try_from(value.as_bytes())
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
			Self::Bip122(GenesisHexHashReference::bitcoin_mainnet())
		}

		/// The chain ID for the Polkadot relaychain.
		pub fn polkadot() -> Self {
			Self::Dotsama(GenesisHexHashReference::polkadot())
		}

		/// The chain ID for the Kusama relaychain.
		pub fn kusama() -> Self {
			Self::Dotsama(GenesisHexHashReference::kusama())
		}

		/// The chain ID for the KILT Spiritnet parachain.
		pub fn kilt_spiritnet() -> Self {
			Self::Dotsama(GenesisHexHashReference::kilt_spiritnet())
		}

		/// The chain ID for the Solana mainnet.
		pub fn solana_mainnet() -> Self {
			Self::Solana(GenesisBase58HashReference::solana_mainnet())
		}
	}

	/// An EIP155 chain reference.
	/// It is a modification of the [CAIP-3 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-3.md)
	/// according to the rules defined in the Asset DID method specification.
	#[non_exhaustive]
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct Eip155Reference(pub BoundedVec<u8, ConstU32<MAXIMUM_REFERENCE_LENGTH_U32>>);

	impl Eip155Reference {
		/// [CAN PANIC]
		/// Tries to create an Eip155Reference reference from the provided
		/// slice, panicking if the slice is longer than the maximum length
		/// allowed.
		pub(crate) fn from_slice_unchecked(slice: &[u8]) -> Self {
			Self(slice.to_vec().try_into().expect("Eip155Reference::from_slice_unchecked should never panic."))
		}

		/// The EIP155 reference for the Ethereum mainnet.
		pub fn ethereum_mainnet() -> Self {
			Self::from_slice_unchecked(b"1")
		}

		/// The EIP155 reference for the Moonriver parachain.
		pub fn moonriver_eth() -> Self {
			// Info taken from https://chainlist.org/
			Self::from_slice_unchecked(b"1285")
		}

		/// The EIP155 reference for the Moonbeam parachain.
		pub fn moonbeam_eth() -> Self {
			// Info taken from https://chainlist.org/
			Self::from_slice_unchecked(b"1284")
		}
	}

	impl TryFrom<&[u8]> for Eip155Reference {
		type Error = ChainIdError;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			let input_length = value.len();
			if input_length < MINIMUM_REFERENCE_LENGTH {
				Err(ReferenceError::TooShort.into())
			} else if input_length > MAXIMUM_REFERENCE_LENGTH {
				Err(ReferenceError::TooLong.into())
			} else {
				value.iter().try_for_each(|c| {
					if !(b'0'..=b'9').contains(c) {
						Err(ReferenceError::InvalidFormat)
					} else {
						Ok(())
					}
				})?;
				// Unchecked since we already checked for length
				Ok(Self::from_slice_unchecked(value))
			}
		}
	}

	// TODO: Add support for compilation-time checks on the value of L when
	// supported.
	/// A chain reference for CAIP-2 chains that are identified by a HEX genesis
	/// hash.
	#[non_exhaustive]
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct GenesisHexHashReference<const L: usize = MAXIMUM_REFERENCE_LENGTH>(pub [u8; L]);

	impl<const L: usize> GenesisHexHashReference<L> {
		/// [CAN PANIC]
		/// Tries to create a GenesisHexHashReference reference from the
		/// provided slice, panicking if the slice is longer than the maximum
		/// length allowed.
		pub(crate) fn from_slice_unchecked(slice: &[u8]) -> Self {
			Self(slice.try_into().expect("GenesisHexHashReference::from_slice_unchecked should never panic."))
		}

		/// The CAIP-2 reference for the Bitcoin mainnet.
		pub fn bitcoin_mainnet() -> Self {
			Self::from_slice_unchecked(b"000000000019d6689c085ae165831e93")
		}

		/// The CAIP-2 reference for the Polkadot relaychain.
		pub fn polkadot() -> Self {
			Self::from_slice_unchecked(b"91b171bb158e2d3848fa23a9f1c25182")
		}

		/// The CAIP-2 reference for the Kusama relaychain.
		pub fn kusama() -> Self {
			Self::from_slice_unchecked(b"b0a8d493285c2df73290dfb7e61f870f")
		}

		/// The CAIP-2 reference for the KILT Spiritnet parachain.
		pub fn kilt_spiritnet() -> Self {
			Self::from_slice_unchecked(b"411f057b9107718c9624d6aa4a3f23c1")
		}
	}

	impl<const L: usize> TryFrom<&[u8]> for GenesisHexHashReference<L> {
		type Error = ChainIdError;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			let input_length = value.len();
			if input_length < MINIMUM_REFERENCE_LENGTH {
				Err(ReferenceError::TooShort.into())
			} else if input_length > MAXIMUM_REFERENCE_LENGTH {
				Err(ReferenceError::TooLong.into())
			} else if input_length % 2 != 0 {
				// Hex encoding can only have 2x characters
				Err(ReferenceError::InvalidFormat.into())
			} else {
				value.iter().try_for_each(|c| {
					if !matches!(c, b'0'..=b'9' | b'a'..=b'f') {
						Err(ReferenceError::InvalidFormat)
					} else {
						Ok(())
					}
				})?;
				value.try_into().map(Self).map_err(|_| ChainIdError::InvalidFormat)
			}
		}
	}

	// FIXME: Ensure that a size is given for the expected hash length (less than
	// the max allowed size).

	/// A chain reference for CAIP-2 chains that are identified by a
	/// Base58-encoded genesis hash.
	#[non_exhaustive]
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct GenesisBase58HashReference<const L: usize = MAXIMUM_REFERENCE_LENGTH>(pub [u8; L]);

	impl<const L: usize> GenesisBase58HashReference<L> {
		/// [CAN PANIC]
		/// Tries to create a GenesisBase58HashReference reference from the
		/// provided slice, panicking if the slice is longer than the maximum
		/// length allowed.
		pub(crate) fn from_slice_unchecked(slice: &[u8]) -> Self {
			Self(slice.to_vec().try_into().expect("GenesisBase58HashReference::from_slice_unchecked should never panic."))
		}

		/// The CAIP-2 reference for the Solana mainnet.
		pub fn solana_mainnet() -> Self {
			Self::from_slice_unchecked(b"4sGjMW1sUnHzSxGspuhpqLDx6wiyjNtZ")
		}
	}

	impl<const L: usize> TryFrom<&[u8]> for GenesisBase58HashReference<L> {
		type Error = ChainIdError;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			let input_length = value.len();
			if input_length < MINIMUM_REFERENCE_LENGTH {
				Err(ReferenceError::TooShort.into())
			} else if input_length > MAXIMUM_REFERENCE_LENGTH {
				Err(ReferenceError::TooLong.into())
			} else {
				let decoded_string = str::from_utf8(value).map_err(|_| ReferenceError::InvalidFormat)?;
				// Check for proper base58 encoding
				decoded_string
					.from_base58()
					.map_err(|_| ReferenceError::InvalidFormat)?;

				value.try_into().map(Self).map_err(|_| ChainIdError::InvalidFormat)
			}
		}
	}

	/// A generic chain ID compliant with the [CAIP-2 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md) that cannot be boxed in any of the supported variants.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct GenericChainId {
		pub namespace: GenericChainNamespace,
		pub reference: GenericChainReference,
	}

	impl GenericChainId {
		/// [CAN PANIC]
		/// Tries to create a GenericChainId identifier from the provided raw
		/// components, panicking if the any of them is longer than the maximum
		/// length allowed.\
		#[allow(dead_code)]
		fn from_raw_unchecked(namespace: &[u8], reference: &[u8]) -> Self {
			Self {
				namespace: GenericChainNamespace::from_slice_unchecked(namespace),
				reference: GenericChainReference::from_slice_unchecked(reference),
			}
		}
	}

	impl TryFrom<&[u8]> for GenericChainId {
		type Error = ChainIdError;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			let input_length = value.len();
			if input_length > MAXIMUM_CHAIN_ID_LENGTH {
				return Err(ChainIdError::InvalidFormat);
			}

			let mut components = value.split(|c| *c == b':');

			let namespace = components
				.next()
				.ok_or(ChainIdError::InvalidFormat)
				.and_then(GenericChainNamespace::try_from)?;
			let reference = components
				.next()
				.ok_or(ChainIdError::InvalidFormat)
				.and_then(GenericChainReference::try_from)?;

			Ok(Self { namespace, reference })
		}
	}

	/// A generic chain namespace as defined in the [CAIP-2 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md).
	#[non_exhaustive]
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct GenericChainNamespace(pub BoundedVec<u8, ConstU32<MAXIMUM_NAMESPACE_LENGTH_U32>>);

	impl GenericChainNamespace {
		/// [CAN PANIC]
		/// Tries to create a GenericChainNamespace namespace from the provided
		/// slice, panicking if the slice is longer than the maximum length
		/// allowed.
		fn from_slice_unchecked(value: &[u8]) -> Self {
			Self(value.to_vec().try_into().expect("GenericChainNamespace::from_slice_unchecked should never panic."))
		}
	}

	impl TryFrom<&[u8]> for GenericChainNamespace {
		type Error = ChainIdError;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			let input_length = value.len();
			if input_length < MINIMUM_NAMESPACE_LENGTH {
				Err(NamespaceError::TooShort.into())
			} else if input_length > MAXIMUM_NAMESPACE_LENGTH {
				Err(NamespaceError::TooLong.into())
			} else {
				value.iter().try_for_each(|c| {
					if !matches!(c, b'-' | b'a'..=b'z' | b'0'..=b'9') {
						Err(NamespaceError::InvalidFormat)
					} else {
						Ok(())
					}
				})?;
				// Unchecked since we already checked for length
				Ok(Self::from_slice_unchecked(value))
			}
		}
	}

	/// A generic chain reference as defined in the [CAIP-2 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md).
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct GenericChainReference(BoundedVec<u8, ConstU32<MAXIMUM_REFERENCE_LENGTH_U32>>);

	impl GenericChainReference {
		/// [CAN PANIC]
		/// Tries to create a GenericChainReference reference from the provided
		/// slice, panicking if the slice is longer than the maximum length
		/// allowed.
		fn from_slice_unchecked(value: &[u8]) -> Self {
			Self(value.to_vec().try_into().expect("GenericChainReference::from_slice_unchecked should never panic."))
		}
	}

	impl TryFrom<&[u8]> for GenericChainReference {
		type Error = ChainIdError;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			let input_length = value.len();
			if input_length < MINIMUM_REFERENCE_LENGTH {
				Err(ReferenceError::TooShort.into())
			} else if input_length > MAXIMUM_REFERENCE_LENGTH {
				Err(ReferenceError::TooLong.into())
			} else {
				value.iter().try_for_each(|c| {
					if !matches!(c, b'-' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9') {
						Err(ReferenceError::InvalidFormat)
					} else {
						Ok(())
					}
				})?;
				// Unchecked since we already checked for length
				Ok(Self::from_slice_unchecked(value))
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
					ChainId::try_from(chain.as_bytes()).is_ok(),
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
					ChainId::try_from(chain.as_bytes()).is_err(),
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
					ChainId::try_from(chain.as_bytes()).is_ok(),
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
					ChainId::try_from(chain.as_bytes()).is_err(),
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
					ChainId::try_from(chain.as_bytes()).is_ok(),
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
					ChainId::try_from(chain.as_bytes()).is_err(),
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
					ChainId::try_from(chain.as_bytes()).is_ok(),
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
					ChainId::try_from(chain.as_bytes()).is_err(),
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
					ChainId::try_from(chain.as_bytes()).is_ok(),
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
					ChainId::try_from(chain.as_bytes()).is_err(),
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
			ChainId::polkadot();
			ChainId::kusama();
			ChainId::kilt_spiritnet();
			ChainId::solana_mainnet();
		}
	}
}
