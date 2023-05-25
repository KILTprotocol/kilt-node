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

use crate::*;

pub use crate::api::api::*;

// The runtime version has to be part of the api module since
// `RUNTIME_API_VERSIONS` is private and can only be accessed from within the
// api module.
/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("mashnet-node"),
	impl_name: create_runtime_str!("mashnet-node"),
	authoring_version: 4,
	spec_version: 11100,
	impl_version: 0,
	apis: crate::api::RUNTIME_API_VERSIONS,
	transaction_version: 6,
	state_version: 0,
};

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	frame_benchmarking::define_benchmarks!(
		[frame_system, SystemBench::<Runtime>]
		[pallet_timestamp, Timestamp]
		[pallet_indices, Indices]
		[pallet_balances, Balances]
		[pallet_session, SessionBench::<Runtime>]
		[parachain_staking, ParachainStaking]
		[pallet_democracy, Democracy]
		[pallet_collective, Council]
		[pallet_collective, TechnicalCommittee]
		[pallet_membership, TechnicalMembership]
		[pallet_treasury, Treasury]
		[pallet_utility, Utility]
		[pallet_vesting, Vesting]
		[pallet_scheduler, Scheduler]
		[pallet_proxy, Proxy]
		[pallet_preimage, Preimage]
		[pallet_tips, Tips]
		[pallet_multisig, Multisig]
		[ctype, Ctype]
		[attestation, Attestation]
		[delegation, Delegation]
		[did, Did]
		[pallet_inflation, Inflation]
		[pallet_did_lookup, DidLookup]
		[pallet_web3_names, Web3Names]
		[public_credentials, PublicCredentials]
		[pallet_xcm, PolkadotXcm]
		[frame_benchmarking::baseline, Baseline::<Runtime>]

	);
}

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
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			frame_system::Pallet::<Runtime>::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}

		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> pallet_transaction_payment::FeeDetails<Balance> {
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
		) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
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

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(block: Block, data: sp_inherents::InherentData) -> sp_inherents::CheckInherentsResult {
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
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}

		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuthorityId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuthorityId> {
			Aura::authorities().into_inner()
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
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
		fn query_by_web3_name(name: Vec<u8>) -> Option<kilt_runtime_api_did::RawDidLinkedInfo<
				DidIdentifier,
				AccountId,
				LinkableAccountId,
				Balance,
				Hash,
				BlockNumber
			>
		> {
			let name: pallet_web3_names::web3_name::AsciiWeb3Name<Runtime> = name.try_into().ok()?;
			pallet_web3_names::Owner::<Runtime>::get(&name)
				.and_then(|owner_info| {
					did::Did::<Runtime>::get(&owner_info.owner).map(|details| (owner_info, details))
				})
				.map(|(owner_info, details)| {
					let accounts = pallet_did_lookup::ConnectedAccounts::<Runtime>::iter_key_prefix(
						&owner_info.owner,
					).collect();
					let service_endpoints = did::ServiceEndpoints::<Runtime>::iter_prefix(&owner_info.owner).map(|e| From::from(e.1)).collect();

					kilt_runtime_api_did::RawDidLinkedInfo{
						identifier: owner_info.owner,
						w3n: Some(name.into()),
						accounts,
						service_endpoints,
						details: details.into(),
					}
			})
		}

		fn query_by_account(account: LinkableAccountId) -> Option<
			kilt_runtime_api_did::RawDidLinkedInfo<
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

					kilt_runtime_api_did::RawDidLinkedInfo {
						identifier: connection_record.did,
						w3n,
						accounts,
						service_endpoints,
						details: details.into(),
					}
				})
		}

		fn query(did: DidIdentifier) -> Option<
			kilt_runtime_api_did::RawDidLinkedInfo<
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

			Some(kilt_runtime_api_did::RawDidLinkedInfo {
				identifier: did,
				w3n,
				accounts,
				service_endpoints,
				details: details.into(),
			})
		}
	}

	impl kilt_runtime_api_public_credentials::PublicCredentials<Block, Vec<u8>, Hash, public_credentials::CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>>, PublicCredentialsFilter<Hash, AccountId>, PublicCredentialsApiError> for Runtime {
		fn get_by_id(credential_id: Hash) -> Option<public_credentials::CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>>> {
			let subject = public_credentials::CredentialSubjects::<Runtime>::get(credential_id)?;
			public_credentials::Credentials::<Runtime>::get(subject, credential_id)
		}

		fn get_by_subject(subject: Vec<u8>, filter: Option<PublicCredentialsFilter<Hash, AccountId>>) -> Result<Vec<(Hash, public_credentials::CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>>)>, PublicCredentialsApiError> {
			let asset_did = AssetDid::try_from(subject).map_err(|_| PublicCredentialsApiError::InvalidSubjectId)?;
			let credentials_prefix = public_credentials::Credentials::<Runtime>::iter_prefix(asset_did);
			if let Some(filter) = filter {
				Ok(credentials_prefix.filter(|(_, entry)| filter.should_include(entry)).collect())
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

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;

			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			use frame_benchmarking::baseline::Pallet as Baseline;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();
			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, TrackedStorageKey};
			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			use frame_benchmarking::baseline::Pallet as Baseline;

			impl frame_system_benchmarking::Config for Runtime {}
			impl cumulus_pallet_session_benchmarking::Config for Runtime {}
			impl frame_benchmarking::baseline::Config for Runtime {}

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
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmarks!(params, batches);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: UpgradeCheckSelect) -> (Weight, Weight) {
			log::info!("try-runtime::on_runtime_upgrade peregrine.");
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, BlockWeights::get().max_block)
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
}
