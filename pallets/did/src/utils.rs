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
	did_details::{DidCreationDetails, DidPublicKey},
	BalanceOf, Config, KeyIdOf,
};
use parity_scale_codec::Encode;
use sp_core::Get;
use sp_runtime::traits::{Hash, Zero};
use sp_std::vec::Vec;

pub fn calculate_key_id<T: Config>(key: &DidPublicKey) -> KeyIdOf<T> {
	let hashed_values: Vec<u8> = key.encode();
	T::Hashing::hash(&hashed_values)
}

pub fn calculate_deposit<T: Config>(details: &DidCreationDetails<T>) -> BalanceOf<T>
where
	BalanceOf<T>: From<u32>,
{
	let mut deposit: BalanceOf<T> = T::BaseDeposit::get();

	let count_service_endpoint: BalanceOf<T> = (details.new_service_details.len() as u32).into();
	deposit += count_service_endpoint * T::DepositServiceEndpoint::get();

	let count_key_agreements: BalanceOf<T> = (details.new_key_agreement_keys.len() as u32).into();
	deposit += count_key_agreements * T::DepositKey::get();

	deposit += match details.new_attestation_key {
		Some(_) => T::DepositKey::get(),
		_ => Zero::zero(),
	};

	deposit += match details.new_delegation_key {
		Some(_) => T::DepositKey::get(),
		_ => Zero::zero(),
	};

	deposit
}

/// Verifies that an input string contains only traditional (non-extended) ASCII
/// characters.
pub(crate) fn is_valid_ascii_string(input: &str) -> bool {
	input.chars().all(|c| c.is_ascii())
}

#[test]
fn check_is_valid_ascii_string() {
	let test_cases = [
		("kilt.io", true),
		("super.long.domain.com:12345/path/to/directory#fragment?arg=value", true),
		("super.long.domain.com:12345/path/to/directory/file.txt", true),
		("domain.with.only.valid.characters.:/?#[]@!$&'()*+,;=-._~", true),
		("invalid.châracter.domain.org", false),
		("âinvalid.character.domain.org", false),
		("invalid.character.domain.orgâ", false),
		("", true),
		("例子.領域.cn", false),
		("kilt.io/%3Ctag%3E/encoded_upper_case_ascii.com", true),
		("kilt.io/%3ctag%3e/encoded_lower_case_ascii.com", true),
	];

	test_cases.iter().for_each(|(input, expected_result)| {
		assert_eq!(
			is_valid_ascii_string(input),
			*expected_result,
			"Test case for \"{}\" returned wrong result.",
			input
		);
	});
}
