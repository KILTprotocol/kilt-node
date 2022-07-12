// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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

use frame_support::{
	parameter_types,
	weights::{constants::WEIGHT_PER_SECOND, Weight},
};
use sp_runtime::{Perbill, Percent, Perquintill};

use parachain_staking::InflationInfo;

use crate::{Balance, BlockNumber};

/// This determines the average expected block time that we are targetting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 12_000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;
// Julian year as Substrate handles it
pub const BLOCKS_PER_YEAR: BlockNumber = DAYS * 36525 / 100;

pub const MAX_COLLATOR_STAKE: Balance = 200_000 * KILT;

/// One KILT
pub const KILT: Balance = 10u128.pow(15);

/// 0.001 KILT
pub const MILLI_KILT: Balance = 10u128.pow(12);

/// 0.000_001 KILT
pub const MICRO_KILT: Balance = 10u128.pow(9);

// 1 in 4 blocks (on average, not counting collisions) will be primary babe
// blocks.
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

/// We assume that ~10% of the block weight is consumed by `on_initalize`
/// handlers. This is used to limit the maximal weight of a single extrinsic.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be
/// used by  Operational  extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 0.5 seconds of compute with a 12 second average block time.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;

pub const INFLATION_CONFIG: (Perquintill, Perquintill, Perquintill, Perquintill) = (
	// max collator staking rate
	Perquintill::from_percent(40),
	// collator reward rate
	Perquintill::from_percent(10),
	// max delegator staking rate
	Perquintill::from_percent(10),
	// delegator reward rate
	Perquintill::from_percent(8),
);

/// Inflation configuration which is used at genesis
pub fn kilt_inflation_config() -> InflationInfo {
	InflationInfo::new(
		BLOCKS_PER_YEAR,
		// max collator staking rate
		Perquintill::from_percent(40),
		// collator reward rate
		Perquintill::from_percent(10),
		// max delegator staking rate
		Perquintill::from_percent(10),
		// delegator reward rate
		Perquintill::from_percent(8),
	)
}

/// Calculate the storage deposit based on the number of storage items and the
/// combined byte size of those items.
pub const fn deposit(items: u32, bytes: u32) -> Balance {
	items as Balance * 56 * MILLI_KILT + (bytes as Balance) * 50 * MICRO_KILT
}

/// The size of an index in the index pallet.
/// The size is checked in the runtime by a test.
pub const MAX_INDICES_BYTE_LENGTH: u32 = 49;

/// Copied from Kusama & Polkadot runtime
pub const MAX_VESTING_SCHEDULES: u32 = 28;

parameter_types! {
	/// Vesting Pallet. Copied from Kusama & Polkadot runtime
	pub const MinVestedTransfer: Balance = 100 * MILLI_KILT;
	/// Deposits per byte
	pub const ByteDeposit: Balance = deposit(0, 1);
	/// Index Pallet. Deposit taken for an account index
	pub const IndicesDeposit: Balance = deposit(1, MAX_INDICES_BYTE_LENGTH);
	/// CType Pallet. Per byte fee for a ctype.
	pub const CtypeFee: Balance = MILLI_KILT;
}

pub mod attestation {
	use super::*;

	/// The size is checked in the runtime by a test.
	pub const MAX_ATTESTATION_BYTE_LENGTH: u32 = 179;
	pub const ATTESTATION_DEPOSIT: Balance = deposit(2, MAX_ATTESTATION_BYTE_LENGTH);

	parameter_types! {
		pub const MaxDelegatedAttestations: u32 = 1000;
		pub const AttestationDeposit: Balance = ATTESTATION_DEPOSIT;
	}
}

pub mod delegation {
	use super::*;

	pub const DELEGATION_DEPOSIT: Balance = KILT;
	pub const MAX_SIGNATURE_BYTE_LENGTH: u16 = 64;
	pub const MAX_PARENT_CHECKS: u32 = 5;
	pub const MAX_REVOCATIONS: u32 = 5;
	pub const MAX_REMOVALS: u32 = MAX_REVOCATIONS;
	pub const MAX_CHILDREN: u32 = 1000;

	parameter_types! {
		pub const MaxSignatureByteLength: u16 = MAX_SIGNATURE_BYTE_LENGTH;
		pub const MaxParentChecks: u32 = MAX_PARENT_CHECKS;
		pub const MaxRevocations: u32 = MAX_REVOCATIONS;
		pub const MaxRemovals: u32 = MAX_REMOVALS;
		#[derive(Clone)]
		pub const MaxChildren: u32 = MAX_CHILDREN;
		pub const DelegationDeposit: Balance = DELEGATION_DEPOSIT;
	}
}

pub mod staking {
	use super::*;

