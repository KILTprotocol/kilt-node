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

use frame_support::{parameter_types, traits::AsEnsureOriginWithArg};
use frame_system::{pallet_prelude::BlockNumberFor, EnsureRoot, EnsureSigned};
use kilt_support::traits::InspectMetadata;
use pallet_asset_switch::xcm::{AccountId32ToAccountId32JunctionConverter, MatchesSwitchPairXcmFeeFungibleAsset};
use pallet_deposit_storage::{DepositKeyOf, PalletDepositStorageReason};
use runtime_common::{
	asset_switch::{hooks::RestrictSwitchDestinationToSelf, EnsureRootAsTreasury},
	bonded_coins::{
		hooks::NextAssetIdGenerator, AssetId, FixedPoint, FixedPointInput,
		NativeAndForeignAssets as NativeAndForeignAssetsType, TargetFromLeft,
	},
	deposits::DepositNamespace,
	AccountId, Balance, SendDustAndFeesToTreasury,
};
use sp_core::{crypto::ByteArray, ConstU128, ConstU32, ConstU8};
use sp_runtime::AccountId32;
use sp_std::vec::Vec;
use xcm::v4::{Junctions, Location};
use xcm_builder::{FungiblesAdapter, NoChecking};

use crate::{
	constants, weights,
	xcm::{LocationToAccountIdConverter, UniversalLocation, XcmRouter},
	Balances, BondedCurrencies, BondedFungibles, DepositStorage, Fungibles, PolkadotXcm, Runtime, RuntimeEvent,
	RuntimeFreezeReason, RuntimeHoldReason,
};

pub(crate) mod credential;
pub(crate) mod did;
pub(crate) use did::UniqueLinkingDeployment;
pub use did::{DotName, Web3Name};
pub(crate) mod dip;
pub use dip::{DipProofError, DipProofRequest};

impl parachain_staking::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type CurrencyBalance = Balance;
	type FreezeIdentifier = RuntimeFreezeReason;
	type MinBlocksPerRound = constants::staking::MinBlocksPerRound;
	type DefaultBlocksPerRound = constants::staking::DefaultBlocksPerRound;
	type StakeDuration = constants::staking::StakeDuration;
	type ExitQueueDelay = constants::staking::ExitQueueDelay;
	type MinCollators = constants::staking::MinCollators;
	type MinRequiredCollators = constants::staking::MinRequiredCollators;
	type MaxDelegationsPerRound = constants::staking::MaxDelegationsPerRound;
	type MaxDelegatorsPerCollator = constants::staking::MaxDelegatorsPerCollator;
	type MinCollatorStake = constants::staking::MinCollatorStake;
	type MinCollatorCandidateStake = constants::staking::MinCollatorStake;
	type MaxTopCandidates = constants::staking::MaxCollatorCandidates;
	type MinDelegatorStake = constants::staking::MinDelegatorStake;
	type MaxUnstakeRequests = constants::staking::MaxUnstakeRequests;
	type NetworkRewardRate = constants::staking::NetworkRewardRate;
	type NetworkRewardStart = constants::staking::NetworkRewardStart;
	type NetworkRewardBeneficiary = SendDustAndFeesToTreasury<Runtime>;
	type WeightInfo = weights::parachain_staking::WeightInfo<Runtime>;

	const BLOCKS_PER_YEAR: BlockNumberFor<Self> = constants::BLOCKS_PER_YEAR;
}

impl pallet_inflation::Config for Runtime {
	type Currency = Balances;
	type InitialPeriodLength = constants::treasury::InitialPeriodLength;
	type InitialPeriodReward = constants::treasury::InitialPeriodReward;
	type Beneficiary = SendDustAndFeesToTreasury<Runtime>;
	type WeightInfo = weights::pallet_inflation::WeightInfo<Runtime>;
}

parameter_types! {
	pub CheckingAccount: AccountId = PolkadotXcm::check_account();
}

pub(crate) type KiltToEKiltSwitchPallet = pallet_asset_switch::Instance1;
impl pallet_asset_switch::Config<KiltToEKiltSwitchPallet> for Runtime {
	type AccountIdConverter = AccountId32ToAccountId32JunctionConverter;
	type AssetTransactor = FungiblesAdapter<
		Fungibles,
		MatchesSwitchPairXcmFeeFungibleAsset<Runtime, KiltToEKiltSwitchPallet>,
		LocationToAccountIdConverter,
		AccountId,
		NoChecking,
		CheckingAccount,
	>;
	type FeeOrigin = EnsureRoot<AccountId>;
	type LocalCurrency = Balances;
	type PauseOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type SubmitterOrigin = EnsureSigned<AccountId>;
	type SwitchHooks = RestrictSwitchDestinationToSelf;
	type SwitchOrigin = EnsureRoot<AccountId>;
	type UniversalLocation = UniversalLocation;
	type WeightInfo = weights::pallet_asset_switch::WeightInfo<Runtime>;
	type XcmRouter = XcmRouter;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = crate::benchmarks::asset_switch::CreateFungibleForAssetSwitchPool1;
}

