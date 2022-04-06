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

use codec::Encode;

use frame_support::dispatch::Weight;
use sp_runtime::DispatchError;

use did::{DeriveDidCallAuthorizationVerificationType, DidAuthorizedCallOperation, DidAuthorizedCallOperationWithVerificationRelationship, DidVerificationType, DidVerifiableIdentifier};

pub struct DidCallProxy<Config>(sp_std::marker::PhantomData<Config>);
impl<T: did::Config> did::DidCallProxy<T> for DidCallProxy<T> {
	fn weight(did_call: &DidAuthorizedCallOperation<T>) -> Weight {
		// TODO: Use runtime-specific weights for the inline and stored signature verification
		todo!()
	}

	fn authorise(
		did_call: &DidAuthorizedCallOperation<T>,
		signature: &did::DidSignature,
	) -> Result<(), DispatchError> {
		let verification_key_relationship = did_call
			.call
			.derive_verification_key_relationship()
			.map_err(did::Error::<T>::from)?;

		match verification_key_relationship {
			DidVerificationType::Inline => {
				did_call.did
					.verify_and_recover_signature(&did_call.encode(), &signature)
					.map_err(did::Error::<T>::from)?;
				Ok(())
			}
			DidVerificationType::StoredVerificationKey(key_relationship) => {
				let wrapped_operation = DidAuthorizedCallOperationWithVerificationRelationship {
					operation: did_call.clone(),
					verification_key_relationship: key_relationship,
				};

				did::Pallet::<T>::verify_did_operation_signature_and_increase_nonce(&wrapped_operation, &signature)
					.map_err(did::Error::<T>::from)?;
				Ok(())
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use super::DidCallProxy as Proxy;

	use frame_support::{
		assert_err,
		assert_ok,
		parameter_types,
		weights::constants::RocksDbWeight,
	};
	use frame_system::EnsureSigned;
	use sp_core::{ed25519, sr25519, Pair};
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
		MultiSignature, MultiSigner,
	};
	use sp_std::vec::Vec;

	use did::{DidDetails, DidSignature, DidCallProxy};

	pub(crate) type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
	pub(crate) type Block = frame_system::mocking::MockBlock<TestRuntime>;
	pub(crate) type Hash = sp_core::H256;
	pub(crate) type Balance = u128;
	pub(crate) type Signature = MultiSignature;
	pub(crate) type AccountPublic = <Signature as Verify>::Signer;
	pub(crate) type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
	pub(crate) type Index = u64;
	pub(crate) type BlockNumber = u64;

	pub(crate) type DidIdentifier = AccountId;

	const MICRO_KILT: Balance = 10u128.pow(9);

	frame_support::construct_runtime!(
		pub enum TestRuntime where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			Did: did::{Pallet, Call, Storage, Event<T>, Origin<T>},
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		}
	);

	impl did::DeriveDidCallAuthorizationVerificationType for Call {
		fn derive_verification_key_relationship(&self) -> did::DeriveDidVerificationTypeResult {
			match *self {
				Call::System(frame_system::Call::remark { .. }) => Ok(did::DidVerificationType::inline()),
				Call::System(frame_system::Call::remark_with_event { .. }) => Ok(did::DidVerificationType::StoredVerificationKey(did::DidVerificationKeyRelationship::Authentication)),
				_ => Err(did::RelationshipDeriveError::NotCallableByDid)
			}
		}
	}

	parameter_types! {
		pub const SS58Prefix: u8 = 38;
		pub const BlockHashCount: u64 = 250;
	}

	impl frame_system::Config for TestRuntime {
		type Origin = Origin;
		type Call = Call;
		type Index = Index;
		type BlockNumber = BlockNumber;
		type Hash = Hash;
		type Hashing = BlakeTwo256;
		type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type DbWeight = RocksDbWeight;
		type Version = ();

		type PalletInfo = PalletInfo;
		type AccountData = pallet_balances::AccountData<Balance>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type BaseCallFilter = frame_support::traits::Everything;
		type SystemWeightInfo = ();
		type BlockWeights = ();
		type BlockLength = ();
		type SS58Prefix = SS58Prefix;
		type OnSetCode = ();
		type MaxConsumers = frame_support::traits::ConstU32<16>;
	}

	parameter_types! {
		#[derive(Debug, Clone, PartialEq)]
		pub const MaxUrlLength: u32 = 200u32;
		#[derive(Debug, Clone, PartialEq)]
		pub const MaxTotalKeyAgreementKeys: u32 = 10u32;
		// IMPORTANT: Needs to be at least MaxTotalKeyAgreementKeys + 3 (auth, delegation, attestation keys) for benchmarks!
		#[derive(Debug, Clone)]
		pub const MaxPublicKeysPerDid: u32 = 13u32;
		pub const MaxBlocksTxValidity: u64 = 300u64;
		pub const Deposit: Balance = 10 * MICRO_KILT;
		pub const DidFee: Balance = MICRO_KILT;
		pub const MaxNumberOfServicesPerDid: u32 = 25u32;
		pub const MaxServiceIdLength: u32 = 50u32;
		pub const MaxServiceTypeLength: u32 = 50u32;
		pub const MaxServiceUrlLength: u32 = 100u32;
		pub const MaxNumberOfTypesPerService: u32 = 1u32;
		pub const MaxNumberOfUrlsPerService: u32 = 1u32;
	}

	impl did::Config for TestRuntime {
		type DidIdentifier = DidIdentifier;
		type Origin = Origin;
		type Call = Call;
		type DidCallProxy = super::DidCallProxy<Self>;
		type EnsureOrigin = EnsureSigned<DidIdentifier>;
		type OriginSuccess = AccountId;
		type Event = ();
		type Currency = ();
		type Deposit = ();
		type Fee = ();
		type FeeCollector = ();
		type MaxTotalKeyAgreementKeys = MaxTotalKeyAgreementKeys;
		type MaxPublicKeysPerDid = MaxPublicKeysPerDid;
		type MaxBlocksTxValidity = MaxBlocksTxValidity;
		type WeightInfo = ();
		type MaxNumberOfServicesPerDid = MaxNumberOfServicesPerDid;
		type MaxServiceIdLength = MaxServiceIdLength;
		type MaxServiceTypeLength = MaxServiceTypeLength;
		type MaxServiceUrlLength = MaxServiceUrlLength;
		type MaxNumberOfTypesPerService = MaxNumberOfTypesPerService;
		type MaxNumberOfUrlsPerService = MaxNumberOfUrlsPerService;
	}

	#[derive(Clone, Default)]
	struct ExtBuilder {
		dids_stored: Vec<(DidIdentifier, DidDetails<TestRuntime>)>,
	}

	impl ExtBuilder {
		#[must_use]
		pub fn with_dids(mut self, dids: Vec<(DidIdentifier, DidDetails<TestRuntime>)>) -> Self {
			self.dids_stored = dids;
			self
		}

		pub fn build(self, ext: Option<sp_io::TestExternalities>) -> sp_io::TestExternalities {
			let mut ext = if let Some(ext) = ext {
				ext
			} else {
				let storage = frame_system::GenesisConfig::default().build_storage::<TestRuntime>().unwrap();
				sp_io::TestExternalities::new(storage)
			};

			ext.execute_with(|| {
				for did in self.dids_stored.iter() {
					did::Did::<TestRuntime>::insert(&did.0, did.1.clone());
				}
			});

			ext
		}
	}

	const ACCOUNT_00: AccountId = AccountId::new([1u8; 32]);
	const DEFAULT_AUTH_SEED: [u8; 32] = [4u8; 32];
	const ALTERNATIVE_AUTH_SEED: [u8; 32] = [40u8; 32];

	fn get_ed25519_authentication_key(default: bool) -> ed25519::Pair {
		if default {
			ed25519::Pair::from_seed(&DEFAULT_AUTH_SEED)
		} else {
			ed25519::Pair::from_seed(&ALTERNATIVE_AUTH_SEED)
		}
	}

	fn get_sr25519_authentication_key(default: bool) -> sr25519::Pair {
		if default {
			sr25519::Pair::from_seed(&DEFAULT_AUTH_SEED)
		} else {
			sr25519::Pair::from_seed(&ALTERNATIVE_AUTH_SEED)
		}
	}

	fn get_did_identifier_from_ed25519_key(public_key: ed25519::Public) -> DidIdentifier {
		MultiSigner::from(public_key).into_account()
	}

	fn get_did_identifier_from_sr25519_key(public_key: sr25519::Public) -> DidIdentifier {
		MultiSigner::from(public_key).into_account()
	}

	#[test]
	fn test_correct_inline_operation() {
		let auth_key = get_ed25519_authentication_key(true);
		let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

		let call = Call::System(frame_system::Call::<TestRuntime>::remark { remark: vec![1; 32] });
		let did_call = DidAuthorizedCallOperation::<TestRuntime> { block_number: 0u64.into(), call, did: alice_did, submitter: ACCOUNT_00, tx_counter: 0 };
		let encoded_did_call = did_call.encode();
		let did_signature = DidSignature::from(auth_key.sign(&encoded_did_call));

		assert_ok!(Proxy::authorise(&did_call, &did_signature));
	}

	#[test]
	fn test_wrong_signature_format_inline_operation() {
		let auth_key = get_ed25519_authentication_key(true);
		let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

		let call = Call::System(frame_system::Call::<TestRuntime>::remark { remark: vec![1; 32] });
		let did_call = DidAuthorizedCallOperation::<TestRuntime> { block_number: 0u64.into(), call, did: alice_did, submitter: ACCOUNT_00, tx_counter: 0 };
		let encoded_did_call = did_call.encode();

		let alternative_auth_key = get_sr25519_authentication_key(true);
		let did_signature = DidSignature::from(alternative_auth_key.sign(&encoded_did_call));

		// Fails with InvalidSignature because it tries to re-create an Sr25519 key given the Sr25519 signature but of course it fails.
		assert_err!(Proxy::authorise(&did_call, &did_signature), did::Error::<TestRuntime>::InvalidSignature);
	}

	#[test]
	fn test_wrong_signature_inline_operation() {
		let auth_key = get_ed25519_authentication_key(true);
		let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

		let call = Call::System(frame_system::Call::<TestRuntime>::remark { remark: vec![1; 32] });
		let did_call = DidAuthorizedCallOperation::<TestRuntime> { block_number: 0u64.into(), call, did: alice_did, submitter: ACCOUNT_00, tx_counter: 0 };
		// Sign a random byte array
		let did_signature = DidSignature::from(auth_key.sign(&vec![10; 64]));

		// Fails with InvalidSignature because it tries to re-create an Sr25519 key given the Sr25519 signature but of course it fails.
		assert_err!(Proxy::authorise(&did_call, &did_signature), did::Error::<TestRuntime>::InvalidSignature);
	}
}
