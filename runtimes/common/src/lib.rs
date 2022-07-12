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

//! Primitives used by the Parachains Tick, Trick and Track.

#![cfg_attr(not(feature = "std"), no_std)]

use constants::{AVERAGE_ON_INITIALIZE_RATIO, MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO};
use fees::SplitFeesByRatio;

pub use sp_consensus_aura::sr25519::AuthorityId;

pub use opaque::*;

pub use frame_support::weights::constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight};
use frame_support::{
	parameter_types,
	traits::{Contains, ContainsLengthBound, Currency, Get, SortedMembers},
	weights::DispatchClass,
};
use frame_system::limits;
use pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};
use sp_runtime::{
	generic,
	traits::{IdentifyAccount, Verify},
	FixedPointNumber, MultiSignature, Perquintill, SaturatedConversion,
};
use sp_std::marker::PhantomData;

pub mod authorization;
pub mod constants;
pub mod fees;
pub mod migrations;
pub mod pallet_id;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarks;

/// Opaque types. These are used by the CLI to instantiate machinery that don't
/// need to know the specifics of the runtime. They can then be made to be
/// agnostic over specific formats of data like extrinsics, allowing for them to
/// continue syncing the network through upgrades to even the core data
/// structures.
pub mod opaque {
	use super::*;
	use sp_runtime::{generic, traits::BlakeTwo256};

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
}

/// An index to a block.
pub type BlockNumber = u64;

/// Alias to 512-bit hash when used in the context of a transaction signature on
/// the chain.
pub type Signature = MultiSignature;

/// Alias to the public key used for this chain, actually a `MultiSigner`. Like
/// the signature, this also isn't a fixed size when encoded, as different
/// cryptos have different size public keys.
pub type AccountPublic = <Signature as Verify>::Signer;

/// Alias to the opaque account ID type for this chain, actually a
/// `AccountId32`. This is always 32 bytes.
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of
/// them, but you never know...
pub type AccountIndex = u32;

/// Identifier for a chain. 32-bit should be plenty.
pub type ChainId = u32;

/// Balance of an account.
pub type Balance = u128;
pub type Amount = i128;

/// Index of a transaction in the chain.
pub type Index = u64;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Digest item type.
pub type DigestItem = generic::DigestItem;

/// A Kilt DID subject identifier.
pub type DidIdentifier = AccountId;

pub type NegativeImbalanceOf<T> =
	<pallet_balances::Pallet<T> as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

// Common constants used in all runtimes.
parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	/// The portion of the `NORMAL_DISPATCH_RATIO` that we adjust the fees with. Blocks filled less
	/// than this will decrease the weight and more will increase.
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	/// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
	/// change the fees more rapidly.
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(3, 100_000);
	/// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
	/// that combined with `AdjustmentVariable`, we can recover from the minimum.
	/// See `multiplier_can_grow_from_zero`.
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000u128);
	/// Maximum length of block. Up to 5MB.
	pub BlockLength: limits::BlockLength =
		limits::BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	/// Block weights base values and limits.
	pub BlockWeights: limits::BlockWeights = limits::BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have an extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT,
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();

	/// Fee split ratio between treasury and block author (order is important).
	pub const FeeSplitRatio: (u32, u32) = (50, 50);
}

/// Split the fees using a preconfigured Ratio
/// (`runtime_common::FeeSplitRatio`).
pub type FeeSplit<R, B1, B2> = SplitFeesByRatio<R, FeeSplitRatio, B1, B2>;

/// Parameterized slow adjusting fee updated based on
/// https://w3f-research.readthedocs.io/en/latest/polkadot/Token%20Economics.html#-2.-slow-adjusting-mechanism
pub type SlowAdjustingFeeUpdate<R> =
	TargetedFeeAdjustment<R, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;

pub struct Tippers<R, I>(PhantomData<R>, PhantomData<I>);
impl<R, I: 'static> ContainsLengthBound for Tippers<R, I>
where
	R: pallet_membership::Config<I>,
{
	fn max_len() -> usize {
		<R as pallet_membership::Config<I>>::MaxMembers::get().saturated_into()
	}

	fn min_len() -> usize {
		0
	}
}

impl<R, I: 'static> SortedMembers<R::AccountId> for Tippers<R, I>
where
	R: pallet_membership::Config<I>,
	pallet_membership::Pallet<R, I>: SortedMembers<R::AccountId> + Contains<R::AccountId>,
{
	fn sorted_members() -> sp_std::vec::Vec<R::AccountId> {
		pallet_membership::Pallet::<R, I>::sorted_members()
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn add(who: &R::AccountId) {
		pallet_membership::Members::<R, I>::mutate(|members| match members.binary_search_by(|m| m.cmp(who)) {
			Ok(_) => (),
			Err(pos) => members.insert(pos, who.clone()),
		})
	}
}
