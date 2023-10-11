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

use core::{marker::PhantomData, ops::ControlFlow};
use frame_support::{log, match_types, parameter_types, traits::ProcessMessageError, weights::Weight};
use polkadot_parachain::primitives::Sibling;
use xcm::latest::prelude::*;
use xcm_builder::{AccountId32Aliases, CurrencyAdapter, IsConcrete, ParentIsPreset, SiblingParachainConvertsVia};
use xcm_executor::traits::{Properties, ShouldExecute};

use crate::AccountId;

parameter_types! {
	// One XCM operation is 1_000_000_000 weight, almost certainly a conservative estimate.
	pub UnitWeightCost: Weight = Weight::from_parts(1_000_000_000, 64 * 1024);
	pub const MaxInstructions: u32 = 100;
	pub const MaxAssetsIntoHolding: u32 = 64;
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
		instructions: &mut [Instruction<Call>],
		max_weight: Weight,
		properties: &mut Properties,
	) -> Result<(), ProcessMessageError> {
		Deny::should_execute(origin, instructions, max_weight, properties)?;
		Allow::should_execute(origin, instructions, max_weight, properties)
	}
}

/// Reserved funds to the relay chain can't return. See https://github.com/paritytech/polkadot/issues/5233
/// Usage of the new xcm matcher. See https://github.com/paritytech/polkadot/pull/7098
pub struct DenyReserveTransferToRelayChain;
impl ShouldExecute for DenyReserveTransferToRelayChain {
	fn should_execute<RuntimeCall>(
		origin: &MultiLocation,
		message: &mut [Instruction<RuntimeCall>],
		_max_weight: Weight,
		_properties: &mut Properties,
	) -> Result<(), ProcessMessageError> {
		xcm_builder::MatchXcm::match_next_inst_while(
			xcm_builder::CreateMatcher::matcher(message),
			|_| true,
			|inst| match inst {
				InitiateReserveWithdraw {
					reserve: MultiLocation {
						parents: 1,
						interior: Here,
					},
					..
				}
				| DepositReserveAsset {
					dest: MultiLocation {
						parents: 1,
						interior: Here,
					},
					..
				}
				| TransferReserveAsset {
					dest: MultiLocation {
						parents: 1,
						interior: Here,
					},
					..
				} => {
					Err(ProcessMessageError::Unsupported) // Deny
				}

				// An unexpected reserve transfer has arrived from the Relay Chain. Generally,
				// `IsReserve` should not allow this, but we just log it here.
				ReserveAssetDeposited { .. }
					if matches!(
						origin,
						MultiLocation {
							parents: 1,
							interior: Here
						}
					) =>
				{
					log::warn!(
						target: "xcm::barrier",
						"Unexpected ReserveAssetDeposited from the Relay Chain",
					);
					Ok(ControlFlow::Continue(()))
				}

				_ => Ok(ControlFlow::Continue(())),
			},
		)?;

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
pub type LocationToAccountId<NetworkId> = (
	// The parent (Relay-chain) origin converts to the b"parent" `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<NetworkId, AccountId>,
);

/// Means for transacting assets on this chain.
pub type LocalAssetTransactor<Currency, NetworkId> = CurrencyAdapter<
	// Use this currency:
	Currency,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<HereLocation>,
	// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
	LocationToAccountId<NetworkId>,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports.
	(),
>;
