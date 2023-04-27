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

pub mod v1 {
	use crate::errors::asset::{Error, IdentifierError, NamespaceError, ReferenceError};

	use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
	use scale_info::TypeInfo;

	use core::{format_args, str};

	use frame_support::{sp_runtime::RuntimeDebug, traits::ConstU32, BoundedVec};
	use sp_core::U256;
	use sp_std::{fmt::Display, vec::Vec};

	/// The minimum length, including separator symbols, an asset ID can have
	/// according to the minimum values defined by the CAIP-19 definition.
	pub const MINIMUM_ASSET_ID_LENGTH: usize = MINIMUM_ASSET_NAMESPACE_LENGTH + 1 + MINIMUM_ASSET_REFERENCE_LENGTH;
	/// The maximum length, including separator symbols, an asset ID can have
	/// according to the minimum values defined by the CAIP-19 definition.
	pub const MAXIMUM_ASSET_ID_LENGTH: usize =
		MAXIMUM_NAMESPACE_LENGTH + 1 + MAXIMUM_ASSET_REFERENCE_LENGTH + 1 + MAXIMUM_ASSET_IDENTIFIER_LENGTH;

	/// The minimum length of a valid asset ID namespace.
	pub const MINIMUM_ASSET_NAMESPACE_LENGTH: usize = 3;
	/// The maximum length of a valid asset ID namespace.
	pub const MAXIMUM_NAMESPACE_LENGTH: usize = 8;
	const MAXIMUM_ASSET_NAMESPACE_LENGTH_U32: u32 = MAXIMUM_NAMESPACE_LENGTH as u32;
	/// The minimum length of a valid asset ID reference.
	pub const MINIMUM_ASSET_REFERENCE_LENGTH: usize = 1;
	/// The maximum length of a valid asset ID reference.
	pub const MAXIMUM_ASSET_REFERENCE_LENGTH: usize = 128;
	const MAXIMUM_ASSET_REFERENCE_LENGTH_U32: u32 = MAXIMUM_ASSET_REFERENCE_LENGTH as u32;
	/// The minimum length of a valid asset ID identifier.
	pub const MINIMUM_ASSET_IDENTIFIER_LENGTH: usize = 1;
	/// The maximum length of a valid asset ID reference.
	pub const MAXIMUM_ASSET_IDENTIFIER_LENGTH: usize = 78;
	const MAXIMUM_ASSET_IDENTIFIER_LENGTH_U32: u32 = MAXIMUM_ASSET_IDENTIFIER_LENGTH as u32;

	/// Separator between asset namespace and asset reference.
	const ASSET_NAMESPACE_REFERENCE_SEPARATOR: u8 = b':';
	/// Separator between asset reference and asset identifier.
	const ASSET_REFERENCE_IDENTIFIER_SEPARATOR: u8 = b':';

	/// Namespace for Slip44 assets.
	pub const SLIP44_NAMESPACE: &[u8] = b"slip44";
	/// Namespace for Erc20 assets.
	pub const ERC20_NAMESPACE: &[u8] = b"erc20";
	/// Namespace for Erc721 assets.
	pub const ERC721_NAMESPACE: &[u8] = b"erc721";
	/// Namespace for Erc1155 assets.
	pub const ERC1155_NAMESPACE: &[u8] = b"erc1155";

	// TODO: Add link to the Asset DID spec once merged.

