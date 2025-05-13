#![allow(unused_imports)]
use frame_support::parameter_types;
use ismp::{host::StateMachine, module::IsmpModule, router::IsmpRouter};
use runtime_common::{AccountId, Balance};
use sp_core::{ConstU8, Get};
use sp_std::{boxed::Box, vec::Vec};
use xcm::v4::prelude::*;

use crate::{
	governance::{RootOrCollectiveProportion, TechnicalCollective},
	Balances, Fungibles, Ismp, IsmpParachain, Runtime, RuntimeEvent, Timestamp, Treasury,
};

parameter_types! {
	// The hyperbridge parachain on Polkadot
	pub const Coprocessor: Option<StateMachine> = Some(StateMachine::Polkadot(3367));
	pub const HostStateMachine: StateMachine = StateMachine::Polkadot(2086);
}

#[derive(Default)]
pub struct Router;

impl IsmpRouter for Router {
	fn module_for_id(&self, id: Vec<u8>) -> Result<Box<dyn IsmpModule>, anyhow::Error> {
		match id.as_slice() {
			//		pallet_hyperbridge::PALLET_HYPERBRIDGE_ID =>
			// Ok(Box::new(pallet_hyperbridge::Pallet::<Runtime>::default())),
			// id if TokenGateway::is_token_gateway(&id) => {
			// 	Ok(Box::new(pallet_token_gateway::Pallet::<Runtime>::default()))
			// }
			_ => Err(ismp::Error::ModuleNotFound(id))?,
		}
	}
}

impl pallet_ismp::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	// Modify the consensus client's permissions, for example, TechAdmin
	type AdminOrigin = RootOrCollectiveProportion<TechnicalCollective, 1, 2>;
	// The state machine identifier of the chain -- parachain id
	type HostStateMachine = HostStateMachine;
	type TimestampProvider = Timestamp;
	type Balance = Balance;
	type Currency = Balances;
	// Co-processor
	type Coprocessor = Coprocessor;
	// A tuple of types implementing the ConsensusClient interface, which defines
	// all consensus algorithms supported by this protocol deployment
	type ConsensusClients = (ismp_parachain::ParachainConsensusClient<Runtime, IsmpParachain>,);
	type Router = Router;

	type WeightProvider = ();
	type OffchainDB = ();
}

impl pallet_hyperbridge::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	// pallet-ismp implements the IsmpHost
	type IsmpHost = Ismp;
}

impl ismp_parachain::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type IsmpHost = Ismp;
	type WeightInfo = ();
}

pub struct AssetAdmin;
impl Get<AccountId> for AssetAdmin {
	fn get() -> AccountId {
		Treasury::account_id()
	}
}

parameter_types! {
	// The hyperbridge parachain on Polkadot
	pub const NativeAssetId: AssetId = AssetId::from(0);

}

impl pallet_token_gateway::Config for Runtime {
	// configure the runtime event
	type RuntimeEvent = RuntimeEvent;
	// Configured as Pallet Ismp
	type Dispatcher = Ismp;
	type Assets = Fungibles;
	type CreateOrigin = RootOrCollectiveProportion<TechnicalCollective, 2, 3>;
	// AssetAdmin account
	type AssetAdmin = AssetAdmin;
	type Decimals = ConstU8<15>;
	type NativeCurrency = Balances;
	type NativeAssetId = ();
	type EvmToSubstrate = ();
	type WeightInfo = ();
}
