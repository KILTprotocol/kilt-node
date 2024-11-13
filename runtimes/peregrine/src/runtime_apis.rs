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

use cumulus_pallet_aura_ext::FixedVelocityConsensusHook;
use frame_support::{
	genesis_builder_helper::{build_config, create_default_config},
	pallet_prelude::{TransactionSource, TransactionValidity},
	weights::Weight,
};
use kilt_support::traits::ItemFilter;
use pallet_asset_switch::xcm::AccountId32ToAccountId32JunctionConverter;
use pallet_did_lookup::{linkable_account::LinkableAccountId, ConnectionRecord};
use pallet_web3_names::web3_name::{AsciiWeb3Name, Web3NameOwnership};
use runtime_common::{
	asset_switch::runtime_api::Error as AssetSwitchApiError,
	assets::{AssetDid, PublicCredentialsFilter},
	authorization::AuthorizationId,
	constants::{EXISTENTIAL_DEPOSIT, SLOT_DURATION},
	dip::merkle::{CompleteMerkleProof, DidMerkleProofOf, DidMerkleRootGenerator},
	errors::PublicCredentialsApiError,
	AccountId, AuthorityId, Balance, BlockNumber, BlockWeights, DidIdentifier, Hash, Nonce,
};
use sp_api::impl_runtime_apis;
use sp_core::{storage::TrackedStorageKey, OpaqueMetadata};
use sp_runtime::{traits::Block as BlockT, ApplyExtrinsicResult};
use sp_version::RuntimeVersion;
use unique_linking_runtime_api::{AddressResult, NameResult};
use xcm::{v4::Location, VersionedAssetId, VersionedLocation, VersionedXcm};

