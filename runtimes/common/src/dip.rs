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

use codec::{Decode, Encode, MaxEncodedLen};
use did::{did_details::DidPublicKeyDetails, DidVerificationKeyRelationship};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
use sp_std::{marker::PhantomData, vec::Vec};

#[derive(Clone, RuntimeDebug, Encode, Decode, PartialEq, Eq, TypeInfo, PartialOrd, Ord)]
pub enum KeyRelationship {
	Encryption,
	Verification(DidVerificationKeyRelationship),
}

impl From<DidVerificationKeyRelationship> for KeyRelationship {
	fn from(value: DidVerificationKeyRelationship) -> Self {
		Self::Verification(value)
	}
}

// #[derive(Clone, RuntimeDebug, Encode, Decode, PartialEq, Eq, TypeInfo,
// PartialOrd, Ord)] pub enum LeafKey<KeyId> {
// 	KeyReference(KeyId, KeyRelationship),
// 	KeyDetails(KeyId),
// }

// #[derive(Clone, RuntimeDebug, Encode, Decode, PartialEq, Eq, TypeInfo)]
// pub enum LeafValue<BlockNumber: MaxEncodedLen> {
// 	KeyReference,
// 	KeyDetails(DidPublicKeyDetails<BlockNumber>),
// }

#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug)]
pub struct KeyReferenceKey<KeyId>(pub KeyId, pub KeyRelationship);
#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug)]
pub struct KeyReferenceValue;

#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug)]
pub struct KeyDetailsKey<KeyId>(pub KeyId);
#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug)]
pub struct KeyDetailsValue<BlockNumber: MaxEncodedLen>(pub DidPublicKeyDetails<BlockNumber>);

#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug)]
pub enum ProofLeaf<KeyId, BlockNumber: MaxEncodedLen> {
	KeyReference(KeyReferenceKey<KeyId>, KeyReferenceValue),
	KeyDetails(KeyDetailsKey<KeyId>, KeyDetailsValue<BlockNumber>),
}

impl<KeyId, BlockNumber> ProofLeaf<KeyId, BlockNumber>
where
	KeyId: Encode,
	BlockNumber: MaxEncodedLen + Encode,
{
	pub(crate) fn encoded_key(&self) -> Vec<u8> {
		match self {
			ProofLeaf::KeyReference(key, _) => key.encode(),
			ProofLeaf::KeyDetails(key, _) => key.encode(),
		}
	}

	pub(crate) fn encoded_value(&self) -> Vec<u8> {
		match self {
			ProofLeaf::KeyReference(_, value) => value.encode(),
			ProofLeaf::KeyDetails(_, value) => value.encode(),
		}
	}
}

pub mod sender {
	use super::*;

	use did::{did_details::DidDetails, KeyIdOf};
	use dip_support::latest::Proof;
	use pallet_dip_sender::traits::{IdentityProofGenerator, IdentityProvider};
	use sp_std::borrow::ToOwned;
	use sp_trie::{LayoutV1, MemoryDB};

	pub type BlindedValue = Vec<u8>;

	pub type DidMerkleProof<T> =
		Proof<Vec<BlindedValue>, ProofLeaf<KeyIdOf<T>, <T as frame_system::Config>::BlockNumber>>;

	#[derive(Encode, Decode, TypeInfo)]
	pub struct CompleteMerkleProof<Root, Proof> {
		merkle_root: Root,
		merkle_proof: Proof,
	}

	pub struct DidMerkleRootGenerator<T>(PhantomData<T>);

