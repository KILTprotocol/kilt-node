// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

use frame_support::parameter_types;
use frame_system::{pallet_prelude::BlockNumberFor, EnsureRoot, EnsureSigned};
use pallet_asset_switch::xcm::{AccountId32ToAccountId32JunctionConverter, MatchesSwitchPairXcmFeeFungibleAsset};
use runtime_common::{asset_switch::hooks::RestrictSwitchDestinationToSelf, AccountId, Balance};
use xcm_builder::{FungiblesAdapter, NoChecking};

use crate::{
	constants, weights,
	xcm::{LocationToAccountIdConverter, UniversalLocation, XcmRouter},
	Balances, Fungibles, PolkadotXcm, Runtime, RuntimeEvent, RuntimeFreezeReason,
};

pub(crate) mod credential;
pub(crate) mod did;
pub use did::Web3Name;
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
	type NetworkRewardBeneficiary = ();
	type WeightInfo = weights::parachain_staking::WeightInfo<Runtime>;

	const BLOCKS_PER_YEAR: BlockNumberFor<Self> = constants::BLOCKS_PER_YEAR;
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
