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

#![allow(clippy::unused_unit)]

use codec::{Decode, Encode, FullCodec};
pub use cumulus_primitives_core::ParaId;
use frame_support::traits::Get;
use frame_system::Config;
use scale_info::TypeInfo;
use sp_std::{boxed::Box, marker::PhantomData, prelude::*};
use xcm::latest::prelude::*;

use crate::traits::RelayRuntimeCallBuilder;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum UtilityRuntimeCall<RelayChainRuntimeCall> {
	#[codec(index = 1)]
	AsDerivative(u16, RelayChainRuntimeCall),
	#[codec(index = 2)]
	BatchAll(Vec<RelayChainRuntimeCall>),
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum RegistrarRuntimeCall {
	#[codec(index = 4)]
	Swap(ParaId, ParaId),
}

/// The encoded index correspondes to Kusama's Runtime module configuration.
/// https://github.com/paritytech/polkadot/blob/444e96ae34bcec8362f0f947a07bd912b32ca48f/runtime/kusama/src/lib.rs#L1379
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum RelayChainRuntimeCall {
	#[codec(index = 24)]
	Utility(Box<UtilityRuntimeCall<Self>>),
	#[codec(index = 70)]
	Registrar(RegistrarRuntimeCall),
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct RelayChainRuntimeCallBuilder<T: Config, ParachainId: Get<ParaId>>(PhantomData<(T, ParachainId)>);

impl<T: Config, ParachainId: Get<ParaId>> RelayRuntimeCallBuilder for RelayChainRuntimeCallBuilder<T, ParachainId>
where
	T::AccountId: FullCodec,
	RelayChainRuntimeCall: FullCodec,
{
	type AccountId = T::AccountId;
	type Balance = polkadot_core_primitives::Balance;
	type RelayChainRuntimeCall = RelayChainRuntimeCall;

	fn utility_batch_call(calls: Vec<Self::RelayChainRuntimeCall>) -> Self::RelayChainRuntimeCall {
		RelayChainRuntimeCall::Utility(Box::new(UtilityRuntimeCall::BatchAll(calls)))
	}

	fn utility_as_derivative_call(call: Self::RelayChainRuntimeCall, index: u16) -> Self::RelayChainRuntimeCall {
		RelayChainRuntimeCall::Utility(Box::new(UtilityRuntimeCall::AsDerivative(index, call)))
	}

	fn swap_call(id: ParaId, other: ParaId) -> Self::RelayChainRuntimeCall {
		RelayChainRuntimeCall::Registrar(RegistrarRuntimeCall::Swap(id, other))
	}

	fn finalize_call_into_xcm_message(call: Vec<u8>, extra_fee: Self::Balance, weight: u64) -> Xcm<()> {
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
