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

#[derive(Clone, RuntimeDebug, Encode, Decode, PartialEq, Eq, TypeInfo, PartialOrd, Ord)]
pub enum LeafKey<KeyId> {
	KeyReference(KeyId, KeyRelationship),
	KeyDetails(KeyId),
}

#[derive(Clone, RuntimeDebug, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub enum LeafValue<BlockNumber: MaxEncodedLen> {
	KeyReference,
	KeyDetails(DidPublicKeyDetails<BlockNumber>),
}

pub mod sender {
	use super::*;

	use did::{did_details::DidDetails, KeyIdOf};
	use dip_support::latest::Proof;
	use pallet_dip_sender::traits::{IdentityProofGenerator, IdentityProvider};
	use sp_std::{
		borrow::ToOwned,
		collections::{btree_map::BTreeMap, btree_set::BTreeSet},
	};
	use sp_trie::{LayoutV1, MemoryDB};

	pub struct DidMerkleRootGenerator<T>(PhantomData<T>);

	pub type DidMerkleProof<T> =
		Proof<Vec<Vec<u8>>, LeafKey<KeyIdOf<T>>, LeafValue<<T as frame_system::Config>::BlockNumber>>;

	impl<T> DidMerkleRootGenerator<T>
	where
		T: did::Config,
	{
		fn calculate_root_with_db(identity: &DidDetails<T>, db: &mut MemoryDB<T::Hashing>) -> Result<T::Hash, ()> {
			use sp_trie::{TrieDBMutBuilder, TrieHash, TrieMut};

			let mut trie = TrieHash::<LayoutV1<T::Hashing>>::default();
			let mut trie_builder = TrieDBMutBuilder::<LayoutV1<T::Hashing>>::new(db, &mut trie).build();

			// Authentication key
			trie_builder
				.insert(
					LeafKey::KeyReference(
						identity.authentication_key,
						DidVerificationKeyRelationship::Authentication.into(),
					)
					.encode()
					.as_slice(),
					LeafValue::<T::BlockNumber>::KeyReference.encode().as_slice(),
				)
				.map_err(|_| ())?;
			// Attestation key
			if let Some(att_key_id) = identity.attestation_key {
				trie_builder
					.insert(
						LeafKey::KeyReference(att_key_id, DidVerificationKeyRelationship::AssertionMethod.into())
							.encode()
							.as_slice(),
						LeafValue::<T::BlockNumber>::KeyReference.encode().as_slice(),
					)
					.map_err(|_| ())?;
			};
			// Delegation key
			if let Some(del_key_id) = identity.delegation_key {
				trie_builder
					.insert(
						LeafKey::KeyReference(del_key_id, DidVerificationKeyRelationship::CapabilityDelegation.into())
							.encode()
							.as_slice(),
						LeafValue::<T::BlockNumber>::KeyReference.encode().as_slice(),
					)
					.map_err(|_| ())?;
			};
			// Key agreement keys
			identity
				.key_agreement_keys
				.iter()
				.try_for_each(|id| -> Result<(), ()> {
					trie_builder
						.insert(
							LeafKey::KeyReference(*id, KeyRelationship::Encryption)
								.encode()
								.as_slice(),
							LeafValue::<T::BlockNumber>::KeyReference.encode().as_slice(),
						)
						.map_err(|_| ())?;
					Ok(())
				})?;
			// Public keys
			identity
				.public_keys
				.iter()
				.try_for_each(|(id, key_details)| -> Result<(), ()> {
					trie_builder
						.insert(
							LeafKey::KeyDetails(*id).encode().as_slice(),
							LeafValue::KeyDetails(key_details.clone()).encode().as_slice(),
						)
						.map_err(|_| ())?;
					Ok(())
				})?;
			trie_builder.commit();
			Ok(trie_builder.root().to_owned())
		}

		// TODO: Better error handling
		#[allow(clippy::result_unit_err)]
		pub fn generate_proof(
			identity: &DidDetails<T>,
			key_ids: BTreeSet<KeyIdOf<T>>,
		) -> Result<(T::Hash, DidMerkleProof<T>), ()> {
			use sp_trie::generate_trie_proof;

			let mut db = MemoryDB::default();
			let root = Self::calculate_root_with_db(identity, &mut db)?;

			#[allow(clippy::type_complexity)]
			let mut leaves: BTreeMap<LeafKey<KeyIdOf<T>>, LeafValue<T::BlockNumber>> = BTreeMap::new();
			key_ids.iter().try_for_each(|key_id| {
				let key_details = identity.public_keys.get(key_id).ok_or(())?;
				if *key_id == identity.authentication_key {
					leaves.insert(
						LeafKey::KeyReference(*key_id, DidVerificationKeyRelationship::Authentication.into()),
						LeafValue::KeyReference,
					);
					Ok(())
				} else if let Some(key_id) = identity.attestation_key {
					leaves.insert(
						LeafKey::KeyReference(key_id, DidVerificationKeyRelationship::AssertionMethod.into()),
						LeafValue::KeyReference,
					);
					Ok(())
				} else if let Some(key_id) = identity.delegation_key {
					leaves.insert(
						LeafKey::KeyReference(key_id, DidVerificationKeyRelationship::CapabilityDelegation.into()),
						LeafValue::KeyReference,
					);
					Ok(())
				} else if identity.key_agreement_keys.contains(key_id) {
					leaves.insert(
						LeafKey::KeyReference(*key_id, KeyRelationship::Encryption),
						LeafValue::KeyReference,
					);
					Ok(())
				} else {
					Err(())
				}?;
				leaves.insert(LeafKey::KeyDetails(*key_id), LeafValue::KeyDetails(key_details.clone()));
				Ok::<_, ()>(())
			})?;
			let encoded_keys: Vec<Vec<u8>> = leaves.keys().map(|k| k.encode()).collect();
			let proof =
				generate_trie_proof::<LayoutV1<T::Hashing>, _, _, _>(&db, root, &encoded_keys).map_err(|_| ())?;
			Ok((
				root,
				DidMerkleProof::<T> {
					blinded: proof,
					revealed: leaves.into_iter().collect::<Vec<_>>(),
				},
			))
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

	#[derive(RuntimeDebug)]
	pub struct ProofEntry<BlockNumber: MaxEncodedLen> {
		pub key: DidPublicKeyDetails<BlockNumber>,
		pub relationship: KeyRelationship,
	}

	#[derive(RuntimeDebug)]
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
		type LeafKey = LeafKey<KeyId>;
		type LeafValue = LeafValue<BlockNumber>;
		type ProofDigest = <Hasher as sp_core::Hasher>::Out;
		type VerificationResult = VerificationResult<BlockNumber>;

		fn verify_proof_against_digest(
			proof: VersionedIdentityProof<Self::BlindedValue, Self::LeafKey, Self::LeafValue>,
			digest: Self::ProofDigest,
		) -> Result<Self::VerificationResult, Self::Error> {
			use dip_support::v1;
			use sp_trie::verify_trie_proof;

			let proof: v1::Proof<_, _, _> = proof.try_into()?;
			// TODO: more efficient by removing cloning and/or collecting. Did not find
			// another way of mapping a Vec<(Vec<u8>, Vec<u8>)> to a Vec<(Vec<u8>,
			// Option<Vec<u8>>)>.
			let proof_leaves = proof
				.revealed
				.iter()
				.map(|(key, value)| (key.encode(), Some(value.encode())))
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
				.filter_map(|(key, value)| {
					if let (LeafKey::KeyDetails(key_id), LeafValue::KeyDetails(key_details)) = (key, value) {
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
				.filter_map(|(key, value)| {
					if let (LeafKey::KeyReference(key_id, key_rel), LeafValue::KeyReference) = (key, value) {
						// TODO: Better error handling.
						let key_details = public_keys
							.get(&key_id)
							.expect("Key ID should be present in the map of revealed public keys.");
						Some(ProofEntry {
							key: key_details.clone(),
							relationship: key_rel,
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

	use did::{did_details::DidCreationDetails, KeyIdOf};
	use frame_support::{
		assert_ok, construct_runtime, parameter_types, traits::Everything, weights::constants::RocksDbWeight,
	};
	use frame_system::{
		mocking::{MockBlock, MockUncheckedExtrinsic},
		EnsureSigned, RawOrigin,
	};
	use pallet_dip_receiver::traits::IdentityProofVerifier;
	use sp_core::{ed25519, ConstU16, ConstU32, ConstU64, Hasher, Pair};
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
		type MaxNewKeyAgreementKeys = ConstU32<1>;
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
	fn authentication_merkle_proof_works() {
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
			assert_ok!(Did::create(
				RawOrigin::Signed(ALICE).into(),
				Box::new(create_details.clone()),
				did_auth_key.sign(&create_details.encode()).into()
			));
			let did_details = Did::get_did(&did).expect("DID should be present");
			let (root, proof) = DidMerkleRootGenerator::<TestRuntime>::generate_proof(
				&did_details,
				BTreeSet::from_iter(vec![did_details.authentication_key]),
			)
			.expect("Merkle proof generation should not fail.");
			println!("{:?} - {:?}", root, proof);
			assert_ok!(
				DidMerkleProofVerifier::<KeyIdOf<TestRuntime>, BlockNumber, Hashing>::verify_proof_against_digest(
					proof.into(),
					root
				)
			);
		})
	}
}