	/// Minimum round length is 1 hour (300 * 12 second block times)
	#[cfg(feature = "fast-gov")]
	pub const MIN_BLOCKS_PER_ROUND: BlockNumber = 10;
	#[cfg(not(feature = "fast-gov"))]
	pub const MIN_BLOCKS_PER_ROUND: BlockNumber = HOURS;

	#[cfg(feature = "fast-gov")]
	pub const DEFAULT_BLOCKS_PER_ROUND: BlockNumber = 20;
	#[cfg(not(feature = "fast-gov"))]
	pub const DEFAULT_BLOCKS_PER_ROUND: BlockNumber = 2 * HOURS;

	#[cfg(feature = "fast-gov")]
	pub const STAKE_DURATION: BlockNumber = 30;
	#[cfg(not(feature = "fast-gov"))]
	pub const STAKE_DURATION: BlockNumber = 7 * DAYS;

	#[cfg(feature = "fast-gov")]
	pub const MIN_COLLATORS: u32 = 4;
	#[cfg(not(feature = "fast-gov"))]
	pub const MIN_COLLATORS: u32 = 16;

	#[cfg(feature = "fast-gov")]
	pub const MAX_CANDIDATES: u32 = 16;
	#[cfg(not(feature = "fast-gov"))]
	pub const MAX_CANDIDATES: u32 = 75;

	pub const MAX_DELEGATORS_PER_COLLATOR: u32 = 35;
	pub const MIN_DELEGATOR_STAKE: Balance = 20 * KILT;

	pub const NETWORK_REWARD_RATE: Perquintill = Perquintill::from_percent(10);

	parameter_types! {
		/// Minimum round length is 1 hour
		pub const MinBlocksPerRound: BlockNumber = MIN_BLOCKS_PER_ROUND;
		/// Default length of a round/session is 2 hours
		pub const DefaultBlocksPerRound: BlockNumber = DEFAULT_BLOCKS_PER_ROUND;
		/// Unstaked balance can be unlocked after 7 days
		pub const StakeDuration: BlockNumber = STAKE_DURATION;
		/// Collator exit requests are delayed by 4 hours (2 rounds/sessions)
		pub const ExitQueueDelay: u32 = 2;
		/// Minimum 16 collators selected per round, default at genesis and minimum forever after
		pub const MinCollators: u32 = MIN_COLLATORS;
		/// At least 4 candidates which cannot leave the network if there are no other candidates.
		pub const MinRequiredCollators: u32 = 4;
		/// We only allow one delegation per round.
		pub const MaxDelegationsPerRound: u32 = 1;
		/// Maximum 25 delegators per collator at launch, might be increased later
		#[derive(Debug, PartialEq)]
		pub const MaxDelegatorsPerCollator: u32 = MAX_DELEGATORS_PER_COLLATOR;
		/// Maximum 1 collator per delegator at launch, will be increased later
		#[derive(Debug, PartialEq)]
		pub const MaxCollatorsPerDelegator: u32 = 1;
		/// Minimum stake required to be reserved to be a collator is 10_000
		pub const MinCollatorStake: Balance = 10_000 * KILT;
		/// Minimum stake required to be reserved to be a delegator is 1000
		pub const MinDelegatorStake: Balance = MIN_DELEGATOR_STAKE;
		/// Maximum number of collator candidates
		#[derive(Debug, PartialEq)]
		pub const MaxCollatorCandidates: u32 = MAX_CANDIDATES;
		/// Maximum number of concurrent requests to unlock unstaked balance
		pub const MaxUnstakeRequests: u32 = 10;
		/// The starting block number for the network rewards
		pub const NetworkRewardStart: BlockNumber = super::treasury::INITIAL_PERIOD_LENGTH;
		/// The rate in percent for the network rewards
		pub const NetworkRewardRate: Perquintill = NETWORK_REWARD_RATE;
	}
}

pub mod governance {
	use super::*;

	pub const MIN_DEPOSIT: Balance = KILT;

	#[cfg(feature = "fast-gov")]
	pub const LAUNCH_PERIOD: BlockNumber = 7 * MINUTES;
	#[cfg(not(feature = "fast-gov"))]
	pub const LAUNCH_PERIOD: BlockNumber = 7 * DAYS;

	#[cfg(feature = "fast-gov")]
	pub const VOTING_PERIOD: BlockNumber = 7 * MINUTES;
	#[cfg(not(feature = "fast-gov"))]
	pub const VOTING_PERIOD: BlockNumber = 7 * DAYS;

	#[cfg(feature = "fast-gov")]
	pub const FAST_TRACK_VOTING_PERIOD: BlockNumber = 3 * MINUTES;
	#[cfg(not(feature = "fast-gov"))]
	pub const FAST_TRACK_VOTING_PERIOD: BlockNumber = 3 * HOURS;

