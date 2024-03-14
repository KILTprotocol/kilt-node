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

use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
use sp_core::Get;
use sp_std::marker::PhantomData;

/// There are some pallets without a storage version.
/// Based on the changes in the PR <https://github.com/paritytech/substrate/pull/13417>,
/// pallets without a storage version or with a wrong version throw an error
/// in the try state tests.
pub struct BumpStorageVersion<T>(PhantomData<T>);

impl<T> OnRuntimeUpgrade for BumpStorageVersion<T>
where
	T: frame_system::Config,
{
	fn on_runtime_upgrade() -> Weight {
		log::info!("BumpStorageVersion: Initiating migration");

		<T as frame_system::Config>::DbWeight::get().writes(0)
	}
}
