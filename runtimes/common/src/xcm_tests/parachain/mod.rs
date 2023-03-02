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

use dip_support::location_conversion::ForeignChainAliasAccount;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{Everything, Nothing},
	weights::Weight,
};
use pallet_xcm::{TestWeightInfo, XcmPassthrough};
use sp_core::{ConstU32, ConstU64, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};
use sp_std::prelude::*;
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AllowUnpaidExecutionFrom, EnsureXcmOrigin, FixedWeightBounds, SignedAccountId32AsNative,
	SignedToAccountId32, SovereignSignedViaLocation,
};
use xcm_executor::XcmExecutor;

mod mock_dip;
mod mock_msg_queue;

parameter_types! {
	pub ExistentialDeposit: Balance = 1;
	pub const RelayNetworkId: Option<NetworkId> = None;
	pub const UnitWeightCost: Weight = Weight::from_parts(1, 1);
}

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	ReachableDest: Option<MultiLocation> = Some(Parent.into());
}

pub type XcmOriginToTransactDispatchOrigin<RuntimeOrigin, NetworkId> = (
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	SignedAccountId32AsNative<NetworkId, RuntimeOrigin>,
	XcmPassthrough<RuntimeOrigin>,
);

pub(super) type AccountId = AccountId32;
pub(super) type Balance = u128;

type Block<Runtime> = frame_system::mocking::MockBlock<Runtime>;
type Identifier = [u8; 4];
type IdentityProofOutput = [u8; 32];
type UncheckedExtrinsic<Runtime> = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type LocationToAccountId = ForeignChainAliasAccount<AccountId>;
type XcmRouter<MsgQueue> = super::ParachainXcmRouter<MsgQueue>;

pub(super) mod sender {
	use codec::Encode;
	use dip_sender::traits::{DefaultIdentityProofGenerator, DefaultIdentityProvider, TxBuilder};
	use dip_support::latest::IdentityProofAction;
	use xcm::DoubleEncoded;

	use crate::{
		dip::identity_dispatch::DidXcmV3ViaXcmPalletDispatcher,
		xcm_tests::parachain::mock_dip::{ReceiverParachainCalls, ReceiverParachainDipReceiverCalls},
	};

	use super::*;

