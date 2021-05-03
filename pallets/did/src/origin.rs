// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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
	codec::{Decode, Encode},
	traits::EnsureOrigin,
};
use sp_runtime::RuntimeDebug;
use sp_std::default::Default;
use sp_std::marker::PhantomData;

/// Origin for modules that support DID-based authorization.
#[derive(Clone, Decode, Encode, Eq, PartialEq, RuntimeDebug)]
pub struct RawOrigin<DidIdentifier>
{
	pub id: DidIdentifier,
}

pub struct EnsureDid<DidIdentifier>(PhantomData<DidIdentifier>);

impl<Origin, DidIdentifier> EnsureOrigin<Origin> for EnsureDid<DidIdentifier>
where
	Origin: Into<Result<RawOrigin<DidIdentifier>, Origin>> + From<RawOrigin<DidIdentifier>>,
	DidIdentifier: Default
{
	type Success = DidIdentifier;

	fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
		o.into().and_then(|o| Ok(o.id))
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> Origin {
		Origin::from(RawOrigin { id: Default::default() })
	}
}
