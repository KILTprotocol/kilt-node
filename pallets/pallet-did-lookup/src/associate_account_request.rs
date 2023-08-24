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

use crate::{
	account::{AccountId20, EthereumSignature},
	linkable_account::LinkableAccountId,
	signature::get_wrapped_payload,
};

use base58::ToBase58;
use blake2::{Blake2b512, Digest};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::{
	prelude::{format, string::String},
	TypeInfo,
};
use sp_runtime::{traits::Verify, AccountId32, MultiSignature};
use sp_std::{fmt::Debug, vec, vec::Vec};

#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum AssociateAccountRequest {
	Polkadot(AccountId32, MultiSignature),
	Ethereum(AccountId20, EthereumSignature),
}

impl AssociateAccountRequest {
	pub fn verify<DidIdentifier: AsRef<[u8]>, BlockNumber: Debug>(
		&self,
		did_identifier: &DidIdentifier,
		expiration: BlockNumber,
	) -> bool {
		let encoded_payload = get_challenge(did_identifier, expiration).into_bytes();
		match self {
			AssociateAccountRequest::Polkadot(acc, proof) => proof.verify(
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
			AssociateAccountRequest::Polkadot(acc, _) => LinkableAccountId::AccountId32(acc.clone()),
			AssociateAccountRequest::Ethereum(acc, _) => LinkableAccountId::AccountId20(*acc),
		}
	}
}

/// Build the challenge that must be signed to prove the consent for an
/// account to be linked to a DID.
pub fn get_challenge<DidIdentifier: AsRef<[u8]>, BlockNumber: Debug>(
	did_identifier: &DidIdentifier,
	expiration: BlockNumber,
) -> String {
	format!(
		"Publicly link the signing address to did:kilt:{} before block number {:?}",
		to_ss58(did_identifier.as_ref(), 38),
		expiration
	)
}

// Copied from https://github.com/paritytech/substrate/blob/ad5399644aebc54e32a107ac37ae08e6cd1f0cfb/primitives/core/src/crypto.rs#L324
// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0
fn to_ss58(public_key: &[u8], prefix: u16) -> String {
	// We mask out the upper two bits of the ident - SS58 Prefix currently only
	// supports 14-bits
	let ident: u16 = prefix & 0b0011_1111_1111_1111;
	let mut v = match ident {
		0..=63 => vec![ident as u8],
		64..=16_383 => {
			// upper six bits of the lower byte(!)
			let first = ((ident & 0b0000_0000_1111_1100) as u8) >> 2;
			// lower two bits of the lower byte in the high pos,
			// lower bits of the upper byte in the low pos
			let second = ((ident >> 8) as u8) | ((ident & 0b0000_0000_0000_0011) as u8) << 6;
			vec![first | 0b01000000, second]
		}
		_ => unreachable!("masked out the upper two bits; qed"),
	};
	v.extend(public_key);
	let r = ss58hash(&v);
	v.extend(&r[0..2]);
	v.to_base58()
}

const PREFIX: &[u8] = b"SS58PRE";

fn ss58hash(data: &[u8]) -> Vec<u8> {
	let mut ctx = Blake2b512::new();
	ctx.update(PREFIX);
	ctx.update(data);
	ctx.finalize().to_vec()
}

#[cfg(test)]
mod tests {
	use super::get_challenge;

	#[test]
	fn test_get_challenge() {
		assert_eq!(
			get_challenge(&[1u8; 32], 5),
			"Publicly link the signing address to did:kilt:4nwPAmtsK5toZfBM9WvmAe4Fa3LyZ3X3JHt7EUFfrcPPAZAm before block number 5"
		);
	}
}
