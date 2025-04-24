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

pub use frame_support::weights::constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight};
use frame_system::Pallet as SystemPallet;
use pallet_membership::{Config as MembershipConfig, Instance3, Pallet as MembershipPallet};
use pallet_session::SessionManager as SessionManagerTrait;
pub use sp_consensus_aura::sr25519::AuthorityId;
use sp_core::Get;
use sp_runtime::SaturatedConversion;
use sp_staking::SessionIndex;
use sp_std::{marker::PhantomData, vec::Vec};

use crate::constants::staking::DefaultBlocksPerRound;

type AccountIdOf<Runtime> = <Runtime as frame_system::Config>::AccountId;

/// The session manager for the collator set.
pub struct SessionManager<Runtime>(PhantomData<Runtime>);

impl<Runtime> SessionManagerTrait<AccountIdOf<Runtime>> for SessionManager<Runtime>
where
	Runtime: MembershipConfig<Instance3> + pallet_session::Config,
	<Runtime as pallet_session::Config>::ValidatorId: From<AccountIdOf<Runtime>>,
{
	fn new_session(new_index: SessionIndex) -> Option<Vec<AccountIdOf<Runtime>>> {
		let collators = MembershipPallet::<Runtime, Instance3>::members().to_vec();

		log::debug!(
			"assembling new collators for new session {} at #{:?} with {:?}",
			new_index,
			SystemPallet::<Runtime>::block_number(),
			collators
		);

		let has_collator_keys = collators.iter().any(|collator| {
			pallet_session::NextKeys::<Runtime>::contains_key(<Runtime as pallet_session::Config>::ValidatorId::from(
				collator.clone(),
			))
		});

		SystemPallet::<Runtime>::register_extra_weight_unchecked(
			<Runtime as frame_system::Config>::DbWeight::get()
				.reads(2u64.saturating_add(collators.len().saturated_into::<u64>())),
			frame_support::pallet_prelude::DispatchClass::Mandatory,
		);

		if collators.is_empty() || !has_collator_keys {
			// we never want to pass an empty set of collators. This would brick the chain.
			log::error!("💥 keeping old session because of empty collator set!");
			return None;
		}

		Some(collators)
	}

	fn start_session(_start_index: SessionIndex) {
		// We don't care
	}

	fn end_session(_end_index: SessionIndex) {
		// We don't care
	}
}

pub type FixedLengthSession = pallet_session::PeriodicSessions<DefaultBlocksPerRound, DefaultBlocksPerRound>;