	impl<T> DidMerkleRootGenerator<T>
	where
		T: did::Config,
	{
		fn calculate_root_with_db(identity: &DidDetails<T>, db: &mut MemoryDB<T::Hashing>) -> Result<T::Hash, ()> {
			use sp_trie::{TrieDBMutBuilder, TrieHash, TrieMut};

			let mut trie = TrieHash::<LayoutV1<T::Hashing>>::default();
			let mut trie_builder = TrieDBMutBuilder::<LayoutV1<T::Hashing>>::new(db, &mut trie).build();

			// Authentication key
			let auth_leaf = ProofLeaf::<_, T::BlockNumber>::KeyReference(
				KeyReferenceKey(
					identity.authentication_key,
					DidVerificationKeyRelationship::Authentication.into(),
				),
				KeyReferenceValue,
			);
			trie_builder
				.insert(auth_leaf.encoded_key().as_slice(), auth_leaf.encoded_value().as_slice())
				.map_err(|_| ())?;
			// Attestation key
			if let Some(att_key_id) = identity.attestation_key {
				let att_leaf = ProofLeaf::<_, T::BlockNumber>::KeyReference(
					KeyReferenceKey(att_key_id, DidVerificationKeyRelationship::AssertionMethod.into()),
					KeyReferenceValue,
				);
				trie_builder
					.insert(att_leaf.encoded_key().as_slice(), att_leaf.encoded_value().as_slice())
					.map_err(|_| ())?;
			};
			// Delegation key
			if let Some(del_key_id) = identity.delegation_key {
				let del_leaf = ProofLeaf::<_, T::BlockNumber>::KeyReference(
					KeyReferenceKey(del_key_id, DidVerificationKeyRelationship::CapabilityDelegation.into()),
					KeyReferenceValue,
				);
				trie_builder
					.insert(del_leaf.encoded_key().as_slice(), del_leaf.encoded_value().as_slice())
					.map_err(|_| ())?;
			};
			// Key agreement keys
			identity
				.key_agreement_keys
				.iter()
				.try_for_each(|id| -> Result<(), ()> {
					let enc_leaf = ProofLeaf::<_, T::BlockNumber>::KeyReference(
						KeyReferenceKey(*id, KeyRelationship::Encryption),
						KeyReferenceValue,
					);
					trie_builder
						.insert(enc_leaf.encoded_key().as_slice(), enc_leaf.encoded_value().as_slice())
						.map_err(|_| ())?;
					Ok(())
				})?;
			// Public keys
			identity
				.public_keys
				.iter()
				.try_for_each(|(id, key_details)| -> Result<(), ()> {
					let key_leaf = ProofLeaf::KeyDetails(KeyDetailsKey(*id), KeyDetailsValue(key_details.clone()));
					trie_builder
						.insert(key_leaf.encoded_key().as_slice(), key_leaf.encoded_value().as_slice())
						.map_err(|_| ())?;
					Ok(())
				})?;
			trie_builder.commit();
			Ok(trie_builder.root().to_owned())
		}

		// TODO: Better error handling
		#[allow(clippy::result_unit_err)]
		pub fn generate_proof<'a, K>(
			identity: &DidDetails<T>,
			mut key_ids: K,
		) -> Result<CompleteMerkleProof<T::Hash, DidMerkleProof<T>>, ()>
		where
			K: Iterator<Item = &'a KeyIdOf<T>>,
		{
			use sp_std::collections::btree_set::BTreeSet;
			use sp_trie::generate_trie_proof;

			let mut db = MemoryDB::default();
			let root = Self::calculate_root_with_db(identity, &mut db)?;

			#[allow(clippy::type_complexity)]
			let leaves: BTreeSet<ProofLeaf<KeyIdOf<T>, T::BlockNumber>> =
				key_ids.try_fold(BTreeSet::new(), |mut set, key_id| -> Result<_, ()> {
					let key_details = identity.public_keys.get(key_id).ok_or(())?;
					if *key_id == identity.authentication_key {
						set.insert(ProofLeaf::KeyReference(
							KeyReferenceKey(*key_id, DidVerificationKeyRelationship::Authentication.into()),
							KeyReferenceValue,
						));
					}
					if Some(*key_id) == identity.attestation_key {
						set.insert(ProofLeaf::KeyReference(
							KeyReferenceKey(*key_id, DidVerificationKeyRelationship::AssertionMethod.into()),
							KeyReferenceValue,
						));
					}
					if Some(*key_id) == identity.delegation_key {
						set.insert(ProofLeaf::KeyReference(
							KeyReferenceKey(*key_id, DidVerificationKeyRelationship::CapabilityDelegation.into()),
							KeyReferenceValue,
						));
					}
					if identity.key_agreement_keys.contains(key_id) {
						set.insert(ProofLeaf::KeyReference(
							KeyReferenceKey(*key_id, KeyRelationship::Encryption),
							KeyReferenceValue,
						));
					};
					let key_leaf = ProofLeaf::KeyDetails(KeyDetailsKey(*key_id), KeyDetailsValue(key_details.clone()));
					if !set.contains(&key_leaf) {
						set.insert(key_leaf);
					}
					Ok(set)
				})?;
			let encoded_keys: Vec<Vec<u8>> = leaves.iter().map(|l| l.encoded_key()).collect();
			let proof =
				generate_trie_proof::<LayoutV1<T::Hashing>, _, _, _>(&db, root, &encoded_keys).map_err(|_| ())?;
			Ok(CompleteMerkleProof {
				merkle_root: root,
				merkle_proof: DidMerkleProof::<T> {
					blinded: proof,
					revealed: leaves.into_iter().collect::<Vec<_>>(),
				},
			})
		}
	}

	impl<T> IdentityProofGenerator<T::DidIdentifier, DidDetails<T>> for DidMerkleRootGenerator<T>
	where
		T: did::Config,
	{
		// TODO: Proper error handling
		type Error = ();
		type Output = T::Hash;

		fn generate_commitment(
			_identifier: &T::DidIdentifier,
			identity: &DidDetails<T>,
		) -> Result<T::Hash, Self::Error> {
			let mut db = MemoryDB::default();
			Self::calculate_root_with_db(identity, &mut db)
		}
	}

	pub struct DidIdentityProvider<T>(PhantomData<T>);

	impl<T> IdentityProvider<T::DidIdentifier, DidDetails<T>, ()> for DidIdentityProvider<T>
	where
		T: did::Config,
	{
		// TODO: Proper error handling
		type Error = ();

		fn retrieve(identifier: &T::DidIdentifier) -> Result<Option<(DidDetails<T>, ())>, Self::Error> {
			match (
				did::Pallet::<T>::get_did(identifier),
				did::Pallet::<T>::get_deleted_did(identifier),
			) {
				(Some(details), _) => Ok(Some((details, ()))),
				(None, Some(_)) => Ok(None),
				_ => Err(()),
			}
		}
	}
}

