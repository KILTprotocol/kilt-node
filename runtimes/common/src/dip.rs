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

pub mod proof_generation {
	use codec::Encode;
	use sp_std::marker::PhantomData;
	use sp_trie::{generate_trie_proof, verify_trie_proof, LayoutV1, MemoryDB, TrieDBMutBuilder, TrieHash, TrieMut};

	use did::{
		did_details::{DidDetails, DidPublicKeyDetails},
		DidIdentifierOf, KeyIdOf,
	};
	use dip_sender::traits::IdentityProofGenerator;

	pub struct DidPalletMerkleRootHasher<T>(PhantomData<T>);

	pub type DidMerkleRootHasherOutput<T> = <T as frame_system::Config>::Hash;

	impl<T: did::Config> IdentityProofGenerator<DidIdentifierOf<T>, DidDetails<T>, DidMerkleRootHasherOutput<T>>
		for DidPalletMerkleRootHasher<T>
	{
		fn generate_proof(
			identifier: &DidIdentifierOf<T>,
			identity: &DidDetails<T>,
		) -> Result<DidMerkleRootHasherOutput<T>, sp_runtime::DispatchError> {
			let mut db = MemoryDB::default();
			Self::calculate_root_with_db(identifier, identity, &mut db)
		}
	}

	impl<T: did::Config> DidPalletMerkleRootHasher<T> {
		fn calculate_root_with_db(
			did: &DidIdentifierOf<T>,
			details: &DidDetails<T>,
			db: &mut MemoryDB<<T as frame_system::Config>::Hashing>,
		) -> Result<DidMerkleRootHasherOutput<T>, sp_runtime::DispatchError> {
			let mut trie = TrieHash::<LayoutV1<T::Hashing>>::default();
			let mut trie_builder = TrieDBMutBuilder::<LayoutV1<T::Hashing>>::new(db, &mut trie).build();

			// Should never happen
			trie_builder
				.insert(b"did", &did.encode())
				.map_err(|_| did::Error::<T>::InternalError)?;
			// Should never happen
			trie_builder
				.insert(b"auth", &details.authentication_key.encode())
				.map_err(|_| did::Error::<T>::InternalError)?;
			if let Some(att_key) = details.attestation_key {
				trie_builder
					.insert(b"att", &att_key.encode())
					.map_err(|_| did::Error::<T>::InternalError)?;
			}
			if let Some(del_key) = details.delegation_key {
				trie_builder
					.insert(b"del", &del_key.encode())
					.map_err(|_| did::Error::<T>::InternalError)?;
			}
			details
				.key_agreement_keys
				.iter()
				.enumerate()
				.try_for_each(|(i, k)| -> Result<(), did::Error<T>> {
					trie_builder
						.insert(&[b"enc-".as_slice(), i.to_be_bytes().as_slice()].concat(), &k.encode())
						.map_err(|_| did::Error::<T>::InternalError)?;
					Ok(())
				})?;
			details
				.public_keys
				.iter()
				.enumerate()
				.try_for_each(|(i, (id, k))| -> Result<(), did::Error<T>> {
					trie_builder
						.insert(
							&[b"pub-".as_slice(), i.to_be_bytes().as_slice()].concat(),
							&(id, k).encode(),
						)
						.map_err(|_| did::Error::<T>::InternalError)?;
					Ok(())
				})?;
			trie_builder.commit();
			// TODO: Benchmark weight
			Ok(trie_builder.root().to_owned())
		}

		pub fn generate_proof(
			did_identifier: &DidIdentifierOf<T>,
			details: &DidDetails<T>,
			key_id: KeyIdOf<T>,
		) -> Result<(DidMerkleRootHasherOutput<T>, Vec<Vec<u8>>), sp_runtime::DispatchError> {
			let mut db = MemoryDB::default();
			let merkle_root = Self::calculate_root_with_db(did_identifier, details, &mut db)?;
			let proof_key: Vec<u8> = if key_id == details.authentication_key {
				Ok(b"auth".to_vec())
			} else if details.attestation_key == Some(key_id) {
				Ok(b"att".to_vec())
			} else if details.delegation_key == Some(key_id) {
				Ok(b"del".to_vec())
			} else if let Some((i, _)) = details
				.key_agreement_keys
				.iter()
				.enumerate()
				.find(|(_, enc_key_id)| **enc_key_id == key_id)
			{
				Ok([b"enc-".as_slice(), i.to_be_bytes().as_slice()].concat())
			} else if let Some((i, _)) = details
				.public_keys
				.iter()
				.enumerate()
				.find(|(_, (public_key_id, _))| **public_key_id == key_id)
			{
				Ok([b"pub-".as_slice(), i.to_be_bytes().as_slice()].concat())
			} else {
				Err(did::Error::<T>::VerificationKeyNotPresent)
			}?;
			let merkle_proof =
				generate_trie_proof::<LayoutV1<T::Hashing>, _, _, _>(&db, merkle_root, &[proof_key]).unwrap();
			Ok((merkle_root, merkle_proof))
		}

