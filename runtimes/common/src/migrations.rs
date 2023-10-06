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

use frame_support::{
	traits::{GetStorageVersion, OnRuntimeUpgrade},
	weights::Weight,
};

use pallet_membership::Instance2;
use sp_core::Get;
use sp_std::marker::PhantomData;

/// There are some pallets without a storage version.
/// Based on the changes in the PR (https://github.com/paritytech/substrate/pull/13417),
/// pallets without a storage version or with a wrong version throw an error
/// in the try state tests.
pub struct BumpStorageVersion<T>(PhantomData<T>);

impl<T> OnRuntimeUpgrade for BumpStorageVersion<T>
where
	T: frame_system::Config,
	T: pallet_tips::Config,
	T: pallet_multisig::Config,
	T: cumulus_pallet_parachain_system::Config,
	T: pallet_membership::Config<Instance2>,
	T: cumulus_pallet_xcmp_queue::Config,
	T: cumulus_pallet_dmp_queue::Config,
{
	fn on_runtime_upgrade() -> Weight {
		log::info!("BumpStorageVersion: Initiating migration");

		pallet_tips::Pallet::<T>::current_storage_version().put::<pallet_tips::Pallet<T>>();
		cumulus_pallet_parachain_system::Pallet::<T>::current_storage_version()
			.put::<cumulus_pallet_parachain_system::Pallet<T>>();
		cumulus_pallet_xcmp_queue::Pallet::<T>::current_storage_version().put::<cumulus_pallet_xcmp_queue::Pallet<T>>();
		pallet_membership::Pallet::<T, Instance2>::current_storage_version()
			.put::<pallet_membership::Pallet<T, Instance2>>();
		cumulus_pallet_dmp_queue::Pallet::<T>::current_storage_version().put::<cumulus_pallet_dmp_queue::Pallet<T>>();
		pallet_multisig::Pallet::<T>::current_storage_version().put::<pallet_multisig::Pallet<T>>();

		<T as frame_system::Config>::DbWeight::get().writes(6)
	}
}
