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

//! # RelayChain Support Module
//!
//! Provides means of of handling relaychain related utilities and
//! business logic such as finalizing XCM calls.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::{Decode, Encode, FullCodec};
use frame_support::{traits::Get, weights::Weight};
use frame_system::Config;
use scale_info::TypeInfo;
use sp_std::{boxed::Box, marker::PhantomData, prelude::*};
use xcm::latest::prelude::*;

pub use cumulus_primitives_core::ParaId;

use crate::traits::RelayCallBuilder;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum UtilityCall<RelayChainCall> {
	#[codec(index = 1)]
	AsDerivative(u16, RelayChainCall),
	#[codec(index = 2)]
	BatchAll(Vec<RelayChainCall>),
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum RegistrarCall {
	#[codec(index = 4)]
	Swap(ParaId, ParaId),
}

/// The encoded index correspondes to Kusama's Runtime module configuration.
/// https://github.com/paritytech/polkadot/blob/444e96ae34bcec8362f0f947a07bd912b32ca48f/runtime/kusama/src/lib.rs#L1379
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum RelayChainCall {
	#[codec(index = 24)]
	Utility(Box<UtilityCall<Self>>),
	#[codec(index = 70)]
	Registrar(RegistrarCall),
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct RelayChainCallBuilder<T: Config, ParachainId: Get<ParaId>>(PhantomData<(T, ParachainId)>);

impl<T: Config, ParachainId: Get<ParaId>> RelayCallBuilder for RelayChainCallBuilder<T, ParachainId>
where
	T::AccountId: FullCodec,
	RelayChainCall: FullCodec,
{
	type AccountId = T::AccountId;
	type Balance = polkadot_core_primitives::Balance;
	type RelayChainCall = RelayChainCall;

	fn utility_batch_call(calls: Vec<Self::RelayChainCall>) -> Self::RelayChainCall {
		RelayChainCall::Utility(Box::new(UtilityCall::BatchAll(calls)))
	}

	fn utility_as_derivative_call(call: Self::RelayChainCall, index: u16) -> Self::RelayChainCall {
		RelayChainCall::Utility(Box::new(UtilityCall::AsDerivative(index, call)))
	}

	fn swap_call(id: ParaId, other: ParaId) -> Self::RelayChainCall {
		RelayChainCall::Registrar(RegistrarCall::Swap(id, other))
	}

	fn finalize_call_into_xcm_message(call: Vec<u8>, extra_fee: Self::Balance, weight: Weight) -> Xcm<()> {
		let asset = MultiAsset {
			id: Concrete(MultiLocation::here()),
			fun: Fungibility::Fungible(extra_fee),
		};
		Xcm(vec![
			WithdrawAsset(asset.clone().into()),
			BuyExecution {
				fees: asset,
				weight_limit: Unlimited,
			},
			Transact {
				origin_type: OriginKind::Native,
				require_weight_at_most: weight,
				call: call.into(),
			},
			RefundSurplus,
			DepositAsset {
				assets: All.into(),
				max_assets: 1,
				beneficiary: MultiLocation {
					parents: 0,
					interior: X1(Parachain(ParachainId::get().into())),
				},
			},
		])
	}
}