	#[cfg(feature = "fast-gov")]
	pub const ENACTMENT_PERIOD: BlockNumber = 8 * MINUTES;
	#[cfg(not(feature = "fast-gov"))]
	pub const ENACTMENT_PERIOD: BlockNumber = DAYS;

	#[cfg(feature = "fast-gov")]
	pub const COOLOFF_PERIOD: BlockNumber = 7 * MINUTES;
	#[cfg(not(feature = "fast-gov"))]
	pub const COOLOFF_PERIOD: BlockNumber = 7 * DAYS;

	#[cfg(feature = "fast-gov")]
	pub const SPEND_PERIOD: BlockNumber = 6 * MINUTES;
	#[cfg(not(feature = "fast-gov"))]
	pub const SPEND_PERIOD: BlockNumber = 6 * DAYS;

	#[cfg(feature = "fast-gov")]
	pub const ROTATION_PERIOD: BlockNumber = 80 * MINUTES;
	#[cfg(not(feature = "fast-gov"))]
	pub const ROTATION_PERIOD: BlockNumber = 80 * HOURS;

	#[cfg(feature = "fast-gov")]
	pub const TERM_DURATION: BlockNumber = 15 * MINUTES;
	#[cfg(not(feature = "fast-gov"))]
	pub const TERM_DURATION: BlockNumber = DAYS;

	#[cfg(feature = "fast-gov")]
	pub const COUNCIL_MOTION_DURATION: BlockNumber = 4 * MINUTES;
	#[cfg(not(feature = "fast-gov"))]
	pub const COUNCIL_MOTION_DURATION: BlockNumber = 3 * DAYS;

	#[cfg(feature = "fast-gov")]
	pub const TECHNICAL_MOTION_DURATION: BlockNumber = 4 * MINUTES;
	#[cfg(not(feature = "fast-gov"))]
	pub const TECHNICAL_MOTION_DURATION: BlockNumber = 3 * DAYS;

	parameter_types! {
		// Democracy Pallet
		pub const LaunchPeriod: BlockNumber = LAUNCH_PERIOD;
		pub const VotingPeriod: BlockNumber = VOTING_PERIOD;
		pub const FastTrackVotingPeriod: BlockNumber = FAST_TRACK_VOTING_PERIOD;
		pub const MinimumDeposit: Balance = MIN_DEPOSIT;
		pub const EnactmentPeriod: BlockNumber = ENACTMENT_PERIOD;
		pub const CooloffPeriod: BlockNumber = COOLOFF_PERIOD;
		// Council Pallet
		pub const CouncilMotionDuration: BlockNumber = COUNCIL_MOTION_DURATION;
		pub const CouncilMaxProposals: u32 = 100;
		pub const CouncilMaxMembers: u32 = 100;
		// Technical Committee
		pub const TechnicalMotionDuration: BlockNumber = TECHNICAL_MOTION_DURATION;
		pub const TechnicalMaxProposals: u32 = 100;
		pub const TechnicalMaxMembers: u32 = 100;
	}
}

pub mod did {
	use super::*;

	/// The size is checked in the runtime by a test.
	pub const MAX_DID_BYTE_LENGTH: u32 = 9918;

	pub const DID_DEPOSIT: Balance = deposit(2 + MAX_NUMBER_OF_SERVICES_PER_DID, MAX_DID_BYTE_LENGTH);
	pub const DID_FEE: Balance = 50 * MILLI_KILT;
	pub const MAX_KEY_AGREEMENT_KEYS: u32 = 10;
	pub const MAX_URL_LENGTH: u32 = 200;
	// This has been reduced from the previous 100, but it might still need
	// fine-tuning depending on our needs.
	pub const MAX_PUBLIC_KEYS_PER_DID: u32 = 20;
	// At most the max number of keys - 1 for authentication
	pub const MAX_TOTAL_KEY_AGREEMENT_KEYS: u32 = MAX_PUBLIC_KEYS_PER_DID - 1;
	pub const MAX_ENDPOINT_URLS_COUNT: u32 = 3;
	pub const MAX_BLOCKS_TX_VALIDITY: BlockNumber = HOURS;

	pub const MAX_NUMBER_OF_SERVICES_PER_DID: u32 = 25;
	pub const MAX_SERVICE_ID_LENGTH: u32 = 50;
	pub const MAX_SERVICE_TYPE_LENGTH: u32 = 50;
	pub const MAX_NUMBER_OF_TYPES_PER_SERVICE: u32 = 1;
	pub const MAX_SERVICE_URL_LENGTH: u32 = 200;
	pub const MAX_NUMBER_OF_URLS_PER_SERVICE: u32 = 1;

