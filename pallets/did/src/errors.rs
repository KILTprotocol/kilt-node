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

use scale_info::TypeInfo;

use crate::did_details::DidVerificationKeyRelationship;

/// All the errors that can be generated when validating a DID operation.
#[derive(Debug, Eq, PartialEq, TypeInfo)]
pub enum Error {
	/// See [StorageError].
	Storage(Storage),
	/// See [SignatureError].
	Signature(Signature),
	/// See [InputError].
	Input(Input),
	/// An error that is not supposed to take place, yet it happened.
	Internal,
}

impl From<Storage> for Error {
	fn from(err: Storage) -> Self {
		Error::Storage(err)
	}
}

impl From<Input> for Error {
	fn from(err: Input) -> Self {
		Error::Input(err)
	}
}

/// Error involving the pallet's storage.
#[derive(Debug, Eq, PartialEq, TypeInfo)]
pub enum Storage {
	/// The DID being created is already present on chain.
	AlreadyExists,
	/// The expected DID cannot be found on chain.
	NotFound,
	/// The given DID does not contain the right key to verify the signature
	/// of a DID operation.
	DidKeyNotFound(DidVerificationKeyRelationship),
	/// At least one key referenced is not stored under the given DID.
	KeyNotFound,
	/// The maximum number of public keys for this DID key identifier has
	/// been reached.
	MaxPublicKeysExceeded,
	/// The maximum number of key agreements has been reached for the DID
	/// subject.
	MaxTotalKeyAgreementKeysExceeded,
	/// The DID has already been previously deleted.
	AlreadyDeleted,
}

/// Error generated when validating a DID operation.
#[derive(Debug, Eq, PartialEq, TypeInfo)]
pub enum Signature {
	/// The signature is not in the format the verification key expects.
	InvalidSignatureFormat,
	/// The signature is invalid for the payload and the verification key
	/// provided.
	InvalidSignature,
	/// The operation nonce is not equal to the current DID nonce + 1.
	InvalidNonce,
	/// The provided operation block number is not valid.
	TransactionExpired,
}

/// Error generated when some extrinsic input does not respect the pallet's
/// constraints.
#[derive(Debug, Eq, PartialEq, TypeInfo)]
pub enum Input {
	/// A number of new key agreement keys greater than the maximum allowed has
	/// been provided.
	MaxKeyAgreementKeysLimitExceeded,
	/// The maximum number of service endpoints for a DID has been exceeded.
	MaxServicesCountExceeded,
	/// The maximum number of URLs for a service endpoint has been exceeded.
	MaxUrlCountExceeded,
	/// The maximum number of types for a service endpoint has been exceeded.
	MaxTypeCountExceeded,
	/// The service endpoint ID exceeded the maximum allowed length.
	MaxIdLengthExceeded,
	/// One of the service endpoint URLs exceeded the maximum allowed length.
	MaxUrlLengthExceeded,
	/// One of the service endpoint types exceeded the maximum allowed length.
	MaxTypeLengthExceeded,
	/// One of the service endpoint details contains non-ASCII characters.
	InvalidEncoding,
}
