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

// If you feel like getting in touch with us, you can do so at <hello@kilt.org>

use frame_support::traits::InstanceFilter;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use runtime_common::constants;
use sp_core::RuntimeDebug;
use sp_runtime::traits::BlakeTwo256;

use crate::{weights, Balances, Runtime, RuntimeCall, RuntimeEvent};

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	scale_info::TypeInfo,
	Default,
)]
pub enum ProxyType {
	/// Allow for any call.
	#[default]
	Any,
	/// Allow for calls that do not move tokens out of the caller's account.
	NonTransfer,
	/// Allow for governance-related calls.
	Governance,
	/// Allow for staking-related calls.
	ParachainStaking,
	/// Allow for calls that cancel proxy information.
	CancelProxy,
	/// Allow for calls that do not result in a deposit being claimed (e.g., for
	/// attestations, delegations, or DIDs).
	NonDepositClaiming,
}

impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => matches!(
				c,
				RuntimeCall::Attestation(..)
					// Excludes `Balances`
					| RuntimeCall::Council(..)
					| RuntimeCall::Ctype(..)
					| RuntimeCall::Delegation(..)
					| RuntimeCall::Democracy(..)
					| RuntimeCall::DepositStorage(..)
					| RuntimeCall::Did(..)
					| RuntimeCall::DidLookup(..)
					| RuntimeCall::DipProvider(..)
					| RuntimeCall::DotNames(..)
					| RuntimeCall::Indices(
						// Excludes `force_transfer`, and `transfer`
						pallet_indices::Call::claim { .. }
							| pallet_indices::Call::free { .. }
							| pallet_indices::Call::freeze { .. }
					)
					| RuntimeCall::Multisig(..)
					| RuntimeCall::ParachainStaking(..)
					// Excludes `ParachainSystem`
					| RuntimeCall::Preimage(..)
					| RuntimeCall::Proxy(..)
					| RuntimeCall::PublicCredentials(..)
					| RuntimeCall::Scheduler(..)
					| RuntimeCall::Session(..)
					| RuntimeCall::System(..)
					| RuntimeCall::TechnicalCommittee(..)
					| RuntimeCall::TechnicalMembership(..)
					| RuntimeCall::TipsMembership(..)
					| RuntimeCall::Timestamp(..)
					| RuntimeCall::Treasury(..)
					| RuntimeCall::UniqueLinking(..)
					| RuntimeCall::Utility(..)
					| RuntimeCall::Vesting(
						// Excludes `force_vested_transfer`, `merge_schedules`, and `vested_transfer`
						pallet_vesting::Call::vest { .. }
							| pallet_vesting::Call::vest_other { .. }
					)
					| RuntimeCall::Web3Names(..),
			),
			ProxyType::NonDepositClaiming => matches!(
				c,
				RuntimeCall::Attestation(
						// Excludes `reclaim_deposit`
						attestation::Call::add { .. }
							| attestation::Call::remove { .. }
							| attestation::Call::revoke { .. }
							| attestation::Call::change_deposit_owner { .. }
							| attestation::Call::update_deposit { .. }
					)
					// Excludes `Balances`
					| RuntimeCall::Council(..)
					| RuntimeCall::Ctype(..)
					| RuntimeCall::Delegation(
						// Excludes `reclaim_deposit`
						delegation::Call::add_delegation { .. }
							| delegation::Call::create_hierarchy { .. }
							| delegation::Call::remove_delegation { .. }
							| delegation::Call::revoke_delegation { .. }
							| delegation::Call::update_deposit { .. }
							| delegation::Call::change_deposit_owner { .. }
					)
					| RuntimeCall::Democracy(..)
					// Excludes `DepositStorage`
					| RuntimeCall::Did(
						// Excludes `reclaim_deposit`
						did::Call::add_key_agreement_key { .. }
							| did::Call::add_service_endpoint { .. }
							| did::Call::create { .. }
							| did::Call::delete { .. }
							| did::Call::remove_attestation_key { .. }
							| did::Call::remove_delegation_key { .. }
							| did::Call::remove_key_agreement_key { .. }
							| did::Call::remove_service_endpoint { .. }
							| did::Call::set_attestation_key { .. }
							| did::Call::set_authentication_key { .. }
							| did::Call::set_delegation_key { .. }
							| did::Call::submit_did_call { .. }
							| did::Call::update_deposit { .. }
							| did::Call::change_deposit_owner { .. }
							| did::Call::create_from_account { .. }
							| did::Call::dispatch_as { .. }
					)
					| RuntimeCall::DidLookup(
						// Excludes `reclaim_deposit`
						pallet_did_lookup::Call::associate_account { .. }
							| pallet_did_lookup::Call::associate_sender { .. }
							| pallet_did_lookup::Call::remove_account_association { .. }
							| pallet_did_lookup::Call::remove_sender_association { .. }
							| pallet_did_lookup::Call::update_deposit { .. }
							| pallet_did_lookup::Call::change_deposit_owner { .. }
					)
					| RuntimeCall::DipProvider(..)
					| RuntimeCall::DotNames(
						// Excludes `ban`, and `reclaim_deposit`
						pallet_web3_names::Call::claim { .. }
							| pallet_web3_names::Call::release_by_owner { .. }
							| pallet_web3_names::Call::unban { .. }
							| pallet_web3_names::Call::update_deposit { .. }
							| pallet_web3_names::Call::change_deposit_owner { .. }
					)
					| RuntimeCall::Indices(..)
					| RuntimeCall::Multisig(..)
					| RuntimeCall::ParachainStaking(..)
					// Excludes `ParachainSystem`
					| RuntimeCall::Preimage(..)
					| RuntimeCall::Proxy(..)
					| RuntimeCall::PublicCredentials(
						// Excludes `reclaim_deposit`
						public_credentials::Call::add { .. }
						| public_credentials::Call::revoke { .. }
						| public_credentials::Call::unrevoke { .. }
						| public_credentials::Call::remove { .. }
						| public_credentials::Call::update_deposit { .. }
						| public_credentials::Call::change_deposit_owner { .. }
					)
					| RuntimeCall::Scheduler(..)
					| RuntimeCall::Session(..)
					| RuntimeCall::System(..)
					| RuntimeCall::TechnicalCommittee(..)
					| RuntimeCall::TechnicalMembership(..)
					| RuntimeCall::TipsMembership(..)
					| RuntimeCall::Timestamp(..)
					| RuntimeCall::Treasury(..)
					| RuntimeCall::UniqueLinking(
						// Excludes `reclaim_deposit`
						pallet_did_lookup::Call::associate_account { .. }
							| pallet_did_lookup::Call::associate_sender { .. }
							| pallet_did_lookup::Call::remove_account_association { .. }
							| pallet_did_lookup::Call::remove_sender_association { .. }
							| pallet_did_lookup::Call::update_deposit { .. }
							| pallet_did_lookup::Call::change_deposit_owner { .. }
					)
					| RuntimeCall::Utility(..)
					| RuntimeCall::Vesting(..)
					| RuntimeCall::Web3Names(
						// Excludes `ban`, and `reclaim_deposit`
						pallet_web3_names::Call::claim { .. }
							| pallet_web3_names::Call::release_by_owner { .. }
							| pallet_web3_names::Call::unban { .. }
							| pallet_web3_names::Call::update_deposit { .. }
							| pallet_web3_names::Call::change_deposit_owner { .. }
					),
			),
			ProxyType::Governance => matches!(
				c,
				RuntimeCall::Council(..)
					| RuntimeCall::Democracy(..)
					| RuntimeCall::TechnicalCommittee(..)
					| RuntimeCall::TechnicalMembership(..)
					| RuntimeCall::TipsMembership(..)
					| RuntimeCall::Treasury(..)
					| RuntimeCall::Utility(..)
			),
			ProxyType::ParachainStaking => {
				matches!(
					c,
					RuntimeCall::ParachainStaking(..) | RuntimeCall::Session(..) | RuntimeCall::Utility(..)
				)
			}
			ProxyType::CancelProxy => matches!(c, RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. })),
		}
	}

	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			// "anything" always contains any subset
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			// reclaiming deposits is part of NonTransfer but not in NonDepositClaiming
			(ProxyType::NonDepositClaiming, ProxyType::NonTransfer) => false,
			// everything except NonTransfer and Any is part of NonDepositClaiming
			(ProxyType::NonDepositClaiming, _) => true,
			// Transfers are part of NonDepositClaiming but not in NonTransfer
			(ProxyType::NonTransfer, ProxyType::NonDepositClaiming) => false,
			// everything except NonDepositClaiming and Any is part of NonTransfer
			(ProxyType::NonTransfer, _) => true,
			_ => false,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = constants::proxy::ProxyDepositBase;
	type ProxyDepositFactor = constants::proxy::ProxyDepositFactor;
	type MaxProxies = constants::proxy::MaxProxies;
	type MaxPending = constants::proxy::MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = constants::proxy::AnnouncementDepositBase;
	type AnnouncementDepositFactor = constants::proxy::AnnouncementDepositFactor;
	type WeightInfo = weights::pallet_proxy::WeightInfo<Runtime>;
}
