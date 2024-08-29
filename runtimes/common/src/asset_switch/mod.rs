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

pub mod hooks;
pub mod runtime_api;

use frame_support::traits::EnsureOrigin;
use frame_system::EnsureRoot;
use sp_std::marker::PhantomData;

#[cfg(feature = "runtime-benchmarks")]
pub struct NoopBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_assets::BenchmarkHelper<xcm::v4::Location> for NoopBenchmarkHelper {
	fn create_asset_id_parameter(_id: u32) -> xcm::v4::Location {
		xcm::v4::Location {
			parents: 0,
			interior: xcm::v4::Junctions::Here,
		}
	}
}

/// Returns the `treasury` address if the origin is the root origin.
///
/// Required by `type CreateOrigin` in `pallet_assets`.
pub struct EnsureRootAsTreasury<Runtime>(PhantomData<Runtime>);

impl<Runtime> EnsureOrigin<Runtime::RuntimeOrigin> for EnsureRootAsTreasury<Runtime>
where
	Runtime: pallet_treasury::Config,
{
	type Success = Runtime::AccountId;

	fn try_origin(o: Runtime::RuntimeOrigin) -> Result<Self::Success, Runtime::RuntimeOrigin> {
		EnsureRoot::try_origin(o)?;

		// Return treasury account ID if successful.
		Ok(pallet_treasury::Pallet::<Runtime>::account_id())
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<Runtime::RuntimeOrigin, ()> {
		EnsureRoot::try_successful_origin()
	}
}
