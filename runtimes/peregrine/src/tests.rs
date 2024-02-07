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

use frame_support::{traits::Currency, BoundedVec};
use pallet_dip_provider::IdentityCommitmentOf;
use parity_scale_codec::MaxEncodedLen;

use did::DeriveDidCallAuthorizationVerificationKeyRelationship;
use pallet_did_lookup::associate_account_request::AssociateAccountRequest;
use pallet_treasury::BalanceOf;
use pallet_web3_names::{Web3NameOf, Web3OwnershipOf};
use runtime_common::{
	constants::{
		attestation::MAX_ATTESTATION_BYTE_LENGTH,
		deposit_storage::MAX_DEPOSIT_PALLET_KEY_LENGTH,
		did::{MAX_KEY_LENGTH, MAX_SERVICE_ENDPOINT_BYTE_LENGTH},
		did_lookup::MAX_CONNECTION_BYTE_LENGTH,
		dip_provider::MAX_COMMITMENT_BYTE_LENGTH,
		public_credentials::MAX_PUBLIC_CREDENTIAL_STORAGE_LENGTH,
		web3_names::MAX_NAME_BYTE_LENGTH,
		MAX_INDICES_BYTE_LENGTH,
	},
	AccountId, BlockNumber,
};

use crate::dip::deposit::DepositKey;

use super::{Runtime, RuntimeCall};

#[test]
fn call_size() {
	assert!(
		core::mem::size_of::<RuntimeCall>() <= 240,
		"size of Call is more than 240 bytes: some calls have too big arguments, use Box to reduce \
		the size of Call.
		If the limit is too strong, maybe consider increase the limit to 300.",
	);
}

#[test]
fn attestation_storage_sizes() {
	type DelegationRecord =
		BoundedVec<<Runtime as frame_system::Config>::Hash, <Runtime as attestation::Config>::MaxDelegatedAttestations>;

	let attestation_record = attestation::AttestationDetailsOf::<Runtime>::max_encoded_len();
	let delegation_record = DelegationRecord::max_encoded_len()
		/ (<Runtime as attestation::Config>::MaxDelegatedAttestations::get() as usize);
	assert_eq!(
		attestation_record + delegation_record,
		MAX_ATTESTATION_BYTE_LENGTH as usize
	)
}

#[test]
fn did_storage_sizes() {
	// Service endpoint
	let max_did_endpoint_size = did::service_endpoints::DidEndpoint::<Runtime>::max_encoded_len();
	assert_eq!(max_did_endpoint_size, MAX_SERVICE_ENDPOINT_BYTE_LENGTH as usize);

	// DID key
	let max_did_key_size = did::did_details::DidPublicKey::<AccountId>::max_encoded_len();
	assert_eq!(max_did_key_size, MAX_KEY_LENGTH as usize);
}

#[test]
fn did_lookup_storage_sizes() {
	type DidConnection =
		pallet_did_lookup::ConnectionRecord<
			<Runtime as pallet_did_lookup::Config>::DidIdentifier,
			<Runtime as frame_system::Config>::AccountId,
			<<Runtime as pallet_did_lookup::Config>::Currency as Currency<
				<Runtime as frame_system::Config>::AccountId,
			>>::Balance,
		>;

	let did_connection_size = DidConnection::max_encoded_len();

	assert_eq!(did_connection_size, MAX_CONNECTION_BYTE_LENGTH as usize)
}

#[test]
fn web3_name_storage_sizes() {
	let owner_size = Web3NameOf::<Runtime>::max_encoded_len();
	let name_size = Web3OwnershipOf::<Runtime>::max_encoded_len();

	assert_eq!(owner_size + name_size, MAX_NAME_BYTE_LENGTH as usize)
}

#[test]
fn indices_storage_sizes() {
	type Indices = (<Runtime as frame_system::Config>::AccountId, BalanceOf<Runtime>, bool);

	let size = Indices::max_encoded_len();
	assert_eq!(size, MAX_INDICES_BYTE_LENGTH as usize)
}

