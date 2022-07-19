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

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

use frame_support::RuntimeDebug;

use kilt_support::deposit::Deposit;

use crate::{BalanceOf, Config};

/// The bulk of the credential, i.e., its (encoded) claims, subject, and Ctype.
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, PartialOrd, Ord, TypeInfo)]
pub struct Claim<CtypeHash, SubjectIdentifier, Content> {
	/// The Ctype of the credential.
	pub ctype_hash: CtypeHash,
	/// The credential subject ID as specified by the attester.
	pub subject: SubjectIdentifier,
	/// The credential claims.
	pub contents: Content,
}

/// The type of a claimer's signature to prove the claimer's involvement in the
/// public credential issuance process.
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, PartialOrd, Ord, TypeInfo)]
pub struct ClaimerSignatureInfo<ClaimerIdentifier, Signature> {
	/// The identifier of the claimer.
	pub claimer_id: ClaimerIdentifier,
	/// The signature of the claimer.
	pub signature_payload: Signature,
}

/// The type of a credentials as incoming from the outside world.
/// Some of its fields are parsed and/or transformed inside the `add` operation.
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, PartialOrd, Ord, TypeInfo)]
pub struct Credential<
	CtypeHash,
	SubjectIdentifier,
	ClaimContent,
	ClaimHash,
	Nonce,
	ClaimerSignature,
	AuthorizationControl,
> {
	/// The credential content.
	pub claim: Claim<CtypeHash, SubjectIdentifier, ClaimContent>,
	/// The nonce used to generate the root hash.
	pub nonce: Nonce,
	/// The root hash of the credential claims.
	pub claim_hash: ClaimHash,
	/// The claimer's signature information.
	pub claimer_signature: Option<ClaimerSignature>,
	/// The authorization info to attest the credential.
	pub authorization_info: Option<AuthorizationControl>,
}

/// The entry in the blockchain state corresponding to a successful public
/// credential attestation. It is meant to be an index for clients to query the
/// block number in which a tx for a credential creation was included in a
/// block. The block number is used to query the full content of the credential
/// from archive nodes.
#[derive(Encode, Decode, Clone, MaxEncodedLen, RuntimeDebug, PartialEq, Eq, PartialOrd, Ord, TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "std", serde(bound = ""))]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct CredentialEntryOf<T: Config> {
	/// The block number in which the credential tx was evaluated and included
	/// in the block.
	pub block_number: T::BlockNumber,
	/// The info about the credential deposit.
	pub deposit: Deposit<T::AccountId, BalanceOf<T>>,
}
