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

use kilt_support::deposit::Deposit;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// A record in the ConnectedDid map.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Decode, Debug, Encode, TypeInfo, Eq, PartialEq, MaxEncodedLen)]
pub struct ConnectionRecord<DidIdentifier, Account, Balance> {
	/// The did that is connected to the key under which the record was stored.
	pub did: DidIdentifier,

	/// The deposit that was reserved in order to incentivise fair blockchain
	/// use.
	pub deposit: Deposit<Account, Balance>,
}