#[test]
fn public_credentials_storage_sizes() {
	// Stored in Credentials
	let credential_entry_max_size = public_credentials::CredentialEntryOf::<Runtime>::max_encoded_len();
	// Stored in CredentialsUnicityIndex
	let subject_id_max_size = <Runtime as public_credentials::Config>::SubjectId::max_encoded_len();

	// Each credential would have a different deposit, so no multiplier here
	assert_eq!(
		credential_entry_max_size + subject_id_max_size,
		MAX_PUBLIC_CREDENTIAL_STORAGE_LENGTH as usize
	)
}

#[test]
fn pallet_deposit_storage_max_key_length() {
	assert_eq!(DepositKey::max_encoded_len(), MAX_DEPOSIT_PALLET_KEY_LENGTH as usize)
}

#[test]
fn pallet_dip_provider_commitment_max_length() {
	assert_eq!(
		IdentityCommitmentOf::<Runtime>::max_encoded_len(),
		MAX_COMMITMENT_BYTE_LENGTH as usize
	)
}

#[test]
fn test_derive_did_verification_relation_ctype() {
	let c1 = RuntimeCall::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3],
	});
	let c2 = RuntimeCall::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3, 3],
	});
	let c3 = RuntimeCall::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3, 3],
	});
	let c4 = RuntimeCall::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 100],
	});

	let cb = RuntimeCall::Utility(pallet_utility::Call::batch {
		calls: vec![c1, c2, c3, c4],
	});
	assert_eq!(
		cb.derive_verification_key_relationship(),
		Ok(did::DidVerificationKeyRelationship::AssertionMethod)
	);
}

#[test]
fn test_derive_did_key_web3name() {
	assert_eq!(
		RuntimeCall::Web3Names(pallet_web3_names::Call::claim {
			name: b"test-name".to_vec().try_into().unwrap()
		})
		.derive_verification_key_relationship(),
		Ok(did::DidVerificationKeyRelationship::Authentication)
	);

	assert_eq!(
		RuntimeCall::Web3Names(pallet_web3_names::Call::release_by_owner {}).derive_verification_key_relationship(),
		Ok(did::DidVerificationKeyRelationship::Authentication)
	);
}

#[test]
fn test_derive_did_key_lookup() {
	assert_eq!(
		RuntimeCall::DidLookup(pallet_did_lookup::Call::associate_account {
			req: AssociateAccountRequest::Polkadot(
				AccountId::new([1u8; 32]),
				sp_runtime::MultiSignature::from(sp_core::ed25519::Signature([0; 64]))
			),
			expiration: BlockNumber::default(),
		})
		.derive_verification_key_relationship(),
		Ok(did::DidVerificationKeyRelationship::Authentication)
	);

	assert_eq!(
		RuntimeCall::DidLookup(pallet_did_lookup::Call::remove_account_association {
			account: AccountId::new([1u8; 32]).into(),
		})
		.derive_verification_key_relationship(),
		Ok(did::DidVerificationKeyRelationship::Authentication)
	);
}

#[test]
fn test_derive_did_verification_relation_fail() {
	let c1 = RuntimeCall::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3],
	});
	let c2 = RuntimeCall::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3, 3],
	});
	let c3 = RuntimeCall::System(frame_system::Call::remark {
		remark: vec![0, 1, 2, 3, 3],
	});
	let c4 = RuntimeCall::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 100],
	});

	let cb = RuntimeCall::Utility(pallet_utility::Call::batch {
		calls: vec![c1, c2, c3, c4],
	});

	#[cfg(feature = "runtime-benchmarks")]
	assert_eq!(
		cb.derive_verification_key_relationship(),
		Err(did::RelationshipDeriveError::InvalidCallParameter)
	);
	#[cfg(not(feature = "runtime-benchmarks"))]
	assert_eq!(
		cb.derive_verification_key_relationship(),
		Err(did::RelationshipDeriveError::NotCallableByDid)
	);
}

