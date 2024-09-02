var srcIndex = JSON.parse('{\
"attestation":["",[],["access_control.rs","attestations.rs","authorized_by.rs","benchmarking.rs","default_weights.rs","lib.rs","migrations.rs","mock.rs","try_state.rs"]],\
"ctype":["",[],["benchmarking.rs","ctype_entry.rs","default_weights.rs","lib.rs","mock.rs"]],\
"delegation":["",[],["access_control.rs","benchmarking.rs","default_weights.rs","delegation_hierarchy.rs","lib.rs","migrations.rs","mock.rs","try_state.rs"]],\
"did":["",[],["benchmarking.rs","default_weights.rs","did_details.rs","errors.rs","lib.rs","migrations.rs","mock_utils.rs","origin.rs","service_endpoints.rs","signature.rs","try_state.rs","utils.rs"]],\
"dip_consumer_node_template":["",[],["chain_spec.rs","cli.rs","command.rs","main.rs","rpc.rs","service.rs"]],\
"dip_consumer_runtime_template":["",[["weights",[],["frame_system.rs","mod.rs","pallet_dip_consumer.rs","pallet_relay_store.rs"]]],["dip.rs","lib.rs","origin_adapter.rs"]],\
"dip_provider_node_template":["",[],["chain_spec.rs","cli.rs","command.rs","main.rs","rpc.rs","service.rs"]],\
"dip_provider_runtime_template":["",[["weights",[],["did.rs","frame_system.rs","mod.rs","pallet_deposit_storage.rs","pallet_did_lookup.rs","pallet_dip_provider.rs","pallet_web3_names.rs"]]],["dip.rs","lib.rs"]],\
"kestrel_runtime":["",[],["lib.rs"]],\
"kilt_asset_dids":["",[],["asset.rs","chain.rs","errors.rs","lib.rs","v1.rs"]],\
"kilt_dip_primitives":["",[["merkle_proofs",[["v0",[["dip_subject_state",[],["mod.rs"]],["provider_state",[],["mod.rs"]],["relay_state",[],["mod.rs"]]],["error.rs","input_common.rs","mod.rs","output_common.rs"]]],["mod.rs"]],["state_proofs",[],["error.rs","mod.rs","substrate_no_std_port.rs"]],["verifier",[["parachain",[["v0",[],["mod.rs"]]],["error.rs","mod.rs"]],["relaychain",[["v0",[],["mod.rs"]]],["error.rs","mod.rs"]]],["errors.rs","mod.rs"]]],["lib.rs","traits.rs","utils.rs"]],\
"kilt_parachain":["",[["chain_spec",[["peregrine",[],["dev.rs","mod.rs","new.rs"]],["rilt",[],["mod.rs","new.rs"]],["spiritnet",[],["dev.rs","mod.rs","new.rs"]]],["mod.rs","utils.rs"]]],["cli.rs","command.rs","main.rs","rpc.rs","service.rs"]],\
"kilt_runtime_api_did":["",[],["did_details.rs","lib.rs","service_endpoint.rs"]],\
"kilt_runtime_api_dip_provider":["",[],["lib.rs"]],\
"kilt_runtime_api_public_credentials":["",[],["lib.rs"]],\
"kilt_runtime_api_staking":["",[],["lib.rs"]],\
"kilt_support":["",[],["benchmark.rs","deposit.rs","lib.rs","migration.rs","mock.rs","signature.rs","test_utils.rs","traits.rs"]],\
"pallet_asset_switch":["",[["xcm",[["convert",[],["mod.rs"]],["match",[],["mod.rs"]],["trade",[["switch_pair_remote_asset",[],["mod.rs"]],["xcm_fee_asset",[],["mod.rs"]]],["mod.rs"]],["transact",[],["mod.rs"]],["transfer",[["switch_pair_remote_asset",[],["mod.rs"]],["xcm_fee_asset",[],["mod.rs"]]],["mod.rs"]]],["mod.rs"]]],["benchmarking.rs","default_weights.rs","lib.rs","switch.rs","traits.rs","try_state.rs"]],\
"pallet_asset_switch_runtime_api":["",[],["lib.rs"]],\
"pallet_configuration":["",[],["benchmarking.rs","configuration.rs","default_weights.rs","lib.rs"]],\
"pallet_deposit_storage":["",[["deposit",[],["mod.rs"]]],["benchmarking.rs","default_weights.rs","lib.rs","traits.rs"]],\
"pallet_did_lookup":["",[],["account.rs","associate_account_request.rs","benchmarking.rs","connection_record.rs","default_weights.rs","lib.rs","linkable_account.rs","migrations.rs","signature.rs","try_state.rs"]],\
"pallet_dip_consumer":["",[],["benchmarking.rs","default_weights.rs","lib.rs","origin.rs","traits.rs"]],\
"pallet_dip_provider":["",[],["benchmarking.rs","default_weights.rs","lib.rs","traits.rs"]],\
"pallet_inflation":["",[],["benchmarking.rs","default_weights.rs","lib.rs"]],\
"pallet_migration":["",[],["benchmarking.rs","default_weights.rs","lib.rs","mock.rs"]],\
"pallet_postit":["",[],["lib.rs","post.rs","traits.rs"]],\
"pallet_relay_store":["",[],["benchmarking.rs","default_weights.rs","lib.rs","relay.rs"]],\
"pallet_web3_names":["",[],["benchmarking.rs","default_weights.rs","lib.rs","migrations.rs","mock.rs","try_state.rs","web3_name.rs"]],\
"parachain_staking":["",[],["api.rs","benchmarking.rs","default_weights.rs","inflation.rs","lib.rs","set.rs","try_state.rs","types.rs"]],\
"peregrine_runtime":["",[["dip",[],["mod.rs","runtime_api.rs"]],["weights",[],["attestation.rs","ctype.rs","cumulus_pallet_dmp_queue.rs","cumulus_pallet_parachain_system.rs","delegation.rs","did.rs","frame_system.rs","mod.rs","pallet_asset_switch.rs","pallet_assets.rs","pallet_balances.rs","pallet_collective_council.rs","pallet_collective_technical_committee.rs","pallet_democracy.rs","pallet_deposit_storage.rs","pallet_did_lookup.rs","pallet_dip_provider.rs","pallet_indices.rs","pallet_inflation.rs","pallet_membership.rs","pallet_message_queue.rs","pallet_migration.rs","pallet_multisig.rs","pallet_preimage.rs","pallet_proxy.rs","pallet_scheduler.rs","pallet_session.rs","pallet_sudo.rs","pallet_timestamp.rs","pallet_tips.rs","pallet_treasury.rs","pallet_utility.rs","pallet_vesting.rs","pallet_web3_names.rs","pallet_xcm.rs","parachain_staking.rs","public_credentials.rs","rocksdb_weights.rs"]]],["lib.rs","xcm_config.rs"]],\
"public_credentials":["",[],["access_control.rs","benchmarking.rs","credentials.rs","default_weights.rs","lib.rs","migrations.rs","mock.rs","try_state.rs"]],\
"runtime_common":["",[["asset_switch",[],["hooks.rs","mod.rs","runtime_api.rs"]],["benchmarks",[],["mod.rs","treasury.rs","xcm.rs"]],["dip",[["deposit",[],["mod.rs"]],["did",[],["mod.rs"]],["merkle",[["v0",[],["mod.rs"]]],["mod.rs"]]],["mod.rs"]]],["assets.rs","authorization.rs","constants.rs","errors.rs","fees.rs","lib.rs","migrations.rs","pallet_id.rs","xcm_config.rs"]],\
"spiritnet_runtime":["",[["dip",[],["mod.rs","runtime_api.rs"]],["weights",[],["attestation.rs","ctype.rs","cumulus_pallet_dmp_queue.rs","cumulus_pallet_parachain_system.rs","delegation.rs","did.rs","frame_system.rs","mod.rs","pallet_asset_switch.rs","pallet_assets.rs","pallet_balances.rs","pallet_collective_council.rs","pallet_collective_technical_committee.rs","pallet_democracy.rs","pallet_deposit_storage.rs","pallet_did_lookup.rs","pallet_dip_provider.rs","pallet_indices.rs","pallet_inflation.rs","pallet_membership.rs","pallet_message_queue.rs","pallet_migration.rs","pallet_multisig.rs","pallet_preimage.rs","pallet_proxy.rs","pallet_scheduler.rs","pallet_session.rs","pallet_timestamp.rs","pallet_tips.rs","pallet_treasury.rs","pallet_utility.rs","pallet_vesting.rs","pallet_web3_names.rs","pallet_xcm.rs","parachain_staking.rs","public_credentials.rs","rocksdb_weights.rs"]]],["lib.rs","xcm_config.rs"]],\
"standalone_node":["",[],["chain_spec.rs","cli.rs","command.rs","main.rs","rpc.rs","service.rs"]],\
"xcm_integration_tests":["",[],["lib.rs"]]\
}');
createSrcSidebar();