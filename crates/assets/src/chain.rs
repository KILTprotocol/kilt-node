// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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

// Exported types. This will always only re-export the latest version by
// default.
pub use v1::*;

mod v1 {
	use crate::errors::chain::{Error, NamespaceError, ReferenceError};

	use base58::{FromBase58, ToBase58};
	use hex_literal::hex;

	use core::str;

	use codec::{Decode, Encode, MaxEncodedLen};
	use scale_info::TypeInfo;

	use frame_support::{sp_runtime::RuntimeDebug, traits::ConstU32, BoundedVec};
	use sp_std::{fmt::Display, vec, vec::Vec};

	/// The minimum length, including separator symbols, a chain ID can have
	/// according to the minimum values defined by the CAIP-2 definition.
	pub const MINIMUM_CHAIN_ID_LENGTH: usize = MINIMUM_NAMESPACE_LENGTH + 1 + MINIMUM_REFERENCE_LENGTH;
	/// The maximum length, including separator symbols, a chain ID can have
	/// according to the minimum values defined by the CAIP-2 definition.
	pub const MAXIMUM_CHAIN_ID_LENGTH: usize = MAXIMUM_NAMESPACE_LENGTH + 1 + MAXIMUM_REFERENCE_LENGTH;

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

	/// Separator between chain namespace and chain reference.
	const NAMESPACE_REFERENCE_SEPARATOR: u8 = b':';
	/// Namespace for Eip155 chains.
	pub const EIP155_NAMESPACE: &[u8] = b"eip155";
	/// Namespace for Bip122 chains.
	pub const BIP122_NAMESPACE: &[u8] = b"bip122";
	/// Namespace for Dotsama chains.
	pub const DOTSAMA_NAMESPACE: &[u8] = b"polkadot";
	/// Namespace for Solana chains.
	pub const SOLANA_NAMESPACE: &[u8] = b"solana";

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
		/// input, according to the AssetDID method rules.
		pub fn from_utf8_encoded<I>(input: I) -> Result<Self, Error>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let input = input.as_ref();
			let input_length = input.len();
			if !(MINIMUM_CHAIN_ID_LENGTH..=MAXIMUM_CHAIN_ID_LENGTH).contains(&input_length) {
				log::trace!(
					"Length of provided input {} is not included in the inclusive range [{},{}]",
					input_length,
					MINIMUM_CHAIN_ID_LENGTH,
					MAXIMUM_CHAIN_ID_LENGTH
				);
				return Err(Error::InvalidFormat);
			}

			let ChainComponents { namespace, reference } = split_components(input);

