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

mod peregrine;
mod relaychain;
mod utils;

use crate::{xcm_config::UniversalLocation as PeregrineUniversalLocation, PolkadotXcm as PeregrineXcm};
use frame_support::{assert_ok, traits::fungible::Inspect};
use frame_system::RawOrigin;
use parity_scale_codec::Encode;
use peregrine::{Runtime as PeregrineRuntime, System as PeregrineSystem};
use polkadot_primitives::{AccountId, Balance};
use polkadot_service::chain_spec::get_account_id_from_seed;
use relaychain::{
	set_free_balance as set_rococo_free_balance, LocationConverter as RococoLocationConverter,
	Runtime as RococoRuntime, RuntimeEvent as RococoRuntimeEvent, System as RococoSystem,
};
use rococo_runtime::xcm_config::UniversalLocation as RococoUniversalLocation;
use rococo_runtime_constants::currency::UNITS;
use runtime_common::constants::KILT;
use sp_core::{sr25519, Get};
use xcm::prelude::*;
use xcm_emulator::{decl_test_networks, BridgeMessageHandler, Parachain, RelayChain, TestExt};
use xcm_executor::traits::ConvertLocation;

decl_test_networks! {
	pub struct RococoNetwork {
		relay_chain = RococoRuntime,
		parachains = vec![
			PeregrineRuntime,
		],
		bridge = ()
	}
}

use crate::Balances;

#[test]
fn test_ump_message_from_regular_account() {
	env_logger::init();
	let rococo_universal_location = RococoUniversalLocation::get();

	let account_id_on_peregrine = get_account_id_from_seed::<sr25519::Public>("Alice");
	let account_id_universal_location = PeregrineRuntime::execute_with(|| {
		PeregrineUniversalLocation::get()
			.pushed_with(AccountId32 {
				network: None,
				id: account_id_on_peregrine.clone().into(),
			})
			.expect("Should not fail to create absolute account ID.")
	})
	.into_location();

	let message: Xcm<_> = vec![
		WithdrawAsset((Here, 100).into()),
		BuyExecution {
			fees: (Here, 100).into(),
			weight_limit: WeightLimit::Unlimited,
		},
	]
	.into();
	PeregrineRuntime::execute_with(|| {
		assert_ok!(PeregrineXcm::send(
			RawOrigin::Signed(account_id_on_peregrine).into(),
			Box::new(Parent.into()),
			Box::new(VersionedXcm::from(message)),
		));
	});
	// Regular parachain accounts cannot be translated to account IDs, hence the
	// conversion in `WithdrawAsset` should fail.
	RococoRuntime::execute_with(|| {
		assert_eq!(RococoSystem::events().len(), 1);
		assert!(matches!(
			RococoSystem::events().first().unwrap().event,
			RococoRuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed { success: false, .. })
		));
	})
}

#[test]
fn test_ump_message_from_parachain_account() {
	env_logger::init();
	let rococo_universal_location = RococoUniversalLocation::get();

	let peregrine_universal_location =
		PeregrineRuntime::execute_with(|| PeregrineUniversalLocation::get()).into_location();
	let peregrine_account_id_on_rococo = RococoLocationConverter::convert_location(&peregrine_universal_location)
		.expect("Should not fail to convert parachain location to Rococo account ID.");
	println!("Account ID: {:#?}", &peregrine_account_id_on_rococo);
	set_rococo_free_balance((peregrine_account_id_on_rococo, 200 * UNITS));
	let message = b"Hello!";
	let weight = <<rococo_runtime::Runtime as frame_system::Config>::SystemWeightInfo as frame_system::WeightInfo>::remark_with_event(message.len() as u32);

	let message: Xcm<_> = vec![
		WithdrawAsset((Here, 100 * UNITS).into()),
		BuyExecution {
			fees: (Here, 100 * UNITS).into(),
			weight_limit: WeightLimit::Unlimited,
		},
		// TODO: `Transact` gives a `!! ERROR: NoPermission` error.
		Transact {
			origin_kind: OriginKind::Native,
			require_weight_at_most: weight,
			call: RococoRuntime::execute_with(|| {
				rococo_runtime::RuntimeCall::System(frame_system::Call::remark_with_event {
					remark: message.to_vec(),
				})
				.encode()
				.into()
			}),
		},
		RefundSurplus,
		DepositAsset {
			assets: Wild(All),
			beneficiary: peregrine_universal_location,
		},
	]
	.into();
	PeregrineRuntime::execute_with(|| {
		assert_ok!(PeregrineXcm::send(
			RawOrigin::Root.into(),
			Box::new(Parent.into()),
			Box::new(VersionedXcm::from(message)),
		));
	});
	// // Regular parachain accounts cannot be translated to account IDs, hence
	// the // conversion in `WithdrawAsset` should fail.
	// RococoRuntime::execute_with(|| {
	// 	assert_eq!(RococoSystem::events().len(), 1);
	// 	assert!(matches!(
	// 		RococoSystem::events().first().unwrap().event,
	// 		RococoRuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed {
	// success: false, .. }) 	));
	// })
}