	parameter_types! {
		pub const MaxNewKeyAgreementKeys: u32 = MAX_KEY_AGREEMENT_KEYS;
		#[derive(Debug, Clone, PartialEq)]
		pub const MaxUrlLength: u32 = MAX_URL_LENGTH;
		pub const MaxPublicKeysPerDid: u32 = MAX_PUBLIC_KEYS_PER_DID;
		#[derive(Debug, Clone, PartialEq)]
		pub const MaxTotalKeyAgreementKeys: u32 = MAX_TOTAL_KEY_AGREEMENT_KEYS;
		#[derive(Debug, Clone, PartialEq)]
		pub const MaxEndpointUrlsCount: u32 = MAX_ENDPOINT_URLS_COUNT;
		// Standalone block time is half the duration of a parachain block.
		pub const MaxBlocksTxValidity: BlockNumber = MAX_BLOCKS_TX_VALIDITY;
		pub const DidDeposit: Balance = DID_DEPOSIT;
		pub const DidFee: Balance = DID_FEE;
		pub const MaxNumberOfServicesPerDid: u32 = MAX_NUMBER_OF_SERVICES_PER_DID;
		pub const MaxServiceIdLength: u32 = MAX_SERVICE_ID_LENGTH;
		pub const MaxServiceTypeLength: u32 = MAX_SERVICE_TYPE_LENGTH;
		pub const MaxServiceUrlLength: u32 = MAX_SERVICE_URL_LENGTH;
		pub const MaxNumberOfTypesPerService: u32 = MAX_NUMBER_OF_TYPES_PER_SERVICE;
		pub const MaxNumberOfUrlsPerService: u32 = MAX_NUMBER_OF_URLS_PER_SERVICE;
	}
}

pub mod did_lookup {
	use super::*;

	/// The size is checked in the runtime by a test.
	pub const MAX_CONNECTION_BYTE_LENGTH: u32 = 80;
	pub const DID_CONNECTION_DEPOSIT: Balance = deposit(1, MAX_CONNECTION_BYTE_LENGTH);

	parameter_types! {
		pub const DidLookupDeposit: Balance = DID_CONNECTION_DEPOSIT;
	}
}

pub mod treasury {
	use super::*;

	pub const INITIAL_PERIOD_LENGTH: BlockNumber = BLOCKS_PER_YEAR.saturating_mul(5);
	const YEARLY_REWARD: Balance = 2_000_000u128 * KILT;
	pub const INITIAL_PERIOD_REWARD_PER_BLOCK: Balance = YEARLY_REWARD / (BLOCKS_PER_YEAR as Balance);

	parameter_types! {
		pub const InitialPeriodLength: BlockNumber = INITIAL_PERIOD_LENGTH;
		pub const InitialPeriodReward: Balance = INITIAL_PERIOD_REWARD_PER_BLOCK;
	}
}

pub mod proxy {
	use super::*;

	parameter_types! {
		// One storage item; key size 32, value size 8; .
		pub const ProxyDepositBase: Balance = deposit(1, 8);
		// Additional storage item size of 33 bytes.
		pub const ProxyDepositFactor: Balance = deposit(0, 33);
		pub const MaxProxies: u16 = 10;
		pub const AnnouncementDepositBase: Balance = deposit(1, 8);
		pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
		pub const MaxPending: u16 = 10;
	}
}

pub mod web3_names {
	use super::*;

	pub const MIN_LENGTH: u32 = 3;
	pub const MAX_LENGTH: u32 = 32;

	/// The size is checked in the runtime by a test.
	pub const MAX_NAME_BYTE_LENGTH: u32 = 121;
	pub const DEPOSIT: Balance = deposit(2, MAX_NAME_BYTE_LENGTH);

	parameter_types! {
		pub const Web3NameDeposit: Balance = DEPOSIT;
		pub const MinNameLength: u32 = MIN_LENGTH;
		pub const MaxNameLength: u32 = MAX_LENGTH;
	}
}

pub mod preimage {
	use super::*;
	parameter_types! {
		pub const PreimageMaxSize: u32 = 4096 * 1024;
		pub const PreimageBaseDeposit: Balance = deposit(2, 64);
	}
}

pub mod tips {
	use super::*;

	parameter_types! {
		pub const MaximumReasonLength: u32 = 16384;
		pub const TipCountdown: BlockNumber = DAYS;
		pub const TipFindersFee: Percent = Percent::from_percent(20);
		pub const TipReportDepositBase: Balance = deposit(1, 1);
	}
}

pub mod fee {
	use super::*;

	parameter_types! {
		/// This value increases the priority of `Operational` transactions by adding
		/// a "virtual tip" that's equal to the `OperationalFeeMultiplier * final_fee`.
		pub const OperationalFeeMultiplier: u8 = 5;
		pub const TransactionByteFee: Balance = MICRO_KILT;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	// TODO: static assert
	#[allow(clippy::assertions_on_constants)]
	#[test]
	fn blocks_per_year_saturation() {
		assert!(BLOCKS_PER_YEAR < u64::MAX);
	}
}
