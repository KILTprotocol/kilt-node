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

use kilt_support::Deposit;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// An on-chain attestation written by an attester.
#[derive(Clone, Debug, Encode, Decode, Eq, PartialEq, TypeInfo, MaxEncodedLen)]
pub struct AttestationDetails<CtypeHash, AttesterId, AuthorizationId, AccountId, Balance> {
	/// The hash of the CType used for this attestation.
	pub ctype_hash: CtypeHash,
	/// The ID of the attester.
	pub attester: AttesterId,
	/// \[OPTIONAL\] The ID of the delegation node used to authorize the
	/// attester.
	pub authorization_id: Option<AuthorizationId>,
	/// The flag indicating whether the attestation has been revoked or not.
	pub revoked: bool,
	/// The deposit that was taken to incentivise fair use of the on chain
	/// storage.
	pub deposit: Deposit<AccountId, Balance>,
}

#[cfg(test)]
mod tests {
	use ctype::CtypeHashOf;

	use super::*;
	use crate::{mock::*, AccountIdOf, AttestationDetailsOf, AttesterOf, BalanceOf};

	type OldAttestationDetailsOf<Test> =
		OldAttestationDetails<CtypeHashOf<Test>, AttesterOf<Test>, AccountIdOf<Test>, BalanceOf<Test>>;
	/// Old Attestation
	#[derive(Clone, Debug, Encode, Decode, Eq, PartialEq, TypeInfo, MaxEncodedLen)]

	pub struct OldAttestationDetails<CtypeHash, Attester, AccountId, Balance> {
		/// The hash of the CType used for this attestation.
		pub ctype_hash: CtypeHash,
		/// The ID of the attester.
		pub attester: Attester,
		/// \[OPTIONAL\] The ID of the delegation node used to authorize the
		/// attester.
		pub delegation_id: Option<[u8; 32]>,
		/// The flag indicating whether the attestation has been revoked or not.
		pub revoked: bool,
		/// The deposit that was taken to incentivise fair use of the on chain
		/// storage.
		pub deposit: Deposit<AccountId, Balance>,
	}

	#[test]
	fn test_no_need_to_migrate_if_none() {
		let old = OldAttestationDetailsOf::<Test> {
			ctype_hash: claim_hash_from_seed(CLAIM_HASH_SEED_01),
			attester: sr25519_did_from_seed(&ALICE_SEED),
			delegation_id: None,
			revoked: true,
			deposit: Deposit {
				owner: ACCOUNT_00,
				amount: ATTESTATION_DEPOSIT,
			},
		};
		let encoded = old.encode();

		let new = AttestationDetailsOf::<Test>::decode(&mut &encoded[..]);
		assert_eq!(
			new,
			Ok(AttestationDetailsOf::<Test> {
				ctype_hash: claim_hash_from_seed(CLAIM_HASH_SEED_01),
				attester: sr25519_did_from_seed(&ALICE_SEED),
				authorization_id: None,
				revoked: true,
				deposit: Deposit {
					owner: ACCOUNT_00,
					amount: ATTESTATION_DEPOSIT,
				},
			})
		);
	}
}
