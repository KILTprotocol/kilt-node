// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

use frame_support::weights::{constants::WEIGHT_PER_SECOND, Weight};
use sp_runtime::{Perbill, Perquintill};

use crate::*;

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

pub const MIN_VESTED_TRANSFER_AMOUNT: Balance = 1000 * KILT;
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

pub mod staking {
	#[cfg(not(feature = "fast-gov"))]
	use super::{DAYS, HOURS};
	use crate::BlockNumber;

	// Minimum round length is 1 hour (600 * 6 second block times)
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
}

pub mod governance {
	#[cfg(feature = "fast-gov")]
	use super::MINUTES;
	#[cfg(not(feature = "fast-gov"))]
	use super::{DAYS, HOURS};
	use crate::BlockNumber;

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
	pub const ENACTMENT_PERIOD: BlockNumber = 8 * DAYS;

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
	pub const CHALLENGE_PERIOD: BlockNumber = 7 * MINUTES;
	#[cfg(not(feature = "fast-gov"))]
	pub const CHALLENGE_PERIOD: BlockNumber = 7 * DAYS;

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
