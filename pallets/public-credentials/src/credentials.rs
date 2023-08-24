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

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

use frame_support::RuntimeDebug;

use kilt_support::deposit::Deposit;

/// The type of a credentials as incoming from the outside world.
/// Some of its fields are parsed and/or transformed inside the `add` operation.
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, PartialOrd, Ord, TypeInfo)]
pub struct Credential<CtypeHash, SubjectIdentifier, Claims, AccessControl> {
	/// The Ctype of the credential.
	pub ctype_hash: CtypeHash,
	/// The credential subject ID as specified by the attester.
	pub subject: SubjectIdentifier,
	/// The credential claims.
	pub claims: Claims,
	/// The access control logic to authorize the creation operation.
	pub authorization: Option<AccessControl>,
}

/// The entry in the blockchain state corresponding to a successful public
/// credential attestation. It is meant to be an index for clients to query the
/// block number in which a tx for a credential creation was included in a
/// block. The block number is used to query the full content of the credential
/// from archive nodes.
#[derive(Encode, Decode, Clone, MaxEncodedLen, RuntimeDebug, PartialEq, Eq, PartialOrd, Ord, TypeInfo)]
pub struct CredentialEntry<CTypeHash, Attester, BlockNumber, AccountId, Balance, AuthorizationId> {
	/// The hash of the CType used for this attestation.
	pub ctype_hash: CTypeHash,
	/// The attester of the credential.
	pub attester: Attester,
	/// A flag indicating the revocation status of the credential.
	pub revoked: bool,
	/// The block number in which the credential tx was evaluated and included
	/// in the block.
	pub block_number: BlockNumber,
	/// The info about the credential deposit.
	pub deposit: Deposit<AccountId, Balance>,
	/// The ID of the authorization information (e.g., a delegation node) used
	/// to authorize the operation.
	pub authorization_id: Option<AuthorizationId>,
}
