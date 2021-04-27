// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

//! The KILT runtime. This can be compiled with `#[no_std]`, ready for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
// The `from_over_into` warning originates from `construct_runtime` macro.
#![allow(clippy::from_over_into)]

// Make the WASM binary available
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use grandpa::{fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use kilt_primitives::{
	constants::{DOLLARS, MIN_VESTED_TRANSFER_AMOUNT, SLOT_DURATION},
	AccountId, Balance, BlockNumber, Hash, Index, Signature,
};
use pallet_transaction_payment::{CurrencyAdapter, FeeDetails};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::{ed25519::AuthorityId as AuraId, SlotDuration};
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto, NumberFor, OpaqueKeys, Verify},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// pub use consensus::Call as ConsensusCall;
pub use balances::Call as BalancesCall;

pub use frame_support::{
	construct_runtime, parameter_types,
	traits::{Currency, FindAuthor, Imbalance, KeyOwnerProofSystem, OnUnbalanced, Randomness},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
		IdentityFee, Weight,
	},
	ConsensusEngineId, StorageValue,
};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};
pub use timestamp::Call as TimestampCall;

pub use attestation;
pub use ctype;
pub use delegation;
pub use did;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

pub type NegativeImbalance<T> =
	<balances::Pallet<T> as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

/// Opaque types. These are used by the CLI to instantiate machinery that don't
/// need to know the specifics of the runtime. They can then be made to be
/// agnostic over specific formats of data like extrinsics, allowing for them to
/// continue syncing the network through upgrades to even the core data
/// structures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
		}
	}
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("mashnet-node"),
	impl_name: create_runtime_str!("mashnet-node"),
	authoring_version: 4,
	spec_version: 8,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 2,
};

/// The version information used to identify this runtime when compiled
/// natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const BlockHashCount: BlockNumber = 2400;
	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
		::with_sensible_defaults(2 * WEIGHT_PER_SECOND, NORMAL_DISPATCH_RATIO);
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 38;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = ();
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = BlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = BlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in
	/// dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest
	/// pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// Converts a Pallet to the index of the Pallet in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The data to be stored in an account.
	type AccountData = balances::AccountData<Balance>;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// This is used as an identifier of the chain. 42 is the generic substrate
	/// prefix.
	type SS58Prefix = SS58Prefix;
	/// The set code logic, just the default since we're not a parachain.
	type OnSetCode = ();
}

impl aura::Config for Runtime {
	type AuthorityId = AuraId;
}

impl grandpa::Config for Runtime {
	type Event = Event;
	type Call = Call;

	type KeyOwnerProofSystem = ();

	type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

	type KeyOwnerIdentification =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::IdentificationTuple;

	type HandleEquivocation = ();

	type WeightInfo = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const Deposit: Balance = 1_000;
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = Index;
	type Currency = Balances;
	type Deposit = Deposit;
	type Event = Event;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: Balance = 500;
	pub const MaxLocks: u32 = 50;
}

impl balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = balances::weights::SubstrateWeight<Runtime>;
}

/// Logic for the author to get a portion of fees.
pub struct ToAuthor<R>(sp_std::marker::PhantomData<R>);

impl<R> OnUnbalanced<NegativeImbalance<R>> for ToAuthor<R>
where
	R: balances::Config + authorship::Config,
	<R as frame_system::Config>::AccountId: From<AccountId>,
	<R as frame_system::Config>::AccountId: Into<AccountId>,
	<R as frame_system::Config>::Event: From<balances::Event<Runtime>>,
	<R as balances::Config>::Balance: Into<u128>,
{
	fn on_nonzero_unbalanced(amount: NegativeImbalance<R>) {
		let numeric_amount = amount.peek();
		let author = <authorship::Pallet<R>>::author();
		<balances::Pallet<R>>::resolve_creating(&author, amount);
		<frame_system::Pallet<R>>::deposit_event(balances::Event::Deposit(author.into(), numeric_amount.into()));
	}
}

parameter_types! {
	pub const MaxClaims: u32 = 300;
	pub const UsableBalance: Balance = DOLLARS;
}

impl kilt_launch::Config for Runtime {
	type Event = Event;
	type MaxClaims = MaxClaims;
	type UsableBalance = UsableBalance;
	type WeightInfo = kilt_launch::default_weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const TransactionByteFee: Balance = 0;
}

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = CurrencyAdapter<Balances, ToAuthor<Runtime>>;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
}

impl sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

impl attestation::Config for Runtime {
	/// The ubiquitous event type.
	type Event = Event;
	type WeightInfo = ();
}

pub struct AttestationStructRuntimeUpgrade;
impl attestation::migration::V23ToV24 for AttestationStructRuntimeUpgrade {
	type Hash = Hash;
	type DelegationNodeId = Hash;
	type AccountId = AccountId;
	type Module = Attestation;
}
impl frame_support::traits::OnRuntimeUpgrade for AttestationStructRuntimeUpgrade {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		attestation::migration::apply::<Self>()
	}
}

impl ctype::Config for Runtime {
	/// The ubiquitous event type.
	type Event = Event;
	type WeightInfo = ();
}