#[test]
fn test_derive_did_verification_relation_nested_fail() {
	let c1 = RuntimeCall::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3],
	});
	let c2 = RuntimeCall::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3, 3],
	});
	let f3 = RuntimeCall::System(frame_system::Call::remark {
		remark: vec![0, 1, 2, 3, 3],
	});
	let c4 = RuntimeCall::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 100],
	});

	let cb = RuntimeCall::Utility(pallet_utility::Call::batch {
		calls: vec![c1.clone(), c2.clone(), c4.clone()],
	});

	let cb = RuntimeCall::Utility(pallet_utility::Call::batch {
		calls: vec![c1, c2, cb, f3, c4],
	});

	#[cfg(feature = "runtime-benchmarks")]
	assert_eq!(
		cb.derive_verification_key_relationship(),
		Err(did::RelationshipDeriveError::InvalidCallParameter)
	);
	#[cfg(not(feature = "runtime-benchmarks"))]
	assert_eq!(
		cb.derive_verification_key_relationship(),
		Err(did::RelationshipDeriveError::NotCallableByDid)
	);
}

#[test]
fn test_derive_did_verification_relation_nested() {
	let c1 = RuntimeCall::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3],
	});
	let c2 = RuntimeCall::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3, 3],
	});
	let c4 = RuntimeCall::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 100],
	});

	let cb = RuntimeCall::Utility(pallet_utility::Call::batch {
		calls: vec![c1.clone(), c2.clone(), c4.clone()],
	});

	let cb = RuntimeCall::Utility(pallet_utility::Call::batch {
		calls: vec![c1, c2, cb, c4],
	});
	assert_eq!(
		cb.derive_verification_key_relationship(),
		Ok(did::DidVerificationKeyRelationship::AssertionMethod)
	);
}

#[test]
fn test_derive_did_verification_relation_single() {
	let c1 = RuntimeCall::Ctype(ctype::Call::add {
		ctype: vec![0, 1, 2, 3],
	});

	let cb = RuntimeCall::Utility(pallet_utility::Call::batch { calls: vec![c1] });

	assert_eq!(
		cb.derive_verification_key_relationship(),
		Ok(did::DidVerificationKeyRelationship::AssertionMethod)
	);
}

#[test]
fn test_derive_did_verification_relation_empty() {
	let cb = RuntimeCall::Utility(pallet_utility::Call::batch { calls: vec![] });

	assert_eq!(
		cb.derive_verification_key_relationship(),
		Err(did::RelationshipDeriveError::InvalidCallParameter)
	);
}

mod dip_provider {
	use super::*;
	use did::{
		did_details::{DidDetails, DidEncryptionKey, DidVerificationKey},
		DidRawOrigin, DidSignature, EnsureDidOrigin, KeyIdOf,
	};
	use frame_support::{construct_runtime, parameter_types, traits::Everything};
	use frame_system::EnsureRoot;
	use hex_literal::hex;
	use kilt_dip_primitives::{DipDidProofWithVerifiedCommitment, TimeBoundDidSignature};
	use pallet_did_lookup::linkable_account::LinkableAccountId;
	use pallet_dip_provider::{traits::IdentityProvider, NoopHooks};
	use runtime_common::{
		constants::{self, EXISTENTIAL_DEPOSIT, KILT},
		dip::{did::LinkedDidInfoProvider, merkle::DidMerkleRootGenerator},
		Balance, BlockHashCount, BlockLength, BlockWeights, DidIdentifier, Hash, Nonce,
	};
	use sp_core::{crypto::Ss58Codec, ed25519, sr25519, ConstU32};
	use sp_runtime::{
		traits::{AccountIdLookup, BlakeTwo256},
		AccountId32, BuildStorage,
	};

	construct_runtime!(
		pub struct TestRuntime {
			System: frame_system,
			Balances: pallet_balances,
			Did: did,
			DidLookup: pallet_did_lookup,
			Web3Names: pallet_web3_names,
			DipProvider: pallet_dip_provider,
		}
	);

