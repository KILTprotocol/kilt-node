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

use frame_support::{traits::ConstU32, BoundedVec};

const MINIMUM_NAMESPACE_LENGTH: usize = 3;
const MAXIMUM_NAMESPACE_LENGTH: usize = 8;
const MAXIMUM_NAMESPACE_LENGTH_U32: u32 = MAXIMUM_NAMESPACE_LENGTH as u32;
const MINIMUM_REFERENCE_LENGTH: usize = 1;
const MAXIMUM_REFERENCE_LENGTH: usize = 64;
const MAXIMUM_REFERENCE_LENGTH_U32: u32 = MAXIMUM_REFERENCE_LENGTH as u32;
const MINIMUM_IDENTIFIER_LENGTH: usize = 1;
const MAXIMUM_IDENTIFIER_LENGTH: usize = 78;
const MAXIMUM_IDENTIFIER_LENGTH_U32: u32 = MAXIMUM_IDENTIFIER_LENGTH as u32;

// 20 bytes -> 40 HEX characters
const EVM_SMART_CONTRACT_ADDRESS_LENGTH: usize = 40;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum AssetIdError {
	Namespace(NamespaceError),
	Reference(ReferenceError),
	Identifier(IdentifierError),
	InvalidFormat,
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum NamespaceError {
	TooLong,
	TooShort,
	InvalidCharacter,
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ReferenceError {
	TooLong,
	TooShort,
	InvalidCharacter,
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum IdentifierError {
	TooLong,
	TooShort,
	InvalidCharacter,
}

pub use v1::*;

pub mod v1 {
	use super::*;

	#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
	pub enum AssetId {
		Slip44(Slip44Reference),
		Erc20(EvmSmartContractFungibleReference),
		Erc721(EvmSmartContractNonFungibleReference),
		Erc1155(EvmSmartContractNonFungibleReference),
		Generic(GenericAssetId),
	}

	impl TryFrom<&[u8]> for AssetId {
		type Error = AssetIdError;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			match value {
				// "slip44:" tokens -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-20.md
				[b's', b'l', b'i', b'p', b'4', b'4', b':', asset_reference @ ..] => {
					Slip44Reference::try_from(asset_reference).map(Self::Slip44)
				}
				// "erc20:" tokens -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-21.md
				[b'e', b'r', b'c', b'2', b'0', b':', asset_reference @ ..] => {
					EvmSmartContractFungibleReference::try_from(asset_reference).map(Self::Erc20)
				}
				// "erc721:" tokens -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-22.md
				[b'e', b'r', b'c', b'7', b'2', b'1', b':', asset_reference @ ..] => {
					EvmSmartContractNonFungibleReference::try_from(asset_reference).map(Self::Erc721)
				}
				// "erc1155:" tokens -> https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-29.md
				[b'e', b'r', b'c', b'1', b'1', b'5', b'5', b':', asset_reference @ ..] => {
					EvmSmartContractNonFungibleReference::try_from(asset_reference).map(Self::Erc1155)
				}
				asset_id => GenericAssetId::try_from(asset_id).map(Self::Generic),
			}
		}
	}

	#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
	pub struct Slip44Reference(BoundedVec<u8, ConstU32<MAXIMUM_REFERENCE_LENGTH_U32>>);

	// Values taken from https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-20.md
	impl Slip44Reference {
		#[allow(dead_code)]
		pub(crate) fn from_slice_unchecked(slice: &[u8]) -> Self {
			Self(slice.to_vec().try_into().unwrap())
		}
	}

	impl TryFrom<&[u8]> for Slip44Reference {
		type Error = AssetIdError;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			let input_length = value.len();
			if input_length < MINIMUM_REFERENCE_LENGTH {
				Err(AssetIdError::Reference(ReferenceError::TooShort))
			} else if input_length > MAXIMUM_REFERENCE_LENGTH {
				Err(AssetIdError::Reference(ReferenceError::TooLong))
			} else {
				value.iter().try_for_each(|c| {
					if !(b'0'..=b'9').contains(c) {
						Err(AssetIdError::Reference(ReferenceError::InvalidCharacter))
					} else {
						Ok(())
					}
				})?;
				// Unchecked since we already checked for length
				Ok(Self::from_slice_unchecked(value))
			}
		}
	}

	#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
	pub struct EvmSmartContractFungibleReference([u8; EVM_SMART_CONTRACT_ADDRESS_LENGTH]);

	// Values taken from https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-20.md
	impl EvmSmartContractFungibleReference {
		#[allow(dead_code)]
		pub(crate) fn from_slice_unchecked(slice: &[u8]) -> Self {
			Self(slice.try_into().unwrap())
		}
	}

	impl TryFrom<&[u8]> for EvmSmartContractFungibleReference {
		type Error = AssetIdError;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			match value {
				// If the prefix is "0x" => parse the address
				[b'0', b'x', contract_address @ ..] => {
					let inner: [u8; EVM_SMART_CONTRACT_ADDRESS_LENGTH] =
						contract_address.try_into().map_err(|_| AssetIdError::InvalidFormat)?;
					inner.iter().try_for_each(|c| {
						if !matches!(c, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F') {
							Err(AssetIdError::Reference(ReferenceError::InvalidCharacter))
						} else {
							Ok(())
						}
					})?;
					// Unchecked since we already checked for length
					Ok(Self::from_slice_unchecked(contract_address))
				}
				// Otherwise fail
				_ => Err(AssetIdError::InvalidFormat),
			}
		}
	}

	#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
	pub struct EvmSmartContractNonFungibleReference(
		EvmSmartContractFungibleReference,
		Option<EvmSmartContractNonFungibleIdentifier>,
	);

	impl EvmSmartContractNonFungibleReference {
		#[allow(dead_code)]
		pub(crate) fn from_raw_unchecked(reference: &[u8], id: Option<&[u8]>) -> Self {
			Self(reference.try_into().unwrap(), id.map(|id| id.try_into().unwrap()))
		}
	}

	impl TryFrom<&[u8]> for EvmSmartContractNonFungibleReference {
		type Error = AssetIdError;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			let mut components = value.split(|c| *c == b':');

			let reference = components
				.next()
				.ok_or(AssetIdError::InvalidFormat)
				.and_then(EvmSmartContractFungibleReference::try_from)?;

			let id = components
				.next()
				// Transform Option<Result> to Result<Option> and bubble Err case up, keeping Ok(Option) for successful
				// cases.
				.map_or(Ok(None), |id| {
					EvmSmartContractNonFungibleIdentifier::try_from(id).map(Some)
				})?;
			Ok(Self(reference, id))
		}
	}

	#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
	pub struct EvmSmartContractNonFungibleIdentifier(BoundedVec<u8, ConstU32<MAXIMUM_IDENTIFIER_LENGTH_U32>>);

	impl EvmSmartContractNonFungibleIdentifier {
		#[allow(dead_code)]
		pub(crate) fn from_slice_unchecked(value: &[u8]) -> Self {
			Self(value.to_vec().try_into().unwrap())
		}
	}

	impl TryFrom<&[u8]> for EvmSmartContractNonFungibleIdentifier {
		type Error = AssetIdError;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			let input_length = value.len();
			if input_length < MINIMUM_IDENTIFIER_LENGTH {
				Err(AssetIdError::Identifier(IdentifierError::TooShort))
			} else if input_length > MAXIMUM_IDENTIFIER_LENGTH {
				Err(AssetIdError::Identifier(IdentifierError::TooLong))
			} else {
				value.iter().try_for_each(|c| {
					if !matches!(c, b'0'..=b'9') {
						Err(AssetIdError::Identifier(IdentifierError::InvalidCharacter))
					} else {
						Ok(())
					}
				})?;
				value
					.to_vec()
					.try_into()
					.map(Self)
					.map_err(|_| AssetIdError::InvalidFormat)
			}
		}
	}

	#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
	pub struct GenericAssetId {
		pub namespace: GenericAssetNamespace,
		pub reference: GenericAssetReference,
		pub id: Option<GenericAssetIdentifier>,
	}

	impl GenericAssetId {
		#[allow(dead_code)]
		fn from_raw_unchecked(namespace: &[u8], reference: &[u8], id: Option<&[u8]>) -> Self {
			Self {
				namespace: GenericAssetNamespace::from_slice_unchecked(namespace),
				reference: GenericAssetReference::from_slice_unchecked(reference),
				id: id.map(GenericAssetIdentifier::from_slice_unchecked).or(None),
			}
		}
	}

	impl TryFrom<&[u8]> for GenericAssetId {
		type Error = AssetIdError;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			let input_length = value.len();
			if input_length > MAXIMUM_NAMESPACE_LENGTH + MAXIMUM_REFERENCE_LENGTH + MAXIMUM_IDENTIFIER_LENGTH + 2 {
				return Err(AssetIdError::InvalidFormat);
			}

			let mut components = value.split(|c| *c == b':');

			let namespace = components
				.next()
				.ok_or(AssetIdError::InvalidFormat)
				.and_then(GenericAssetNamespace::try_from)?;
			let reference = components
				.next()
				.ok_or(AssetIdError::InvalidFormat)
				.and_then(GenericAssetReference::try_from)?;
			let id = components
				.next()
				// Transform Option<Result> to Result<Option> and bubble Err case up, keeping Ok(Option) for successful
				// cases.
				.map_or(Ok(None), |id| GenericAssetIdentifier::try_from(id).map(Some))?;

			Ok(Self {
				namespace,
				reference,
				id,
			})
		}
	}

	#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
	pub struct GenericAssetNamespace(BoundedVec<u8, ConstU32<MAXIMUM_NAMESPACE_LENGTH_U32>>);

	impl GenericAssetNamespace {
		fn from_slice_unchecked(value: &[u8]) -> Self {
			Self(value.to_vec().try_into().unwrap())
		}
	}

	impl TryFrom<&[u8]> for GenericAssetNamespace {
		type Error = AssetIdError;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			let input_length = value.len();
			if input_length < MINIMUM_NAMESPACE_LENGTH {
				Err(AssetIdError::Namespace(NamespaceError::TooShort))
			} else if input_length > MAXIMUM_NAMESPACE_LENGTH {
				Err(AssetIdError::Namespace(NamespaceError::TooLong))
			} else {
				value.iter().try_for_each(|c| {
					if !matches!(c, b'-' | b'a'..=b'z' | b'0'..=b'9') {
						Err(AssetIdError::Namespace(NamespaceError::InvalidCharacter))
					} else {
						Ok(())
					}
				})?;
				// Unchecked since we already checked for length
				Ok(Self::from_slice_unchecked(value))
			}
		}
	}

	#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
	pub struct GenericAssetReference(BoundedVec<u8, ConstU32<MAXIMUM_REFERENCE_LENGTH_U32>>);

	impl GenericAssetReference {
		fn from_slice_unchecked(value: &[u8]) -> Self {
			Self(value.to_vec().try_into().unwrap())
		}
	}

	impl TryFrom<&[u8]> for GenericAssetReference {
		type Error = AssetIdError;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			let input_length = value.len();
			if input_length < MINIMUM_REFERENCE_LENGTH {
				Err(AssetIdError::Reference(ReferenceError::TooShort))
			} else if input_length > MAXIMUM_REFERENCE_LENGTH {
				Err(AssetIdError::Reference(ReferenceError::TooLong))
			} else {
				value.iter().try_for_each(|c| {
					if !matches!(c, b'-' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9') {
						Err(AssetIdError::Reference(ReferenceError::InvalidCharacter))
					} else {
						Ok(())
					}
				})?;
				// Unchecked since we already checked for length
				Ok(Self::from_slice_unchecked(value))
			}
		}
	}

	#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
	pub struct GenericAssetIdentifier(BoundedVec<u8, ConstU32<MAXIMUM_IDENTIFIER_LENGTH_U32>>);

	impl GenericAssetIdentifier {
		fn from_slice_unchecked(value: &[u8]) -> Self {
			Self(value.to_vec().try_into().unwrap())
		}
	}

	impl TryFrom<&[u8]> for GenericAssetIdentifier {
		type Error = AssetIdError;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			let input_length = value.len();
			if input_length < MINIMUM_IDENTIFIER_LENGTH {
				Err(AssetIdError::Identifier(IdentifierError::TooShort))
			} else if input_length > MAXIMUM_IDENTIFIER_LENGTH {
				Err(AssetIdError::Identifier(IdentifierError::TooLong))
			} else {
				value.iter().try_for_each(|c| {
					if !matches!(c, b'-' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9') {
						Err(AssetIdError::Identifier(IdentifierError::InvalidCharacter))
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
				assert!(
					AssetId::try_from(asset.as_bytes()).is_ok(),
					"Asset ID {:?} should not fail to parse for slip44 assets",
					asset
				);
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
				"slip44:99999999999999999999999999999999999999999999999999999999999999999",
			];
			for asset in invalid_assets {
				assert!(
					AssetId::try_from(asset.as_bytes()).is_err(),
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
				assert!(
					AssetId::try_from(asset.as_bytes()).is_ok(),
					"Asset ID {:?} should not fail to parse for erc20 assets",
					asset
				);
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
				// Max chars - 1
				"erc20:0x8f8221AFBB33998D8584A2B05749BA73C37A938",
				// Max chars + 1
				"erc20:0x8f8221AFBB33998D8584A2B05749BA73C37A938A1",
				// Asset ID (not supported for erc20 standard)
				"erc20:0x8f8221AFBB33998D8584A2B05749BA73C37A938A1:1",
			];
			for asset in invalid_assets {
				assert!(
					AssetId::try_from(asset.as_bytes()).is_err(),
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
				assert!(
					AssetId::try_from(asset.as_bytes()).is_ok(),
					"Asset ID {:?} should not fail to parse for erc721 assets",
					asset
				);
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
		];
			for asset in invalid_assets {
				assert!(
					AssetId::try_from(asset.as_bytes()).is_err(),
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
				assert!(
					AssetId::try_from(asset.as_bytes()).is_ok(),
					"Asset ID {:?} should not fail to parse for erc1155 assets",
					asset
				);
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
		];
			for asset in invalid_assets {
				assert!(
					AssetId::try_from(asset.as_bytes()).is_err(),
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
			"12345678:-abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-:-abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ012345678901234567890123-",
			"para:411f057b9107718c9624d6aa4a3f23c1",
			"para:kilt-spiritnet",
			"w3n:john-doe",
		];

			for asset in valid_assets {
				assert!(
					AssetId::try_from(asset.as_bytes()).is_ok(),
					"Asset ID {:?} should not fail to parse for generic assets",
					asset
				);
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
				"valid:too-loooooooooooooooooooooooooooooooooooooooooooooooooooooooooong",
				"valid:valid:too-loooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong",
				// Wrong characters
				"no-val!d:valid",
				"valid:no-val!d",
				"valid:valid:no-val!d",
			];
			for asset in invalid_assets {
				assert!(
					AssetId::try_from(asset.as_bytes()).is_err(),
					"Asset ID {:?} should fail to parse for generic assets",
					asset
				);
			}
		}
	}
}
