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

use frame_support::RuntimeDebug;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// The identity entry for any given user that uses the DIP protocol.
#[derive(Encode, Decode, MaxEncodedLen, Default, TypeInfo, RuntimeDebug)]
pub struct IdentityDetails<Digest, Details> {
	/// The identity digest information, typically used to verify identity
	/// proofs.
	pub digest: Digest,
	/// The details related to the user, stored in the pallet storage.
	pub details: Details,
}

impl<Digest, Details> From<Digest> for IdentityDetails<Digest, Details>
where
	Details: Default,
{
	fn from(value: Digest) -> Self {
		Self {
			digest: value,
			details: Details::default(),
		}
	}
}