		pub fn verify_proof(
			merkle_root: <T as frame_system::Config>::Hash,
			proof: Vec<Vec<u8>>,
			merkle_key: &[u8],
			did_key: (KeyIdOf<T>, DidPublicKeyDetails<T::BlockNumber>),
		) -> bool {
			verify_trie_proof::<LayoutV1<T::Hashing>, _, _, _>(
				&merkle_root,
				&proof,
				&[(&merkle_key, Some(did_key.encode()))],
			)
			.is_ok()
		}
	}
}

pub mod identity_retrieval {
	use sp_std::marker::PhantomData;

	use did::{did_details::DidDetails, DidIdentifierOf};
	use dip_sender::traits::IdentityProvider;

	pub struct DidPalletProvider<T>(PhantomData<T>);

	impl<T: did::Config> IdentityProvider<DidIdentifierOf<T>, DidDetails<T>> for DidPalletProvider<T> {
		fn retrieve(identifier: &DidIdentifierOf<T>) -> Result<Option<DidDetails<T>>, sp_runtime::DispatchError> {
			if let Some(did_details) = did::Did::<T>::get(identifier) {
				Ok(Some(did_details))
			} else if did::Pallet::<T>::get_deleted_did(identifier).is_some() {
				Ok(None)
			} else {
				Err(did::Error::<T>::DidNotPresent.into())
			}
		}
	}
}

pub mod identity_dispatch {
	use codec::Encode;
	use frame_support::weights::Weight;
	use frame_system::{pallet_prelude::OriginFor, RawOrigin};
	use sp_std::marker::PhantomData;
	use xcm::latest::prelude::*;

	use dip_sender::traits::{IdentityProofDispatcher, TxBuilder};
	use dip_support::latest::IdentityProofAction;
	use xcm_executor::traits::Convert;

	// Dispatcher wrapping the XCM pallet.
	// It basically properly encodes the Transact operation, then delegates
	// everything else to the pallet's `send_xcm` function, similarly to what the
	// pallet's `send` extrinsic does.
	pub struct DidXcmV3ViaXcmPalletDispatcher<T, I, P, C>(PhantomData<(T, I, P, C)>);

	impl<T, I, P, C> IdentityProofDispatcher<I, <T as frame_system::Config>::AccountId, P>
		for DidXcmV3ViaXcmPalletDispatcher<T, I, P, C>
	where
		T: pallet_xcm::Config,
		I: Encode,
		P: Encode,
		C: Convert<OriginFor<T>, MultiLocation>,
	{
		type Error = SendError;

		fn dispatch<B: TxBuilder<I, P>>(
			action: IdentityProofAction<I, P>,
			dispatcher: T::AccountId,
			asset: MultiAsset,
			destination: MultiLocation,
		) -> Result<(), Self::Error> {
			println!("DidXcmV3ViaXcmPalletDispatcher::dispatch 1");
			let origin_location =
				C::convert(RawOrigin::Signed(dispatcher).into()).map_err(|_| SendError::DestinationUnsupported)?;
			println!(
				"DidXcmV3ViaXcmPalletDispatcher::dispatch 2 with origin_location: {:?}",
				origin_location
			);
			let interior: Junctions = origin_location
				.try_into()
				.map_err(|_| SendError::DestinationUnsupported)?;
			println!(
				"DidXcmV3ViaXcmPalletDispatcher::dispatch 3 with interior: {:?}",
				interior
			);
			// TODO: Replace with proper error handling
			let dest_tx = B::build(destination, action)
				.map_err(|_| ())
				.expect("Failed to build call");
			let dest_xcm = Xcm(vec![
				BuyExecution {
					fees: asset,
					weight_limit: Limited(Weight::from_parts(1_000_000, 1_000_000)),
				},
				// TODO: Insert a `ExpectPallet` instruction
				// TODO: Replace `action.encode().into()` with actual encoded call
				Transact {
					origin_kind: OriginKind::SovereignAccount,
					require_weight_at_most: Weight::from_ref_time(100_000_000),
					call: dest_tx,
				},
				RefundSurplus,
			]);
			println!("DidXcmV3ViaXcmPalletDispatcher::dispatch 4");
			let res = pallet_xcm::Pallet::<T>::send_xcm(interior, destination, dest_xcm).map(|_| ());
			println!("DidXcmV3ViaXcmPalletDispatcher::dispatch 5");
			res
		}
	}
}