	construct_runtime!(
		pub enum Runtime where
			Block = Block<Runtime>,
			NodeBlock = Block<Runtime>,
			UncheckedExtrinsic = UncheckedExtrinsic<Runtime>,
		{
			System: frame_system::{Pallet, Call, Storage, Config, Event<T>} = 1,
			Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 2,
			MsgQueue: mock_msg_queue::{Pallet, Storage, Event<T>} = 3,
			PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin} = 4,
			DipProvider: dip_sender::{Pallet, Call, Storage, Event<T>} = 5,
		}
	);

	parameter_types! {
		pub UniversalLocation: InteriorMultiLocation = Parachain(MsgQueue::parachain_id().into()).into();
	}

	impl frame_system::Config for Runtime {
		type AccountData = pallet_balances::AccountData<Balance>;
		type AccountId = AccountId;
		type BaseCallFilter = Everything;
		type BlockHashCount = ConstU64<250>;
		type BlockLength = ();
		type BlockNumber = u64;
		type BlockWeights = ();
		type DbWeight = ();
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type Header = Header;
		type Index = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type MaxConsumers = ConstU32<16>;
		type OnKilledAccount = ();
		type OnNewAccount = ();
		type OnSetCode = ();
		type PalletInfo = PalletInfo;
		type RuntimeCall = RuntimeCall;
		type RuntimeEvent = RuntimeEvent;
		type RuntimeOrigin = RuntimeOrigin;
		type SS58Prefix = ();
		type SystemWeightInfo = ();
		type Version = ();
	}

	impl pallet_balances::Config for Runtime {
		type AccountStore = System;
		type Balance = Balance;
		type DustRemoval = ();
		type ExistentialDeposit = ExistentialDeposit;
		type MaxLocks = ConstU32<50>;
		type MaxReserves = ConstU32<50>;
		type ReserveIdentifier = [u8; 8];
		type RuntimeEvent = RuntimeEvent;
		type WeightInfo = ();
	}

	pub struct XcmConfig;
	impl xcm_executor::Config for XcmConfig {
		type AssetClaims = ();
		type AssetExchanger = ();
		type AssetLocker = ();
		type AssetTransactor = ();
		type AssetTrap = ();
		type Barrier = AllowUnpaidExecutionFrom<Everything>;
		type CallDispatcher = RuntimeCall;
		type FeeManager = ();
		type IsReserve = ();
		type IsTeleporter = ();
		type MaxAssetsIntoHolding = ConstU32<64>;
		type MessageExporter = ();
		type OriginConverter = XcmOriginToTransactDispatchOrigin<RuntimeOrigin, RelayNetworkId>;
		type PalletInstancesInfo = ();
		type ResponseHandler = ();
		type RuntimeCall = RuntimeCall;
		type SafeCallFilter = Everything;
		type SubscriptionService = ();
		type Trader = ();
		type UniversalAliases = Nothing;
		type UniversalLocation = UniversalLocation;
		type XcmSender = XcmRouter<MsgQueue>;
		type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, ConstU32<100>>;
	}

	impl mock_msg_queue::Config for Runtime {
		type RuntimeEvent = RuntimeEvent;
		type XcmExecutor = XcmExecutor<XcmConfig>;
	}

	impl pallet_xcm::Config for Runtime {
		const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;

		type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
		type Currency = Balances;
		type CurrencyMatcher = ();
		type ExecuteXcmOrigin =
			EnsureXcmOrigin<RuntimeOrigin, SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetworkId>>;
		type MaxLockers = ConstU32<8>;
		type RuntimeCall = RuntimeCall;
		type RuntimeEvent = RuntimeEvent;
		type RuntimeOrigin = RuntimeOrigin;
		type SendXcmOrigin =
			EnsureXcmOrigin<RuntimeOrigin, SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetworkId>>;
		type SovereignAccountOf = AccountId32Aliases<RelayNetworkId, AccountId>;
		type TrustedLockers = ();
		type UniversalLocation = UniversalLocation;
		type XcmExecuteFilter = Everything;
		type XcmExecutor = XcmExecutor<XcmConfig>;
		type XcmReserveTransferFilter = Nothing;
		type XcmRouter = XcmRouter<MsgQueue>;
		type XcmTeleportFilter = Nothing;
		type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, ConstU32<100>>;
		type WeightInfo = TestWeightInfo;

		#[cfg(feature = "runtime-benchmarks")]
		type ReachableDest = ReachableDest;
	}

	pub struct ReceiverParachainTxBuilder;
	impl
		TxBuilder<
			<receiver::Runtime as dip_receiver::Config>::Identifier,
			<receiver::Runtime as dip_receiver::Config>::Proof,
		> for ReceiverParachainTxBuilder
	{
		type Error = ();

		fn build(
			_dest: MultiLocation,
			action: IdentityProofAction<
				<receiver::Runtime as dip_receiver::Config>::Identifier,
				<receiver::Runtime as dip_receiver::Config>::Proof,
			>,
		) -> Result<DoubleEncoded<()>, Self::Error> {
			let double_encoded: DoubleEncoded<()> =
				ReceiverParachainCalls::DipReceiver(ReceiverParachainDipReceiverCalls::ProcessIdentityAction(action))
					.encode()
					.into();
			println!("ReceiverParachainTxBuilder::build 1");
			Ok(double_encoded)
		}
	}

	impl dip_sender::Config for Runtime {
		type Identifier = Identifier;
		type Identity = u32;
		type IdentityProofDispatcher = DidXcmV3ViaXcmPalletDispatcher<
			Runtime,
			Identifier,
			IdentityProofOutput,
			SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetworkId>,
		>;
		type IdentityProofGenerator = DefaultIdentityProofGenerator;
		type IdentityProvider = DefaultIdentityProvider;
		type ProofOutput = IdentityProofOutput;
		type RuntimeEvent = RuntimeEvent;
		type TxBuilder = ReceiverParachainTxBuilder;
	}
}

pub(super) mod receiver {
	use super::*;