			match (namespace, reference) {
				// "eip155:" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-3.md
				(Some(EIP155_NAMESPACE), Some(eip155_reference)) => {
					Eip155Reference::from_utf8_encoded(eip155_reference).map(Self::Eip155)
				}
				// "bip122:" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-4.md
				(Some(BIP122_NAMESPACE), Some(bip122_reference)) => {
					GenesisHexHash32Reference::from_utf8_encoded(bip122_reference).map(Self::Bip122)
				}
				// "polkadot:" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-13.md
				(Some(DOTSAMA_NAMESPACE), Some(dotsama_reference)) => {
					GenesisHexHash32Reference::from_utf8_encoded(dotsama_reference).map(Self::Dotsama)
				}
				// "solana:" chains -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-30.md
				(Some(SOLANA_NAMESPACE), Some(solana_reference)) => {
					GenesisBase58Hash32Reference::from_utf8_encoded(solana_reference).map(Self::Solana)
				}
				// Other chains that are still compatible with the CAIP-2 spec -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md
				_ => GenericChainId::from_utf8_encoded(input).map(Self::Generic),
			}
		}
	}

	impl Display for ChainId {
		fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
			match self {
				Self::Bip122(reference) => {
					write!(
						f,
						"{}",
						str::from_utf8(BIP122_NAMESPACE)
							.expect("Conversion of Bip122 namespace to string should never fail.")
					)?;
					write!(f, "{}", char::from(NAMESPACE_REFERENCE_SEPARATOR))?;
					reference.fmt(f)?;
				}
				Self::Eip155(reference) => {
					write!(
						f,
						"{}",
						str::from_utf8(EIP155_NAMESPACE)
							.expect("Conversion of Eip155 namespace to string should never fail.")
					)?;
					write!(f, "{}", char::from(NAMESPACE_REFERENCE_SEPARATOR))?;
					reference.fmt(f)?;
				}
				Self::Dotsama(reference) => {
					write!(
						f,
						"{}",
						str::from_utf8(DOTSAMA_NAMESPACE)
							.expect("Conversion of Dotsama namespace to string should never fail.")
					)?;
					write!(f, "{}", char::from(NAMESPACE_REFERENCE_SEPARATOR))?;
					reference.fmt(f)?;
				}
				Self::Solana(reference) => {
					write!(
						f,
						"{}",
						str::from_utf8(SOLANA_NAMESPACE)
							.expect("Conversion of Solana namespace to string should never fail.")
					)?;
					write!(f, "{}", char::from(NAMESPACE_REFERENCE_SEPARATOR))?;
					reference.fmt(f)?;
				}
				Self::Generic(GenericChainId { namespace, reference }) => {
					namespace.fmt(f)?;
					write!(f, "{}", char::from(NAMESPACE_REFERENCE_SEPARATOR))?;
					reference.fmt(f)?;
				}
			}
			Ok(())
		}
	}

	const fn check_namespace_length_bounds(namespace: &[u8]) -> Result<(), NamespaceError> {
		let namespace_length = namespace.len();
		if namespace_length < MINIMUM_NAMESPACE_LENGTH {
			Err(NamespaceError::TooShort)
		} else if namespace_length > MAXIMUM_NAMESPACE_LENGTH {
			Err(NamespaceError::TooLong)
		} else {
			Ok(())
		}
	}

	const fn check_reference_length_bounds(reference: &[u8]) -> Result<(), ReferenceError> {
		let reference_length = reference.len();
		if reference_length < MINIMUM_REFERENCE_LENGTH {
			Err(ReferenceError::TooShort)
		} else if reference_length > MAXIMUM_REFERENCE_LENGTH {
			Err(ReferenceError::TooLong)
		} else {
			Ok(())
		}
	}

	/// Split the given input into its components, i.e., namespace, and
	/// reference, if the proper separator is found.
	fn split_components(input: &[u8]) -> ChainComponents {
		let mut split = input.as_ref().splitn(2, |c| *c == NAMESPACE_REFERENCE_SEPARATOR);
		ChainComponents {
			namespace: split.next(),
			reference: split.next(),
		}
	}

	struct ChainComponents<'a> {
		namespace: Option<&'a [u8]>,
		reference: Option<&'a [u8]>,
	}

	/// An EIP155 chain reference.
	/// It is a modification of the [CAIP-3 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-3.md)
	/// according to the rules defined in the Asset DID method specification.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct Eip155Reference(pub(crate) u128);

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
		/// Parse a UTF8-encoded EIP155 chain reference, failing if the input
		/// string is not valid.
		pub(crate) fn from_utf8_encoded<I>(input: I) -> Result<Self, Error>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let input = input.as_ref();
			check_reference_length_bounds(input)?;

			let decoded = str::from_utf8(input).map_err(|_| {
				log::trace!("Provided input is not a valid UTF8 string as expected by an Eip155 reference.");
				ReferenceError::InvalidFormat
			})?;
			let parsed = decoded.parse::<u128>().map_err(|_| {
				log::trace!("Provided input is not a valid u128 value as expected by an Eip155 reference.");
				ReferenceError::InvalidFormat
			})?;
			// Unchecked since we already checked for maximum length and hence maximum value
			Ok(Self(parsed))
		}
	}

	// Getters
	impl Eip155Reference {
		pub fn inner(&self) -> &u128 {
			&self.0
		}
	}

	impl TryFrom<u128> for Eip155Reference {
		type Error = Error;

		fn try_from(value: u128) -> Result<Self, Self::Error> {
			// Max value for 32-digit decimal values (used for EIP chains so far).
			// TODO: This could be enforced at compilation time once constraints on generics
			// will be available.
			// https://rust-lang.github.io/rfcs/2000-const-generics.html
			if value <= 99999999999999999999999999999999 {
				Ok(Self(value))
			} else {
				Err(ReferenceError::TooLong.into())
			}
		}
	}

	impl From<u64> for Eip155Reference {
		fn from(value: u64) -> Self {
			Self(value.into())
		}
	}

	impl Display for Eip155Reference {
		fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
			write!(f, "{}", self.0)
		}
	}

	/// A chain reference for CAIP-2 chains that are identified by a HEX genesis
	/// hash of 32 characters.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct GenesisHexHash32Reference(pub(crate) [u8; 16]);

	impl GenesisHexHash32Reference {
		/// The CAIP-2 reference for the Bitcoin mainnet.
		pub const fn bitcoin_mainnet() -> Self {
			Self(hex!("000000000019d6689c085ae165831e93"))
		}

		/// The CAIP-2 reference for the Litecoin mainnet.
		pub const fn litecoin_mainnet() -> Self {
			Self(hex!("12a765e31ffd4059bada1e25190f6e98"))
		}

		/// The CAIP-2 reference for the Polkadot relaychain.
		pub const fn polkadot() -> Self {
			Self(hex!("91b171bb158e2d3848fa23a9f1c25182"))
		}

		/// The CAIP-2 reference for the Kusama relaychain.
		pub const fn kusama() -> Self {
			Self(hex!("b0a8d493285c2df73290dfb7e61f870f"))
		}

		/// The CAIP-2 reference for the KILT Spiritnet parachain.
		pub const fn kilt_spiritnet() -> Self {
			Self(hex!("411f057b9107718c9624d6aa4a3f23c1"))
		}
	}

	impl GenesisHexHash32Reference {
		/// Parse a UTF8-encoded HEX chain reference, failing if the input
		/// string is not valid.
		pub(crate) fn from_utf8_encoded<I>(input: I) -> Result<Self, Error>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let input = input.as_ref();
			check_reference_length_bounds(input)?;

			let decoded = hex::decode(input).map_err(|_| {
				log::trace!("Provided input is not a valid hex value as expected by a genesis HEX reference.");
				ReferenceError::InvalidFormat
			})?;
			let inner: [u8; 16] = decoded.try_into().map_err(|_| {
				log::trace!("Provided input is not 16 bytes long as expected by a genesis HEX reference.");
				ReferenceError::InvalidFormat
			})?;
			Ok(Self(inner))
		}
	}

	// Getters
	impl GenesisHexHash32Reference {
		pub fn inner(&self) -> &[u8] {
			&self.0
		}
	}

	impl Display for GenesisHexHash32Reference {
		fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
			write!(f, "{}", hex::encode(self.0))
		}
	}

	/// A chain reference for CAIP-2 chains that are identified by a
	/// Base58-encoded genesis hash of 32 characters.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct GenesisBase58Hash32Reference(pub(crate) BoundedVec<u8, ConstU32<32>>);

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
		pub(crate) fn from_utf8_encoded<I>(input: I) -> Result<Self, Error>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let input = input.as_ref();
			check_reference_length_bounds(input)?;

			let decoded_string = str::from_utf8(input).map_err(|_| {
				log::trace!("Provided input is not a valid UTF8 string as expected by a genesis base58 reference.");
				ReferenceError::InvalidFormat
			})?;
			let decoded = decoded_string.from_base58().map_err(|_| {
				log::trace!("Provided input is not a valid base58 value as expected by a genesis base58 reference.");
				ReferenceError::InvalidFormat
			})?;
			// Max length in bytes of a 32-character Base58 string is 32 -> it is the string
			// formed by all "1". Otherwise, it is always between 23 and 24 characters.
			let inner: BoundedVec<u8, ConstU32<32>> = decoded.try_into().map_err(|_| ReferenceError::InvalidFormat)?;
			Ok(Self(inner))
		}
	}

	// Getters
	impl GenesisBase58Hash32Reference {
		pub fn inner(&self) -> &[u8] {
			&self.0
		}
	}

	impl Display for GenesisBase58Hash32Reference {
		fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
			write!(f, "{}", &self.0.to_base58())
		}
	}

	/// A generic chain ID compliant with the [CAIP-2 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md) that cannot be boxed in any of the supported variants.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct GenericChainId {
		pub(crate) namespace: GenericChainNamespace,
		pub(crate) reference: GenericChainReference,
	}

	impl GenericChainId {
		/// Parse a generic UTF8-encoded chain ID, failing if the input does not
		/// respect the CAIP-2 requirements.
		pub(crate) fn from_utf8_encoded<I>(input: I) -> Result<Self, Error>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let ChainComponents { namespace, reference } = split_components(input.as_ref());

			match (namespace, reference) {
				(Some(namespace), Some(reference)) => Ok(Self {
					namespace: GenericChainNamespace::from_utf8_encoded(namespace)?,
					reference: GenericChainReference::from_utf8_encoded(reference)?,
				}),
				_ => Err(Error::InvalidFormat),
			}
		}
	}

	// Getters
	impl GenericChainId {
		pub fn namespace(&self) -> &GenericChainNamespace {
			&self.namespace
		}
		pub fn reference(&self) -> &GenericChainReference {
			&self.reference
		}
	}

	/// A generic chain namespace as defined in the [CAIP-2 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md).
	/// It stores the provided UTF8-encoded namespace without trying to apply
	/// any parsing/decoding logic.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct GenericChainNamespace(pub(crate) BoundedVec<u8, ConstU32<MAXIMUM_NAMESPACE_LENGTH_U32>>);

	impl GenericChainNamespace {
		/// Parse a generic UTF8-encoded chain namespace, failing if the input
		/// does not respect the CAIP-2 requirements.
		pub(crate) fn from_utf8_encoded<I>(input: I) -> Result<Self, Error>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let input = input.as_ref();
			check_namespace_length_bounds(input)?;

			input.iter().try_for_each(|c| {
				if !matches!(c, b'-' | b'a'..=b'z' | b'0'..=b'9') {
					log::trace!("Provided input has some invalid values as expected by a generic chain namespace.");
					Err(NamespaceError::InvalidFormat)
				} else {
					Ok(())
				}
			})?;
			Ok(Self(
				Vec::<u8>::from(input)
					.try_into()
					.map_err(|_| NamespaceError::InvalidFormat)?,
			))
		}
	}

	// Getters
	impl GenericChainNamespace {
		pub fn inner(&self) -> &[u8] {
			&self.0
		}
	}

	impl Display for GenericChainNamespace {
		fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
			// We checked when the type is created that all characters are valid UTF8
			// (actually ASCII) characters.
			write!(
				f,
				"{}",
				str::from_utf8(&self.0).expect("Conversion of GenericChainNamespace to string should never fail.")
			)
		}
	}

	/// A generic chain reference as defined in the [CAIP-2 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md).
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct GenericChainReference(pub(crate) BoundedVec<u8, ConstU32<MAXIMUM_REFERENCE_LENGTH_U32>>);

	impl GenericChainReference {
		/// Parse a generic UTF8-encoded chain reference, failing if the input
		/// does not respect the CAIP-2 requirements.
		pub(crate) fn from_utf8_encoded<I>(input: I) -> Result<Self, Error>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let input = input.as_ref();
			check_reference_length_bounds(input)?;

			input.iter().try_for_each(|c| {
				if !matches!(c, b'-' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9') {
					log::trace!("Provided input has some invalid values as expected by a generic chain reference.");
					Err(ReferenceError::InvalidFormat)
				} else {
					Ok(())
				}
			})?;
			Ok(Self(
				Vec::<u8>::from(input)
					.try_into()
					.map_err(|_| ReferenceError::InvalidFormat)?,
			))
		}
	}

	// Getters
	impl GenericChainReference {
		pub fn inner(&self) -> &[u8] {
			&self.0
		}
	}

	impl Display for GenericChainReference {
		fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
			// We checked when the type is created that all characters are valid UTF8
			// (actually ASCII) characters.
			write!(
				f,
				"{}",
				str::from_utf8(&self.0).expect("Conversion of GenericChainReference to string should never fail.")
			)
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
				let chain_id = ChainId::from_utf8_encoded(chain.as_bytes())
					.unwrap_or_else(|_| panic!("Chain ID {:?} should not fail for eip155 chains", chain));
				// Verify that the ToString implementation prints exactly the original input
				assert_eq!(chain_id.to_string(), chain);
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
				let chain_id = ChainId::from_utf8_encoded(chain.as_bytes())
					.unwrap_or_else(|_| panic!("Chain ID {:?} should not fail for bip122 chains", chain));
				// Verify that the ToString implementation prints exactly the original input
				assert_eq!(chain_id.to_string(), chain);
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
				let chain_id = ChainId::from_utf8_encoded(chain.as_bytes())
					.unwrap_or_else(|_| panic!("Chain ID {:?} should not fail for dotsama chains", chain));
				// Verify that the ToString implementation prints exactly the original input
				assert_eq!(chain_id.to_string(), chain);
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
				let chain_id = ChainId::from_utf8_encoded(chain.as_bytes())
					.unwrap_or_else(|_| panic!("Chain ID {:?} should not fail for solana chains", chain));
				// Verify that the ToString implementation prints exactly the original input
				assert_eq!(chain_id.to_string(), chain);
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
				let chain_id = ChainId::from_utf8_encoded(chain.as_bytes())
					.unwrap_or_else(|_| panic!("Chain ID {:?} should not fail for generic chains", chain));
				// Verify that the ToString implementation prints exactly the original input
				assert_eq!(chain_id.to_string(), chain);
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
			assert_eq!(ChainId::ethereum_mainnet().to_string(), "eip155:1");
			assert_eq!(ChainId::moonbeam_eth().to_string(), "eip155:1284");
			assert_eq!(
				ChainId::bitcoin_mainnet().to_string(),
				"bip122:000000000019d6689c085ae165831e93"
			);
			assert_eq!(
				ChainId::litecoin_mainnet().to_string(),
				"bip122:12a765e31ffd4059bada1e25190f6e98"
			);
			assert_eq!(
				ChainId::polkadot().to_string(),
				"polkadot:91b171bb158e2d3848fa23a9f1c25182"
			);
			assert_eq!(
				ChainId::kusama().to_string(),
				"polkadot:b0a8d493285c2df73290dfb7e61f870f"
			);
			assert_eq!(
				ChainId::kilt_spiritnet().to_string(),
				"polkadot:411f057b9107718c9624d6aa4a3f23c1"
			);
			assert_eq!(
				ChainId::solana_mainnet().to_string(),
				"solana:4sGjMW1sUnHzSxGspuhpqLDx6wiyjNtZ"
			);
		}
	}
}
