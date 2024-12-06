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
use pallet_asset_switch::xcm::{AccountId32ToAccountId32JunctionConverter, MatchesSwitchPairXcmFeeFungibleAsset};
use runtime_common::{
	asset_switch::{hooks::RestrictSwitchDestinationToSelf, EnsureRootAsTreasury},
	bonded_coins::{
		hooks::NextAssetIdGenerator, AssetId, FixedPoint, FixedPointInput,
		NativeAndForeignAssets as NativeAndForeignAssetsType, TargetFromLeft,
	},
	AccountId, Balance, SendDustAndFeesToTreasury,
};
use sp_core::{ConstU128, ConstU32, ConstU8, Get};
use xcm::v4::{Junctions, Location};
use xcm_builder::{FungiblesAdapter, NoChecking};

use crate::{
	constants,
	system::CURRENCY_SYMBOL,
	weights,
	xcm::{LocationToAccountIdConverter, UniversalLocation, XcmRouter},
	Balances, BondedCurrencies, BondedFungibles, Fungibles, PolkadotXcm, Runtime, RuntimeEvent, RuntimeFreezeReason,
	RuntimeHoldReason,
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
impl Get<(u8, Vec<u8>, Vec<u8>)> for MetadataProvider {
	fn get() -> (u8, Vec<u8>, Vec<u8>) {
		(
			constants::DENOMINATION,
			constants::CURRENCY_NAME.to_vec(),
			CURRENCY_SYMBOL.to_vec(),
		)
	}
}

pub type NativeAndForeignAssets =
	NativeAndForeignAssetsType<Balances, Fungibles, TargetFromLeft<NativeAsset>, Location, AccountId>;

impl pallet_bonded_coins::Config for Runtime {
	type BaseDeposit = ConstU128<{ constants::bonded_coins::BASE_DEPOSIT }>;
	type Collaterals = NativeAndForeignAssets;
	type CurveParameterInput = FixedPointInput;
	type CurveParameterType = FixedPoint;
	type DefaultOrigin = EnsureSigned<AccountId>;
	type DepositCurrency = Balances;
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
	type RuntimeHoldReason = RuntimeHoldReason;
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
