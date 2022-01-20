// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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

use codec::{Decode, Encode};
use scale_info::TypeInfo;

/// Storage version of the DID pallet.
#[derive(Copy, Clone, Encode, Eq, Decode, Ord, PartialEq, PartialOrd, TypeInfo)]
pub enum DidStorageVersion {
	V1,
	V2,
	V3,
}

#[cfg(feature = "try-runtime")]
impl DidStorageVersion {
	/// The latest storage version.
	#[allow(dead_code)]
	fn latest() -> Self {
		Self::V3
	}
}

// All nodes will default to this, which is not bad, as in case the "real"
// version is a later one (i.e. the node has been started with already the
// latest version), the migration will simply do nothing as there's nothing in
// the old storage entries to migrate from.
//
// It might get updated in the future when we know that no node is running this
// old version anymore.
impl Default for DidStorageVersion {
	fn default() -> Self {
		Self::V2
	}
}