pub mod receiver {
	use super::*;

	use dip_support::VersionedIdentityProof;
	use pallet_dip_receiver::traits::IdentityProofVerifier;
	use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
	use sp_trie::LayoutV1;

	// TODO: Avoid repetition of the same key if it appears multiple times.
	#[derive(RuntimeDebug, PartialEq, Eq)]
	pub struct ProofEntry<BlockNumber: MaxEncodedLen> {
		pub key: DidPublicKeyDetails<BlockNumber>,
		pub relationship: KeyRelationship,
	}

	#[derive(RuntimeDebug, PartialEq, Eq)]
	pub struct VerificationResult<BlockNumber: MaxEncodedLen>(pub Vec<ProofEntry<BlockNumber>>);

	impl<BlockNumber> From<Vec<ProofEntry<BlockNumber>>> for VerificationResult<BlockNumber>
	where
		BlockNumber: MaxEncodedLen,
	{
		fn from(value: Vec<ProofEntry<BlockNumber>>) -> Self {
			Self(value)
		}
	}

	pub struct DidMerkleProofVerifier<KeyId, BlockNumber, Hasher>(PhantomData<(KeyId, BlockNumber, Hasher)>);

	impl<KeyId, BlockNumber, Hasher> IdentityProofVerifier for DidMerkleProofVerifier<KeyId, BlockNumber, Hasher>
	where
		KeyId: MaxEncodedLen + Clone + Ord,
		BlockNumber: MaxEncodedLen + Clone + Ord,
		Hasher: sp_core::Hasher,
	{
		type BlindedValue = Vec<Vec<u8>>;
		// TODO: Proper error handling
		type Error = ();
		type ProofDigest = <Hasher as sp_core::Hasher>::Out;
		type ProofLeaf = ProofLeaf<KeyId, BlockNumber>;
		type VerificationResult = VerificationResult<BlockNumber>;

		fn verify_proof_against_digest(
			proof: VersionedIdentityProof<Self::BlindedValue, Self::ProofLeaf>,
			digest: Self::ProofDigest,
		) -> Result<Self::VerificationResult, Self::Error> {
			use dip_support::v1;
			use sp_trie::verify_trie_proof;

			let proof: v1::Proof<_, _> = proof.try_into()?;
			// TODO: more efficient by removing cloning and/or collecting.
			// Did not find another way of mapping a Vec<(Vec<u8>, Vec<u8>)> to a
			// Vec<(Vec<u8>, Option<Vec<u8>>)>.
			let proof_leaves = proof
				.revealed
				.iter()
				.map(|leaf| (leaf.encoded_key(), Some(leaf.encoded_value())))
				.collect::<Vec<(Vec<u8>, Option<Vec<u8>>)>>();
			verify_trie_proof::<LayoutV1<Hasher>, _, _, _>(&digest, &proof.blinded, &proof_leaves).map_err(|_| ())?;

			// At this point, we know the proof is valid. We just need to map the revealed
			// leaves to something the consumer can easily operate on.

			// Create a map of the revealed public keys
			//TODO: Avoid cloning, and use a map of references for the lookup
			let public_keys: BTreeMap<KeyId, DidPublicKeyDetails<BlockNumber>> = proof
				.revealed
				.clone()
				.into_iter()
				.filter_map(|leaf| {
					if let ProofLeaf::KeyDetails(KeyDetailsKey(key_id), KeyDetailsValue(key_details)) = leaf {
						Some((key_id, key_details))
					} else {
						None
					}
				})
				.collect();
			// Create a list of the revealed keys
			let keys: Vec<ProofEntry<BlockNumber>> = proof
				.revealed
				.into_iter()
				.filter_map(|leaf| {
					if let ProofLeaf::KeyReference(KeyReferenceKey(key_id, key_relationship), KeyReferenceValue) = leaf
					{
						// TODO: Better error handling.
						let key_details = public_keys
							.get(&key_id)
							.expect("Key ID should be present in the map of revealed public keys.");
						Some(ProofEntry {
							key: key_details.clone(),
							relationship: key_relationship,
						})
					} else {
						None
					}
				})
				.collect();
			Ok(keys.into())
		}
	}
}