	impl frame_system::Config for TestRuntime {
		type AccountId = AccountId;
		type RuntimeCall = RuntimeCall;
		type Lookup = AccountIdLookup<AccountId, ()>;
		type Nonce = Nonce;
		type Block = frame_system::mocking::MockBlock<TestRuntime>;
		type Hash = Hash;
		type Hashing = BlakeTwo256;
		type RuntimeEvent = RuntimeEvent;
		type RuntimeOrigin = RuntimeOrigin;
		type BlockHashCount = BlockHashCount;
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = pallet_balances::AccountData<Balance>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type DbWeight = ();
		type BaseCallFilter = Everything;
		type SystemWeightInfo = ();
		type BlockWeights = BlockWeights;
		type BlockLength = BlockLength;
		type SS58Prefix = ();
		type OnSetCode = ();
		type MaxConsumers = ConstU32<16>;
	}

	parameter_types! {
		pub const ExistentialDeposit: u128 = EXISTENTIAL_DEPOSIT;
		pub const MaxLocks: u32 = 50;
		pub const MaxReserves: u32 = 50;
		pub const MaxHolds: u32 = 50;
		pub const MaxFreezes: u32 = 50;
	}

	impl pallet_balances::Config for TestRuntime {
		type Balance = Balance;
		type FreezeIdentifier = RuntimeFreezeReason;
		type RuntimeHoldReason = RuntimeHoldReason;
		type MaxFreezes = MaxFreezes;
		type MaxHolds = MaxHolds;

		/// The ubiquitous event type.
		type RuntimeEvent = RuntimeEvent;
		type DustRemoval = ();
		type ExistentialDeposit = ExistentialDeposit;
		type AccountStore = System;
		type WeightInfo = ();
		type MaxLocks = MaxLocks;
		type MaxReserves = MaxReserves;
		type ReserveIdentifier = [u8; 8];
	}

	impl did::Config for TestRuntime {
		type RuntimeEvent = RuntimeEvent;
		type RuntimeCall = RuntimeCall;
		type RuntimeHoldReason = RuntimeHoldReason;
		type RuntimeOrigin = RuntimeOrigin;
		type Currency = Balances;
		type DidIdentifier = DidIdentifier;
		type KeyDeposit = constants::did::KeyDeposit;
		type ServiceEndpointDeposit = constants::did::ServiceEndpointDeposit;
		type BaseDeposit = constants::did::DidBaseDeposit;
		type Fee = constants::did::DidFee;
		type FeeCollector = ();

		#[cfg(not(feature = "runtime-benchmarks"))]
		type EnsureOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
		#[cfg(not(feature = "runtime-benchmarks"))]
		type OriginSuccess = did::DidRawOrigin<AccountId, DidIdentifier>;

		#[cfg(feature = "runtime-benchmarks")]
		type EnsureOrigin = frame_system::EnsureSigned<DidIdentifier>;
		#[cfg(feature = "runtime-benchmarks")]
		type OriginSuccess = DidIdentifier;

		type MaxNewKeyAgreementKeys = constants::did::MaxNewKeyAgreementKeys;
		type MaxTotalKeyAgreementKeys = constants::did::MaxTotalKeyAgreementKeys;
		type MaxPublicKeysPerDid = constants::did::MaxPublicKeysPerDid;
		type MaxBlocksTxValidity = constants::did::MaxBlocksTxValidity;
		type MaxNumberOfServicesPerDid = constants::did::MaxNumberOfServicesPerDid;
		type MaxServiceIdLength = constants::did::MaxServiceIdLength;
		type MaxServiceTypeLength = constants::did::MaxServiceTypeLength;
		type MaxServiceUrlLength = constants::did::MaxServiceUrlLength;
		type MaxNumberOfTypesPerService = constants::did::MaxNumberOfTypesPerService;
		type MaxNumberOfUrlsPerService = constants::did::MaxNumberOfUrlsPerService;
		type WeightInfo = ();
		type BalanceMigrationManager = ();
	}

	impl pallet_did_lookup::Config for TestRuntime {
		type RuntimeHoldReason = RuntimeHoldReason;
		type RuntimeEvent = RuntimeEvent;

		type DidIdentifier = DidIdentifier;

		type Currency = Balances;
		type Deposit = constants::did_lookup::DidLookupDeposit;

