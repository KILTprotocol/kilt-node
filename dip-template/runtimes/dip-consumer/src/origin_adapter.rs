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

use crate::{AccountId, DidIdentifier, MerkleProofVerifierOutput, RuntimeOrigin, Web3Name};
use frame_support::traits::EnsureOrigin;
use pallet_dip_consumer::{DipOrigin, EnsureDipOrigin};
use pallet_postit::traits::GetUsername;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;

/// An origin adapter which is used to make sure that a given [`DipOrigin`]
/// contains, among other things, a web3name. If a pallet extrinsic that
/// requires this origin is called with a DIP proof that does not revealed the
/// web3name linked to the subject, the extrinsic will fail with a `BadOrigin`
/// error.
pub struct EnsureDipOriginAdapter;

impl EnsureOrigin<RuntimeOrigin> for EnsureDipOriginAdapter {
	type Success = DipOriginAdapter;

	fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
		EnsureDipOrigin::try_origin(o).map(DipOriginAdapter)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
		EnsureDipOrigin::<DidIdentifier, AccountId, MerkleProofVerifierOutput>::try_successful_origin()
	}
}

/// A wrapper around a [`DipOrigin`] that makes sure the origin has a web3name,
/// or else the origin is invalid.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct DipOriginAdapter(DipOrigin<DidIdentifier, AccountId, MerkleProofVerifierOutput>);

impl GetUsername for DipOriginAdapter {
	type Username = Web3Name;

	fn username(&self) -> Result<Self::Username, &'static str> {
		self.0
			.details
			.web3_name
			.as_ref()
			.map(|leaf| leaf.web3_name.clone())
			.ok_or("No username for the subject.")
	}
}