#[cfg(test)]
mod test {
	use crate::dip::{receiver::DidMerkleProofVerifier, sender::DidMerkleRootGenerator};

	use super::*;

	use did::{
		did_details::{DidCreationDetails, DidEncryptionKey},
		KeyIdOf,
	};
	use dip_support::latest::Proof;
	use frame_support::{
		assert_err, assert_ok, construct_runtime, parameter_types, traits::Everything,
		weights::constants::RocksDbWeight,
	};
	use frame_system::{
		mocking::{MockBlock, MockUncheckedExtrinsic},
		EnsureSigned, RawOrigin,
	};
	use pallet_dip_receiver::traits::IdentityProofVerifier;
	use sp_core::{ecdsa, ed25519, sr25519, ConstU16, ConstU32, ConstU64, Hasher, Pair};
	use sp_io::TestExternalities;
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentifyAccount, IdentityLookup},
		AccountId32,
	};
	use sp_std::collections::btree_set::BTreeSet;

	pub(crate) type AccountId = AccountId32;
	pub(crate) type Balance = u128;
	pub(crate) type Block = MockBlock<TestRuntime>;
	pub(crate) type BlockNumber = u64;
	pub(crate) type Hashing = BlakeTwo256;
	pub(crate) type Index = u64;
	pub(crate) type UncheckedExtrinsic = MockUncheckedExtrinsic<TestRuntime>;

	construct_runtime!(
		pub enum TestRuntime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system,
			Balances: pallet_balances,
			Did: did,
		}
	);

	impl frame_system::Config for TestRuntime {
		type AccountData = pallet_balances::AccountData<Balance>;
		type AccountId = AccountId;
		type BaseCallFilter = Everything;
		type BlockHashCount = ConstU64<250>;
		type BlockLength = ();
		type BlockNumber = BlockNumber;
		type BlockWeights = ();
		type DbWeight = RocksDbWeight;
		type Hash = <Hashing as Hasher>::Out;
		type Hashing = Hashing;
		type Header = Header;
		type Index = Index;
		type Lookup = IdentityLookup<Self::AccountId>;
		type MaxConsumers = ConstU32<16>;
		type OnKilledAccount = ();
		type OnNewAccount = ();
		type OnSetCode = ();
		type PalletInfo = PalletInfo;
		type RuntimeCall = RuntimeCall;
		type RuntimeEvent = RuntimeEvent;
		type RuntimeOrigin = RuntimeOrigin;
		type SS58Prefix = ConstU16<38>;
		type SystemWeightInfo = ();
		type Version = ();
	}

	parameter_types! {
		pub ExistentialDeposit: Balance = 500u64.into();
	}

	impl pallet_balances::Config for TestRuntime {
		type AccountStore = System;
		type Balance = Balance;
		type DustRemoval = ();
		type ExistentialDeposit = ExistentialDeposit;
		type MaxLocks = ConstU32<50>;
		type MaxReserves = ConstU32<50>;
		type ReserveIdentifier = [u8; 8];
		type RuntimeEvent = RuntimeEvent;
		type WeightInfo = ();
	}

	parameter_types! {
		pub Deposit: Balance = 500u64.into();
		pub Fee: Balance = 500u64.into();
		pub MaxBlocksTxValidity: BlockNumber = 10u64;
		#[derive(Debug, Clone, Eq, PartialEq)]
		pub const MaxTotalKeyAgreementKeys: u32 = 2;
	}

	impl did::DeriveDidCallAuthorizationVerificationKeyRelationship for RuntimeCall {
		fn derive_verification_key_relationship(&self) -> did::DeriveDidCallKeyRelationshipResult {
			Ok(DidVerificationKeyRelationship::Authentication)
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn get_call_for_did_call_benchmark() -> Self {
			RuntimeCall::System(frame_system::Call::remark { remark: vec![] })
		}
	}

	impl did::Config for TestRuntime {
		type Currency = Balances;
		type Deposit = Deposit;
		type DidIdentifier = AccountId;
		type EnsureOrigin = EnsureSigned<AccountId>;
		type Fee = Fee;
		type FeeCollector = ();
		type MaxBlocksTxValidity = MaxBlocksTxValidity;
		type MaxNewKeyAgreementKeys = ConstU32<2>;
		type MaxNumberOfServicesPerDid = ConstU32<1>;
		type MaxNumberOfTypesPerService = ConstU32<1>;
		type MaxNumberOfUrlsPerService = ConstU32<1>;
		type MaxPublicKeysPerDid = ConstU32<5>;
		type MaxServiceIdLength = ConstU32<100>;
		type MaxServiceTypeLength = ConstU32<100>;
		type MaxServiceUrlLength = ConstU32<100>;
		type MaxTotalKeyAgreementKeys = MaxTotalKeyAgreementKeys;
		type OriginSuccess = AccountId;
		type RuntimeCall = RuntimeCall;
		type RuntimeEvent = RuntimeEvent;
		type RuntimeOrigin = RuntimeOrigin;
		type WeightInfo = ();
	}

	fn base_ext() -> TestExternalities {
		TestExternalities::new(
			frame_system::GenesisConfig::default()
				.build_storage::<TestRuntime>()
				.unwrap(),
		)
	}

	const ALICE: AccountId = AccountId::new([1u8; 32]);

	#[test]
	fn minimal_did_merkle_proof() {
		base_ext().execute_with(|| {
			// Give Alice some balance
			assert_ok!(Balances::set_balance(RawOrigin::Root.into(), ALICE, 1_000_000_000, 0));
			// Generate a DID for alice
			let did_auth_key = ed25519::Pair::from_seed(&[100u8; 32]);
			let did: AccountId = did_auth_key.public().into_account().into();
			let create_details = DidCreationDetails {
				did: did.clone(),
				submitter: ALICE,
				new_attestation_key: None,
				new_delegation_key: None,
				new_key_agreement_keys: BTreeSet::new().try_into().unwrap(),
				new_service_details: vec![],
			};
			// Create Alice's DID with only authentication key
			assert_ok!(Did::create(
				RawOrigin::Signed(ALICE).into(),
				Box::new(create_details.clone()),
				did_auth_key.sign(&create_details.encode()).into()
			));
			let did_details = Did::get_did(&did).expect("DID should be present");

			// 1. Create the DID merkle proof revealing only the authentication key
			let (root, proof) = DidMerkleRootGenerator::<TestRuntime>::generate_proof(
				&did_details,
				[did_details.authentication_key].iter(),
			)
			.expect("Merkle proof generation should not fail.");
			println!("{:?} - {:?} - {:?} bytes", root, proof, proof.encoded_size());
			// Verify the generated merkle proof
			assert_ok!(
				DidMerkleProofVerifier::<KeyIdOf<TestRuntime>, BlockNumber, Hashing>::verify_proof_against_digest(
					proof.clone().into(),
					root
				)
			);

			// 2. Fail to generate a Merkle proof for a key that does not exist
			assert_err!(
				DidMerkleRootGenerator::<TestRuntime>::generate_proof(
					&did_details,
					[<<Hashing as Hasher>::Out>::default()].iter()
				),
				()
			);

			// 3. Fail to verify a merkle proof with a compromised merkle root
			let new_root = <<Hashing as Hasher>::Out>::default();
			assert_err!(
				DidMerkleProofVerifier::<KeyIdOf<TestRuntime>, BlockNumber, Hashing>::verify_proof_against_digest(
					proof.into(),
					new_root
				),
				()
			);
		})
	}

	#[test]
	fn complete_did_merkle_proof() {
		base_ext().execute_with(|| {
			// Give Alice some balance
			assert_ok!(Balances::set_balance(RawOrigin::Root.into(), ALICE, 1_000_000_000, 0));
			// Generate a DID for alice
			let did_auth_key = ed25519::Pair::from_seed(&[100u8; 32]);
			let did_att_key = sr25519::Pair::from_seed(&[150u8; 32]);
			let did_del_key = ecdsa::Pair::from_seed(&[200u8; 32]);
			let enc_keys = BTreeSet::from_iter(vec![
				DidEncryptionKey::X25519([250u8; 32]),
				DidEncryptionKey::X25519([251u8; 32]),
			]);
			let did: AccountId = did_auth_key.public().into_account().into();
			let create_details = DidCreationDetails {
				did: did.clone(),
				submitter: ALICE,
				new_attestation_key: Some(did_att_key.public().into()),
				new_delegation_key: Some(did_del_key.public().into()),
				new_key_agreement_keys: enc_keys
					.try_into()
					.expect("BTreeSet to BoundedBTreeSet should not fail"),
				new_service_details: vec![],
			};
			// Create Alice's DID with only authentication key
			assert_ok!(Did::create(
				RawOrigin::Signed(ALICE).into(),
				Box::new(create_details.clone()),
				did_auth_key.sign(&create_details.encode()).into()
			));
			let did_details = Did::get_did(&did).expect("DID should be present");

			// 1. Create the DID merkle proof revealing only the authentication key
			let (root, proof) = DidMerkleRootGenerator::<TestRuntime>::generate_proof(
				&did_details,
				[did_details.authentication_key].iter(),
			)
			.expect("Merkle proof generation should not fail.");
			// Verify the generated merkle proof
			assert_ok!(
				DidMerkleProofVerifier::<KeyIdOf<TestRuntime>, BlockNumber, Hashing>::verify_proof_against_digest(
					proof.into(),
					root
				)
			);

			// 2. Create the DID merkle proof revealing all the keys
			let (root, proof) = DidMerkleRootGenerator::<TestRuntime>::generate_proof(
				&did_details,
				[
					did_details.authentication_key,
					did_details.attestation_key.unwrap(),
					did_details.delegation_key.unwrap(),
				]
				.iter()
				.chain(did_details.key_agreement_keys.iter()),
			)
			.expect("Merkle proof generation should not fail.");
			// Verify the generated merkle proof
			assert_ok!(
				DidMerkleProofVerifier::<KeyIdOf<TestRuntime>, BlockNumber, Hashing>::verify_proof_against_digest(
					proof.into(),
					root
				)
			);

			// 2. Create the DID merkle proof revealing only the key reference and not the
			// key ID
			let (root, proof) = DidMerkleRootGenerator::<TestRuntime>::generate_proof(
				&did_details,
				[did_details.authentication_key].iter(),
			)
			.expect("Merkle proof generation should not fail.");
			let reference_only_authentication_leaf: Vec<_> = proof
				.revealed
				.into_iter()
				.filter(|l| !matches!(l, ProofLeaf::KeyDetails(_, _)))
				.collect();
			// Fail to verify the generated merkle proof
			assert_err!(
				DidMerkleProofVerifier::<KeyIdOf<TestRuntime>, BlockNumber, Hashing>::verify_proof_against_digest(
					Proof {
						blinded: proof.blinded,
						revealed: reference_only_authentication_leaf
					}
					.into(),
					root
				),
				()
			);
		})
	}
}
