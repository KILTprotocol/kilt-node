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

use core::marker::PhantomData;
use frame_support::{log, match_types, parameter_types};
use polkadot_parachain::primitives::Sibling;
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AllowUnpaidExecutionFrom, CurrencyAdapter, IsConcrete, ParentIsPreset,
	SiblingParachainConvertsVia,
};
use xcm_executor::traits::ShouldExecute;

use crate::AccountId;

parameter_types! {
	// One XCM operation is 1_000_000_000 weight, almost certainly a conservative estimate.
	pub UnitWeightCost: u64 = 1_000_000_000;
	pub const MaxInstructions: u32 = 100;
}

match_types! {
	// The legislative of our parent (i.e. Polkadot majority vote for Spiritnet).
	pub type ParentLegislative: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: X1(Plurality { id: BodyId::Legislative, .. }) }
	};
}

// Note: This might move to polkadot's xcm module.
/// Deny executing the xcm message if it matches any of the Deny filter
/// regardless of anything else. If it passes the Deny and matches one of the
/// Allow cases, then it is let through.
pub struct DenyThenTry<Deny, Allow>(PhantomData<(Deny, Allow)>);

impl<Deny, Allow> ShouldExecute for DenyThenTry<Deny, Allow>
where
	Deny: ShouldExecute,
	Allow: ShouldExecute,
{
	fn should_execute<Call>(
		origin: &MultiLocation,
		message: &mut Xcm<Call>,
		max_weight: u64,
		weight_credit: &mut u64,
	) -> Result<(), ()> {
		Deny::should_execute(origin, message, max_weight, weight_credit)?;
		Allow::should_execute(origin, message, max_weight, weight_credit)
	}
}

/// Explicitly deny ReserveTransfer to the relay chain. Allow calls from the
/// relay chain governance.
pub type XcmBarrier = DenyThenTry<
	DenyReserveTransferToRelayChain,
	(
		// We don't allow anything from any sibling chain, therefore the following is not included here:
		// * TakeWeightCredit
		// * AllowTopLevelPaidExecutionFrom<Everything>

		// We allow everything from the relay chain if it was send by the relay chain legislative.
		// Since the relaychain doesn't own KILTs and missing fees shouldn't prevent calls from the relaychain
		// legislative, we allow unpaid execution.
		AllowUnpaidExecutionFrom<ParentLegislative>,
	),
>;

/// Reserved funds to the relay chain can't return. See https://github.com/paritytech/polkadot/issues/5233
pub struct DenyReserveTransferToRelayChain;
impl ShouldExecute for DenyReserveTransferToRelayChain {
	fn should_execute<Call>(
		origin: &MultiLocation,
		message: &mut Xcm<Call>,
		_max_weight: u64,
		_weight_credit: &mut u64,
	) -> Result<(), ()> {
		if message.0.iter().any(|inst| {
			matches!(
				inst,
				InitiateReserveWithdraw {
					reserve: MultiLocation {
						parents: 1,
						interior: Here
					},
					..
				} | DepositReserveAsset {
					dest: MultiLocation {
						parents: 1,
						interior: Here
					},
					..
				} | TransferReserveAsset {
					dest: MultiLocation {
						parents: 1,
						interior: Here
					},
					..
				}
			)
		}) {
			return Err(()); // Deny
		}

		// Allow reserve transfers to arrive from relay chain
		if matches!(
			origin,
			MultiLocation {
				parents: 1,
				interior: Here
			}
		) && message
			.0
			.iter()
			.any(|inst| matches!(inst, ReserveAssetDeposited { .. }))
		{
			log::warn!(
				target: "xcm::barriers",
				"Unexpected ReserveAssetDeposited from the relay chain",
			);
		}
		// Permit everything else
		Ok(())
	}
}

parameter_types! {
	pub const RelayLocation: MultiLocation = MultiLocation::parent();
	pub const HereLocation: MultiLocation = MultiLocation::here();
}

/// Type for specifying how a `MultiLocation` can be converted into an
/// `AccountId`. This is used when determining ownership of accounts for asset
/// transacting and when attempting to use XCM `Transact` in order to determine
/// the dispatch Origin.
pub type LocationToAccountId<RelayNetwork> = (
	// The parent (Relay-chain) origin converts to the parent `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
);

/// Means for transacting assets on this chain.
pub type LocalAssetTransactor<Currency, RelayNetwork> = CurrencyAdapter<
	// Use this currency:
	Currency,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<HereLocation>,
	// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
	LocationToAccountId<RelayNetwork>,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports.
	(),
>;
