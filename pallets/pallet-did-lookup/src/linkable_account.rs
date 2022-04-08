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
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
use sp_runtime::{AccountId32, MultiSignature, MultiSigner};

use crate::{
	account::{AccountId20, EthereumSignature, EthereumSigner},
	signature,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum LinkableAccountId {
	AccountId20(AccountId20),
	AccountId32(AccountId32),
}

impl From<AccountId20> for LinkableAccountId {
	fn from(account_id: AccountId20) -> Self {
		Self::AccountId20(account_id)
	}
}

impl From<AccountId32> for LinkableAccountId {
	fn from(account_id: AccountId32) -> Self {
		Self::AccountId32(account_id)
	}
}

impl signature::GetWrapType for LinkableAccountId {
	fn get_wrap_type(&self) -> signature::WrapType {
		match self {
			Self::AccountId20(_) => signature::WrapType::Ethereum,
			Self::AccountId32(_) => signature::WrapType::Substrate,
		}
	}
}

impl signature::GetWrapType for AccountId32 {
	fn get_wrap_type(&self) -> signature::WrapType {
		signature::WrapType::Substrate
	}
}

impl signature::GetWrapType for AccountId20 {
	fn get_wrap_type(&self) -> signature::WrapType {
		signature::WrapType::Ethereum
	}
}

#[cfg(feature = "std")]
impl std::fmt::Display for LinkableAccountId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::AccountId20(account_id) => write!(f, "{}", account_id),
			Self::AccountId32(account_id) => write!(f, "{}", account_id),
		}
	}
}

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum LinkableAccountSignature {
	MultiSignature(MultiSignature),
	EthereumSignature(EthereumSignature),
}

impl From<MultiSignature> for LinkableAccountSignature {
	fn from(signature: MultiSignature) -> Self {
		Self::MultiSignature(signature)
	}
}

impl From<EthereumSignature> for LinkableAccountSignature {
	fn from(signature: EthereumSignature) -> Self {
		Self::EthereumSignature(signature)
	}
}

impl From<sp_core::sr25519::Signature> for LinkableAccountSignature {
	fn from(signature: sp_core::sr25519::Signature) -> Self {
		Self::MultiSignature(signature.into())
	}
}

impl sp_runtime::traits::Verify for LinkableAccountSignature {
	type Signer = LinkableAccountSigner;
	fn verify<L: sp_runtime::traits::Lazy<[u8]>>(&self, msg: L, signer: &LinkableAccountId) -> bool {
		match self {
			LinkableAccountSignature::MultiSignature(sig) => match signer {
				LinkableAccountId::AccountId32(id) => sig.verify(msg, id),
				LinkableAccountId::AccountId20(_) => false,
			},
			LinkableAccountSignature::EthereumSignature(sig) => match signer {
				LinkableAccountId::AccountId20(id) => sig.verify(msg, id),
				LinkableAccountId::AccountId32(_) => false,
			},
		}
	}
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum LinkableAccountSigner {
	MultiSigner(MultiSigner),
	EthereumSigner(EthereumSigner),
}

impl sp_runtime::traits::IdentifyAccount for LinkableAccountSigner {
	type AccountId = LinkableAccountId;
	fn into_account(self) -> LinkableAccountId {
		match self {
			Self::MultiSigner(signer) => LinkableAccountId::AccountId32(signer.into_account()),
			Self::EthereumSigner(signer) => LinkableAccountId::AccountId20(signer.into_account()),
		}
	}
}