use crate::{
	add_benchmarks,
	dip::runtime_api::{DipProofError, DipProofRequest},
	kilt::{DotName, UniqueLinkingDeployment},
	list_benchmarks,
	parachain::ConsensusHook,
	system::SessionKeys,
	xcm_config::{UniversalLocation, XcmConfig},
	AllPalletsWithSystem, AssetSwitchPool1, Aura, Balances, Block, DotNames, Executive, ParachainStaking,
	ParachainSystem, Runtime, RuntimeCall, RuntimeGenesisConfig, System, TransactionPayment, UniqueLinking, VERSION,
};

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

		fn metadata_versions() -> sp_std::vec::Vec<u32> {
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
			sp_consensus_aura::SlotDuration::from_millis(SLOT_DURATION)
		}

		fn authorities() -> Vec<AuthorityId> {
			Aura::authorities().into_inner()
		}
	}

	impl cumulus_primitives_aura::AuraUnincludedSegmentApi<Block> for Runtime {
		fn can_build_upon(
			included_hash: <Block as BlockT>::Hash,
			slot: cumulus_primitives_aura::Slot,
			) -> bool {
				ConsensusHook::can_build_upon(included_hash, slot)
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

					kilt_runtime_api_did::RawDidLinkedInfo{
						identifier: owner_info.owner,
						w3n: Some(parsed_name.into()),
						accounts,
						service_endpoints,
						details: details.into(),
					}
			})
		}

		fn batch_query_by_web3_name(names: Vec<Vec<u8>>) -> Vec<Option<kilt_runtime_api_did::RawDidLinkedInfo<
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

		fn batch_query_by_account(accounts: Vec<LinkableAccountId>) -> Vec<Option<
			kilt_runtime_api_did::RawDidLinkedInfo<
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

		fn batch_query(dids: Vec<DidIdentifier>) -> Vec<Option<
			kilt_runtime_api_did::RawDidLinkedInfo<
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

	impl kilt_runtime_api_public_credentials::PublicCredentials<Block, Vec<u8>, Hash, public_credentials::CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>>, PublicCredentialsFilter<Hash, AccountId>, PublicCredentialsApiError> for Runtime {
		fn get_by_id(credential_id: Hash) -> Option<public_credentials::CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>>> {
			let subject = public_credentials::CredentialSubjects::<Runtime>::get(credential_id)?;
			public_credentials::Credentials::<Runtime>::get(subject, credential_id)
		}

		fn get_by_subject(subject: Vec<u8>, filter: Option<PublicCredentialsFilter<Hash, AccountId>>) -> Result<Vec<(Hash, public_credentials::CredentialEntry<Hash, DidIdentifier, BlockNumber, AccountId, Balance, AuthorizationId<<Runtime as delegation::Config>::DelegationNodeId>>)>, PublicCredentialsApiError> {
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
			use pallet_dip_provider::traits::IdentityProvider;

			let identity_details = pallet_dip_provider::IdentityProviderOf::<Runtime>::retrieve(&request.identifier).map_err(DipProofError::IdentityProvider)?;
			log::info!(target: "runtime_api::dip_provider", "Identity details retrieved for request {:#?}: {:#?}", request, identity_details);

			DidMerkleRootGenerator::<Runtime>::generate_proof(&identity_details, request.version, request.keys.iter(), request.should_include_web3_name, request.accounts.iter()).map_err(DipProofError::MerkleProof)
		}
	}

	impl pallet_asset_switch_runtime_api::AssetSwitch<Block, VersionedAssetId, AccountId, u128, VersionedLocation, AssetSwitchApiError, VersionedXcm<()>> for Runtime {
		fn pool_account_id(pair_id: Vec<u8>, asset_id: VersionedAssetId) -> Result<AccountId, AssetSwitchApiError> {
			use core::str;
			use frame_support::traits::PalletInfoAccess;

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
			use core::str;
			use frame_support::traits::PalletInfoAccess;
			use sp_runtime::traits::TryConvert;
			use xcm::v4::{AssetId, Asset};

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

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();
			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{BenchmarkBatch, BenchmarkError};
			use runtime_common::benchmarks::xcm as xcm_benchmarking;
			use xcm::v4::{Asset, Assets, Fungibility};

			impl pallet_xcm::benchmarking::Config for Runtime {
				type DeliveryHelper = xcm_benchmarking::ParachainDeliveryHelper<ParachainSystem, XcmConfig>;

				fn reachable_dest() -> Option<Location> {
					ParachainSystem::open_outbound_hrmp_channel_for_benchmarks_or_tests(xcm_benchmarking::RandomParaId::get());
					Some(xcm_benchmarking::ParachainLocation::get())
				}

				fn reserve_transferable_asset_and_dest() -> Option<(Asset, Location)> {
					ParachainSystem::open_outbound_hrmp_channel_for_benchmarks_or_tests(xcm_benchmarking::RandomParaId::get());
					Some((
						xcm_benchmarking::NativeAsset::get(),
						xcm_benchmarking::ParachainLocation::get(),
					))
				}

				fn set_up_complex_asset_transfer() -> Option<(Assets, u32, Location, Box<dyn FnOnce()>)> {
					let (transferable_asset, dest) = Self::reserve_transferable_asset_and_dest().unwrap();

					let fee_amount = EXISTENTIAL_DEPOSIT;
					let fee_asset: Asset = (Location::here(), fee_amount).into();

					// Make account free to pay the fee
					let who = frame_benchmarking::whitelisted_caller();
					let balance = fee_amount + EXISTENTIAL_DEPOSIT * 1000;
					let _ = <Balances as frame_support::traits::Currency<_>>::make_free_balance_be(
						&who, balance,
					);

					// verify initial balance
					assert_eq!(Balances::free_balance(&who), balance);


					let assets: Assets = vec![fee_asset.clone(), transferable_asset.clone()].into();
					let fee_index = if assets.get(0).unwrap().eq(&fee_asset) { 0 } else { 1 };

					let verify = Box::new( move || {
						let Fungibility::Fungible(transferable_amount) = transferable_asset.fun else { return; };
						assert_eq!(Balances::free_balance(&who), balance - fee_amount - transferable_amount);
					});

					Some((assets,fee_index , dest, verify))
				}

				fn get_asset() -> Asset {
					xcm_benchmarking::NativeAsset::get()
				}
			}

			impl frame_system_benchmarking::Config for Runtime {
				   fn setup_set_code_requirements(code: &sp_std::vec::Vec<u8>) -> Result<(), BenchmarkError> {
					   ParachainSystem::initialize_for_set_code_benchmark(code.len() as u32);
					   Ok(())
				   }

				fn verify_set_code() {
					System::assert_last_event(cumulus_pallet_parachain_system::Event::<Runtime>::ValidationFunctionStored.into());
				}
			}

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
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
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

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {

		fn create_default_config() -> Vec<u8> {
			create_default_config::<RuntimeGenesisConfig>()
		}

		fn build_config(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_config::<RuntimeGenesisConfig>(config)
		}

	}
}
