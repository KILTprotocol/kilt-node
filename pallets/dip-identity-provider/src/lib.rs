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

// TODO: Pallet description

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::weights::Weight;
use xcm::{
	v2::{
		Instruction::{BuyExecution, RefundSurplus, Transact, WithdrawAsset},
		Junctions, MultiAsset, MultiLocation, OriginKind, SendXcm, WeightLimit, Xcm,
	},
	VersionedMultiLocation,
};

pub use crate::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0); // No need to write a migration to store it.

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type InfoRoot;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type XcmRouter: SendXcm;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Map from MultiLocation to a version number.
	#[pallet::storage]
	pub type ServicedSystems<T> = StorageMap<_, Blake2_128Concat, MultiLocation, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {
		BadVersion,
		InvalidOrigin,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

impl<T, DidIdentifier, AccountId, Root>
	did::traits::DidRootDispatcher<DidIdentifier, AccountId, Root, Box<VersionedMultiLocation>> for Pallet<T>
where
	T: Config,
	AccountId: TryInto<Junctions>,
{
	fn dispatch(
		action: did::traits::DidRootStateAction<DidIdentifier, Root>,
		dispatcher: AccountId,
		asset: MultiAsset,
		location: Box<VersionedMultiLocation>,
	) -> Result<frame_support::weights::Weight, frame_support::sp_runtime::DispatchError> {
		let interior: Junctions = dispatcher.try_into().map_err(|_| Error::<T>::InvalidOrigin)?;
		let destination = MultiLocation::try_from(*location).map_err(|()| Error::<T>::BadVersion)?;

		let withdraw_asset_instruction = WithdrawAsset(asset.clone().into());
		let buy_execution_instruction = BuyExecution {
			fees: asset,
			weight_limit: WeightLimit::Limited(1_000_000_000),
		};
		let transact_instruction = Transact {
			origin_type: OriginKind::SovereignAccount,
			require_weight_at_most: 1_000_000_000,
			call: vec![].into(),
		};
		let refund_surplus_instruction = RefundSurplus;
		let xcm = Xcm(vec![
			withdraw_asset_instruction,
			buy_execution_instruction,
			transact_instruction,
			refund_surplus_instruction,
		]);
		T::XcmRouter::send_xcm(destination, xcm).map_err(|_| Error::<T>::InvalidOrigin)?;
		Ok(Weight::from_ref_time(0))
	}
}
