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

#[frame_benchmarking::v2::instance_benchmarks(where LocalCurrencyBalanceOf<T, I>: Into<u128>)]
mod benchmarks {

	use frame_support::traits::{
		fungible::{Inspect as InspectFungible, Mutate as MutateFungible},
		EnsureOrigin,
	};
	use frame_system::RawOrigin;
	use sp_runtime::traits::TryConvert;
	use sp_runtime::traits::Zero;
	use xcm::{
		v3::{AssetId, Fungibility, Junction, Junctions, MultiAsset, MultiLocation, XcmContext},
		VersionedAssetId, VersionedMultiAsset, VersionedMultiLocation,
	};
	use xcm_executor::traits::TransactAsset;

	use crate::{Call, Config, LocalCurrencyBalanceOf, Pallet, SwitchPairStatus};

	const RESERVE_LOCATION: MultiLocation = MultiLocation {
		parents: 1,
		interior: Junctions::X1(Junction::Parachain(1_000)),
	};
	const REMOTE_ASSET_ID: AssetId = AssetId::Concrete(RESERVE_LOCATION);
	const REMOTE_FEE: MultiAsset = MultiAsset {
		id: REMOTE_ASSET_ID,
		fun: Fungibility::Fungible(1_000),
	};

	fn configure_switch_pair<T, I>()
	where
		T: Config<I>,
		I: 'static,
		LocalCurrencyBalanceOf<T, I>: Into<u128>,
	{
		let reserve_location = Box::new(VersionedMultiLocation::from(RESERVE_LOCATION));
		let remote_asset_id = Box::new(VersionedAssetId::from(REMOTE_ASSET_ID));
		let remote_fee = Box::new(VersionedMultiAsset::from(REMOTE_FEE));

		Pallet::<T, I>::force_set_switch_pair(
			T::RuntimeOrigin::from(RawOrigin::Root),
			reserve_location,
			remote_asset_id,
			remote_fee,
			u128::MAX,
			u128::zero(),
		)
		.unwrap();
		assert!(Pallet::<T, I>::switch_pair().is_some());
	}

	#[benchmark]
	fn set_switch_pair() {
		let origin = <T as Config<I>>::SwitchOrigin::try_successful_origin().unwrap();
		let reserve_location = Box::new(VersionedMultiLocation::from(RESERVE_LOCATION));
		let remote_asset_id = Box::new(VersionedAssetId::from(REMOTE_ASSET_ID));
		let remote_fee = Box::new(VersionedMultiAsset::from(REMOTE_FEE));

		#[extrinsic_call]
		Pallet::<T, I>::set_switch_pair(
			origin as T::RuntimeOrigin,
			reserve_location,
			remote_asset_id,
			remote_fee,
			u128::MAX,
			u128::zero(),
		);

		assert!(Pallet::<T, I>::switch_pair().is_some());
	}

	#[benchmark]
	fn force_set_switch_pair() {
		configure_switch_pair::<T, I>();

		let origin: T::RuntimeOrigin = RawOrigin::Root.into();
		let reserve_location = Box::new(VersionedMultiLocation::from(RESERVE_LOCATION));
		let remote_asset_id = Box::new(VersionedAssetId::from(REMOTE_ASSET_ID));
		let remote_fee = Box::new(VersionedMultiAsset::from(REMOTE_FEE));

		#[extrinsic_call]
		Pallet::<T, I>::force_set_switch_pair(
			origin as T::RuntimeOrigin,
			reserve_location,
			remote_asset_id,
			remote_fee,
			u128::MAX,
			u128::zero(),
		);

		assert!(Pallet::<T, I>::switch_pair().is_some());
	}

	#[benchmark]
	fn force_unset_switch_pair() {
		configure_switch_pair::<T, I>();

		let origin: T::RuntimeOrigin = RawOrigin::Root.into();

		#[extrinsic_call]
		Pallet::<T, I>::force_unset_switch_pair(origin as T::RuntimeOrigin);

		assert!(Pallet::<T, I>::switch_pair().is_none());
	}

	#[benchmark]
	fn pause_switch_pair() {
		configure_switch_pair::<T, I>();

		let origin = <T as Config<I>>::PauseOrigin::try_successful_origin().unwrap();

		#[extrinsic_call]
		Pallet::<T, I>::pause_switch_pair(origin as T::RuntimeOrigin);

		assert_eq!(Pallet::<T, I>::switch_pair().unwrap().status, SwitchPairStatus::Paused);
	}

	#[benchmark]
	fn resume_switch_pair() {
		configure_switch_pair::<T, I>();

		let origin = <T as Config<I>>::SwitchOrigin::try_successful_origin().unwrap();

		#[extrinsic_call]
		Pallet::<T, I>::resume_switch_pair(origin as T::RuntimeOrigin);

		assert_eq!(Pallet::<T, I>::switch_pair().unwrap().status, SwitchPairStatus::Running);
	}

	#[benchmark]
	fn update_remote_fee() {
		configure_switch_pair::<T, I>();

		let origin = <T as Config<I>>::FeeOrigin::try_successful_origin().unwrap();
		let remote_fee = Box::new(VersionedMultiAsset::from(REMOTE_FEE));

		#[extrinsic_call]
		Pallet::<T, I>::update_remote_fee(origin as T::RuntimeOrigin, remote_fee);
	}

	#[benchmark]
	fn switch() {
		configure_switch_pair::<T, I>();
		Pallet::<T, I>::resume_switch_pair(<T as Config<I>>::SwitchOrigin::try_successful_origin().unwrap()).unwrap();

		let origin = <T as Config<I>>::SubmitterOrigin::try_successful_origin().unwrap();
		let reserve_location = Box::new(VersionedMultiLocation::from(RESERVE_LOCATION));

		let account_id = <T as Config<I>>::SubmitterOrigin::ensure_origin(origin.clone()).unwrap();
		let local_account_id_junction = <T as Config<I>>::AccountIdConverter::try_convert(account_id.clone()).unwrap();
		let minimum_balance = <T as Config<I>>::LocalCurrency::minimum_balance();
		<T as Config<I>>::LocalCurrency::set_balance(&account_id, minimum_balance + 1_000u32.into());
		<T as Config<I>>::AssetTransactor::deposit_asset(
			&REMOTE_FEE,
			&(local_account_id_junction.into()),
			&XcmContext::with_message_id(Default::default()),
		)
		.unwrap();

		#[extrinsic_call]
		Pallet::<T, I>::switch(origin as T::RuntimeOrigin, 1_000u32.into(), reserve_location);

		assert_eq!(
			<T as Config<I>>::LocalCurrency::balance(
				&Pallet::<T, I>::pool_account_id_for_remote_asset(&VersionedAssetId::from(REMOTE_ASSET_ID)).unwrap(),
			),
			1_000u32.into()
		);
	}

	#[cfg(test)]
	mod benchmark_tests {
		use crate::Pallet;

		frame_benchmarking::impl_benchmark_test_suite!(
			Pallet,
			crate::mock::ExtBuilder::default().build_with_keystore(),
			crate::mock::MockRuntime
		);
	}
}
