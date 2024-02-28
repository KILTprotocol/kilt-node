
#[cfg(test)]
mod tests {
	mod parachains {
		use crate::{
			xcm_config::tests::{
				relaychain::{accounts, collators, polkadot::ED},
				ALICE,
			},
			AccountId, Balance,
		};
		pub(crate) use crate::{
			xcm_config::{
				tests::utils::{get_account_id_from_seed, get_from_seed},
				RelayNetworkId,
			},
			AuthorityId, BalancesConfig, ParachainInfoConfig, PolkadotXcmConfig, RuntimeGenesisConfig, SessionConfig,
			SessionKeys, SystemConfig, WASM_BINARY,
		};
		use asset_hub_polkadot_runtime::Runtime;
		use cumulus_primitives_core::MultiLocation;
		use parity_scale_codec::Encode;
		use runtime_common::constants::EXISTENTIAL_DEPOSIT;
		pub(crate) use runtime_common::{xcm_config::LocationToAccountId, AccountPublic};
		use sp_core::sr25519;
		use sp_runtime::{BuildStorage, Storage};
		use xcm::DoubleEncoded;
		use xcm_emulator::{decl_test_parachains, BridgeMessageHandler, Parachain, TestExt};
		const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;
		pub mod spiritnet {
			use super::*;
			pub const PARA_ID: u32 = 2_000;
			pub fn genesis() -> Storage {
				RuntimeGenesisConfig {
					system: SystemConfig {
						code: WASM_BINARY
							.expect("WASM binary was not build, please build it!")
							.to_vec(),
						..Default::default()
					},
					parachain_info: ParachainInfoConfig {
						parachain_id: PARA_ID.into(),
						..Default::default()
					},
					polkadot_xcm: PolkadotXcmConfig {
						safe_xcm_version: Some(SAFE_XCM_VERSION),
						..Default::default()
					},
					session: SessionConfig {
						keys: <[_]>::into_vec(
							#[rustc_box]
							::alloc::boxed::Box::new([(
								get_account_id_from_seed::<AccountPublic, sr25519::Public>("Alice"),
								get_from_seed::<AuthorityId>("Alice"),
							)]),
						)
						.iter()
						.map(|(acc, key)| (acc.clone(), acc.clone(), SessionKeys { aura: key.clone() }))
						.collect::<Vec<_>>(),
					},
					balances: BalancesConfig {
						balances: accounts::init_balances()
							.iter()
							.cloned()
							.map(|k| (k, EXISTENTIAL_DEPOSIT * 4096))
							.collect(),
					},
					..Default::default()
				}
				.build_storage()
				.unwrap()
			}
		}
		pub mod asset_hub_polkadot {
			use super::*;
			use asset_hub_polkadot_runtime::RuntimeCall;
			pub const PARA_ID: u32 = 1000;
			pub fn genesis() -> Storage {
				let genesis_config = asset_hub_polkadot_runtime::RuntimeGenesisConfig {
					system: asset_hub_polkadot_runtime::SystemConfig {
						code: asset_hub_polkadot_runtime::WASM_BINARY
							.expect("WASM binary was not build, please build it!")
							.to_vec(),
						..Default::default()
					},
					balances: asset_hub_polkadot_runtime::BalancesConfig {
						balances: accounts::init_balances()
							.iter()
							.cloned()
							.map(|k| (k, ED * 4096))
							.collect(),
					},
					parachain_info: asset_hub_polkadot_runtime::ParachainInfoConfig {
						parachain_id: PARA_ID.into(),
						..Default::default()
					},
					collator_selection: asset_hub_polkadot_runtime::CollatorSelectionConfig {
						invulnerables: collators::invulnerables_asset_hub_polkadot()
							.iter()
							.cloned()
							.map(|(acc, _)| acc)
							.collect(),
						candidacy_bond: ED * 16,
						..Default::default()
					},
					session: asset_hub_polkadot_runtime::SessionConfig {
						keys: collators::invulnerables_asset_hub_polkadot()
							.into_iter()
							.map(|(acc, aura)| (acc.clone(), acc, asset_hub_polkadot_runtime::SessionKeys { aura }))
							.collect(),
					},
					polkadot_xcm: asset_hub_polkadot_runtime::PolkadotXcmConfig {
						safe_xcm_version: Some(SAFE_XCM_VERSION),
						..Default::default()
					},
					..Default::default()
				};
				genesis_config.build_storage().unwrap()
			}
			pub fn force_create_asset_call(
				asset_id: MultiLocation,
				owner: AccountId,
				is_sufficient: bool,
				min_balance: Balance,
			) -> DoubleEncoded<()> {
				RuntimeCall::ForeignAssets(pallet_assets::Call::<Runtime, pallet_assets::Instance2>::force_create {
					id: asset_id.into(),
					owner: owner.into(),
					is_sufficient,
					min_balance,
				})
				.encode()
				.into()
			}
		}
		pub struct AssetHubPolkadot;
		impl Parachain for AssetHubPolkadot {
			type Runtime = asset_hub_polkadot_runtime::Runtime;
			type RuntimeOrigin = asset_hub_polkadot_runtime::RuntimeOrigin;
			type RuntimeCall = asset_hub_polkadot_runtime::RuntimeCall;
			type RuntimeEvent = asset_hub_polkadot_runtime::RuntimeEvent;
			type XcmpMessageHandler = asset_hub_polkadot_runtime::XcmpQueue;
			type DmpMessageHandler = asset_hub_polkadot_runtime::DmpQueue;
			type LocationToAccountId = asset_hub_polkadot_runtime::xcm_config::LocationToAccountId;
			type System = asset_hub_polkadot_runtime::System;
			type Balances = asset_hub_polkadot_runtime::Balances;
			type ParachainSystem = asset_hub_polkadot_runtime::ParachainSystem;
			type ParachainInfo = asset_hub_polkadot_runtime::ParachainInfo;
		}
		pub trait AssetHubPolkadotPallet {
			type PolkadotXcm;
			type Assets;
		}
		impl AssetHubPolkadotPallet for AssetHubPolkadot {
			type PolkadotXcm = asset_hub_polkadot_runtime::PolkadotXcm;
			type Assets = asset_hub_polkadot_runtime::Assets;
		}
		impl ::xcm_emulator::XcmpMessageHandler for AssetHubPolkadot {
			fn handle_xcmp_messages<
				'a,
				I: Iterator<Item = (::xcm_emulator::ParaId, ::xcm_emulator::RelayBlockNumber, &'a [u8])>,
			>(
				iter: I,
				max_weight: ::xcm_emulator::Weight,
			) -> ::xcm_emulator::Weight {
				use ::xcm_emulator::{TestExt, XcmpMessageHandler};
				AssetHubPolkadot::execute_with(|| {
					<Self as Parachain>::XcmpMessageHandler::handle_xcmp_messages(iter, max_weight)
				})
			}
		}
		impl ::xcm_emulator::DmpMessageHandler for AssetHubPolkadot {
			fn handle_dmp_messages(
				iter: impl Iterator<Item = (::xcm_emulator::RelayBlockNumber, Vec<u8>)>,
				max_weight: ::xcm_emulator::Weight,
			) -> ::xcm_emulator::Weight {
				use ::xcm_emulator::{DmpMessageHandler, TestExt};
				AssetHubPolkadot::execute_with(|| {
					<Self as Parachain>::DmpMessageHandler::handle_dmp_messages(iter, max_weight)
				})
			}
		}
		pub const EXT_ASSETHUBPOLKADOT: ::std::thread::LocalKey<
			::xcm_emulator::RefCell<::xcm_emulator::sp_io::TestExternalities>,
		> = {
			#[inline]
			fn __init() -> ::xcm_emulator::RefCell<::xcm_emulator::sp_io::TestExternalities> {
				::xcm_emulator::RefCell::new(<AssetHubPolkadot>::build_new_ext(asset_hub_polkadot::genesis()))
			}
			#[inline]
			unsafe fn __getit(
				init: ::std::option::Option<
					&mut ::std::option::Option<::xcm_emulator::RefCell<::xcm_emulator::sp_io::TestExternalities>>,
				>,
			) -> ::std::option::Option<&'static ::xcm_emulator::RefCell<::xcm_emulator::sp_io::TestExternalities>> {
				#[thread_local]
				static __KEY:
                            ::std::thread::local_impl::Key<::xcm_emulator::RefCell<::xcm_emulator::sp_io::TestExternalities>>
                            =
                            ::std::thread::local_impl::Key::<::xcm_emulator::RefCell<::xcm_emulator::sp_io::TestExternalities>>::new();
				unsafe {
					__KEY.get(move || {
						if let ::std::option::Option::Some(init) = init {
							if let ::std::option::Option::Some(value) = init.take() {
								return value;
							} else if true {
								{
									::core::panicking::panic_fmt(format_args!(
										"internal error: entered unreachable code: {0}",
										format_args!("missing default value")
									));
								};
							}
						}
						__init()
					})
				}
			}
			unsafe { ::std::thread::LocalKey::new(__getit) }
		};
		impl TestExt for AssetHubPolkadot {
			fn build_new_ext(storage: ::xcm_emulator::Storage) -> ::xcm_emulator::sp_io::TestExternalities {
				let mut ext = sp_io::TestExternalities::new(storage);
				ext.execute_with(|| {
					#[allow(clippy::no_effect)]
					();
					sp_tracing::try_init_simple();
					<Self as Parachain>::System::set_block_number(1);
				});
				ext
			}
			fn new_ext() -> ::xcm_emulator::sp_io::TestExternalities {
				<AssetHubPolkadot>::build_new_ext(asset_hub_polkadot::genesis())
			}
			fn reset_ext() {
				EXT_ASSETHUBPOLKADOT
					.with(|v| *v.borrow_mut() = <AssetHubPolkadot>::build_new_ext(asset_hub_polkadot::genesis()));
			}
			fn execute_with<R>(execute: impl FnOnce() -> R) -> R {
				use ::xcm_emulator::{Bridge, Get, Hooks, Network, NetworkComponent};
				<AssetHubPolkadot as NetworkComponent>::Network::init();
				let mut relay_block_number = <AssetHubPolkadot as NetworkComponent>::Network::relay_block_number();
				relay_block_number += 1;
				<AssetHubPolkadot as NetworkComponent>::Network::set_relay_block_number(relay_block_number);
				let para_id = <AssetHubPolkadot>::para_id().into();
				EXT_ASSETHUBPOLKADOT.with(|v| {
					v.borrow_mut().execute_with(|| {
						let relay_block_number = <AssetHubPolkadot as NetworkComponent>::Network::relay_block_number();
						let _ = <Self as Parachain>::ParachainSystem::set_validation_data(
							<Self as Parachain>::RuntimeOrigin::none(),
							<AssetHubPolkadot as NetworkComponent>::Network::hrmp_channel_parachain_inherent_data(
								para_id,
								relay_block_number,
							),
						);
					})
				});
				let r = EXT_ASSETHUBPOLKADOT.with(|v| v.borrow_mut().execute_with(execute));
				EXT_ASSETHUBPOLKADOT.with(|v| {
					v.borrow_mut().execute_with(|| {
						use sp_runtime::traits::Header as HeaderT;
						let block_number = <Self as Parachain>::System::block_number();
						let mock_header = HeaderT::new(
							0,
							Default::default(),
							Default::default(),
							Default::default(),
							Default::default(),
						);
						<Self as Parachain>::ParachainSystem::on_finalize(block_number);
						let collation_info = <Self as Parachain>::ParachainSystem::collect_collation_info(&mock_header);
						let relay_block_number = <AssetHubPolkadot as NetworkComponent>::Network::relay_block_number();
						for msg in collation_info.upward_messages.clone() {
							<AssetHubPolkadot>::send_upward_message(para_id, msg);
						}
						for msg in collation_info.horizontal_messages {
							<AssetHubPolkadot>::send_horizontal_messages(
								msg.recipient.into(),
								<[_]>::into_vec(
									#[rustc_box]
									::alloc::boxed::Box::new([(para_id.into(), relay_block_number, msg.data)]),
								)
								.into_iter(),
							);
						}
						type NetworkBridge = <<AssetHubPolkadot as NetworkComponent>::Network as Network>::Bridge;
						let bridge_messages = <NetworkBridge as Bridge>::Handler::get_source_outbound_messages();
						for msg in bridge_messages {
							<AssetHubPolkadot>::send_bridged_messages(msg);
						}
						<Self as Parachain>::ParachainSystem::on_initialize(block_number);
					})
				});
				<AssetHubPolkadot as NetworkComponent>::Network::process_messages();
				r
			}
			fn ext_wrapper<R>(func: impl FnOnce() -> R) -> R {
				EXT_ASSETHUBPOLKADOT.with(|v| v.borrow_mut().execute_with(|| func()))
			}
		}
		pub struct SpiritnetPolkadot;
		impl Parachain for SpiritnetPolkadot {
			type Runtime = crate::Runtime;
			type RuntimeOrigin = crate::RuntimeOrigin;
			type RuntimeCall = crate::RuntimeCall;
			type RuntimeEvent = crate::RuntimeEvent;
			type XcmpMessageHandler = crate::XcmpQueue;
			type DmpMessageHandler = crate::DmpQueue;
			type LocationToAccountId = LocationToAccountId<RelayNetworkId>;
			type System = crate::System;
			type Balances = crate::Balances;
			type ParachainSystem = crate::ParachainSystem;
			type ParachainInfo = crate::ParachainInfo;
		}
		pub trait SpiritnetPolkadotPallet {}
		impl SpiritnetPolkadotPallet for SpiritnetPolkadot {}
		impl ::xcm_emulator::XcmpMessageHandler for SpiritnetPolkadot {
			fn handle_xcmp_messages<
				'a,
				I: Iterator<Item = (::xcm_emulator::ParaId, ::xcm_emulator::RelayBlockNumber, &'a [u8])>,
			>(
				iter: I,
				max_weight: ::xcm_emulator::Weight,
			) -> ::xcm_emulator::Weight {
				use ::xcm_emulator::{TestExt, XcmpMessageHandler};
				SpiritnetPolkadot::execute_with(|| {
					<Self as Parachain>::XcmpMessageHandler::handle_xcmp_messages(iter, max_weight)
				})
			}
		}
		impl ::xcm_emulator::DmpMessageHandler for SpiritnetPolkadot {
			fn handle_dmp_messages(
				iter: impl Iterator<Item = (::xcm_emulator::RelayBlockNumber, Vec<u8>)>,
				max_weight: ::xcm_emulator::Weight,
			) -> ::xcm_emulator::Weight {
				use ::xcm_emulator::{DmpMessageHandler, TestExt};
				SpiritnetPolkadot::execute_with(|| {
					<Self as Parachain>::DmpMessageHandler::handle_dmp_messages(iter, max_weight)
				})
			}
		}
		pub const EXT_SPIRITNETPOLKADOT: ::std::thread::LocalKey<
			::xcm_emulator::RefCell<::xcm_emulator::sp_io::TestExternalities>,
		> = {
			#[inline]
			fn __init() -> ::xcm_emulator::RefCell<::xcm_emulator::sp_io::TestExternalities> {
				::xcm_emulator::RefCell::new(<SpiritnetPolkadot>::build_new_ext(spiritnet::genesis()))
			}
			#[inline]
			unsafe fn __getit(
				init: ::std::option::Option<
					&mut ::std::option::Option<::xcm_emulator::RefCell<::xcm_emulator::sp_io::TestExternalities>>,
				>,
			) -> ::std::option::Option<&'static ::xcm_emulator::RefCell<::xcm_emulator::sp_io::TestExternalities>> {
				#[thread_local]
				static __KEY:
                            ::std::thread::local_impl::Key<::xcm_emulator::RefCell<::xcm_emulator::sp_io::TestExternalities>>
                            =
                            ::std::thread::local_impl::Key::<::xcm_emulator::RefCell<::xcm_emulator::sp_io::TestExternalities>>::new();
				unsafe {
					__KEY.get(move || {
						if let ::std::option::Option::Some(init) = init {
							if let ::std::option::Option::Some(value) = init.take() {
								return value;
							} else if true {
								{
									::core::panicking::panic_fmt(format_args!(
										"internal error: entered unreachable code: {0}",
										format_args!("missing default value")
									));
								};
							}
						}
						__init()
					})
				}
			}
			unsafe { ::std::thread::LocalKey::new(__getit) }
		};
		impl TestExt for SpiritnetPolkadot {
			fn build_new_ext(storage: ::xcm_emulator::Storage) -> ::xcm_emulator::sp_io::TestExternalities {
				let mut ext = sp_io::TestExternalities::new(storage);
				ext.execute_with(|| {
					#[allow(clippy::no_effect)]
					();
					sp_tracing::try_init_simple();
					<Self as Parachain>::System::set_block_number(1);
				});
				ext
			}
			fn new_ext() -> ::xcm_emulator::sp_io::TestExternalities {
				<SpiritnetPolkadot>::build_new_ext(spiritnet::genesis())
			}
			fn reset_ext() {
				EXT_SPIRITNETPOLKADOT
					.with(|v| *v.borrow_mut() = <SpiritnetPolkadot>::build_new_ext(spiritnet::genesis()));
			}
			fn execute_with<R>(execute: impl FnOnce() -> R) -> R {
				use ::xcm_emulator::{Bridge, Get, Hooks, Network, NetworkComponent};
				<SpiritnetPolkadot as NetworkComponent>::Network::init();
				let mut relay_block_number = <SpiritnetPolkadot as NetworkComponent>::Network::relay_block_number();
				relay_block_number += 1;
				<SpiritnetPolkadot as NetworkComponent>::Network::set_relay_block_number(relay_block_number);
				let para_id = <SpiritnetPolkadot>::para_id().into();
				EXT_SPIRITNETPOLKADOT.with(|v| {
					v.borrow_mut().execute_with(|| {
						let relay_block_number = <SpiritnetPolkadot as NetworkComponent>::Network::relay_block_number();
						let _ = <Self as Parachain>::ParachainSystem::set_validation_data(
							<Self as Parachain>::RuntimeOrigin::none(),
							<SpiritnetPolkadot as NetworkComponent>::Network::hrmp_channel_parachain_inherent_data(
								para_id,
								relay_block_number,
							),
						);
					})
				});
				let r = EXT_SPIRITNETPOLKADOT.with(|v| v.borrow_mut().execute_with(execute));
				EXT_SPIRITNETPOLKADOT.with(|v| {
					v.borrow_mut().execute_with(|| {
						use sp_runtime::traits::Header as HeaderT;
						let block_number = <Self as Parachain>::System::block_number();
						let mock_header = HeaderT::new(
							0,
							Default::default(),
							Default::default(),
							Default::default(),
							Default::default(),
						);
						<Self as Parachain>::ParachainSystem::on_finalize(block_number);
						let collation_info = <Self as Parachain>::ParachainSystem::collect_collation_info(&mock_header);
						let relay_block_number = <SpiritnetPolkadot as NetworkComponent>::Network::relay_block_number();
						for msg in collation_info.upward_messages.clone() {
							<SpiritnetPolkadot>::send_upward_message(para_id, msg);
						}
						for msg in collation_info.horizontal_messages {
							<SpiritnetPolkadot>::send_horizontal_messages(
								msg.recipient.into(),
								<[_]>::into_vec(
									#[rustc_box]
									::alloc::boxed::Box::new([(para_id.into(), relay_block_number, msg.data)]),
								)
								.into_iter(),
							);
						}
						type NetworkBridge = <<SpiritnetPolkadot as NetworkComponent>::Network as Network>::Bridge;
						let bridge_messages = <NetworkBridge as Bridge>::Handler::get_source_outbound_messages();
						for msg in bridge_messages {
							<SpiritnetPolkadot>::send_bridged_messages(msg);
						}
						<Self as Parachain>::ParachainSystem::on_initialize(block_number);
					})
				});
				<SpiritnetPolkadot as NetworkComponent>::Network::process_messages();
				r
			}
			fn ext_wrapper<R>(func: impl FnOnce() -> R) -> R {
				EXT_SPIRITNETPOLKADOT.with(|v| v.borrow_mut().execute_with(|| func()))
			}
		}
	}
	mod relaychain {
		use crate::xcm_config::tests::utils::get_from_seed;
		use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
		use polkadot_primitives::{AccountId, AssignmentId, Balance, BlockNumber, ValidatorId};
		pub(crate) use polkadot_runtime::System;
		use polkadot_runtime_parachains::{
			configuration::HostConfiguration,
			paras::{ParaGenesisArgs, ParaKind},
		};
		use polkadot_service::chain_spec::get_authority_keys_from_seed_no_beefy;
		use sc_consensus_grandpa::AuthorityId as GrandpaId;
		use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
		use sp_consensus_babe::AuthorityId as BabeId;
		use sp_core::{sr25519, storage::Storage, Pair, Public};
		use sp_runtime::{
			traits::{IdentifyAccount, Verify},
			BuildStorage, MultiSignature, Perbill,
		};
		use xcm_emulator::{decl_test_relay_chains, RelayChain, TestExt, XcmHash};
		type AccountPublic = <MultiSignature as Verify>::Signer;
		const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;
		/// Helper function to generate an account ID from seed.
		fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
		where
			AccountPublic: From<<TPublic::Pair as Pair>::Public>,
		{
			AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
		}
		pub fn get_host_config() -> HostConfiguration<BlockNumber> {
			HostConfiguration {
				max_upward_queue_count: 10,
				max_upward_queue_size: 51200,
				max_upward_message_size: 51200,
				max_upward_message_num_per_candidate: 10,
				max_downward_message_size: 51200,
				hrmp_sender_deposit: 100_000_000_000,
				hrmp_recipient_deposit: 100_000_000_000,
				hrmp_channel_max_capacity: 1000,
				hrmp_channel_max_message_size: 102400,
				hrmp_channel_max_total_size: 102400,
				hrmp_max_parachain_outbound_channels: 30,
				hrmp_max_parachain_inbound_channels: 30,
				..Default::default()
			}
		}
		fn session_keys(
			babe: BabeId,
			grandpa: GrandpaId,
			im_online: ImOnlineId,
			para_validator: ValidatorId,
			para_assignment: AssignmentId,
			authority_discovery: AuthorityDiscoveryId,
		) -> polkadot_runtime::SessionKeys {
			polkadot_runtime::SessionKeys {
				babe,
				grandpa,
				im_online,
				para_validator,
				para_assignment,
				authority_discovery,
			}
		}
		pub fn initial_authorities() -> Vec<(
			AccountId,
			AccountId,
			BabeId,
			GrandpaId,
			ImOnlineId,
			ValidatorId,
			AssignmentId,
			AuthorityDiscoveryId,
		)> {
			<[_]>::into_vec(
				#[rustc_box]
				::alloc::boxed::Box::new([get_authority_keys_from_seed_no_beefy("Alice")]),
			)
		}
		pub mod accounts {
			use super::*;
			pub const ALICE: &str = "Alice";
			pub const BOB: &str = "Bob";
			pub const CHARLIE: &str = "Charlie";
			pub const DAVE: &str = "Dave";
			pub const EVE: &str = "Eve";
			pub const FERDIE: &str = "Ferdei";
			pub const ALICE_STASH: &str = "Alice//stash";
			pub const BOB_STASH: &str = "Bob//stash";
			pub const CHARLIE_STASH: &str = "Charlie//stash";
			pub const DAVE_STASH: &str = "Dave//stash";
			pub const EVE_STASH: &str = "Eve//stash";
			pub const FERDIE_STASH: &str = "Ferdie//stash";
			pub const FERDIE_BEEFY: &str = "Ferdie//stash";
			pub fn init_balances() -> Vec<AccountId> {
				<[_]>::into_vec(
					#[rustc_box]
					::alloc::boxed::Box::new([
						get_account_id_from_seed::<sr25519::Public>(ALICE),
						get_account_id_from_seed::<sr25519::Public>(BOB),
						get_account_id_from_seed::<sr25519::Public>(CHARLIE),
						get_account_id_from_seed::<sr25519::Public>(DAVE),
						get_account_id_from_seed::<sr25519::Public>(EVE),
						get_account_id_from_seed::<sr25519::Public>(FERDIE),
						get_account_id_from_seed::<sr25519::Public>(ALICE_STASH),
						get_account_id_from_seed::<sr25519::Public>(BOB_STASH),
						get_account_id_from_seed::<sr25519::Public>(CHARLIE_STASH),
						get_account_id_from_seed::<sr25519::Public>(DAVE_STASH),
						get_account_id_from_seed::<sr25519::Public>(EVE_STASH),
						get_account_id_from_seed::<sr25519::Public>(FERDIE_STASH),
					]),
				)
			}
		}
		pub mod collators {
			use super::*;
			use asset_hub_polkadot_runtime::common::{AssetHubPolkadotAuraId, AuraId};
			pub fn invulnerables_asset_hub_polkadot() -> Vec<(AccountId, AssetHubPolkadotAuraId)> {
				<[_]>::into_vec(
					#[rustc_box]
					::alloc::boxed::Box::new([
						(
							get_account_id_from_seed::<sr25519::Public>("Alice"),
							get_from_seed::<AssetHubPolkadotAuraId>("Alice"),
						),
						(
							get_account_id_from_seed::<sr25519::Public>("Bob"),
							get_from_seed::<AssetHubPolkadotAuraId>("Bob"),
						),
					]),
				)
			}
			pub fn invulnerables() -> Vec<(AccountId, AuraId)> {
				<[_]>::into_vec(
					#[rustc_box]
					::alloc::boxed::Box::new([
						(
							get_account_id_from_seed::<sr25519::Public>("Alice"),
							get_from_seed::<AuraId>("Alice"),
						),
						(
							get_account_id_from_seed::<sr25519::Public>("Bob"),
							get_from_seed::<AuraId>("Bob"),
						),
					]),
				)
			}
		}
		pub mod validators {
			use super::*;
			pub fn initial_authorities() -> Vec<(
				AccountId,
				AccountId,
				BabeId,
				GrandpaId,
				ImOnlineId,
				ValidatorId,
				AssignmentId,
				AuthorityDiscoveryId,
			)> {
				<[_]>::into_vec(
					#[rustc_box]
					::alloc::boxed::Box::new([get_authority_keys_from_seed_no_beefy("Alice")]),
				)
			}
		}
		pub mod polkadot {
			use super::*;
			use crate::xcm_config::tests::parachains::asset_hub_polkadot;
			use polkadot_primitives::{HeadData, ValidationCode};
			pub const ED: Balance = polkadot_runtime_constants::currency::EXISTENTIAL_DEPOSIT;
			const STASH: u128 = 100 * polkadot_runtime_constants::currency::UNITS;
			pub fn get_host_config() -> HostConfiguration<BlockNumber> {
				HostConfiguration {
					max_upward_queue_count: 10,
					max_upward_queue_size: 51200,
					max_upward_message_size: 51200,
					max_upward_message_num_per_candidate: 10,
					max_downward_message_size: 51200,
					hrmp_sender_deposit: 100_000_000_000,
					hrmp_recipient_deposit: 100_000_000_000,
					hrmp_channel_max_capacity: 1000,
					hrmp_channel_max_message_size: 102400,
					hrmp_channel_max_total_size: 102400,
					hrmp_max_parachain_outbound_channels: 30,
					hrmp_max_parachain_inbound_channels: 30,
					..Default::default()
				}
			}
			fn session_keys(
				babe: BabeId,
				grandpa: GrandpaId,
				im_online: ImOnlineId,
				para_validator: ValidatorId,
				para_assignment: AssignmentId,
				authority_discovery: AuthorityDiscoveryId,
			) -> polkadot_runtime::SessionKeys {
				polkadot_runtime::SessionKeys {
					babe,
					grandpa,
					im_online,
					para_validator,
					para_assignment,
					authority_discovery,
				}
			}
			pub fn genesis() -> Storage {
				let genesis_config = polkadot_runtime::RuntimeGenesisConfig {
					system: polkadot_runtime::SystemConfig {
						code: polkadot_runtime::WASM_BINARY.unwrap().to_vec(),
						..Default::default()
					},
					balances: polkadot_runtime::BalancesConfig {
						balances: accounts::init_balances()
							.iter()
							.cloned()
							.map(|k| (k, ED * 4096))
							.collect(),
					},
					session: polkadot_runtime::SessionConfig {
						keys: validators::initial_authorities()
							.iter()
							.map(|x| {
								(
									x.0.clone(),
									x.0.clone(),
									polkadot::session_keys(
										x.2.clone(),
										x.3.clone(),
										x.4.clone(),
										x.5.clone(),
										x.6.clone(),
										x.7.clone(),
									),
								)
							})
							.collect::<Vec<_>>(),
					},
					staking: polkadot_runtime::StakingConfig {
						validator_count: validators::initial_authorities().len() as u32,
						minimum_validator_count: 1,
						stakers: validators::initial_authorities()
							.iter()
							.map(|x| {
								(
									x.0.clone(),
									x.1.clone(),
									STASH,
									polkadot_runtime::StakerStatus::Validator,
								)
							})
							.collect(),
						invulnerables: validators::initial_authorities().iter().map(|x| x.0.clone()).collect(),
						force_era: pallet_staking::Forcing::ForceNone,
						slash_reward_fraction: Perbill::from_percent(10),
						..Default::default()
					},
					babe: polkadot_runtime::BabeConfig {
						authorities: Default::default(),
						epoch_config: Some(polkadot_runtime::BABE_GENESIS_EPOCH_CONFIG),
						..Default::default()
					},
					configuration: polkadot_runtime::ConfigurationConfig {
						config: get_host_config(),
					},
					paras: polkadot_runtime::ParasConfig {
						paras: <[_]>::into_vec(
							#[rustc_box]
							::alloc::boxed::Box::new([(
								asset_hub_polkadot::PARA_ID.into(),
								ParaGenesisArgs {
									genesis_head: HeadData::default(),
									validation_code: ValidationCode(
										asset_hub_polkadot_runtime::WASM_BINARY.unwrap().to_vec(),
									),
									para_kind: ParaKind::Parachain,
								},
							)]),
						),
						..Default::default()
					},
					..Default::default()
				};
				genesis_config.build_storage().unwrap()
			}
		}
		pub struct Polkadot;
		impl RelayChain for Polkadot {
			type Runtime = polkadot_runtime::Runtime;
			type RuntimeOrigin = polkadot_runtime::RuntimeOrigin;
			type RuntimeCall = polkadot_runtime::RuntimeCall;
			type RuntimeEvent = polkadot_runtime::RuntimeEvent;
			type XcmConfig = polkadot_runtime::xcm_config::XcmConfig;
			type SovereignAccountOf = polkadot_runtime::xcm_config::SovereignAccountOf;
			type System = polkadot_runtime::System;
			type Balances = polkadot_runtime::Balances;
		}
		pub trait PolkadotPallet {
			type XcmPallet;
		}
		impl PolkadotPallet for Polkadot {
			type XcmPallet = polkadot_runtime::XcmPallet;
		}
		impl ::xcm_emulator::ProcessMessage for Polkadot {
			type Origin = ::xcm_emulator::ParaId;
			fn process_message(
				msg: &[u8],
				para: Self::Origin,
				meter: &mut ::xcm_emulator::WeightMeter,
				_id: &mut XcmHash,
			) -> Result<bool, ::xcm_emulator::ProcessMessageError> {
				use ::xcm_emulator::{AggregateMessageOrigin, EnqueueMessage, ServiceQueues, UmpQueueId, Weight};
				use polkadot_runtime::MessageQueue as message_queue;
				use polkadot_runtime::RuntimeEvent as runtime_event;
				Self::execute_with(|| {
					<polkadot_runtime::MessageQueue as EnqueueMessage<AggregateMessageOrigin>>::enqueue_message(
						msg.try_into().expect("Message too long"),
						AggregateMessageOrigin::Ump(UmpQueueId::Para(para.clone())),
					);
					<polkadot_runtime::System>::reset_events();
					<polkadot_runtime::MessageQueue as ServiceQueues>::service_queues(Weight::MAX);
					let events = <polkadot_runtime::System>::events();
					let event = events.last().expect("There must be at least one event");
					match &event.event {
						runtime_event::MessageQueue(::xcm_emulator::pallet_message_queue::Event::Processed {
							origin,
							..
						}) => {
							match (&origin, &&AggregateMessageOrigin::Ump(UmpQueueId::Para(para))) {
								(left_val, right_val) => {
									if !(*left_val == *right_val) {
										let kind = ::core::panicking::AssertKind::Eq;
										::core::panicking::assert_failed(
											kind,
											&*left_val,
											&*right_val,
											::core::option::Option::None,
										);
									}
								}
							};
						}
						event => {
							::core::panicking::panic_fmt(format_args!("Unexpected event: {0:#?}", event));
						}
					}
					Ok(true)
				})
			}
		}
		pub const EXT_POLKADOT: ::std::thread::LocalKey<
			::xcm_emulator::RefCell<::xcm_emulator::sp_io::TestExternalities>,
		> = {
			#[inline]
			fn __init() -> ::xcm_emulator::RefCell<::xcm_emulator::sp_io::TestExternalities> {
				::xcm_emulator::RefCell::new(<Polkadot>::build_new_ext(polkadot::genesis()))
			}
			#[inline]
			unsafe fn __getit(
				init: ::std::option::Option<
					&mut ::std::option::Option<::xcm_emulator::RefCell<::xcm_emulator::sp_io::TestExternalities>>,
				>,
			) -> ::std::option::Option<&'static ::xcm_emulator::RefCell<::xcm_emulator::sp_io::TestExternalities>> {
				#[thread_local]
				static __KEY:
                            ::std::thread::local_impl::Key<::xcm_emulator::RefCell<::xcm_emulator::sp_io::TestExternalities>>
                            =
                            ::std::thread::local_impl::Key::<::xcm_emulator::RefCell<::xcm_emulator::sp_io::TestExternalities>>::new();
				unsafe {
					__KEY.get(move || {
						if let ::std::option::Option::Some(init) = init {
							if let ::std::option::Option::Some(value) = init.take() {
								return value;
							} else if true {
								{
									::core::panicking::panic_fmt(format_args!(
										"internal error: entered unreachable code: {0}",
										format_args!("missing default value")
									));
								};
							}
						}
						__init()
					})
				}
			}
			unsafe { ::std::thread::LocalKey::new(__getit) }
		};
		impl TestExt for Polkadot {
			fn build_new_ext(storage: ::xcm_emulator::Storage) -> ::xcm_emulator::sp_io::TestExternalities {
				let mut ext = sp_io::TestExternalities::new(storage);
				ext.execute_with(|| {
					#[allow(clippy::no_effect)]
					();
					sp_tracing::try_init_simple();
					<Self as RelayChain>::System::set_block_number(1);
				});
				ext
			}
			fn new_ext() -> ::xcm_emulator::sp_io::TestExternalities {
				<Polkadot>::build_new_ext(polkadot::genesis())
			}
			fn reset_ext() {
				EXT_POLKADOT.with(|v| *v.borrow_mut() = <Polkadot>::build_new_ext(polkadot::genesis()));
			}
			fn execute_with<R>(execute: impl FnOnce() -> R) -> R {
				use ::xcm_emulator::{Network, NetworkComponent};
				<Polkadot as NetworkComponent>::Network::init();
				let r = EXT_POLKADOT.with(|v| v.borrow_mut().execute_with(execute));
				EXT_POLKADOT.with(|v| {
					v.borrow_mut().execute_with(|| {
						use ::xcm_emulator::polkadot_primitives::runtime_api::runtime_decl_for_parachain_host::ParachainHostV5;
						for para_id in <Polkadot as NetworkComponent>::Network::para_ids() {
							let downward_messages = <Self as RelayChain>::Runtime::dmq_contents(para_id.into())
								.into_iter()
								.map(|inbound| (inbound.sent_at, inbound.msg));
							if downward_messages.len() == 0 {
								continue;
							}
							<Polkadot>::send_downward_messages(para_id, downward_messages.into_iter());
						}
					})
				});
				<Polkadot as NetworkComponent>::Network::process_messages();
				r
			}
			fn ext_wrapper<R>(func: impl FnOnce() -> R) -> R {
				EXT_POLKADOT.with(|v| v.borrow_mut().execute_with(|| func()))
			}
		}
	}
	mod utils {
		use sp_core::{Pair, Public};
		use sp_runtime::traits::IdentifyAccount;
		/// Helper function to generate a crypto pair from seed
		pub(crate) fn get_from_seed<TPublic>(seed: &str) -> <TPublic::Pair as Pair>::Public
		where
			TPublic: Public,
		{
			TPublic::Pair::from_string(
				&{
					let res = ::alloc::fmt::format(format_args!("//{0}", seed));
					res
				},
				None,
			)
			.expect("static values are valid; qed")
			.public()
		}
		/// Helper function to generate an account ID from seed.
		pub(crate) fn get_account_id_from_seed<AccountPublic, TPublic>(seed: &str) -> AccountPublic::AccountId
		where
			AccountPublic: From<<TPublic::Pair as Pair>::Public> + IdentifyAccount,
			TPublic: Public,
		{
			AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
		}
	}
	use self::{
		parachains::asset_hub_polkadot::{self, PARA_ID},
		relaychain::accounts::{ALICE, BOB},
	};
	use crate::{
		xcm_config::tests::{
			parachains::{AssetHubPolkadot, SpiritnetPolkadot},
			relaychain::{Polkadot, System as PolkadotSystem},
		},
		PolkadotXcm as SpiritnetXcm, RuntimeEvent as SpiritnetRuntimeEvent, System as SpiritnetSystem,
	};
	use asset_hub_polkadot_runtime::{RuntimeEvent as AssetHubRuntimeEvent, System as AssetHubSystem};
	use cumulus_pallet_xcmp_queue::Event as XcmpQueueEvent;
	use frame_support::{assert_err, assert_ok};
	use frame_system::RawOrigin;
	use polkadot_primitives::{AccountId, Balance};
	use polkadot_service::chain_spec::get_account_id_from_seed;
	use runtime_common::constants::EXISTENTIAL_DEPOSIT;
	use sp_core::{sr25519, Get};
	use sp_runtime::{DispatchError, ModuleError};
	use xcm::prelude::*;
	use xcm_emulator::{decl_test_networks, BridgeMessageHandler, Parachain, RelayChain, TestExt};
	use xcm_executor::traits::ConvertLocation;
	pub struct PolkadotNetwork;
	impl PolkadotNetwork {
		pub fn reset() {
			use ::xcm_emulator::{TestExt, VecDeque};
			::xcm_emulator::INITIALIZED.with(|b| b.borrow_mut().remove("PolkadotNetwork"));
			::xcm_emulator::DOWNWARD_MESSAGES.with(|b| b.borrow_mut().remove("PolkadotNetwork"));
			::xcm_emulator::DMP_DONE.with(|b| b.borrow_mut().remove("PolkadotNetwork"));
			::xcm_emulator::UPWARD_MESSAGES.with(|b| b.borrow_mut().remove("PolkadotNetwork"));
			::xcm_emulator::HORIZONTAL_MESSAGES.with(|b| b.borrow_mut().remove("PolkadotNetwork"));
			::xcm_emulator::BRIDGED_MESSAGES.with(|b| b.borrow_mut().remove("PolkadotNetwork"));
			::xcm_emulator::RELAY_BLOCK_NUMBER.with(|b| b.borrow_mut().remove("PolkadotNetwork"));
			<Polkadot>::reset_ext();
			<SpiritnetPolkadot>::reset_ext();
			<AssetHubPolkadot>::reset_ext();
		}
	}
	impl ::xcm_emulator::Network for PolkadotNetwork {
		type Bridge = ();
		fn init() {
			if ::xcm_emulator::INITIALIZED.with(|b| b.borrow_mut().get("PolkadotNetwork").is_none()) {
				::xcm_emulator::INITIALIZED.with(|b| b.borrow_mut().insert("PolkadotNetwork".to_string(), true));
				::xcm_emulator::DOWNWARD_MESSAGES.with(|b| {
					b.borrow_mut()
						.insert("PolkadotNetwork".to_string(), ::xcm_emulator::VecDeque::new())
				});
				::xcm_emulator::DMP_DONE.with(|b| {
					b.borrow_mut()
						.insert("PolkadotNetwork".to_string(), ::xcm_emulator::VecDeque::new())
				});
				::xcm_emulator::UPWARD_MESSAGES.with(|b| {
					b.borrow_mut()
						.insert("PolkadotNetwork".to_string(), ::xcm_emulator::VecDeque::new())
				});
				::xcm_emulator::HORIZONTAL_MESSAGES.with(|b| {
					b.borrow_mut()
						.insert("PolkadotNetwork".to_string(), ::xcm_emulator::VecDeque::new())
				});
				::xcm_emulator::BRIDGED_MESSAGES.with(|b| {
					b.borrow_mut()
						.insert("PolkadotNetwork".to_string(), ::xcm_emulator::VecDeque::new())
				});
				::xcm_emulator::RELAY_BLOCK_NUMBER.with(|b| b.borrow_mut().insert("PolkadotNetwork".to_string(), 1));
				::xcm_emulator::PARA_IDS
					.with(|b| b.borrow_mut().insert("PolkadotNetwork".to_string(), Self::para_ids()));
				<SpiritnetPolkadot>::prepare_for_xcmp();
				<AssetHubPolkadot>::prepare_for_xcmp();
			}
		}
		fn para_ids() -> Vec<u32> {
			<[_]>::into_vec(
				#[rustc_box]
				::alloc::boxed::Box::new([
					<SpiritnetPolkadot>::para_id().into(),
					<AssetHubPolkadot>::para_id().into(),
				]),
			)
		}
		fn relay_block_number() -> u32 {
			::xcm_emulator::RELAY_BLOCK_NUMBER.with(|v| *v.clone().borrow().get("PolkadotNetwork").unwrap())
		}
		fn set_relay_block_number(block_number: u32) {
			::xcm_emulator::RELAY_BLOCK_NUMBER
				.with(|v| v.borrow_mut().insert("PolkadotNetwork".to_string(), block_number));
		}
		fn process_messages() {
			while Self::has_unprocessed_messages() {
				Self::process_upward_messages();
				Self::process_horizontal_messages();
				Self::process_downward_messages();
				Self::process_bridged_messages();
			}
		}
		fn has_unprocessed_messages() -> bool {
			::xcm_emulator::DOWNWARD_MESSAGES.with(|b| !b.borrow_mut().get_mut("PolkadotNetwork").unwrap().is_empty())
				|| ::xcm_emulator::HORIZONTAL_MESSAGES
					.with(|b| !b.borrow_mut().get_mut("PolkadotNetwork").unwrap().is_empty())
				|| ::xcm_emulator::UPWARD_MESSAGES
					.with(|b| !b.borrow_mut().get_mut("PolkadotNetwork").unwrap().is_empty())
				|| ::xcm_emulator::BRIDGED_MESSAGES
					.with(|b| !b.borrow_mut().get_mut("PolkadotNetwork").unwrap().is_empty())
		}
		fn process_downward_messages() {
			use ::xcm_emulator::{Bounded, DmpMessageHandler};
			use polkadot_parachain::primitives::RelayChainBlockNumber;
			while let Some((to_para_id, messages)) = ::xcm_emulator::DOWNWARD_MESSAGES
				.with(|b| b.borrow_mut().get_mut("PolkadotNetwork").unwrap().pop_front())
			{
				let para_id: u32 = <SpiritnetPolkadot>::para_id().into();
				if ::xcm_emulator::PARA_IDS
					.with(|b| b.borrow_mut().get_mut("PolkadotNetwork").unwrap().contains(&to_para_id))
					&& para_id == to_para_id
				{
					let mut msg_dedup: Vec<(RelayChainBlockNumber, Vec<u8>)> = Vec::new();
					for m in &messages {
						msg_dedup.push((m.0, m.1.clone()));
					}
					msg_dedup.dedup();
					let msgs = msg_dedup
						.clone()
						.into_iter()
						.filter(|m| {
							!::xcm_emulator::DMP_DONE.with(|b| {
								b.borrow_mut()
									.get_mut("PolkadotNetwork")
									.unwrap_or(&mut ::xcm_emulator::VecDeque::new())
									.contains(&(to_para_id, m.0, m.1.clone()))
							})
						})
						.collect::<Vec<(RelayChainBlockNumber, Vec<u8>)>>();
					if msgs.len() != 0 {
						<SpiritnetPolkadot>::handle_dmp_messages(
							msgs.clone().into_iter(),
							::xcm_emulator::Weight::max_value(),
						);
						{
							let lvl = ::log::Level::Debug;
							if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
								::log::__private_api::log(
									format_args!(
										"DMP messages processed {0:?} to para_id {1:?}",
										msgs.clone(),
										&to_para_id
									),
									lvl,
									&(
										"dmp::PolkadotNetwork",
										"spiritnet_runtime::xcm_config::tests",
										"runtimes/spiritnet/src/xcm_config/tests/mod.rs",
									),
									46u32,
									::log::__private_api::Option::None,
								);
							}
						};
						for m in msgs {
							::xcm_emulator::DMP_DONE.with(|b| {
								b.borrow_mut()
									.get_mut("PolkadotNetwork")
									.unwrap()
									.push_back((to_para_id, m.0, m.1))
							});
						}
					}
				}
				let para_id: u32 = <AssetHubPolkadot>::para_id().into();
				if ::xcm_emulator::PARA_IDS
					.with(|b| b.borrow_mut().get_mut("PolkadotNetwork").unwrap().contains(&to_para_id))
					&& para_id == to_para_id
				{
					let mut msg_dedup: Vec<(RelayChainBlockNumber, Vec<u8>)> = Vec::new();
					for m in &messages {
						msg_dedup.push((m.0, m.1.clone()));
					}
					msg_dedup.dedup();
					let msgs = msg_dedup
						.clone()
						.into_iter()
						.filter(|m| {
							!::xcm_emulator::DMP_DONE.with(|b| {
								b.borrow_mut()
									.get_mut("PolkadotNetwork")
									.unwrap_or(&mut ::xcm_emulator::VecDeque::new())
									.contains(&(to_para_id, m.0, m.1.clone()))
							})
						})
						.collect::<Vec<(RelayChainBlockNumber, Vec<u8>)>>();
					if msgs.len() != 0 {
						<AssetHubPolkadot>::handle_dmp_messages(
							msgs.clone().into_iter(),
							::xcm_emulator::Weight::max_value(),
						);
						{
							let lvl = ::log::Level::Debug;
							if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
								::log::__private_api::log(
									format_args!(
										"DMP messages processed {0:?} to para_id {1:?}",
										msgs.clone(),
										&to_para_id
									),
									lvl,
									&(
										"dmp::PolkadotNetwork",
										"spiritnet_runtime::xcm_config::tests",
										"runtimes/spiritnet/src/xcm_config/tests/mod.rs",
									),
									46u32,
									::log::__private_api::Option::None,
								);
							}
						};
						for m in msgs {
							::xcm_emulator::DMP_DONE.with(|b| {
								b.borrow_mut()
									.get_mut("PolkadotNetwork")
									.unwrap()
									.push_back((to_para_id, m.0, m.1))
							});
						}
					}
				}
			}
		}
		fn process_horizontal_messages() {
			use ::xcm_emulator::{Bounded, XcmpMessageHandler};
			while let Some((to_para_id, messages)) = ::xcm_emulator::HORIZONTAL_MESSAGES
				.with(|b| b.borrow_mut().get_mut("PolkadotNetwork").unwrap().pop_front())
			{
				let iter = messages
					.iter()
					.map(|(p, b, m)| (*p, *b, &m[..]))
					.collect::<Vec<_>>()
					.into_iter();
				let para_id: u32 = <SpiritnetPolkadot>::para_id().into();
				if ::xcm_emulator::PARA_IDS
					.with(|b| b.borrow_mut().get_mut("PolkadotNetwork").unwrap().contains(&to_para_id))
					&& para_id == to_para_id
				{
					<SpiritnetPolkadot>::handle_xcmp_messages(iter.clone(), ::xcm_emulator::Weight::max_value());
					{
						let lvl = ::log::Level::Debug;
						if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
							::log::__private_api::log(
								format_args!("HRMP messages processed {0:?} to para_id {1:?}", &messages, &to_para_id),
								lvl,
								&(
									"hrmp::PolkadotNetwork",
									"spiritnet_runtime::xcm_config::tests",
									"runtimes/spiritnet/src/xcm_config/tests/mod.rs",
								),
								46u32,
								::log::__private_api::Option::None,
							);
						}
					};
				}
				let para_id: u32 = <AssetHubPolkadot>::para_id().into();
				if ::xcm_emulator::PARA_IDS
					.with(|b| b.borrow_mut().get_mut("PolkadotNetwork").unwrap().contains(&to_para_id))
					&& para_id == to_para_id
				{
					<AssetHubPolkadot>::handle_xcmp_messages(iter.clone(), ::xcm_emulator::Weight::max_value());
					{
						let lvl = ::log::Level::Debug;
						if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
							::log::__private_api::log(
								format_args!("HRMP messages processed {0:?} to para_id {1:?}", &messages, &to_para_id),
								lvl,
								&(
									"hrmp::PolkadotNetwork",
									"spiritnet_runtime::xcm_config::tests",
									"runtimes/spiritnet/src/xcm_config/tests/mod.rs",
								),
								46u32,
								::log::__private_api::Option::None,
							);
						}
					};
				}
			}
		}
		fn process_upward_messages() {
			use ::xcm_emulator::{Bounded, ProcessMessage, WeightMeter};
			use sp_core::Encode;
			while let Some((from_para_id, msg)) =
				::xcm_emulator::UPWARD_MESSAGES.with(|b| b.borrow_mut().get_mut("PolkadotNetwork").unwrap().pop_front())
			{
				let mut weight_meter = WeightMeter::max_limit();
				let _ = <Polkadot>::process_message(
					&msg[..],
					from_para_id.into(),
					&mut weight_meter,
					&mut msg.using_encoded(sp_core::blake2_256),
				);
				{
					let lvl = ::log::Level::Debug;
					if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
						::log::__private_api::log(
							format_args!("Upward message processed {0:?} from para_id {1:?}", &msg, &from_para_id),
							lvl,
							&(
								"ump::PolkadotNetwork",
								"spiritnet_runtime::xcm_config::tests",
								"runtimes/spiritnet/src/xcm_config/tests/mod.rs",
							),
							46u32,
							::log::__private_api::Option::None,
						);
					}
				};
			}
		}
		fn process_bridged_messages() {
			use ::xcm_emulator::Bridge;
			<Self::Bridge as Bridge>::init();
			while let Some(msg) = ::xcm_emulator::BRIDGED_MESSAGES
				.with(|b| b.borrow_mut().get_mut("PolkadotNetwork").unwrap().pop_front())
			{
				let dispatch_result =
					<<Self::Bridge as ::xcm_emulator::Bridge>::Target as TestExt>::ext_wrapper(|| {
						<<Self::Bridge as Bridge>::Handler as BridgeMessageHandler>::dispatch_target_inbound_message(
							msg.clone(),
						)
					});
				match dispatch_result {
					Err(e) => {
						::core::panicking::panic_fmt(format_args!(
							"Error {0:?} processing bridged message: {1:?}",
							e,
							msg.clone()
						));
					}
					Ok(()) => {
						<<Self::Bridge as ::xcm_emulator::Bridge>::Source as TestExt>::ext_wrapper(|| {
							<<Self::Bridge as Bridge>::Handler as BridgeMessageHandler>::notify_source_message_delivery(
								msg.id,
							);
						});
						{
							let lvl = ::log::Level::Debug;
							if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
								::log::__private_api::log(
									format_args!("Bridged message processed {0:?}", msg.clone()),
									lvl,
									&(
										"bridge::PolkadotNetwork",
										"spiritnet_runtime::xcm_config::tests",
										"runtimes/spiritnet/src/xcm_config/tests/mod.rs",
									),
									46u32,
									::log::__private_api::Option::None,
								);
							}
						};
					}
				}
			}
		}
		fn hrmp_channel_parachain_inherent_data(
			para_id: u32,
			relay_parent_number: u32,
		) -> ::xcm_emulator::ParachainInherentData {
			use ::xcm_emulator::cumulus_primitives_core::{relay_chain::HrmpChannelId, AbridgedHrmpChannel};
			let mut sproof = ::xcm_emulator::RelayStateSproofBuilder::default();
			sproof.para_id = para_id.into();
			let e_index = sproof.hrmp_egress_channel_index.get_or_insert_with(Vec::new);
			for recipient_para_id in
				::xcm_emulator::PARA_IDS.with(|b| b.borrow_mut().get_mut("PolkadotNetwork").unwrap().clone())
			{
				let recipient_para_id = ::xcm_emulator::ParaId::from(recipient_para_id);
				if let Err(idx) = e_index.binary_search(&recipient_para_id) {
					e_index.insert(idx, recipient_para_id);
				}
				sproof
					.hrmp_channels
					.entry(HrmpChannelId {
						sender: sproof.para_id,
						recipient: recipient_para_id,
					})
					.or_insert_with(|| AbridgedHrmpChannel {
						max_capacity: 1024,
						max_total_size: 1024 * 1024,
						max_message_size: 1024 * 1024,
						msg_count: 0,
						total_size: 0,
						mqc_head: Option::None,
					});
			}
			let (relay_storage_root, proof) = sproof.into_state_root_and_proof();
			::xcm_emulator::ParachainInherentData {
				validation_data: ::xcm_emulator::PersistedValidationData {
					parent_head: Default::default(),
					relay_parent_number,
					relay_parent_storage_root: relay_storage_root,
					max_pov_size: Default::default(),
				},
				relay_chain_state: proof,
				downward_messages: Default::default(),
				horizontal_messages: Default::default(),
			}
		}
	}
	impl ::xcm_emulator::NetworkComponent for Polkadot {
		type Network = PolkadotNetwork;
		fn network_name() -> &'static str {
			"PolkadotNetwork"
		}
	}
	impl Polkadot {
		pub fn child_location_of(id: ::xcm_emulator::ParaId) -> MultiLocation {
			(Ancestor(0), Parachain(id.into())).into()
		}
		pub fn account_id_of(seed: &str) -> ::xcm_emulator::AccountId {
			::xcm_emulator::get_account_id_from_seed::<sr25519::Public>(seed)
		}
		pub fn account_data_of(account: AccountId) -> ::xcm_emulator::AccountData<Balance> {
			Self::ext_wrapper(|| <Self as RelayChain>::System::account(account).data)
		}
		pub fn sovereign_account_id_of(location: ::xcm_emulator::MultiLocation) -> ::xcm_emulator::AccountId {
			<Self as RelayChain>::SovereignAccountOf::convert_location(&location).unwrap()
		}
		pub fn fund_accounts(accounts: Vec<(AccountId, Balance)>) {
			Self::ext_wrapper(|| {
				for account in accounts {
					let _ = <Self as RelayChain>::Balances::force_set_balance(
						<Self as RelayChain>::RuntimeOrigin::root(),
						account.0.into(),
						account.1.into(),
					);
				}
			});
		}
		pub fn events() -> Vec<<Self as RelayChain>::RuntimeEvent> {
			<Self as RelayChain>::System::events()
				.iter()
				.map(|record| record.event.clone())
				.collect()
		}
	}
	impl ::xcm_emulator::NetworkComponent for SpiritnetPolkadot {
		type Network = PolkadotNetwork;
		fn network_name() -> &'static str {
			"PolkadotNetwork"
		}
	}
	impl SpiritnetPolkadot {
		pub fn para_id() -> ::xcm_emulator::ParaId {
			Self::ext_wrapper(|| <Self as Parachain>::ParachainInfo::get())
		}
		pub fn parent_location() -> ::xcm_emulator::MultiLocation {
			(Parent).into()
		}
		pub fn sibling_location_of(para_id: ::xcm_emulator::ParaId) -> ::xcm_emulator::MultiLocation {
			(Parent, X1(Parachain(para_id.into()))).into()
		}
		pub fn account_id_of(seed: &str) -> ::xcm_emulator::AccountId {
			::xcm_emulator::get_account_id_from_seed::<sr25519::Public>(seed)
		}
		pub fn account_data_of(account: AccountId) -> ::xcm_emulator::AccountData<Balance> {
			Self::ext_wrapper(|| <Self as Parachain>::System::account(account).data)
		}
		pub fn sovereign_account_id_of(location: ::xcm_emulator::MultiLocation) -> ::xcm_emulator::AccountId {
			<Self as Parachain>::LocationToAccountId::convert_location(&location).unwrap()
		}
		pub fn fund_accounts(accounts: Vec<(AccountId, Balance)>) {
			Self::ext_wrapper(|| {
				for account in accounts {
					let _ = <Self as Parachain>::Balances::force_set_balance(
						<Self as Parachain>::RuntimeOrigin::root(),
						account.0.into(),
						account.1.into(),
					);
				}
			});
		}
		pub fn events() -> Vec<<Self as Parachain>::RuntimeEvent> {
			<Self as Parachain>::System::events()
				.iter()
				.map(|record| record.event.clone())
				.collect()
		}
		fn prepare_for_xcmp() {
			use ::xcm_emulator::{Network, NetworkComponent};
			let para_id = Self::para_id();
			<Self as TestExt>::ext_wrapper(|| {
				use ::xcm_emulator::{Get, Hooks};
				let block_number = <Self as Parachain>::System::block_number();
				let _ = <Self as Parachain>::ParachainSystem::set_validation_data(
					<Self as Parachain>::RuntimeOrigin::none(),
					<Self as NetworkComponent>::Network::hrmp_channel_parachain_inherent_data(para_id.into(), 1),
				);
				<Self as Parachain>::ParachainSystem::on_initialize(block_number);
			});
		}
	}
	impl ::xcm_emulator::NetworkComponent for AssetHubPolkadot {
		type Network = PolkadotNetwork;
		fn network_name() -> &'static str {
			"PolkadotNetwork"
		}
	}
	impl AssetHubPolkadot {
		pub fn para_id() -> ::xcm_emulator::ParaId {
			Self::ext_wrapper(|| <Self as Parachain>::ParachainInfo::get())
		}
		pub fn parent_location() -> ::xcm_emulator::MultiLocation {
			(Parent).into()
		}
		pub fn sibling_location_of(para_id: ::xcm_emulator::ParaId) -> ::xcm_emulator::MultiLocation {
			(Parent, X1(Parachain(para_id.into()))).into()
		}
		pub fn account_id_of(seed: &str) -> ::xcm_emulator::AccountId {
			::xcm_emulator::get_account_id_from_seed::<sr25519::Public>(seed)
		}
		pub fn account_data_of(account: AccountId) -> ::xcm_emulator::AccountData<Balance> {
			Self::ext_wrapper(|| <Self as Parachain>::System::account(account).data)
		}
		pub fn sovereign_account_id_of(location: ::xcm_emulator::MultiLocation) -> ::xcm_emulator::AccountId {
			<Self as Parachain>::LocationToAccountId::convert_location(&location).unwrap()
		}
		pub fn fund_accounts(accounts: Vec<(AccountId, Balance)>) {
			Self::ext_wrapper(|| {
				for account in accounts {
					let _ = <Self as Parachain>::Balances::force_set_balance(
						<Self as Parachain>::RuntimeOrigin::root(),
						account.0.into(),
						account.1.into(),
					);
				}
			});
		}
		pub fn events() -> Vec<<Self as Parachain>::RuntimeEvent> {
			<Self as Parachain>::System::events()
				.iter()
				.map(|record| record.event.clone())
				.collect()
		}
		fn prepare_for_xcmp() {
			use ::xcm_emulator::{Network, NetworkComponent};
			let para_id = Self::para_id();
			<Self as TestExt>::ext_wrapper(|| {
				use ::xcm_emulator::{Get, Hooks};
				let block_number = <Self as Parachain>::System::block_number();
				let _ = <Self as Parachain>::ParachainSystem::set_validation_data(
					<Self as Parachain>::RuntimeOrigin::none(),
					<Self as NetworkComponent>::Network::hrmp_channel_parachain_inherent_data(para_id.into(), 1),
				);
				<Self as Parachain>::ParachainSystem::on_initialize(block_number);
			});
		}
	}
	extern crate test;
	#[cfg(test)]
	#[rustc_test_marker = "xcm_config::tests::test_reserve_asset_transfer_from_regular_account_to_relay"]
	pub const test_reserve_asset_transfer_from_regular_account_to_relay: test::TestDescAndFn = test::TestDescAndFn {
		desc: test::TestDesc {
			name: test::StaticTestName("xcm_config::tests::test_reserve_asset_transfer_from_regular_account_to_relay"),
			ignore: false,
			ignore_message: ::core::option::Option::None,
			source_file: "runtimes/spiritnet/src/xcm_config/tests/mod.rs",
			start_line: 60usize,
			start_col: 4usize,
			end_line: 60usize,
			end_col: 61usize,
			compile_fail: false,
			no_run: false,
			should_panic: test::ShouldPanic::No,
			test_type: test::TestType::UnitTest,
		},
		testfn: test::StaticTestFn(|| {
			test::assert_test_result(test_reserve_asset_transfer_from_regular_account_to_relay())
		}),
	};
	/// Test that a reserved transfer to the relaychain is failing. We don't want to
	/// allow transfers to the relaychain since the funds might be lost.
	fn test_reserve_asset_transfer_from_regular_account_to_relay() {
		PolkadotNetwork::reset();
		let alice_account_id_on_peregrine = get_account_id_from_seed::<sr25519::Public>(ALICE);
		SpiritnetPolkadot::execute_with(|| {
			let is = SpiritnetXcm::limited_reserve_transfer_assets(
				RawOrigin::Signed(alice_account_id_on_peregrine.clone()).into(),
				Box::new(Parent.into()),
				Box::new(
					X1(AccountId32 {
						network: None,
						id: alice_account_id_on_peregrine.into(),
					})
					.into(),
				),
				Box::new((Here, 1_000_000).into()),
				0,
				WeightLimit::Unlimited,
			);
			match is {
				Ok(_) => (),
				_ => {
					if !false {
						{
							::core::panicking::panic_fmt(format_args!("Expected Ok(_). Got {0:#?}", is));
						}
					}
				}
			};
			if !match SpiritnetSystem::events()
				.first()
				.expect("An event should be emitted when sending an XCM message.")
				.event
			{
				SpiritnetRuntimeEvent::PolkadotXcm(pallet_xcm::Event::Attempted {
					outcome: xcm::latest::Outcome::Error(xcm::latest::Error::Barrier),
				}) => true,
				_ => false,
			} {
				::core::panicking::panic("assertion failed: matches!(SpiritnetSystem::events().first().expect(\"An event should be emitted when sending an XCM message.\").event,\n    SpiritnetRuntimeEvent::PolkadotXcm(pallet_xcm::Event::Attempted\n    { outcome: xcm::latest::Outcome::Error(xcm::latest::Error::Barrier) }))")
			};
		});
		Polkadot::execute_with(|| {
			match (&PolkadotSystem::events().len(), &0) {
				(left_val, right_val) => {
					if !(*left_val == *right_val) {
						let kind = ::core::panicking::AssertKind::Eq;
						::core::panicking::assert_failed(kind, &*left_val, &*right_val, ::core::option::Option::None);
					}
				}
			};
		})
	}
	extern crate test;
	#[cfg(test)]
	#[rustc_test_marker = "xcm_config::tests::test_reserve_asset_transfer_from_regular_account_to_asset_hub"]
	pub const test_reserve_asset_transfer_from_regular_account_to_asset_hub: test::TestDescAndFn =
		test::TestDescAndFn {
			desc: test::TestDesc {
				name: test::StaticTestName(
					"xcm_config::tests::test_reserve_asset_transfer_from_regular_account_to_asset_hub",
				),
				ignore: false,
				ignore_message: ::core::option::Option::None,
				source_file: "runtimes/spiritnet/src/xcm_config/tests/mod.rs",
				start_line: 99usize,
				start_col: 4usize,
				end_line: 99usize,
				end_col: 65usize,
				compile_fail: false,
				no_run: false,
				should_panic: test::ShouldPanic::No,
				test_type: test::TestType::UnitTest,
			},
			testfn: test::StaticTestFn(|| {
				test::assert_test_result(test_reserve_asset_transfer_from_regular_account_to_asset_hub())
			}),
		};
	/// Test that a reserved transfer to the relaychain is failing. We don't want to
	/// allow transfers to the relaychain since the funds might be lost.
	fn test_reserve_asset_transfer_from_regular_account_to_asset_hub() {
		PolkadotNetwork::reset();
		let alice_account_id = get_account_id_from_seed::<sr25519::Public>(ALICE);
		let bob_account_id = get_account_id_from_seed::<sr25519::Public>(BOB);
		SpiritnetPolkadot::execute_with(|| {
			let is = SpiritnetXcm::limited_reserve_transfer_assets(
				RawOrigin::Signed(alice_account_id.clone()).into(),
				Box::new(ParentThen(Junctions::X1(Junction::Parachain(asset_hub_polkadot::PARA_ID))).into()),
				Box::new(
					X1(AccountId32 {
						network: None,
						id: bob_account_id.into(),
					})
					.into(),
				),
				Box::new((Here, 1000 * EXISTENTIAL_DEPOSIT).into()),
				0,
				WeightLimit::Unlimited,
			);
			match is {
				Ok(_) => (),
				_ => {
					if !false {
						{
							::core::panicking::panic_fmt(format_args!("Expected Ok(_). Got {0:#?}", is));
						}
					}
				}
			};
			if !match SpiritnetSystem::events()
				.last()
				.expect("An event should be emitted when sending an XCM message.")
				.event
			{
				SpiritnetRuntimeEvent::PolkadotXcm(pallet_xcm::Event::Attempted {
					outcome: xcm::latest::Outcome::Complete(_),
				}) => true,
				_ => false,
			} {
				{
					::core::panicking::panic_fmt(format_args!("Didn\'t match {0:?}", SpiritnetSystem::events().last()));
				}
			};
		});
		Polkadot::execute_with(|| {
			match (&PolkadotSystem::events().len(), &0) {
				(left_val, right_val) => {
					if !(*left_val == *right_val) {
						let kind = ::core::panicking::AssertKind::Eq;
						::core::panicking::assert_failed(kind, &*left_val, &*right_val, ::core::option::Option::None);
					}
				}
			};
		});
		AssetHubPolkadot::execute_with(|| {
			if !match AssetHubSystem::events()
				.last()
				.expect("An event should be emitted when sending an XCM message.")
				.event
			{
				AssetHubRuntimeEvent::XcmpQueue(XcmpQueueEvent::Fail { .. }) => true,
				_ => false,
			} {
				{
					::core::panicking::panic_fmt(format_args!("Didn\'t match {0:?}", AssetHubSystem::events().last()));
				}
			};
		});
	}
	extern crate test;
	#[cfg(test)]
	#[rustc_test_marker = "xcm_config::tests::test_teleport_asset_from_regular_account_to_asset_hub"]
	pub const test_teleport_asset_from_regular_account_to_asset_hub: test::TestDescAndFn = test::TestDescAndFn {
		desc: test::TestDesc {
			name: test::StaticTestName("xcm_config::tests::test_teleport_asset_from_regular_account_to_asset_hub"),
			ignore: false,
			ignore_message: ::core::option::Option::None,
			source_file: "runtimes/spiritnet/src/xcm_config/tests/mod.rs",
			start_line: 156usize,
			start_col: 4usize,
			end_line: 156usize,
			end_col: 57usize,
			compile_fail: false,
			no_run: false,
			should_panic: test::ShouldPanic::No,
			test_type: test::TestType::UnitTest,
		},
		testfn: test::StaticTestFn(
			|| test::assert_test_result(test_teleport_asset_from_regular_account_to_asset_hub()),
		),
	};
	fn test_teleport_asset_from_regular_account_to_asset_hub() {
		PolkadotNetwork::reset();
		let alice_account_id = get_account_id_from_seed::<sr25519::Public>(ALICE);
		let bob_account_id = get_account_id_from_seed::<sr25519::Public>(BOB);
		asset_hub_polkadot::force_create_asset_call(
			ParentThen(Junctions::X1(Junction::Parachain(PARA_ID))).into(),
			alice_account_id.clone(),
			true,
			0,
		);
		SpiritnetPolkadot::execute_with(|| {
			match (
				&SpiritnetXcm::limited_teleport_assets(
					RawOrigin::Signed(alice_account_id.clone()).into(),
					Box::new(ParentThen(Junctions::X1(Junction::Parachain(asset_hub_polkadot::PARA_ID))).into()),
					Box::new(
						X1(AccountId32 {
							network: None,
							id: bob_account_id.into(),
						})
						.into(),
					),
					Box::new((Here, 1000 * EXISTENTIAL_DEPOSIT).into()),
					0,
					WeightLimit::Unlimited,
				),
				&Err(DispatchError::Module(ModuleError {
					index: 83,
					error: [2, 0, 0, 0],
					message: Some("Filtered"),
				})
				.into()),
			) {
				(left_val, right_val) => {
					if !(*left_val == *right_val) {
						let kind = ::core::panicking::AssertKind::Eq;
						::core::panicking::assert_failed(kind, &*left_val, &*right_val, ::core::option::Option::None);
					}
				}
			};
		});
		Polkadot::execute_with(|| {
			match (&PolkadotSystem::events().len(), &0) {
				(left_val, right_val) => {
					if !(*left_val == *right_val) {
						let kind = ::core::panicking::AssertKind::Eq;
						::core::panicking::assert_failed(kind, &*left_val, &*right_val, ::core::option::Option::None);
					}
				}
			};
		});
		AssetHubPolkadot::execute_with(|| {
			match (&AssetHubSystem::events().len(), &0) {
				(left_val, right_val) => {
					if !(*left_val == *right_val) {
						let kind = ::core::panicking::AssertKind::Eq;
						::core::panicking::assert_failed(kind, &*left_val, &*right_val, ::core::option::Option::None);
					}
				}
			};
		});
	}
}
