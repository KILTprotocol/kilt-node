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
use sp_runtime::{Perbill, Perquintill};

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

pub const MIN_VESTED_TRANSFER_AMOUNT: Balance = 100 * MILLI_KILT;
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

/// Inflation configuration which is used at genesis
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

/// Copied from Kusama & Polkadot runtime
pub const MAX_VESTING_SCHEDULES: u32 = 28;

/// Calculate the storage deposit based on the number of storage items and the
/// combined byte size of those items.
pub const fn deposit(items: u32, bytes: u32) -> Balance {
	items as Balance * 63 * MILLI_KILT + (bytes as Balance) * 50 * MICRO_KILT
}

/// The size of an index in the index pallet.
/// The size is checked in the runtime by a test.
pub const MAX_INDICES_BYTE_LENGTH: u32 = 49;

parameter_types! {
	pub const ByteDeposit: Balance = deposit(0, 1);
	pub const IndicesDeposit: Balance = deposit(1, MAX_INDICES_BYTE_LENGTH);
}

pub mod attestation {
	use super::*;

	/// The size is checked in the runtime by a test.
	pub const MAX_ATTESTATION_BYTE_LENGTH: u32 = 178;
	pub const ATTESTATION_DEPOSIT: Balance = deposit(2, MAX_ATTESTATION_BYTE_LENGTH);
}

pub mod delegation {
	use super::*;

	pub const DELEGATION_DEPOSIT: Balance = KILT;
	pub const MAX_SIGNATURE_BYTE_LENGTH: u16 = 64;
	pub const MAX_PARENT_CHECKS: u32 = 5;
	pub const MAX_REVOCATIONS: u32 = 5;
	pub const MAX_REMOVALS: u32 = MAX_REVOCATIONS;
	pub const MAX_CHILDREN: u32 = 1000;
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
}

pub mod did {
	use super::*;

	/// The size is checked in the runtime by a test.
	pub const MAX_DID_BYTE_LENGTH: u32 = 7418;

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
	pub const MAX_SERVICE_URL_LENGTH: u32 = 100;
	pub const MAX_NUMBER_OF_URLS_PER_SERVICE: u32 = 1;
}

pub mod did_lookup {
	use super::*;

	/// The size is checked in the runtime by a test.
	pub const MAX_CONNECTION_BYTE_LENGTH: u32 = 80;
	pub const DID_CONNECTION_DEPOSIT: Balance = deposit(1, MAX_CONNECTION_BYTE_LENGTH);
}

pub mod treasury {
	use super::*;

	pub const INITIAL_PERIOD_LENGTH: BlockNumber = BLOCKS_PER_YEAR.saturating_mul(5);
	const YEARLY_REWARD: Balance = 2_000_000u128 * KILT;
	pub const INITIAL_PERIOD_REWARD_PER_BLOCK: Balance = YEARLY_REWARD / (BLOCKS_PER_YEAR as Balance);
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
}

pub mod preimage {
	use super::*;
	parameter_types! {
		pub const PreimageMaxSize: u32 = 4096 * 1024;
		pub const PreimageBaseDeposit: Balance = deposit(2, 64);
	}
}