	construct_runtime!(
		pub enum Runtime where
			Block = Block<Runtime>,
			NodeBlock = Block<Runtime>,
			UncheckedExtrinsic = UncheckedExtrinsic<Runtime>,
		{
			System: frame_system::{Pallet, Call, Storage, Config, Event<T>} = 1,
			Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 2,
			MsgQueue: mock_msg_queue::{Pallet, Storage, Event<T>} = 3,
			PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin} = 4,
			DipReceiver: dip_receiver::{Pallet, Call, Storage, Event<T>} = 5,
		}
	);

	parameter_types! {
		pub UniversalLocation: InteriorMultiLocation = Parachain(MsgQueue::parachain_id().into()).into();
	}

	impl frame_system::Config for Runtime {
		type AccountData = pallet_balances::AccountData<Balance>;
		type AccountId = AccountId;
		type BaseCallFilter = Everything;
		type BlockHashCount = ConstU64<250>;
		type BlockLength = ();
		type BlockNumber = u64;
		type BlockWeights = ();
		type DbWeight = ();
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type Header = Header;
		type Index = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type MaxConsumers = ConstU32<16>;
		type OnKilledAccount = ();
		type OnNewAccount = ();
		type OnSetCode = ();
		type PalletInfo = PalletInfo;
		type RuntimeCall = RuntimeCall;
		type RuntimeEvent = RuntimeEvent;
		type RuntimeOrigin = RuntimeOrigin;
		type SS58Prefix = ();
		type SystemWeightInfo = ();
		type Version = ();
	}

	impl pallet_balances::Config for Runtime {
		type AccountStore = System;
		type Balance = Balance;
		type DustRemoval = ();
		type ExistentialDeposit = ExistentialDeposit;
		type MaxLocks = ConstU32<50>;
		type MaxReserves = ConstU32<50>;
		type ReserveIdentifier = [u8; 8];
		type RuntimeEvent = RuntimeEvent;
		type WeightInfo = ();
	}

	pub struct XcmConfig;
	impl xcm_executor::Config for XcmConfig {
		type AssetClaims = ();
		type AssetExchanger = ();
		type AssetLocker = ();
		type AssetTransactor = ();
		type AssetTrap = ();
		type Barrier = AllowUnpaidExecutionFrom<Everything>;
		type CallDispatcher = RuntimeCall;
		type FeeManager = ();
		type IsReserve = ();
		type IsTeleporter = ();
		type MaxAssetsIntoHolding = ConstU32<64>;
		type MessageExporter = ();
		type OriginConverter = XcmOriginToTransactDispatchOrigin<RuntimeOrigin, RelayNetworkId>;
		type PalletInstancesInfo = ();
		type ResponseHandler = ();
		type RuntimeCall = RuntimeCall;
		type SafeCallFilter = Everything;
		type SubscriptionService = ();
		type Trader = ();
		type UniversalAliases = Nothing;
		type UniversalLocation = UniversalLocation;
		type XcmSender = XcmRouter<MsgQueue>;
		type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, ConstU32<100>>;
	}

	impl mock_msg_queue::Config for Runtime {
		type RuntimeEvent = RuntimeEvent;
		type XcmExecutor = XcmExecutor<XcmConfig>;
	}

	impl pallet_xcm::Config for Runtime {
		const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;

		type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
		type Currency = Balances;
		type CurrencyMatcher = ();
		type ExecuteXcmOrigin =
			EnsureXcmOrigin<RuntimeOrigin, SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetworkId>>;
		type MaxLockers = ConstU32<8>;
		type RuntimeCall = RuntimeCall;
		type RuntimeEvent = RuntimeEvent;
		type RuntimeOrigin = RuntimeOrigin;
		type SendXcmOrigin =
			EnsureXcmOrigin<RuntimeOrigin, SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetworkId>>;
		type SovereignAccountOf = AccountId32Aliases<RelayNetworkId, AccountId>;
		type TrustedLockers = ();
		type UniversalLocation = UniversalLocation;
		type XcmExecuteFilter = Everything;
		type XcmExecutor = XcmExecutor<XcmConfig>;
		type XcmReserveTransferFilter = Nothing;
		type XcmRouter = XcmRouter<MsgQueue>;
		type XcmTeleportFilter = Nothing;
		type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, ConstU32<100>>;
		type WeightInfo = TestWeightInfo;

		#[cfg(feature = "runtime-benchmarks")]
		type ReachableDest = ReachableDest;
	}

	impl dip_receiver::Config for Runtime {
		type EnsureSourceXcmOrigin = <Self as pallet_xcm::Config>::ExecuteXcmOrigin;
		type Identifier = Identifier;
		type Proof = IdentityProofOutput;
		type RuntimeEvent = RuntimeEvent;
	}
}
