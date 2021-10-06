// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

use crate::*;

/// All the errors that can be generated when validating a DID operation.
#[derive(Debug, Eq, PartialEq)]
pub enum DidError {
	/// See [StorageError].
	StorageError(StorageError),
	/// See [SignatureError].
	SignatureError(SignatureError),
	/// See [InputError].
	InputError(InputError),
	/// An error that is not supposed to take place, yet it happened.
	InternalError,
}

impl From<StorageError> for DidError {
	fn from(err: StorageError) -> Self {
		DidError::StorageError(err)
	}
}

impl From<InputError> for DidError {
	fn from(err: InputError) -> Self {
		DidError::InputError(err)
	}
}

/// Error involving the pallet's storage.
#[derive(Debug, Eq, PartialEq)]
pub enum StorageError {
	/// The DID being created is already present on chain.
	DidAlreadyPresent,
	/// The expected DID cannot be found on chain.
	DidNotPresent,
	/// The given DID does not contain the right key to verify the signature
	/// of a DID operation.
	DidKeyNotPresent(DidVerificationKeyRelationship),
	/// At least one key referenced is not stored under the given DID.
	KeyNotPresent,
	/// The user tries to delete a verification key that is currently being
	/// used to authorize operations, and this is not allowed.
	CurrentlyActiveKey,
	/// The maximum number of public keys for this DID key identifier has
	/// been reached.
	MaxPublicKeysPerDidExceeded,
	/// The maximum number of key agreements has been reached for the DID
	/// subject.
	MaxTotalKeyAgreementKeysExceeded,
	/// The DID has already been previously deleted.
	DidAlreadyDeleted,
}

/// Error generated when validating a DID operation.
#[derive(Debug, Eq, PartialEq)]
pub enum SignatureError {
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
#[derive(Debug, Eq, PartialEq)]
pub enum InputError {
	/// A number of new key agreement keys greater than the maximum allowed has
	/// been provided.
	MaxKeyAgreementKeysLimitExceeded,
	/// A number of new verification keys to remove greater than the maximum
	/// allowed has been provided.
	MaxVerificationKeysToRemoveLimitExceeded,
}
