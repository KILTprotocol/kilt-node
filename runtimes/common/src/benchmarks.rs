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

use frame_support::traits::EnsureOrigin;
// If you feel like getting in touch with us, you can do so at info@botlabs.org
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

use crate::AccountId;

#[derive(Clone, Copy, Default, Debug, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub struct DummySignature;

impl<A> From<(A, Vec<u8>)> for DummySignature {
	fn from(_: (A, Vec<u8>)) -> Self {
		DummySignature
	}
}

/// [`EnsureOrigin`] implementation that always succeeds.
pub struct BenchmarkOriginHelper;

impl EnsureOrigin<AccountId> for BenchmarkOriginHelper {
	type Success = AccountId;

	fn try_origin(o: AccountId) -> Result<Self::Success, AccountId> {
		Ok(o)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<Self::Success, ()> {
		use sp_core::{sr25519, Pair};
		let (pair, _) = sr25519::Pair::generate();
		Ok(pair.public().into())
	}
}
