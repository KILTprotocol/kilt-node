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

use base58::FromBase58;
use core::str;

use frame_support::{sp_runtime::traits::CheckedConversion, traits::ConstU32, BoundedVec};

const MINIMUM_NAMESPACE_LENGTH: u32 = 3;
const MAXIMUM_NAMESPACE_LENGTH: u32 = 8;
const MINIMUM_REFERENCE_LENGTH: u32 = 1;
const MAXIMUM_REFERENCE_LENGTH: u32 = 32;

pub enum ChainId {
	Eip155(Eip155Reference),
	Bip122(GenesisHexHashReference),
	Dotsama(GenesisHexHashReference),
	Solana(GenesisBase58HashReference),
	Generic(GenericChainId),
}

pub enum ChainIdError {
	Namespace(NamespaceError),
	Reference(ReferenceError),
	InvalidFormat,
}

pub enum NamespaceError {
	TooLong,
	TooShort,
	InvalidCharacter,
}

pub enum ReferenceError {
	TooLong,
	TooShort,
	InvalidCharacter,
}

impl TryFrom<&[u8]> for ChainId {
	type Error = ChainIdError;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		match value {
			// "eip155:" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-3.md
			[b'e', b'i', b'p', b'1', b'5', b'5', b':', chain_id @ ..] => {
				Eip155Reference::try_from(chain_id).map(Self::Eip155)
			}
			// "bip122:" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-4.md
			[b'b', b'i', b'p', b'1', b'2', b'2', b':', chain_id @ ..] => {
				GenesisHexHashReference::try_from(chain_id).map(Self::Bip122)
			}
			// "polkadot:" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-13.md
			[b'p', b'o', b'l', b'k', b'a', b'd', b'o', b't', b':', chain_id @ ..] => {
				GenesisHexHashReference::try_from(chain_id).map(Self::Dotsama)
			}
			// "solana:" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-30.md
			[b's', b'o', b'l', b'a', b'n', b'a', b':', chain_id @ ..] => {
				GenesisBase58HashReference::try_from(chain_id).map(Self::Solana)
			}
			// Other chains that are still compatible with the CAIP-2 spec -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md
			chain_id => GenericChainId::try_from(chain_id).map(Self::Generic),
		}
	}
}

impl ChainId {
	pub fn ethereum_mainnet() -> Self {
		Self::Eip155(Eip155Reference::from_slice_unchecked(b"1"))
	}

	pub fn moonriver_eth() -> Self {
		// Info taken from https://chainlist.org/
		Self::Eip155(Eip155Reference::from_slice_unchecked(b"1285"))
	}

	pub fn moonbeam_eth() -> Self {
		// Info taken from https://chainlist.org/
		Self::Eip155(Eip155Reference::from_slice_unchecked(b"1284"))
	}

	pub fn bitcoin_mainnet() -> Self {
		Self::Bip122(GenesisHexHashReference::from_slice_unchecked(
			b"000000000019d6689c085ae165831e93",
		))
	}

	pub fn polkadot() -> Self {
		Self::Dotsama(GenesisHexHashReference::from_slice_unchecked(
			b"91b171bb158e2d3848fa23a9f1c25182",
		))
	}

	pub fn kusama() -> Self {
		Self::Dotsama(GenesisHexHashReference::from_slice_unchecked(
			b"b0a8d493285c2df73290dfb7e61f870f",
		))
	}

	pub fn kilt_spiritnet() -> Self {
		Self::Dotsama(GenesisHexHashReference::from_slice_unchecked(
			b"411f057b9107718c9624d6aa4a3f23c1",
		))
	}

	pub fn solana_mainnet() -> Self {
		Self::Solana(GenesisBase58HashReference::from_slice_unchecked(
			b"4sGjMW1sUnHzSxGspuhpqLDx6wiyjNtZ",
		))
	}
}

pub struct Eip155Reference(BoundedVec<u8, ConstU32<MAXIMUM_REFERENCE_LENGTH>>);

impl Eip155Reference {
	#[allow(dead_code)]
	pub(crate) fn from_slice_unchecked(slice: &[u8]) -> Self {
		Self(slice.to_vec().try_into().unwrap())
	}
}

impl TryFrom<&[u8]> for Eip155Reference {
	type Error = ChainIdError;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let input_length = value
			.len()
			.checked_into::<u32>()
			.ok_or(ChainIdError::Reference(ReferenceError::TooLong))?;
		if input_length < MINIMUM_REFERENCE_LENGTH {
			Err(ChainIdError::Reference(ReferenceError::TooShort))
		} else if input_length > MAXIMUM_REFERENCE_LENGTH {
			Err(ChainIdError::Reference(ReferenceError::TooLong))
		} else {
			value.iter().try_for_each(|c| {
				if !(b'0'..=b'9').contains(c) {
					Err(ChainIdError::Reference(ReferenceError::InvalidCharacter))
				} else {
					Ok(())
				}
			})?;
			// Unchecked since we already checked for length
			Ok(Self::from_slice_unchecked(value))
		}
	}
}

pub struct GenesisHexHashReference(BoundedVec<u8, ConstU32<MAXIMUM_REFERENCE_LENGTH>>);

impl GenesisHexHashReference {
	#[allow(dead_code)]
	pub(crate) fn from_slice_unchecked(slice: &[u8]) -> Self {
		Self(slice.to_vec().try_into().unwrap())
	}
}

