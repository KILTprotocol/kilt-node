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

use crate::{
	account::{AccountId20, EthereumSignature},
	linkable_account::LinkableAccountId,
	signature::get_wrapped_payload,
};

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::Verify, AccountId32, MultiSignature};

#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum AssociateAccountRequest {
	Dotsama(AccountId32, MultiSignature),
	Ethereum(AccountId20, EthereumSignature),
}

impl AssociateAccountRequest {
	pub fn verify<T: crate::Config>(
		&self,
		did_identifier: <T as crate::Config>::DidIdentifier,
		expiration: <T as frame_system::Config>::BlockNumber,
	) -> bool {
		let encoded_payload = (&did_identifier, expiration).encode();
		match self {
			AssociateAccountRequest::Dotsama(acc, proof) => proof.verify(
				&get_wrapped_payload(&encoded_payload[..], crate::signature::WrapType::Substrate)[..],
				acc,
			),
			AssociateAccountRequest::Ethereum(acc, proof) => proof.verify(
				&get_wrapped_payload(&encoded_payload[..], crate::signature::WrapType::Ethereum)[..],
				acc,
			),
		}
	}

	pub fn get_linkable_account(&self) -> LinkableAccountId {
		match self {
			AssociateAccountRequest::Dotsama(acc, _) => LinkableAccountId::AccountId32(acc.clone()),
			AssociateAccountRequest::Ethereum(acc, _) => LinkableAccountId::AccountId20(*acc),
		}
	}
}
