// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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
use pallet_treasury::ArgumentsFactory;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

use crate::{constants::KILT, AccountId};

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

#[derive(Clone, Copy, Default, Debug, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub struct DummySignature;

impl<A> From<(A, Vec<u8>)> for DummySignature {
	fn from(_: (A, Vec<u8>)) -> Self {
		DummySignature
	}
}

pub mod treasury {
	use super::*;

	pub struct BenchmarkHelper<T>(PhantomData<T>);

	impl<T> ArgumentsFactory<(), AccountIdOf<T>> for BenchmarkHelper<T>
	where
		T: pallet_balances::Config + frame_system::Config,
		<T as pallet_balances::Config>::Balance: From<u128>,
		<T as frame_system::Config>::AccountId: From<sp_runtime::AccountId32>,
	{
		fn create_asset_kind(_seed: u32) {}

		fn create_beneficiary(seed: [u8; 32]) -> AccountIdOf<T> {
			let who: AccountIdOf<T> = AccountId::from(seed).into();

			// endow account with some funds
			let result =
				<pallet_balances::Pallet<T> as frame_support::traits::fungible::Mutate<AccountIdOf<T>>>::mint_into(
					&who,
					KILT.into(),
				);

			debug_assert!(
				result.is_ok(),
				"Could not create account for benchmarking treasury pallet"
			);

			who
		}
	}
}

pub mod xcm_benchmarking {
	use super::*;

	use cumulus_primitives_core::ParaId;
	use frame_support::parameter_types;
	use polkadot_runtime_common::xcm_sender::{NoPriceForMessageDelivery, ToParachainDeliveryHelper};
	use xcm::lts::prelude::*;

	parameter_types! {
		pub const RandomParaId: cumulus_primitives_core::ParaId = cumulus_primitives_core::ParaId::new(42424242);
		pub ExistentialDepositAsset: Option<Asset> = Some((
			Here,
			KILT
		).into());

		pub ParachainLocation: Location = ParentThen(Parachain(RandomParaId::get().into()).into()).into();
		pub NativeAsset: Asset = Asset {
						fun: Fungible(KILT),
						id: AssetId(Here.into())
					};
	}

	pub type ParachainDeliveryHelper<ParachainSystem, XcmConfig> = ToParachainDeliveryHelper<
		XcmConfig,
		ExistentialDepositAsset,
		NoPriceForMessageDelivery<ParaId>,
		RandomParaId,
		ParachainSystem,
	>;
}
