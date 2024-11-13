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

//! The KILT runtime. This can be compiled with `#[no_std]`, ready for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
// Triggered by `impl_runtime_apis` macro
#![allow(clippy::empty_structs_with_brackets)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use cumulus_pallet_parachain_system::register_validate_block;
// Polkadot-sdk crates
use core::str;
use frame_support::construct_runtime;
use frame_system::{
	ChainContext, CheckEra, CheckGenesis, CheckNonZeroSender, CheckNonce, CheckSpecVersion, CheckTxVersion, CheckWeight,
};
use pallet_transaction_payment::ChargeTransactionPayment;
use sp_runtime::{create_runtime_str, generic};
use sp_std::prelude::*;
use sp_version::RuntimeVersion;

use ::xcm::{
	v4::{Asset, AssetId, Location},
	VersionedAssetId, VersionedLocation, VersionedXcm,
};
use cumulus_primitives_aura::Slot;
use cumulus_primitives_core::CollationInfo;
use frame_support::{
	genesis_builder_helper::{build_config, create_default_config},
	pallet_prelude::{TransactionSource, TransactionValidity},
	traits::PalletInfoAccess,
	weights::Weight,
};
use kilt_runtime_api_did::RawDidLinkedInfo;
use kilt_support::traits::ItemFilter;
use pallet_asset_switch::xcm::AccountId32ToAccountId32JunctionConverter;
use pallet_did_lookup::{linkable_account::LinkableAccountId, ConnectionRecord};
use pallet_dip_provider::traits::IdentityProvider;
use pallet_transaction_payment::{FeeDetails, RuntimeDispatchInfo};
use pallet_web3_names::web3_name::{AsciiWeb3Name, Web3NameOwnership};
use public_credentials::CredentialEntry;
use runtime_common::{
	asset_switch::runtime_api::Error as AssetSwitchApiError,
	assets::{AssetDid, PublicCredentialsFilter},
	authorization::AuthorizationId,
	constants::SLOT_DURATION,
	dip::merkle::{CompleteMerkleProof, DidMerkleProofOf, DidMerkleRootGenerator},
	errors::PublicCredentialsApiError,
	AccountId, AuthorityId, Balance, BlockNumber, DidIdentifier, Hash, Header, Nonce,
};
use sp_api::impl_runtime_apis;
use sp_core::OpaqueMetadata;
use sp_inherents::{CheckInherentsResult, InherentData};
use sp_runtime::{
	traits::{Block as BlockT, TryConvert},
	ApplyExtrinsicResult, KeyTypeId,
};
use sp_std::vec::Vec;
use unique_linking_runtime_api::{AddressResult, NameResult};

// Internal crates
pub use parachain_staking::InflationInfo;
pub use public_credentials;
use runtime_common::{constants, fees::WeightToFee, Address, Signature};

use crate::{
	dip::runtime_api::{DipProofError, DipProofRequest},
	kilt::{DotName, UniqueLinkingDeployment},
	migrations::Migrations,
	parachain::ConsensusHook,
	system::SessionKeys,
	xcm::UniversalLocation,
};

mod dip;
mod governance;
mod kilt;
mod migrations;
mod parachain;
mod system;
mod weights;
mod xcm;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;

#[cfg(test)]
mod tests;

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("mashnet-node"),
	impl_name: create_runtime_str!("mashnet-node"),
	authoring_version: 4,
	spec_version: 11500,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 8,
	state_version: 0,
};

