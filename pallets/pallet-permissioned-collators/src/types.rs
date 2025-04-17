use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::{
	prelude::ops::{Add, Sub},
	TypeInfo,
};
use sp_runtime::{RuntimeDebug, Saturating};
use sp_staking::SessionIndex;

/// The current round index and transition information.
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct RoundInfo<BlockNumber> {
	/// Current round index.
	pub current: SessionIndex,
	/// The first block of the current round.
	pub first: BlockNumber,
	/// The length of the current round in blocks.
	pub length: BlockNumber,
}

impl<B> RoundInfo<B>
where
	B: Copy + Saturating + From<u32> + PartialOrd,
{
	pub const fn new(current: SessionIndex, first: B, length: B) -> RoundInfo<B> {
		RoundInfo { current, first, length }
	}

	/// Checks if the round should be updated.
	///
	/// The round should update if `self.length` or more blocks where produced
	/// after `self.first`.
	pub fn should_update(&self, now: B) -> bool {
		let l = now.saturating_sub(self.first);
		l >= self.length
	}

	/// Starts a new round.
	pub fn update(&mut self, now: B) {
		self.current = self.current.saturating_add(1u32);
		self.first = now;
	}
}

impl<B> Default for RoundInfo<B>
where
	B: Copy + Saturating + Add<Output = B> + Sub<Output = B> + From<u32> + PartialOrd,
{
	fn default() -> RoundInfo<B> {
		RoundInfo::new(0u32, 0u32.into(), 20.into())
	}
}