	/// The Asset ID component as specified in the Asset DID specification.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub enum AssetId {
		// A SLIP44 asset reference.
		Slip44(Slip44Reference),
		// An ERC20 asset reference.
		Erc20(EvmSmartContractFungibleReference),
		// An ERC721 asset reference.
		Erc721(EvmSmartContractNonFungibleReference),
		// An ERC1155 asset reference.
		Erc1155(EvmSmartContractNonFungibleReference),
		// A generic asset.
		Generic(GenericAssetId),
	}

	impl From<Slip44Reference> for AssetId {
		fn from(reference: Slip44Reference) -> Self {
			Self::Slip44(reference)
		}
	}

	impl From<EvmSmartContractFungibleReference> for AssetId {
		fn from(reference: EvmSmartContractFungibleReference) -> Self {
			Self::Erc20(reference)
		}
	}

	impl AssetId {
		/// Try to parse an `AssetId` instance from the provided UTF8-encoded
		/// input.
		pub fn from_utf8_encoded<I>(input: I) -> Result<Self, Error>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let input = input.as_ref();
			let input_length = input.len();
			if !(MINIMUM_ASSET_ID_LENGTH..=MAXIMUM_ASSET_ID_LENGTH).contains(&input_length) {
				log::trace!(
					"Length of provided input {} is not included in the inclusive range [{},{}]",
					input_length,
					MINIMUM_ASSET_ID_LENGTH,
					MAXIMUM_ASSET_ID_LENGTH
				);
				return Err(Error::InvalidFormat);
			}

			let AssetComponents {
				namespace,
				reference,
				identifier,
			} = split_components(input);

			match (namespace, reference, identifier) {
				// "slip44:" assets -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-20.md
				(Some(SLIP44_NAMESPACE), _, Some(_)) => {
					log::trace!("Slip44 namespace does not accept an asset identifier.");
					Err(Error::InvalidFormat)
				}
				(Some(SLIP44_NAMESPACE), Some(slip44_reference), None) => {
					Slip44Reference::from_utf8_encoded(slip44_reference).map(Self::Slip44)
				}
				// "erc20:" assets -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-21.md
				(Some(ERC20_NAMESPACE), _, Some(_)) => {
					log::trace!("Erc20 namespace does not accept an asset identifier.");
					Err(Error::InvalidFormat)
				}
				(Some(ERC20_NAMESPACE), Some(erc20_reference), None) => {
					EvmSmartContractFungibleReference::from_utf8_encoded(erc20_reference).map(Self::Erc20)
				}
				// "erc721:" assets -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-22.md
				(Some(ERC721_NAMESPACE), Some(erc721_reference), identifier) => {
					let reference = EvmSmartContractFungibleReference::from_utf8_encoded(erc721_reference)?;
					let identifier = identifier.map_or(Ok(None), |id| {
						EvmSmartContractNonFungibleIdentifier::from_utf8_encoded(id).map(Some)
					})?;
					Ok(Self::Erc721(EvmSmartContractNonFungibleReference(
						reference, identifier,
					)))
				}
				// "erc1155:" assets-> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-29.md
				(Some(ERC1155_NAMESPACE), Some(erc1155_reference), identifier) => {
					let reference = EvmSmartContractFungibleReference::from_utf8_encoded(erc1155_reference)?;
					let identifier = identifier.map_or(Ok(None), |id| {
						EvmSmartContractNonFungibleIdentifier::from_utf8_encoded(id).map(Some)
					})?;
					Ok(Self::Erc1155(EvmSmartContractNonFungibleReference(
						reference, identifier,
					)))
				}
				// Generic yet valid asset IDs
				_ => GenericAssetId::from_utf8_encoded(input).map(Self::Generic),
			}
		}
	}

	impl Display for AssetId {
		fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
			match self {
				Self::Slip44(reference) => {
					write!(
						f,
						"{}",
						str::from_utf8(SLIP44_NAMESPACE)
							.expect("Conversion of Slip44 namespace to string should never fail.")
					)?;
					write!(f, "{}", char::from(ASSET_NAMESPACE_REFERENCE_SEPARATOR))?;
					reference.fmt(f)?;
				}
				Self::Erc20(reference) => {
					write!(
						f,
						"{}",
						str::from_utf8(ERC20_NAMESPACE)
							.expect("Conversion of Erc20 namespace to string should never fail.")
					)?;
					write!(f, "{}", char::from(ASSET_NAMESPACE_REFERENCE_SEPARATOR))?;
					reference.fmt(f)?;
				}
				Self::Erc721(EvmSmartContractNonFungibleReference(reference, identifier)) => {
					write!(
						f,
						"{}",
						str::from_utf8(ERC721_NAMESPACE)
							.expect("Conversion of Erc721 namespace to string should never fail.")
					)?;
					write!(f, "{}", char::from(ASSET_NAMESPACE_REFERENCE_SEPARATOR))?;
					reference.fmt(f)?;
					if let Some(id) = identifier {
						write!(f, "{}", char::from(ASSET_REFERENCE_IDENTIFIER_SEPARATOR))?;
						id.fmt(f)?;
					}
				}
				Self::Erc1155(EvmSmartContractNonFungibleReference(reference, identifier)) => {
					write!(
						f,
						"{}",
						str::from_utf8(ERC1155_NAMESPACE)
							.expect("Conversion of Erc1155 namespace to string should never fail.")
					)?;
					write!(f, "{}", char::from(ASSET_NAMESPACE_REFERENCE_SEPARATOR))?;
					reference.fmt(f)?;
					if let Some(id) = identifier {
						write!(f, "{}", char::from(ASSET_REFERENCE_IDENTIFIER_SEPARATOR))?;
						id.fmt(f)?;
					}
				}
				Self::Generic(GenericAssetId {
					namespace,
					reference,
					id,
				}) => {
					namespace.fmt(f)?;
					write!(f, "{}", char::from(ASSET_NAMESPACE_REFERENCE_SEPARATOR))?;
					reference.fmt(f)?;
					if let Some(identifier) = id {
						write!(f, "{}", char::from(ASSET_REFERENCE_IDENTIFIER_SEPARATOR))?;
						identifier.fmt(f)?;
					}
				}
			}
			Ok(())
		}
	}

	const fn check_namespace_length_bounds(namespace: &[u8]) -> Result<(), NamespaceError> {
		let namespace_length = namespace.len();
		if namespace_length < MINIMUM_ASSET_NAMESPACE_LENGTH {
			Err(NamespaceError::TooShort)
		} else if namespace_length > MAXIMUM_NAMESPACE_LENGTH {
			Err(NamespaceError::TooLong)
		} else {
			Ok(())
		}
	}

	const fn check_reference_length_bounds(reference: &[u8]) -> Result<(), ReferenceError> {
		let reference_length = reference.len();
		if reference_length < MINIMUM_ASSET_REFERENCE_LENGTH {
			Err(ReferenceError::TooShort)
		} else if reference_length > MAXIMUM_ASSET_REFERENCE_LENGTH {
			Err(ReferenceError::TooLong)
		} else {
			Ok(())
		}
	}

	const fn check_identifier_length_bounds(identifier: &[u8]) -> Result<(), IdentifierError> {
		let identifier_length = identifier.len();
		if identifier_length < MINIMUM_ASSET_IDENTIFIER_LENGTH {
			Err(IdentifierError::TooShort)
		} else if identifier_length > MAXIMUM_ASSET_IDENTIFIER_LENGTH {
			Err(IdentifierError::TooLong)
		} else {
			Ok(())
		}
	}

	/// Split the given input into its components, i.e., namespace, reference,
	/// and identifier, if the proper separators are found.
	fn split_components(input: &[u8]) -> AssetComponents {
		let mut split = input.splitn(2, |c| *c == ASSET_NAMESPACE_REFERENCE_SEPARATOR);
		let (namespace, reference) = (split.next(), split.next());

		// Split the remaining reference to extract the identifier, if present
		let (reference, identifier) = if let Some(r) = reference {
			let mut split = r.splitn(2, |c| *c == ASSET_REFERENCE_IDENTIFIER_SEPARATOR);
			// Split the reference further, if present
			(split.next(), split.next())
		} else {
			// Return the old reference, which is None if we are at this point
			(reference, None)
		};

		AssetComponents {
			namespace,
			reference,
			identifier,
		}
	}

	struct AssetComponents<'a> {
		namespace: Option<&'a [u8]>,
		reference: Option<&'a [u8]>,
		identifier: Option<&'a [u8]>,
	}

	/// A Slip44 asset reference.
	/// It is a modification of the [CAIP-20 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-20.md)
	/// according to the rules defined in the Asset DID method specification.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct Slip44Reference(pub(crate) U256);

	impl Slip44Reference {
		/// Parse a UTF8-encoded decimal Slip44 asset reference, failing if the
		/// input string is not valid.
		pub(crate) fn from_utf8_encoded<I>(input: I) -> Result<Self, Error>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let input = input.as_ref();
			check_reference_length_bounds(input)?;

			let decoded = str::from_utf8(input).map_err(|_| {
				log::trace!("Provided input is not a valid UTF8 string as expected by a Slip44 reference.");
				ReferenceError::InvalidFormat
			})?;
			let parsed = U256::from_dec_str(decoded).map_err(|_| {
				log::trace!("Provided input is not a valid u256 value as expected by a Slip44 reference.");
				ReferenceError::InvalidFormat
			})?;
			// Unchecked since we already checked for maximum length and hence maximum value
			Ok(Self(parsed))
		}
	}

	impl TryFrom<U256> for Slip44Reference {
		type Error = Error;

		fn try_from(value: U256) -> Result<Self, Self::Error> {
			// Max value for 64-digit decimal values (used for Slip44 references so far).
			// TODO: This could be enforced at compilation time once constraints on generics
			// will be available.
			// https://rust-lang.github.io/rfcs/2000-const-generics.html
			if value
				<= U256::from_str_radix("9999999999999999999999999999999999999999999999999999999999999999", 10)
					.expect("Casting the maximum value for a Slip44 reference into a U256 should never fail.")
			{
				Ok(Self(value))
			} else {
				Err(ReferenceError::TooLong.into())
			}
		}
	}

	impl From<u128> for Slip44Reference {
		fn from(value: u128) -> Self {
			Self(value.into())
		}
	}

	// Getters
	impl Slip44Reference {
		pub fn inner(&self) -> &U256 {
			&self.0
		}
	}

	impl Display for Slip44Reference {
		fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
			write!(f, "{}", self.0)
		}
	}

	/// An asset reference that is identifiable only by an EVM smart contract
	/// (e.g., a fungible token). It is a modification of the [CAIP-21 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-21.md)
	/// according to the rules defined in the Asset DID method specification.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct EvmSmartContractFungibleReference(pub(crate) [u8; 20]);

	impl EvmSmartContractFungibleReference {
		/// Parse a UTF8-encoded smart contract HEX address (including the `0x`
		/// prefix), failing if the input string is not valid.
		pub(crate) fn from_utf8_encoded<I>(input: I) -> Result<Self, Error>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let input = input.as_ref();
			// If the prefix is "0x" => parse the address
			if let [b'0', b'x', contract_address @ ..] = input {
				check_reference_length_bounds(contract_address)?;

				let decoded = hex::decode(contract_address).map_err(|_| {
					log::trace!("Provided input is not a valid hex value as expected by a smart contract reference.");
					ReferenceError::InvalidFormat
				})?;
				let inner: [u8; 20] = decoded.try_into().map_err(|_| {
					log::trace!("Provided input is not 20 bytes long as expected by a smart contract reference.");
					ReferenceError::InvalidFormat
				})?;
				Ok(Self(inner))
			// Otherwise fail
			} else {
				log::trace!("Provided input does not have the `0x` prefix as expected by a smart contract reference.");
				Err(ReferenceError::InvalidFormat.into())
			}
		}
	}

	// Getters
	impl EvmSmartContractFungibleReference {
		pub fn inner(&self) -> &[u8] {
			&self.0
		}
	}

	impl Display for EvmSmartContractFungibleReference {
		fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
			write!(f, "0x{}", hex::encode(self.0))
		}
	}

	/// An asset reference that is identifiable by an EVM smart contract and an
	/// optional identifier (e.g., an NFT collection or instance thereof). It is
	/// a modification of the [CAIP-22 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-22.md) and
	/// [CAIP-29 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-29.md)
	/// according to the rules defined in the Asset DID method specification.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct EvmSmartContractNonFungibleReference(
		pub(crate) EvmSmartContractFungibleReference,
		pub(crate) Option<EvmSmartContractNonFungibleIdentifier>,
	);

	// Getters
	impl EvmSmartContractNonFungibleReference {
		pub fn smart_contract(&self) -> &EvmSmartContractFungibleReference {
			&self.0
		}

		pub fn identifier(&self) -> &Option<EvmSmartContractNonFungibleIdentifier> {
			&self.1
		}
	}

	/// An asset identifier for an EVM smart contract collection (e.g., an NFT
	/// instance).
	/// Since the identifier can be up to 78 characters long of an unknown
	/// alphabet, this type simply contains the UTF-8 encoding of such an
	/// identifier without applying any special parsing/decoding logic.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct EvmSmartContractNonFungibleIdentifier(
		pub(crate) BoundedVec<u8, ConstU32<MAXIMUM_ASSET_IDENTIFIER_LENGTH_U32>>,
	);

	impl EvmSmartContractNonFungibleIdentifier {
		/// Parse a UTF8-encoded smart contract asset identifier, failing if the
		/// input string is not valid.
		pub(crate) fn from_utf8_encoded<I>(input: I) -> Result<Self, Error>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let input = input.as_ref();
			check_identifier_length_bounds(input)?;

			input.iter().try_for_each(|c| {
				if !c.is_ascii_digit() {
					log::trace!("Provided input has some invalid values as expected by a smart contract-based asset identifier.");
					Err(IdentifierError::InvalidFormat)
				} else {
					Ok(())
				}
			})?;

			Ok(Self(
				Vec::<u8>::from(input)
					.try_into()
					.map_err(|_| IdentifierError::InvalidFormat)?,
			))
		}
	}

	// Getters
	impl EvmSmartContractNonFungibleIdentifier {
		pub fn inner(&self) -> &[u8] {
			&self.0
		}
	}

	impl Display for EvmSmartContractNonFungibleIdentifier {
		fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
			// We checked when the type is created that all characters are valid digits.
			write!(
				f,
				"{}",
				str::from_utf8(&self.0)
					.expect("Conversion of EvmSmartContractNonFungibleIdentifier to string should never fail.")
			)
		}
	}

	/// A generic asset ID compliant with the [CAIP-19 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-19.md) that cannot be boxed in any of the supported variants.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct GenericAssetId {
		pub(crate) namespace: GenericAssetNamespace,
		pub(crate) reference: GenericAssetReference,
		pub(crate) id: Option<GenericAssetIdentifier>,
	}

	impl GenericAssetId {
		/// Parse a generic UTF8-encoded asset ID, failing if the input does not
		/// respect the CAIP-19 requirements.
		pub(crate) fn from_utf8_encoded<I>(input: I) -> Result<Self, Error>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let AssetComponents {
				namespace,
				reference,
				identifier,
			} = split_components(input.as_ref());

			match (namespace, reference, identifier) {
				(Some(namespace), Some(reference), identifier) => Ok(Self {
					namespace: GenericAssetNamespace::from_utf8_encoded(namespace)?,
					reference: GenericAssetReference::from_utf8_encoded(reference)?,
					// Transform Option<Result> to Result<Option> and bubble Err case up, keeping Ok(Option) for
					// successful cases.
					id: identifier.map_or(Ok(None), |id| GenericAssetIdentifier::from_utf8_encoded(id).map(Some))?,
				}),
				_ => Err(Error::InvalidFormat),
			}
		}
	}

	// Getters
	impl GenericAssetId {
		pub fn namespace(&self) -> &GenericAssetNamespace {
			&self.namespace
		}
		pub fn reference(&self) -> &GenericAssetReference {
			&self.reference
		}
		pub fn id(&self) -> &Option<GenericAssetIdentifier> {
			&self.id
		}
	}

	/// A generic asset namespace as defined in the [CAIP-19 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-19.md).
	/// It stores the provided UTF8-encoded namespace without trying to apply
	/// any parsing/decoding logic.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct GenericAssetNamespace(pub(crate) BoundedVec<u8, ConstU32<MAXIMUM_ASSET_NAMESPACE_LENGTH_U32>>);

	impl GenericAssetNamespace {
		/// Parse a generic UTF8-encoded asset namespace, failing if the input
		/// does not respect the CAIP-19 requirements.
		pub(crate) fn from_utf8_encoded<I>(input: I) -> Result<Self, Error>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let input = input.as_ref();
			check_namespace_length_bounds(input)?;

			input.iter().try_for_each(|c| {
				if !matches!(c, b'-' | b'a'..=b'z' | b'0'..=b'9') {
					log::trace!("Provided input has some invalid values as expected by a generic asset namespace.");
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
	impl GenericAssetNamespace {
		pub fn inner(&self) -> &[u8] {
			&self.0
		}
	}

	impl Display for GenericAssetNamespace {
		fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
			// We checked when the type is created that all characters are valid UTF8
			// (actually ASCII) characters.
			write!(
				f,
				"{}",
				str::from_utf8(&self.0).expect("Conversion of GenericAssetNamespace to string should never fail.")
			)
		}
	}

	/// A generic asset reference as defined in the [CAIP-19 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-19.md).
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct GenericAssetReference(pub(crate) BoundedVec<u8, ConstU32<MAXIMUM_ASSET_REFERENCE_LENGTH_U32>>);

	impl GenericAssetReference {
		/// Parse a generic UTF8-encoded asset reference, failing if the input
		/// does not respect the CAIP-19 requirements.
		pub(crate) fn from_utf8_encoded<I>(input: I) -> Result<Self, Error>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let input = input.as_ref();
			check_reference_length_bounds(input)?;

			input.iter().try_for_each(|c| {
				if !matches!(c, b'-' | b'.' | b'%' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9') {
					log::trace!("Provided input has some invalid values as expected by a generic asset reference.");
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
	impl GenericAssetReference {
		pub fn inner(&self) -> &[u8] {
			&self.0
		}
	}

	impl Display for GenericAssetReference {
		fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
			// We checked when the type is created that all characters are valid UTF8
			// (actually ASCII) characters.
			write!(
				f,
				"{}",
				str::from_utf8(&self.0).expect("Conversion of GenericAssetReference to string should never fail.")
			)
		}
	}

	/// A generic asset identifier as defined in the [CAIP-19 spec](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-19.md).
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct GenericAssetIdentifier(pub(crate) BoundedVec<u8, ConstU32<MAXIMUM_ASSET_IDENTIFIER_LENGTH_U32>>);

	impl GenericAssetIdentifier {
		/// Parse a generic UTF8-encoded asset identifier, failing if the input
		/// does not respect the CAIP-19 requirements.
		pub(crate) fn from_utf8_encoded<I>(input: I) -> Result<Self, Error>
		where
			I: AsRef<[u8]> + Into<Vec<u8>>,
		{
			let input = input.as_ref();
			check_identifier_length_bounds(input)?;

			input.iter().try_for_each(|c| {
				if !matches!(c, b'-' | b'.' | b'%' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9') {
					log::trace!("Provided input has some invalid values as expected by a generic asset identifier.");
					Err(IdentifierError::InvalidFormat)
				} else {
					Ok(())
				}
			})?;
			Ok(Self(
				Vec::<u8>::from(input)
					.try_into()
					.map_err(|_| IdentifierError::InvalidFormat)?,
			))
		}
	}

	// Getters
	impl GenericAssetIdentifier {
		pub fn inner(&self) -> &[u8] {
			&self.0
		}
	}

	impl Display for GenericAssetIdentifier {
		fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
			// We checked when the type is created that all characters are valid UTF8
			// (actually ASCII) characters.
			write!(
				f,
				"{}",
				str::from_utf8(&self.0).expect("Conversion of GenericAssetIdentifier to string should never fail.")
			)
		}
	}

	#[cfg(test)]
	mod test {
		use super::*;

		#[test]
		fn test_slip44_assets() {
			let valid_assets = [
				"slip44:60",
				"slip44:0",
				"slip44:2",
				"slip44:714",
				"slip44:234",
				"slip44:134",
				"slip44:0",
				"slip44:9999999999999999999999999999999999999999999999999999999999999999",
			];

			for asset in valid_assets {
				let asset_id = AssetId::from_utf8_encoded(asset.as_bytes())
					.unwrap_or_else(|_| panic!("Asset ID {:?} should not fail to parse for slip44 assets", asset));
				// Verify that the ToString implementation prints exactly the original input
				assert_eq!(asset_id.to_string(), asset);
			}

			let invalid_assets = [
				// Too short
				"",
				"s",
				"sl",
				"sli",
				"slip",
				"slip4",
				"slip44",
				"slip44:",
				// Not a number
				"slip44:a",
				"slip44::",
				"slip44:›",
				"slip44:😁",
				// Max chars + 1
				"slip44:999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999",
			];
			for asset in invalid_assets {
				assert!(
					AssetId::from_utf8_encoded(asset.as_bytes()).is_err(),
					"Asset ID {:?} should fail to parse for slip44 assets",
					asset
				);
			}
		}

		#[test]
		fn test_erc20_assets() {
			let valid_assets = [
				"erc20:0x6b175474e89094c44da98b954eedeac495271d0f",
				"erc20:0x8f8221AFBB33998D8584A2B05749BA73C37A938A",
			];

			for asset in valid_assets {
				let asset_id = AssetId::from_utf8_encoded(asset.as_bytes())
					.unwrap_or_else(|_| panic!("Asset ID {:?} should not fail to parse for erc20 assets", asset));
				// Verify that the ToString implementation prints exactly the original input
				assert_eq!(asset_id.to_string(), asset.to_lowercase());
			}

			let invalid_assets = [
				// Too short
				"",
				"e",
				"er",
				"erc",
				"erc2",
				"erc20",
				"erc20:",
				// Not valid HEX characters
				"erc20::",
				"erc20:›",
				"erc20:😁",
				// Max chars - 2
				"erc20:0x8f8221AFBB33998D8584A2B05749BA73C37A93",
				// Max chars - 1
				"erc20:0x8f8221AFBB33998D8584A2B05749BA73C37A938",
				// Max chars + 1
				"erc20:0x8f8221AFBB33998D8584A2B05749BA73C37A938A1",
				// Asset ID (not supported for erc20 standard)
				"erc20:0x8f8221AFBB33998D8584A2B05749BA73C37A938A1:1",
				// Smart contract without leading `0x`
				"erc20:8f8221AFBB33998D8584A2B05749BA73C37A938A1",
			];
			for asset in invalid_assets {
				assert!(
					AssetId::from_utf8_encoded(asset.as_bytes()).is_err(),
					"Asset ID {:?} should fail to parse for erc20 assets",
					asset
				);
			}
		}

		#[test]
		fn test_erc721_assets() {
			let valid_assets = [
			"erc721:0x6b175474e89094c44da98b954eedeac495271d0f",
			"erc721:0x8f8221AFBB33998D8584A2B05749BA73C37A938A",
			"erc721:0x8f8221AFBB33998D8584A2B05749BA73C37A938A:0",
			"erc721:0x8f8221AFBB33998D8584A2B05749BA73C37A938A:999999999999999999999999999999999999999999999999999999999999999999999999",
		];

			for asset in valid_assets {
				let asset_id = AssetId::from_utf8_encoded(asset.as_bytes())
					.unwrap_or_else(|_| panic!("Asset ID {:?} should not fail to parse for erc721 assets", asset));
				// Verify that the ToString implementation prints exactly the original input
				assert_eq!(asset_id.to_string(), asset.to_lowercase());
			}

			let invalid_assets = [
			// Too short
			"",
			"e",
			"er",
			"erc",
			"erc7",
			"erc72",
			"erc721",
			"erc721:",
			// Not valid HEX characters
			"erc721::",
			"erc721:›",
			"erc721:😁",
			// Max chars - 2
			"erc721:0x8f8221AFBB33998D8584A2B05749BA73C37A93",
			// Max chars - 1
			"erc721:0x8f8221AFBB33998D8584A2B05749BA73C37A938",
			// Max chars + 1
			"erc721:0x8f8221AFBB33998D8584A2B05749BA73C37A938A1",
			// Wrong asset IDs
			"erc721:0x8f8221AFBB33998D8584A2B05749BA73C37A938A1:",
			"erc721:0x8f8221AFBB33998D8584A2B05749BA73C37A938A1:a",
			"erc721:0x8f8221AFBB33998D8584A2B05749BA73C37A938A1:9999999999999999999999999999999999999999999999999999999999999999999999999",
			"erc721:0x8f8221AFBB33998D8584A2B05749BA73C37A938A1:‹",
			"erc721:0x8f8221AFBB33998D8584A2B05749BA73C37A938A1:😁",
			// Smart contract without leading `0x`
			"erc721:8f8221AFBB33998D8584A2B05749BA73C37A938A1",
		];
			for asset in invalid_assets {
				assert!(
					AssetId::from_utf8_encoded(asset.as_bytes()).is_err(),
					"Asset ID {:?} should fail to parse for erc721 assets",
					asset
				);
			}
		}

		#[test]
		fn test_erc1155_assets() {
			let valid_assets = [
			"erc1155:0x6b175474e89094c44da98b954eedeac495271d0f",
			"erc1155:0x8f8221AFBB33998D8584A2B05749BA73C37A938A",
			"erc1155:0x8f8221AFBB33998D8584A2B05749BA73C37A938A:0",
			"erc1155:0x8f8221AFBB33998D8584A2B05749BA73C37A938A:999999999999999999999999999999999999999999999999999999999999999999999999",
		];

			for asset in valid_assets {
				let asset_id = AssetId::from_utf8_encoded(asset.as_bytes())
					.unwrap_or_else(|_| panic!("Asset ID {:?} should not fail to parse for erc1155 assets", asset));
				// Verify that the ToString implementation prints exactly the original input
				assert_eq!(asset_id.to_string(), asset.to_lowercase());
			}

			let invalid_assets = [
			// Too short
			"",
			"e",
			"er",
			"erc",
			"erc1",
			"erc11",
			"erc115",
			"erc1155",
			"erc1155:",
			// Not valid HEX characters
			"erc1155::",
			"erc1155:›",
			"erc1155:😁",
			// Max chars - 2
			"erc1155:0x8f8221AFBB33998D8584A2B05749BA73C37A93",
			// Max chars - 1
			"erc1155:0x8f8221AFBB33998D8584A2B05749BA73C37A938",
			// Max chars + 1
			"erc1155:0x8f8221AFBB33998D8584A2B05749BA73C37A938A1",
			// Wrong asset IDs
			"erc1155:0x8f8221AFBB33998D8584A2B05749BA73C37A938A1:",
			"erc1155:0x8f8221AFBB33998D8584A2B05749BA73C37A938A1:a",
			"erc1155:0x8f8221AFBB33998D8584A2B05749BA73C37A938A1:9999999999999999999999999999999999999999999999999999999999999999999999999",
			"erc1155:0x8f8221AFBB33998D8584A2B05749BA73C37A938A1:‹",
			"erc1155:0x8f8221AFBB33998D8584A2B05749BA73C37A938A1:😁",
			// Smart contract without leading `0x`
			"erc721:8f8221AFBB33998D8584A2B05749BA73C37A938A1",
		];
			for asset in invalid_assets {
				assert!(
					AssetId::from_utf8_encoded(asset.as_bytes()).is_err(),
					"Asset ID {:?} should fail to parse for erc1155 assets",
					asset
				);
			}
		}

		#[test]
		fn test_generic_assets() {
			let valid_assets = [
			"123:a",
			"12345678:-abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-",
			"12345678:-.abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789%-:-.abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ01234567890123456789012%",
			"para:411f057b9107718c9624d6aa4a3f23c1",
			"para:kilt-spiritnet",
			"w3n:john-doe",
		];

			for asset in valid_assets {
				let asset_id = AssetId::from_utf8_encoded(asset.as_bytes())
					.unwrap_or_else(|_| panic!("Asset ID {:?} should not fail to parse for generic assets", asset));
				// Verify that the ToString implementation prints exactly the original input
				assert_eq!(asset_id.to_string(), asset);
			}

			let invalid_assets = [
				// Too short
				"",
				"a",
				"as",
				"as:",
				"‹",
				"‹:",
				"asd:",
				":",
				"::",
				":::",
				"::::",
				"valid:valid:",
				// Too long
				"too-loong:valid",
				"valid:too-loooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong",
				"valid:valid:too-loooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong",
				// Wrong characters
				"no-val!d:valid",
				"valid:no-val!d",
				"valid:valid:no-val!d",
			];
			for asset in invalid_assets {
				assert!(
					AssetId::from_utf8_encoded(asset.as_bytes()).is_err(),
					"Asset ID {:?} should fail to parse for generic assets",
					asset
				);
			}
		}
	}
}