		type EnsureOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
		type OriginSuccess = did::DidRawOrigin<AccountId, DidIdentifier>;

		type WeightInfo = ();
		type BalanceMigrationManager = ();
	}

	impl pallet_web3_names::Config for TestRuntime {
		type RuntimeHoldReason = RuntimeHoldReason;
		type BanOrigin = EnsureRoot<AccountId>;
		type OwnerOrigin = did::EnsureDidOrigin<DidIdentifier, AccountId>;
		type OriginSuccess = did::DidRawOrigin<AccountId, DidIdentifier>;
		type Currency = Balances;
		type Deposit = constants::web3_names::Web3NameDeposit;
		type RuntimeEvent = RuntimeEvent;
		type MaxNameLength = constants::web3_names::MaxNameLength;
		type MinNameLength = constants::web3_names::MinNameLength;
		type Web3Name = pallet_web3_names::web3_name::AsciiWeb3Name<TestRuntime>;
		type Web3NameOwner = DidIdentifier;
		type WeightInfo = ();
		type BalanceMigrationManager = ();
	}

	impl pallet_dip_provider::Config for TestRuntime {
		type CommitOriginCheck = EnsureDidOrigin<DidIdentifier, AccountId>;
		type CommitOrigin = DidRawOrigin<DidIdentifier, AccountId>;
		type Identifier = DidIdentifier;
		type IdentityCommitmentGenerator = DidMerkleRootGenerator<TestRuntime>;
		type IdentityProvider = LinkedDidInfoProvider<20>;
		type ProviderHooks = NoopHooks;
		type RuntimeEvent = RuntimeEvent;
		type WeightInfo = ();
	}

	impl did::DeriveDidCallAuthorizationVerificationKeyRelationship for RuntimeCall {
		fn derive_verification_key_relationship(&self) -> did::DeriveDidCallKeyRelationshipResult {
			match self {
				RuntimeCall::DipProvider { .. } => Ok(did::DidVerificationKeyRelationship::Authentication),
				// DID creation is not allowed through the DID proxy.
				RuntimeCall::Did(did::Call::create { .. }) => Err(did::RelationshipDeriveError::NotCallableByDid),
				RuntimeCall::Did { .. } => Ok(did::DidVerificationKeyRelationship::Authentication),
				RuntimeCall::Web3Names { .. } => Ok(did::DidVerificationKeyRelationship::Authentication),
				RuntimeCall::DidLookup { .. } => Ok(did::DidVerificationKeyRelationship::Authentication),
				#[cfg(not(feature = "runtime-benchmarks"))]
				_ => Err(did::RelationshipDeriveError::NotCallableByDid),
				// By default, returns the authentication key
				#[cfg(feature = "runtime-benchmarks")]
				_ => Ok(did::DidVerificationKeyRelationship::Authentication),
			}
		}

		// Always return a System::remark() extrinsic call
		#[cfg(feature = "runtime-benchmarks")]
		fn get_call_for_did_call_benchmark() -> Self {
			RuntimeCall::System(frame_system::Call::remark { remark: vec![] })
		}
	}

	struct ExtBuilder;

	impl ExtBuilder {
		fn build(self) -> sp_io::TestExternalities {
			sp_io::TestExternalities::new(
				frame_system::GenesisConfig::<TestRuntime>::default()
					.build_storage()
					.unwrap(),
			)
		}
	}

