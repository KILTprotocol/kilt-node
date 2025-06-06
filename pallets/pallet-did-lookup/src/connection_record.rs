// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

use kilt_support::Deposit;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

/// A record in the ConnectedDid map.
#[derive(Clone, Decode, Debug, Encode, TypeInfo, Eq, PartialEq, MaxEncodedLen, Serialize, Deserialize)]
pub struct ConnectionRecord<DidIdentifier, Account, Balance> {
	/// The did that is connected to the key under which the record was stored.
	pub did: DidIdentifier,

	/// The deposit that was reserved in order to incentivise fair blockchain
	/// use.
	pub deposit: Deposit<Account, Balance>,
}