parameter_types! {
	pub const NativeAsset: Location = Junctions::Here.into_location();
}

pub struct MetadataProvider;
impl InspectMetadata for MetadataProvider {
	fn decimals() -> u8 {
		15u8
	}
	fn name() -> Vec<u8> {
		b"KILT".to_vec()
	}
	fn symbol() -> Vec<u8> {
		b"PILT".to_vec()
	}
}

pub type NativeAndForeignAssets =
	NativeAndForeignAssetsType<Balances, Fungibles, TargetFromLeft<NativeAsset>, Location, AccountId, MetadataProvider>;

/// Wrapper around the `PalletDepositStorageReason` that returns a specific
/// `DepositNamespace` for the bonded coins deposits.
#[derive(Debug, Clone)]
pub struct BondedCoinsHoldReason(PalletDepositStorageReason<DepositNamespace, AccountId>);

impl From<AccountId32> for BondedCoinsHoldReason {
	fn from(value: AccountId32) -> Self {
		Self(PalletDepositStorageReason::new(DepositNamespace::BondedTokens, value))
	}
}

impl From<BondedCoinsHoldReason> for RuntimeHoldReason {
	fn from(value: BondedCoinsHoldReason) -> Self {
		pallet_deposit_storage::HoldReason::from(value.0).into()
	}
}

pub struct LocalHoldReason(DepositKeyOf<Runtime>);

impl TryFrom<AccountId32> for LocalHoldReason {
	type Error = ();

	fn try_from(value: AccountId32) -> Result<Self, Self::Error> {
		DepositKeyOf::<Runtime>::try_from(value.to_raw_vec())
			.map(Self)
			.map_err(|_| ())
	}
}

impl From<LocalHoldReason> for PalletDepositStorageReason<DepositNamespace, DepositKeyOf<Runtime>> {
	fn from(value: LocalHoldReason) -> Self {
		Self::new(DepositNamespace::BondedTokens, value.0)
	}
}

impl pallet_bonded_coins::Config for Runtime {
	type BaseDeposit = ConstU128<{ constants::bonded_coins::BASE_DEPOSIT }>;
	type Collaterals = NativeAndForeignAssets;
	type CurveParameterInput = FixedPointInput;
	type CurveParameterType = FixedPoint;
	type DefaultOrigin = EnsureSigned<AccountId>;
	type DepositCurrency = DepositStorage;
	type DepositPerCurrency = ConstU128<{ constants::bonded_coins::DEPOSIT_PER_CURRENCY }>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type Fungibles = BondedFungibles;
	type MaxCurrenciesPerPool = ConstU32<{ constants::bonded_coins::MAX_CURRENCIES }>;
	type MaxDenomination = ConstU8<{ constants::bonded_coins::MAX_DENOMINATION }>;
	type MaxStringInputLength = ConstU32<{ constants::bonded_coins::MAX_STRING_LENGTH }>;
	type NextAssetIds = NextAssetIdGenerator<BondedCurrencies>;
	type PoolCreateOrigin = EnsureSigned<AccountId>;
	type PoolId = AccountId;
	type RuntimeEvent = RuntimeEvent;
	type HoldReason = LocalHoldReason;
	type RuntimeHoldReason = PalletDepositStorageReason<DepositNamespace, DepositKeyOf<Runtime>>;
	type WeightInfo = weights::pallet_bonded_coins::WeightInfo<Runtime>;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = crate::benchmarks::bonded_coins::BondedFungiblesBenchmarkHelper<Runtime>;
}

pub(crate) type BondedFungiblesInstance = pallet_assets::Instance2;
impl pallet_assets::Config<BondedFungiblesInstance> for Runtime {
	type ApprovalDeposit = constants::assets::ApprovalDeposit;
	type AssetAccountDeposit = constants::assets::AssetAccountDeposit;
	type AssetDeposit = constants::assets::AssetDeposit;
	type AssetId = AssetId;
	type AssetIdParameter = AssetId;
	type Balance = Balance;
	type CallbackHandle = ();
	type CreateOrigin = AsEnsureOriginWithArg<EnsureRootAsTreasury<Runtime>>;
	type Currency = Balances;
	type Extra = ();
	type ForceOrigin = EnsureRoot<AccountId>;
	type Freezer = ();
	type MetadataDepositBase = constants::assets::MetaDepositBase;
	type MetadataDepositPerByte = constants::assets::MetaDepositPerByte;
	type RemoveItemsLimit = constants::assets::RemoveItemsLimit;
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = constants::assets::StringLimit;
	type WeightInfo = weights::pallet_bonded_assets::WeightInfo<Runtime>;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}