	// TODO: Set up unit test so that it generates the same proof that fails to
	// verify in the kilt-dip-primitive benchmarking fixture.
	#[test]
	fn test_dip_proof_generation() {
		env_logger::init();
		ExtBuilder.build().execute_with(|| {
			pallet_balances::Pallet::<TestRuntime>::make_free_balance_be(&AccountId32::from([0u8; 32]), 1_000 * KILT);
			frame_system::Pallet::<TestRuntime>::set_block_number(30);

			let did_identifier =
				AccountId32::from_ss58check("5F7Q4Tv8A2Wob14H6V7eGqhhcFEXzjZXSDptYrhxdxATe5qV").unwrap();
			let mut did_details = DidDetails::new(
				DidVerificationKey::Sr25519(sr25519::Public(hex!(
					"86c2871ed4042fc2a4c1399619c4ca17b7c9585768d90b77376045a39d9a702f"
				))),
				30,
				AccountId::from([0u8; 32]),
			)
			.unwrap();
			did_details
				.add_key_agreement_key(
					DidEncryptionKey::X25519(hex!("fe2070c665fa802a3263fc8a89321321184918e584b9499cb84fa38911d11f7f")),
					30,
				)
				.unwrap();
			did_details
				.add_key_agreement_key(
					DidEncryptionKey::X25519(hex!("142d0e8808dd5a1287256cbbd64d06aef686606d872e824cf56b492902000a79")),
					30,
				)
				.unwrap();
			did_details
				.add_key_agreement_key(
					DidEncryptionKey::X25519(hex!("4c0aa2e8f3cf029e08759e8b61d244a6192a2271642bb5da7b5d29990b5da00b")),
					30,
				)
				.unwrap();
			did_details
				.add_key_agreement_key(
					DidEncryptionKey::X25519(hex!("9f97d434ca3cb7b727928beb46cf49f27da871b63ae5447e3ec3b5abb08c9c0e")),
					30,
				)
				.unwrap();
			did_details
				.add_key_agreement_key(
					DidEncryptionKey::X25519(hex!("024b98aac3d5ec1b786293c2a50e7b3ac993919c492f4da5a93f94f0f9cdb241")),
					30,
				)
				.unwrap();
			did_details
				.add_key_agreement_key(
					DidEncryptionKey::X25519(hex!("4a53daee31bc9cdce8896026c31d7621ad90854bdec56f077c2b135aa36f7a18")),
					30,
				)
				.unwrap();
			did_details
				.add_key_agreement_key(
					DidEncryptionKey::X25519(hex!("d523029cf92cbf98206572e11d2c315f9750cf467f7be745034776c8c7552e6f")),
					30,
				)
				.unwrap();
			did_details
				.add_key_agreement_key(
					DidEncryptionKey::X25519(hex!("4ac14eed420bc9f97db98b198133d094dd9c2ebcbe00cc6dd4576b4da6515c65")),
					30,
				)
				.unwrap();
			did_details
				.add_key_agreement_key(
					DidEncryptionKey::X25519(hex!("cabf0743ea77e7a4aec9ab7135700482079a9b4eff89a46add7608192680a413")),
					30,
				)
				.unwrap();
			did_details
				.add_key_agreement_key(
					DidEncryptionKey::X25519(hex!("fe1eb53fd59a1a27b893f0d663df84cc33fc5923d146c26eb57d8158480c7e52")),
					30,
				)
				.unwrap();

			did_details
				.update_delegation_key(
					DidVerificationKey::Ed25519(ed25519::Public(hex!(
						"61d5bd79fe0095640a7bb05e791cae4317d4575e817f41628648fc4ff5271f2d"
					))),
					30,
				)
				.unwrap();
			did_details
				.update_attestation_key(
					DidVerificationKey::Ed25519(ed25519::Public(hex!(
						"34c4685c61e5d6a7ff4e42d1594735285f6c116f1c6e8f70f12a3942655a7c34"
					))),
					30,
				)
				.unwrap();

			did::Pallet::<TestRuntime>::try_insert_did(
				did_identifier.clone(),
				did_details,
				AccountId32::from([0u8; 32]),
			)
			.unwrap();

			pallet_did_lookup::Pallet::<TestRuntime>::add_association(
				AccountId32::from([0u8; 32]),
				did_identifier.clone(),
				LinkableAccountId::AccountId32(
					hex!("a36777a228a2e0651764c7de0be063f9a1cc0281aedececc12cfdc69e048b7cf").into(),
				),
			)
			.unwrap();
			pallet_did_lookup::Pallet::<TestRuntime>::add_association(
				AccountId32::from([0u8; 32]),
				did_identifier.clone(),
				LinkableAccountId::AccountId32(
					hex!("0753e95eac51474dc20653d86195cd11657b3af8f9af52d03f6b42a6cbe78efa").into(),
				),
			)
			.unwrap();
			pallet_did_lookup::Pallet::<TestRuntime>::add_association(
				AccountId32::from([0u8; 32]),
				did_identifier.clone(),
				LinkableAccountId::AccountId32(
					hex!("e154fe906fd0c0227be2967528a972215ceac09ede6167421a761507cea3f1b7").into(),
				),
			)
			.unwrap();
			pallet_did_lookup::Pallet::<TestRuntime>::add_association(
				AccountId32::from([0u8; 32]),
				did_identifier.clone(),
				LinkableAccountId::AccountId32(
					hex!("e0e591ff4d23b8b5cc3655e59aa140c600565c1c21f27960cfb980a4c74b6b03").into(),
				),
			)
			.unwrap();
			pallet_did_lookup::Pallet::<TestRuntime>::add_association(
				AccountId32::from([0u8; 32]),
				did_identifier.clone(),
				LinkableAccountId::AccountId32(
					hex!("98812f5ef930d26a12e8ebcef5b5ce6f5458af9028694d49902172d969e83738").into(),
				),
			)
			.unwrap();
			pallet_did_lookup::Pallet::<TestRuntime>::add_association(
				AccountId32::from([0u8; 32]),
				did_identifier.clone(),
				LinkableAccountId::AccountId32(
					hex!("e60b14479bb4cee4f526ae31541bd3dbc79f541d01cf0a9691c1dbb1bfeb42d4").into(),
				),
			)
			.unwrap();
			pallet_did_lookup::Pallet::<TestRuntime>::add_association(
				AccountId32::from([0u8; 32]),
				did_identifier.clone(),
				LinkableAccountId::AccountId32(
					hex!("36dfc8d778b1b15835d8bc7953b1d782b38aee53ef7785340ec451a1dcc0cdc1").into(),
				),
			)
			.unwrap();
			pallet_did_lookup::Pallet::<TestRuntime>::add_association(
				AccountId32::from([0u8; 32]),
				did_identifier.clone(),
				LinkableAccountId::AccountId32(
					hex!("e808d3d3fe19d0fd336cff473acbc2cf93074c9e163a5d04a9ea33a918e89105").into(),
				),
			)
			.unwrap();
			pallet_did_lookup::Pallet::<TestRuntime>::add_association(
				AccountId32::from([0u8; 32]),
				did_identifier.clone(),
				LinkableAccountId::AccountId32(
					hex!("88dd8180874f51ca369e5a970fc2d09789af161a8ae187df94ff1658884441a4").into(),
				),
			)
			.unwrap();
			pallet_did_lookup::Pallet::<TestRuntime>::add_association(
				AccountId32::from([0u8; 32]),
				did_identifier.clone(),
				LinkableAccountId::AccountId32(
					hex!("5f97e76eefbd998e4886319066e29d7646f7de9b812cef5654536b6a39e257a2").into(),
				),
			)
			.unwrap();

			pallet_web3_names::Pallet::<TestRuntime>::register_name(
				b"b0d832f8c9b145a45537e86".to_vec().try_into().unwrap(),
				did_identifier.clone(),
				AccountId32::from([0u8; 32]),
			)
			.unwrap();

			let identity_details =
				pallet_dip_provider::IdentityProviderOf::<TestRuntime>::retrieve(&did_identifier).unwrap();

			let proof = DidMerkleRootGenerator::<TestRuntime>::generate_proof(
				&identity_details,
				0,
				[
					&hex!("c8585a08f8de24ca2fe9ac237846bfa50ae0be99bc8d1dcd1bbf2d5dcb5469de").into(),
					&hex!("046171527563d8e98a5222de4a9b4141b088da96377e8efc3388c0538a36fe68").into(),
					&hex!("323486d437a096fd79ff975506e5829beace1e88d0348d6e25a6e465a8b89ab6").into(),
					&hex!("7d8c9f4a66a0e93717dddcadba70889c4424f338f589d09714f239043591440d").into(),
					&hex!("8c86c882487160789d32d32e697b2d8e520f1563cb1e4164225753aa28a5ab2f").into(),
					&hex!("8cc1078ba56113813991934ae0ffabbd5cb562cb6a2fc699f9bb8b4aabb8f616").into(),
					&hex!("90fd143d81ddef983d62ce1aa1639168489ff5da0f6702efd258da6c55b0777e").into(),
					&hex!("9aeeb6b118090da14b14065df0bfa29a394e3c57574c2cf72fec38027953f2ca").into(),
					&hex!("c0339da5d721ecf5a599e588a74363db45688e2356ad15bf80f226017bd10248").into(),
					&hex!("cfdcf6e5989bdee27413af55c187c96387dfd2e4f3cde973b250acfac6af6cad").into(),
					&hex!("fda35a39cce44fbfb012b0f6a0a1dfa0b866b0ed8a2f7efb3c7bc944b8b49493").into(),
					&hex!("1b4f12a6cc3a3d8c3d2d508aadbfa6b71edbb9ac7ec3da2f0448ca8035f95c22").into(),
					&hex!("77d0160b28ad4d2f5db38e2192867914fec04eff50dd860bc46df10b36bf3b7b").into(),
				]
				.into_iter(),
				true,
				[
					LinkableAccountId::AccountId32(
						hex!("a36777a228a2e0651764c7de0be063f9a1cc0281aedececc12cfdc69e048b7cf").into(),
					),
					LinkableAccountId::AccountId32(
						hex!("0753e95eac51474dc20653d86195cd11657b3af8f9af52d03f6b42a6cbe78efa").into(),
					),
					LinkableAccountId::AccountId32(
						hex!("e154fe906fd0c0227be2967528a972215ceac09ede6167421a761507cea3f1b7").into(),
					),
					LinkableAccountId::AccountId32(
						hex!("e0e591ff4d23b8b5cc3655e59aa140c600565c1c21f27960cfb980a4c74b6b03").into(),
					),
					LinkableAccountId::AccountId32(
						hex!("98812f5ef930d26a12e8ebcef5b5ce6f5458af9028694d49902172d969e83738").into(),
					),
					LinkableAccountId::AccountId32(
						hex!("e60b14479bb4cee4f526ae31541bd3dbc79f541d01cf0a9691c1dbb1bfeb42d4").into(),
					),
					LinkableAccountId::AccountId32(
						hex!("36dfc8d778b1b15835d8bc7953b1d782b38aee53ef7785340ec451a1dcc0cdc1").into(),
					),
					LinkableAccountId::AccountId32(
						hex!("e808d3d3fe19d0fd336cff473acbc2cf93074c9e163a5d04a9ea33a918e89105").into(),
					),
					LinkableAccountId::AccountId32(
						hex!("88dd8180874f51ca369e5a970fc2d09789af161a8ae187df94ff1658884441a4").into(),
					),
					LinkableAccountId::AccountId32(
						hex!("5f97e76eefbd998e4886319066e29d7646f7de9b812cef5654536b6a39e257a2").into(),
					),
				]
				.iter(),
			)
			.unwrap();
			let root = proof.root;
			assert_eq!(
				root,
				hex!("30abd7efa72c7cbdb7967be6423b4ac91cf2d2e16b09a92865d21942d7104a81").into()
			);
			let proof = proof.proof;
			let external_proof = DipDidProofWithVerifiedCommitment::<
				IdentityCommitmentOf<TestRuntime>,
				KeyIdOf<TestRuntime>,
				AccountId32,
				u64,
				Web3NameOf<TestRuntime>,
				LinkableAccountId,
				u64,
			>::new(
				root,
				proof,
				TimeBoundDidSignature::new(DidSignature::Sr25519(sr25519::Signature([0u8; 64])), 100),
			);
			external_proof.verify_dip_proof::<BlakeTwo256, 50>().unwrap();
			// println!("Generated root: {:#?}", hex::decode(root).unwrap());
			// println!("{:#?}", proof);
		})
	}
}