impl delegation::Config for Runtime {
	/// The ubiquitous event type.
	type Event = Event;
	type Signature = Signature;
	type Signer = <Signature as Verify>::Signer;
	type DelegationNodeId = Hash;
	type WeightInfo = ();
}

pub struct DelegationStructRuntimeUpgrade;
impl delegation::migration::V23ToV24 for DelegationStructRuntimeUpgrade {
	type AccountId = AccountId;
	type DelegationNodeId = Hash;
	type Module = Delegation;
}
impl frame_support::traits::OnRuntimeUpgrade for DelegationStructRuntimeUpgrade {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		delegation::migration::apply::<Self>()
	}
}

impl did::Config for Runtime {
	/// The ubiquitous event type.
	type Event = Event;
	type WeightInfo = ();
	type DidIdentifier = AccountId;
}

pub struct PortableGabiRemoval;
impl frame_support::traits::OnRuntimeUpgrade for PortableGabiRemoval {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		frame_support::storage::unhashed::kill_prefix(&sp_io::hashing::twox_128(b"Portablegabi"));
		Weight::max_value()
	}
}

parameter_types! {
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
	pub const Period: u64 = 0xFFFF_FFFF_FFFF_FFFF;
	pub const Offset: u64 = 0xFFFF_FFFF_FFFF_FFFF;
}

impl session::Config for Runtime {
	type Event = Event;
	type ValidatorId = AccountId;
	type ValidatorIdOf = ();
	type ShouldEndSession = session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = ();
	type SessionManager = ();
	type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = opaque::SessionKeys;
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
	type WeightInfo = ();
}

parameter_types! {
	pub const UncleGenerations: u32 = 0;
}

impl authorship::Config for Runtime {
	type FindAuthor = session::FindAccountFromAuthorIndex<Self, Aura>;
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = ();
}

parameter_types! {
	pub const MinVestedTransfer: Balance = MIN_VESTED_TRANSFER_AMOUNT;
}

impl pallet_vesting::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	// disable vested transfers by setting min amount to max balance
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = pallet_vesting::weights::SubstrateWeight<Runtime>;
}

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
		RandomnessCollectiveFlip: randomness_collective_flip::{Pallet, Call, Storage} = 1,

		Timestamp: timestamp::{Pallet, Call, Storage, Inherent} = 2,
		Aura: aura::{Pallet, Config<T>, Storage} = 3,
		Grandpa: grandpa::{Pallet, Call, Storage, Config, Event} = 4,
		Indices: pallet_indices::{Pallet, Call, Storage, Event<T>} = 5,
		Balances: balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 6,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 7,
		Sudo: sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 8,

		Ctype: ctype::{Pallet, Call, Storage, Event<T>} = 9,
		Attestation: attestation::{Pallet, Call, Storage, Event<T>} = 10,
		Delegation: delegation::{Pallet, Call, Storage, Event<T>} = 11,
		Did: did::{Pallet, Call, Storage, Event<T>} = 12,

		Session: session::{Pallet, Call, Storage, Event, Config<T>} = 15,
		Authorship: authorship::{Pallet, Call, Storage} = 16,

		// // Governance stuff; uncallable initially.
		// Democracy: pallet_democracy::{Module, Call, Storage, Config, Event<T>} = 25,
		// Council: pallet_collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>} = 26,
		// TechnicalCommittee: pallet_collective::<Instance2>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>} = 27,
		// ElectionsPhragmen: pallet_elections_phragmen::{Module, Call, Storage, Event<T>, Config<T>} = 28,
		// TechnicalMembership: pallet_membership::{Module, Call, Storage, Event<T>, Config<T>} = 29,
		// Treasury: pallet_treasury::{Module, Call, Storage, Config, Event<T>} = 30,

		// // Society module.
		// Society: pallet_society::{Module, Call, Storage, Event<T>} = 31,

		// // System scheduler.
		// Scheduler: pallet_scheduler::{Module, Call, Storage, Event<T>} = 32,

		// Vesting. Usable initially, but removed once all vesting is finished.
		Vesting: pallet_vesting::{Pallet, Call, Storage, Event<T>, Config<T>} = 33,
		KiltLaunch: kilt_launch::{Pallet, Call, Storage, Event<T>, Config<T>} = 34,
	}
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various Pallets.
pub type Executive = executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPallets>;

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			frame_system::Pallet::<Runtime>::account_nonce(&account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}

		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}

		fn random_seed() -> <Block as BlockT>::Hash {
			RandomnessCollectiveFlip::random_seed().0
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> SlotDuration {
			SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities()
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			_authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac")
					.to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80")
					.to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a")
					.to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850")
					.to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7")
					.to_vec().into(),
				// KiltLaunch transfer account
				hex_literal::hex!("6a3c793cec9dbe330b349dc4eea6801090f5e71f53b1b41ad11afb4a313a282c").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_balances, Balances);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);
			add_benchmark!(params, batches, attestation, Attestation);
			add_benchmark!(params, batches, ctype, Ctype);
			add_benchmark!(params, batches, delegation, Delegation);
			// add_benchmark!(params, batches, did, Did);
			add_benchmark!(params, batches, kilt_launch, KiltLaunch);
			add_benchmark!(params, batches, pallet_vesting, Vesting);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}
