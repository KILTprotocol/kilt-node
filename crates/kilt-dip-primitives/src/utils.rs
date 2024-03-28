// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

/// The output of a type implementing the [`sp_runtime::traits::Hash`] trait.
pub type OutputOf<Hasher> = <Hasher as sp_runtime::traits::Hash>::Output;

pub(crate) use calculate_parachain_head_storage_key::*;
mod calculate_parachain_head_storage_key {
	use parity_scale_codec::Encode;
	use sp_core::storage::StorageKey;

	pub(crate) fn calculate_parachain_head_storage_key(para_id: u32) -> StorageKey {
		StorageKey(
			[
				frame_support::storage::storage_prefix(b"Paras", b"Heads").as_slice(),
				sp_io::hashing::twox_64(para_id.encode().as_ref()).as_slice(),
				para_id.encode().as_slice(),
			]
			.concat(),
		)
	}

	#[test]
	fn calculate_parachain_head_storage_key_successful_spiritnet_parachain() {
		assert_eq!(
			calculate_parachain_head_storage_key(2_086).0,
			hex_literal::hex!(
				"cd710b30bd2eab0352ddcc26417aa1941b3c252fcb29d88eff4f3de5de4476c32c0cfd6c23b92a7826080000"
			)
			.to_vec()
		);
	}
	#[test]
	fn calculate_parachain_head_storage_key_successful_peregrine_parachain() {
		assert_eq!(
			calculate_parachain_head_storage_key(2_000).0,
			hex_literal::hex!(
				"cd710b30bd2eab0352ddcc26417aa1941b3c252fcb29d88eff4f3de5de4476c363f5a4efb16ffa83d0070000"
			)
			.to_vec()
		);
	}
}

pub(crate) use calculate_dip_identity_commitment_storage_key_for_runtime::*;
mod calculate_dip_identity_commitment_storage_key_for_runtime {
	use pallet_dip_provider::IdentityCommitmentVersion;
	use sp_core::storage::StorageKey;

	pub(crate) fn calculate_dip_identity_commitment_storage_key_for_runtime<Runtime>(
		subject: &Runtime::Identifier,
		version: IdentityCommitmentVersion,
	) -> StorageKey
	where
		Runtime: pallet_dip_provider::Config,
	{
		StorageKey(pallet_dip_provider::IdentityCommitments::<Runtime>::hashed_key_for(
			subject, version,
		))
	}

	#[test]
	fn calculate_dip_identity_commitment_storage_key_for_runtime_successful_peregrine_parachain() {
		use did::DidIdentifierOf;
		use peregrine_runtime::Runtime as PeregrineRuntime;
		use sp_core::crypto::Ss58Codec;

		assert_eq!(
			calculate_dip_identity_commitment_storage_key_for_runtime::<PeregrineRuntime>(&DidIdentifierOf::<PeregrineRuntime>::from_ss58check("4s3jpR7pzrUdhVUqHHdWoBN6oNQHBC7WRo7zsXdjAzQPT7Cf").unwrap(), 0).0,
			hex_literal::hex!("b375edf06348b4330d1e88564111cb3d5bf19e4ed2927982e234d989e812f3f314c9211b34c8b43b2a18d67d5c96de9cb6caebbe9e3adeaaf693a2d198f2881d0b504fc72ed4ac0a7ed24a025fc228ce01a12dfa1fa4ab9a0000")
				.to_vec()
		);
	}
}
