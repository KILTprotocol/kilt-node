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

use crate::{AccountIdOf, BalanceOf, Config};

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, PartialOrd, Ord, TypeInfo)]
pub struct Claim<CtypeHash, SubjectIdentifier, Content> {
	pub ctype_hash: CtypeHash,
	pub subject: SubjectIdentifier,
	pub contents: Content,
}

// TODO: Add support for delegation and claimer's signature.
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, PartialOrd, Ord, TypeInfo)]
pub struct Credential<CtypeHash, SubjectIdentifier, ClaimContent, ClaimHash, Nonce> {
	pub claim: Claim<CtypeHash, SubjectIdentifier, ClaimContent>,
	pub nonce: Nonce,
	pub claim_hash: ClaimHash,
}

#[derive(Encode, Decode, Clone, MaxEncodedLen, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct CredentialEntry<T: Config> {
	pub block_number: T::BlockNumber,
	pub deposit: Deposit<AccountIdOf<T>, BalanceOf<T>>,
}