/// The version information used to identify this runtime when compiled
/// natively.
#[cfg(feature = "std")]
pub fn native_version() -> sp_version::NativeVersion {
	sp_version::NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

construct_runtime! {
	pub enum Runtime
	{
		System: frame_system = 0,
		// DELETED: RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip = 1,

		Timestamp: pallet_timestamp = 2,
		Indices: pallet_indices exclude_parts { Config } = 5,
		Balances: pallet_balances = 6,
		TransactionPayment: pallet_transaction_payment exclude_parts { Config } = 7,
		Sudo: pallet_sudo = 8,
		// Configuration: pallet_configuration = 9,

		// Consensus support.
		// The following order MUST NOT be changed: Aura -> Session -> Staking -> Authorship -> AuraExt
		// Dependencies: AuraExt on Aura, Authorship and Session on ParachainStaking
		Aura: pallet_aura = 23,
		Session: pallet_session = 22,
		ParachainStaking: parachain_staking = 21,
		Authorship: pallet_authorship = 20,
		AuraExt: cumulus_pallet_aura_ext = 24,

		Democracy: pallet_democracy = 30,
		Council: pallet_collective::<Instance1> = 31,
		TechnicalCommittee: pallet_collective::<Instance2> = 32,
		// reserved: parachain council election = 33,
		TechnicalMembership: pallet_membership::<Instance1> = 34,
		Treasury: pallet_treasury = 35,
		// DELETED: RelayMigration: pallet_relay_migration = 36,
		// DELETED: DynFilter: pallet_dyn_filter = 37,

		// A stateless pallet with helper extrinsics (batch extrinsics, send from different origins, ...)
		Utility: pallet_utility = 40,

		// Vesting. Usable initially, but removed once all vesting is finished.
		Vesting: pallet_vesting = 41,

		Scheduler: pallet_scheduler = 42,

		// Allowing accounts to give permission to other accounts to dispatch types of calls from their signed origin
		Proxy: pallet_proxy = 43,

		// Preimage pallet allows the storage of large bytes blob
		Preimage: pallet_preimage = 44,

		// Tips module to reward contributions to the ecosystem with small amount of KILTs.
		TipsMembership: pallet_membership::<Instance2> = 45,
		Tips: pallet_tips = 46,

		Multisig: pallet_multisig = 47,

		AssetSwitchPool1: pallet_asset_switch::<Instance1> = 48,
		Fungibles: pallet_assets = 49,

		// KILT Pallets. Start indices 60 to leave room
		// DELETED: KiltLaunch: kilt_launch = 60,
		Ctype: ctype = 61,
		Attestation: attestation = 62,
		Delegation: delegation = 63,
		Did: did = 64,
		// DELETED: CrowdloanContributors = 65,
		Inflation: pallet_inflation = 66,
		DidLookup: pallet_did_lookup = 67,
		Web3Names: pallet_web3_names = 68,
		PublicCredentials: public_credentials = 69,
		Migration: pallet_migration = 70,
		DipProvider: pallet_dip_provider = 71,
		DepositStorage: pallet_deposit_storage = 72,
		DotNames: pallet_web3_names::<Instance2> = 73,
		UniqueLinking: pallet_did_lookup::<Instance2> = 74,

		// Parachains pallets. Start indices at 80 to leave room.

		// Among others: Send and receive DMP and XCMP messages.
		ParachainSystem: cumulus_pallet_parachain_system = 80,
		ParachainInfo: parachain_info = 81,
		// Wrap and unwrap XCMP messages to send and receive them. Queue them for later processing.
		XcmpQueue: cumulus_pallet_xcmp_queue = 82,
		// Build XCM scripts.
		PolkadotXcm: pallet_xcm = 83,
		// Does nothing cool, just provides an origin.
		CumulusXcm: cumulus_pallet_xcm exclude_parts { Call } = 84,
		// DmpQueue: cumulus_pallet_dmp_queue = 85,
		// Queue and pass DMP messages on to be executed.
		MessageQueue: pallet_message_queue = 86,
	}
}

register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	ChainContext<Runtime>,
	Runtime,
	// Executes pallet hooks in the order of definition in construct_runtime
	AllPalletsWithSystem,
	Migrations,
>;

/// Block header type as expected by this runtime.
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	CheckNonZeroSender<Runtime>,
	CheckSpecVersion<Runtime>,
	CheckTxVersion<Runtime>,
	CheckGenesis<Runtime>,
	CheckEra<Runtime>,
	CheckNonce<Runtime>,
	CheckWeight<Runtime>,
	ChargeTransactionPayment<Runtime>,
);

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
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			frame_system::Pallet::<Runtime>::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}

		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}

		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
	for Runtime
	{
		fn query_call_info(
			call: RuntimeCall,
			len: u32,
		) -> RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(
			call: RuntimeCall,
			len: u32,
		) -> FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(
			extrinsic: <Block as BlockT>::Extrinsic,
		) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(block: Block, data: InherentData) -> CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}

		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuthorityId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(SLOT_DURATION)
		}

		fn authorities() -> Vec<AuthorityId> {
			Aura::authorities().into_inner()
		}
	}

	impl cumulus_primitives_aura::AuraUnincludedSegmentApi<Block> for Runtime {
		fn can_build_upon(
			included_hash: <Block as BlockT>::Hash,
			slot: Slot,
			) -> bool {
				ConsensusHook::can_build_upon(included_hash, slot)
			}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	impl kilt_runtime_api_did::Did<
		Block,
		DidIdentifier,
		AccountId,
		LinkableAccountId,
		Balance,
		Hash,
		BlockNumber
	> for Runtime {
		fn query_by_web3_name(name: Vec<u8>) -> Option<RawDidLinkedInfo<
				DidIdentifier,
				AccountId,
				LinkableAccountId,
				Balance,
				Hash,
				BlockNumber
			>
		> {
			let parsed_name: AsciiWeb3Name<Runtime> = name.try_into().ok()?;
			pallet_web3_names::Owner::<Runtime>::get(&parsed_name)
				.and_then(|owner_info| {
					did::Did::<Runtime>::get(&owner_info.owner).map(|details| (owner_info, details))
				})
				.map(|(owner_info, details)| {
					let accounts = pallet_did_lookup::ConnectedAccounts::<Runtime>::iter_key_prefix(
						&owner_info.owner,
					).collect();
					let service_endpoints = did::ServiceEndpoints::<Runtime>::iter_prefix(&owner_info.owner).map(|e| From::from(e.1)).collect();

					RawDidLinkedInfo{
						identifier: owner_info.owner,
						w3n: Some(parsed_name.into()),
						accounts,
						service_endpoints,
						details: details.into(),
					}
			})
		}

		fn batch_query_by_web3_name(names: Vec<Vec<u8>>) -> Vec<Option<RawDidLinkedInfo<
				DidIdentifier,
				AccountId,
				LinkableAccountId,
				Balance,
				Hash,
				BlockNumber
			>
		>> {
			names.into_iter().map(Self::query_by_web3_name).collect()
		}

		fn query_by_account(account: LinkableAccountId) -> Option<
			RawDidLinkedInfo<
				DidIdentifier,
				AccountId,
				LinkableAccountId,
				Balance,
				Hash,
				BlockNumber
			>
		> {
			pallet_did_lookup::ConnectedDids::<Runtime>::get(account)
				.and_then(|owner_info| {
					did::Did::<Runtime>::get(&owner_info.did).map(|details| (owner_info, details))
				})
				.map(|(connection_record, details)| {
					let w3n = pallet_web3_names::Names::<Runtime>::get(&connection_record.did).map(Into::into);
					let accounts = pallet_did_lookup::ConnectedAccounts::<Runtime>::iter_key_prefix(&connection_record.did).collect();
					let service_endpoints = did::ServiceEndpoints::<Runtime>::iter_prefix(&connection_record.did).map(|e| From::from(e.1)).collect();

					RawDidLinkedInfo {
						identifier: connection_record.did,
						w3n,
						accounts,
						service_endpoints,
						details: details.into(),
					}
				})
		}

		fn batch_query_by_account(accounts: Vec<LinkableAccountId>) -> Vec<Option<
			RawDidLinkedInfo<
				DidIdentifier,
				AccountId,
				LinkableAccountId,
				Balance,
				Hash,
				BlockNumber
			>
		>> {
			accounts.into_iter().map(Self::query_by_account).collect()
		}

		fn query(did: DidIdentifier) -> Option<
			RawDidLinkedInfo<
				DidIdentifier,
				AccountId,
				LinkableAccountId,
				Balance,
				Hash,
				BlockNumber
			>
		> {
			let details = did::Did::<Runtime>::get(&did)?;
			let w3n = pallet_web3_names::Names::<Runtime>::get(&did).map(Into::into);
			let accounts = pallet_did_lookup::ConnectedAccounts::<Runtime>::iter_key_prefix(&did).collect();
			let service_endpoints = did::ServiceEndpoints::<Runtime>::iter_prefix(&did).map(|e| From::from(e.1)).collect();

			Some(RawDidLinkedInfo {
				identifier: did,
				w3n,
				accounts,
				service_endpoints,
				details: details.into(),
			})
		}

		fn batch_query(dids: Vec<DidIdentifier>) -> Vec<Option<
			RawDidLinkedInfo<
				DidIdentifier,
				AccountId,
				LinkableAccountId,
				Balance,
				Hash,
				BlockNumber
			>
		>> {
			dids.into_iter().map(Self::query).collect()
		}
	}

	impl kilt_runtime_api_public_credentials::PublicCredentials<Block, Vec<u8>, Hash, CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>>, PublicCredentialsFilter<Hash, AccountId>, PublicCredentialsApiError> for Runtime {
		fn get_by_id(credential_id: Hash) -> Option<CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>>> {
			let subject = public_credentials::CredentialSubjects::<Runtime>::get(credential_id)?;
			public_credentials::Credentials::<Runtime>::get(subject, credential_id)
		}

		fn get_by_subject(subject: Vec<u8>, filter: Option<PublicCredentialsFilter<Hash, AccountId>>) -> Result<Vec<(Hash, CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>>)>, PublicCredentialsApiError> {
			let asset_did = AssetDid::try_from(subject).map_err(|_| PublicCredentialsApiError::InvalidSubjectId)?;
			let credentials_prefix = public_credentials::Credentials::<Runtime>::iter_prefix(asset_did);
			if let Some(credentials_filter) = filter {
				Ok(credentials_prefix.filter(|(_, entry)| credentials_filter.should_include(entry)).collect())
			} else {
				Ok(credentials_prefix.collect())
			}
		}
	}

	impl kilt_runtime_api_staking::Staking<Block, AccountId, Balance> for Runtime {
		fn get_unclaimed_staking_rewards(account: &AccountId) -> Balance {
			ParachainStaking::get_unclaimed_staking_rewards(account)
		}

		fn get_staking_rates() -> kilt_runtime_api_staking::StakingRates {
			ParachainStaking::get_staking_rates()
		}
	}

	impl kilt_runtime_api_dip_provider::DipProvider<Block, DipProofRequest, CompleteMerkleProof<Hash, DidMerkleProofOf<Runtime>>, DipProofError> for Runtime {
		fn generate_proof(request: DipProofRequest) -> Result<CompleteMerkleProof<Hash, DidMerkleProofOf<Runtime>>, DipProofError> {
			let identity_details = pallet_dip_provider::IdentityProviderOf::<Runtime>::retrieve(&request.identifier).map_err(DipProofError::IdentityProvider)?;
			log::info!(target: "runtime_api::dip_provider", "Identity details retrieved for request {:#?}: {:#?}", request, identity_details);

			DidMerkleRootGenerator::<Runtime>::generate_proof(&identity_details, request.version, request.keys.iter(), request.should_include_web3_name, request.accounts.iter()).map_err(DipProofError::MerkleProof)
		}
	}

	impl pallet_asset_switch_runtime_api::AssetSwitch<Block, VersionedAssetId, AccountId, u128, VersionedLocation, AssetSwitchApiError, VersionedXcm<()>> for Runtime {
		fn pool_account_id(pair_id: Vec<u8>, asset_id: VersionedAssetId) -> Result<AccountId, AssetSwitchApiError> {
			let Ok(pair_id_as_string) = str::from_utf8(pair_id.as_slice()) else {
				return Err(AssetSwitchApiError::InvalidInput);
			};
			match pair_id_as_string {
				kilt_to_ekilt if kilt_to_ekilt == AssetSwitchPool1::name() => {
					AssetSwitchPool1::pool_account_id_for_remote_asset(&asset_id).map_err(|e| {
						log::error!("Failed to calculate pool account address for asset ID {:?} with error: {:?}", asset_id, e);
						AssetSwitchApiError::Internal
					})
				},
				_ => Err(AssetSwitchApiError::SwitchPoolNotFound)
			}
		}

		fn xcm_for_switch(pair_id: Vec<u8>, from: AccountId, to: VersionedLocation, amount: u128) -> Result<VersionedXcm<()>, AssetSwitchApiError> {
			let Ok(pair_id_as_string) = str::from_utf8(pair_id.as_slice()) else {
				return Err(AssetSwitchApiError::InvalidInput);
			};

			if pair_id_as_string != AssetSwitchPool1::name() {
				return Err(AssetSwitchApiError::SwitchPoolNotFound);
			}

			let Some(switch_pair) = AssetSwitchPool1::switch_pair() else {
				return Err(AssetSwitchApiError::SwitchPoolNotSet);
			};

			let from_v4 = AccountId32ToAccountId32JunctionConverter::try_convert(from).map_err(|_| AssetSwitchApiError::Internal)?;
			let to_v4 = Location::try_from(to).map_err(|_| AssetSwitchApiError::Internal)?;
			let our_location_for_destination = {
				let universal_location = UniversalLocation::get();
				universal_location.invert_target(&to_v4)
			}.map_err(|_| AssetSwitchApiError::Internal)?;
			let asset_id_v4 = AssetId::try_from(switch_pair.remote_asset_id).map_err(|_| AssetSwitchApiError::Internal)?;
			let remote_asset_fee_v4 = Asset::try_from(switch_pair.remote_xcm_fee).map_err(|_| AssetSwitchApiError::Internal)?;

			Ok(VersionedXcm::V4(AssetSwitchPool1::compute_xcm_for_switch(&our_location_for_destination, &from_v4.into(), &to_v4, amount, &asset_id_v4, &remote_asset_fee_v4)))
		}
	}

	impl unique_linking_runtime_api::UniqueLinking<Block, LinkableAccountId, DotName, DidIdentifier> for Runtime {
		fn address_for_name(name: DotName) -> Option<AddressResult<LinkableAccountId, DidIdentifier>> {
			let Web3NameOwnership { owner, .. } = DotNames::owner(name)?;

			let (first_account, second_account) = {
				let mut owner_linked_accounts = pallet_did_lookup::ConnectedAccounts::<Runtime, UniqueLinkingDeployment>::iter_key_prefix(&owner);
				(owner_linked_accounts.next(), owner_linked_accounts.next())
			};
			let linked_account = match (first_account, second_account) {
				#[allow(clippy::panic)]
				(Some(_), Some(_)) => { panic!("More than a single account found for DID {:?}.", owner) },
				(first, _) => first
			}?;

			Some(AddressResult::new(linked_account, Some(owner)))
		}

		fn batch_address_for_name(names: Vec<DotName>) -> Vec<Option<AddressResult<LinkableAccountId, DidIdentifier>>> {
			names.into_iter().map(Self::address_for_name).collect()
		}

		fn name_for_address(address: LinkableAccountId) -> Option<NameResult<DotName, DidIdentifier>> {
			let ConnectionRecord { did, .. } = UniqueLinking::connected_dids(address)?;
			let name = DotNames::names(&did)?;

			Some(NameResult::new(name, Some(did)))
		}

		fn batch_name_for_address(addresses: Vec<LinkableAccountId>) -> Vec<Option<NameResult<DotName, DidIdentifier>>> {
			addresses.into_iter().map(Self::name_for_address).collect()
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			log::info!("try-runtime::on_runtime_upgrade peregrine.");
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, runtime_common::BlockWeights::get().max_block)
		}

		fn execute_block(block: Block, state_root_check: bool, sig_check: bool, select: frame_try_runtime::TryStateSelect) -> Weight {
			log::info!(
				target: "runtime::peregrine", "try-runtime: executing block #{} ({:?}) / root checks: {:?} / sig check: {:?} / sanity-checks: {:?}",
				block.header.number,
				block.header.hash(),
				state_root_check,
				sig_check,
				select,
			);
			Executive::try_execute_block(block, state_root_check, sig_check, select).expect("try_execute_block failed")
		}
	}

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {

		fn create_default_config() -> Vec<u8> {
			create_default_config::<RuntimeGenesisConfig>()
		}

		fn build_config(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_config::<RuntimeGenesisConfig>(config)
		}

	}
}
