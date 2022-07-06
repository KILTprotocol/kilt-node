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

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::{BoundedVec, traits::{Currency, Get}};

use attestation::ClaimHashOf;
use ctype::CtypeHashOf;
use kilt_support::traits::{DefaultForLength, GenerateBenchmarkOrigin};

use crate::*;

const SEED: u32 = 0;

fn generate_base_public_credential_creation_op<T: Config>(
	subject_id: BoundedVec<u8, T::MaxSubjectIdLength>,
	claim_hash: ClaimHashOf<T>,
	ctype_hash: CtypeHashOf<T>,
	contents: BoundedVec<u8, T::MaxEncodedClaimsLength>,
	claimer_signature: Option<ClaimerSignatureInfo<T::ClaimerIdentifier, T::ClaimerSignature>>,
) -> CredentialOf<T> {
	CredentialOf::<T> {
		claim: Claim {
			ctype_hash,
			subject: subject_id,
			contents,
		},
		claim_hash,
		claimer_signature,
		nonce: Default::default(),
		authorization_info: Default::default(),
	}
}

#[cfg(test)]
impl<T: Config> DefaultForLength for TestSubjectId {
	// Copied over from the AssetDid implementation, as this pallet does not depend on that.
	fn get_default(length: usize) -> Self {
		// Minimum length is 3 for namespace and 1 for reference
		// https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md
		// Minimum length is 3 for namespace and 1 for reference
		// https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-19.md
		const BASE_ID: &[u8] = b"did:asset:cns:c.asn:a";
		const BASE_LENGTH: usize = BASE_ID.len();
		assert!(length > BASE_LENGTH, "{}", format!(
			"The provided input value {} was not large enough to cover the minimum default case of {}.",
			length,
			BASE_LENGTH
		));
		let remaining_length_for_asset_id = length - BASE_LENGTH;
		// Pad the remaining space with 0s
		let asset_did = [BASE_ID, &vec![b'0'; remaining_length_for_asset_id][..]].concat();
		Self::try_from(asset_did).expect("Asset DID creation failed for the length provided (most likely value too large).")
	}
}

benchmarks! {
	where_clause {
		where
		T: core::fmt::Debug,
		T: Config,
		T: attestation::Config,
		T: ctype::Config<CtypeCreatorId = T::AttesterId>,
		<T as Config>::EnsureOrigin: GenerateBenchmarkOrigin<T::Origin, T::AccountId, T::AttesterId>,
		<T as Config>::SubjectId: DefaultForLength + Into<BoundedVec<u8, T::MaxSubjectIdLength>> + sp_std::fmt::Debug,
	}

	add {
		// Minimum length for a valid asset DID is `did:asset:cns:c:ans:a:0` = 23
		let n in 23 .. T::MaxSubjectIdLength::get();
		let c in 1 .. T::MaxEncodedClaimsLength::get();
		let sender: T::AccountId = account("sender", 0, SEED);
		let attester: T::AttesterId = account("attester", 0, SEED);
		let claim_hash: T::Hash = T::Hash::default();
		let ctype_hash: T::Hash = T::Hash::default();
		let subject_id = <T as Config>::SubjectId::get_default(n as usize);
		let contents = BoundedVec::try_from(vec![0; c as usize]).expect("Contents should not fail.");

		let creation_op = Box::new(generate_base_public_credential_creation_op::<T>(
			subject_id.into(),
			claim_hash,
			ctype_hash,
			contents,
			None,
		));

		ctype::Ctypes::<T>::insert(&ctype_hash, attester.clone());
		CurrencyOf::<T>::make_free_balance_be(&sender, <T as attestation::Config>::Deposit::get() + <T as attestation::Config>::Deposit::get() + <T as Config>::Deposit::get());
		let origin = <T as Config>::EnsureOrigin::generate_origin(sender, attester);
	}: _<T::Origin>(origin, creation_op)
	verify {}
}

impl_benchmark_test_suite! {
	Pallet,
	crate::mock::ExtBuilder::default().build_with_keystore(),
	crate::mock::Test
}