impl TryFrom<&[u8]> for GenesisHexHashReference {
	type Error = ChainIdError;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let input_length = value
			.len()
			.checked_into::<u32>()
			.ok_or(ChainIdError::Reference(ReferenceError::TooLong))?;
		if input_length < MINIMUM_REFERENCE_LENGTH {
			Err(ChainIdError::Reference(ReferenceError::TooShort))
		} else if input_length > MAXIMUM_REFERENCE_LENGTH {
			Err(ChainIdError::Reference(ReferenceError::TooLong))
		} else if input_length % 2 != 0 {
			// Hex encoding can only have 2x characters
			Err(ChainIdError::InvalidFormat)
		} else {
			value.iter().try_for_each(|c| {
				if !matches!(c, b'0'..=b'9' | b'a'..=b'f') {
					Err(ChainIdError::Reference(ReferenceError::InvalidCharacter))
				} else {
					Ok(())
				}
			})?;
			// Unchecked since we already checked for length
			Ok(Self::from_slice_unchecked(value))
		}
	}
}

pub struct GenesisBase58HashReference(BoundedVec<u8, ConstU32<MAXIMUM_REFERENCE_LENGTH>>);

impl GenesisBase58HashReference {
	#[allow(dead_code)]
	pub(crate) fn from_slice_unchecked(slice: &[u8]) -> Self {
		Self(slice.to_vec().try_into().unwrap())
	}
}

impl TryFrom<&[u8]> for GenesisBase58HashReference {
	type Error = ChainIdError;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let input_length = value
			.len()
			.checked_into::<u32>()
			.ok_or(ChainIdError::Reference(ReferenceError::TooLong))?;
		if input_length < MINIMUM_REFERENCE_LENGTH {
			Err(ChainIdError::Reference(ReferenceError::TooShort))
		} else if input_length > MAXIMUM_REFERENCE_LENGTH {
			Err(ChainIdError::Reference(ReferenceError::TooLong))
		} else {
			let decoded_string =
				str::from_utf8(value).map_err(|_| ChainIdError::Reference(ReferenceError::InvalidCharacter))?;
			// Check for proper base58 encoding
			decoded_string
				.from_base58()
				.map_err(|_| ChainIdError::Reference(ReferenceError::InvalidCharacter))?;
			// Unchecked since we already checked for length
			Ok(Self::from_slice_unchecked(value))
		}
	}
}

pub struct GenericChainId {
	pub namespace: ChainNamespace,
	pub reference: ChainReference,
}

impl GenericChainId {
	#[allow(dead_code)]
	fn from_raw_unchecked(namespace: &[u8], reference: &[u8]) -> Self {
		Self {
			namespace: ChainNamespace::from_slice_unchecked(namespace),
			reference: ChainReference::from_slice_unchecked(reference),
		}
	}
}

impl TryFrom<&[u8]> for GenericChainId {
	type Error = ChainIdError;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let input_length = value.len().checked_into::<u32>().ok_or(ChainIdError::InvalidFormat)?;
		if input_length > MINIMUM_NAMESPACE_LENGTH + MAXIMUM_NAMESPACE_LENGTH + 1 {
			return Err(ChainIdError::InvalidFormat);
		}

		let mut components = value.split(|c| *c == b':');

		let namespace = components
			.next()
			.ok_or(ChainIdError::InvalidFormat)
			.and_then(ChainNamespace::try_from)?;
		let reference = components
			.next()
			.ok_or(ChainIdError::InvalidFormat)
			.and_then(ChainReference::try_from)?;

		Ok(Self { namespace, reference })
	}
}

pub struct ChainNamespace(BoundedVec<u8, ConstU32<MAXIMUM_NAMESPACE_LENGTH>>);

impl ChainNamespace {
	fn from_slice_unchecked(value: &[u8]) -> Self {
		Self(value.to_vec().try_into().unwrap())
	}
}

impl TryFrom<&[u8]> for ChainNamespace {
	type Error = ChainIdError;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let input_length = value
			.len()
			.checked_into::<u32>()
			.ok_or(ChainIdError::Namespace(NamespaceError::TooLong))?;
		if input_length < MINIMUM_NAMESPACE_LENGTH {
			Err(ChainIdError::Namespace(NamespaceError::TooShort))
		} else if input_length > MAXIMUM_NAMESPACE_LENGTH {
			Err(ChainIdError::Namespace(NamespaceError::TooLong))
		} else {
			value.iter().try_for_each(|c| {
				if !matches!(c, b'-' | b'a'..=b'z' | b'0'..=b'9') {
					Err(ChainIdError::Namespace(NamespaceError::InvalidCharacter))
				} else {
					Ok(())
				}
			})?;
			// Unchecked since we already checked for length
			Ok(Self::from_slice_unchecked(value))
		}
	}
}

pub struct ChainReference(BoundedVec<u8, ConstU32<MAXIMUM_REFERENCE_LENGTH>>);

impl ChainReference {
	fn from_slice_unchecked(value: &[u8]) -> Self {
		Self(value.to_vec().try_into().unwrap())
	}
}

impl TryFrom<&[u8]> for ChainReference {
	type Error = ChainIdError;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let input_length = value
			.len()
			.checked_into::<u32>()
			.ok_or(ChainIdError::Reference(ReferenceError::TooLong))?;
		if input_length < MINIMUM_REFERENCE_LENGTH {
			Err(ChainIdError::Reference(ReferenceError::TooShort))
		} else if input_length > MAXIMUM_REFERENCE_LENGTH {
			Err(ChainIdError::Reference(ReferenceError::TooLong))
		} else {
			value.iter().try_for_each(|c| {
				if !matches!(c, b'-' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9') {
					Err(ChainIdError::Reference(ReferenceError::InvalidCharacter))
				} else {
					Ok(())
				}
			})?;
			// Unchecked since we already checked for length
			Ok(Self::from_slice_unchecked(value))
		}
	}
}
