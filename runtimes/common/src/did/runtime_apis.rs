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

// If you feel like getting in touch with us, you can do so at <hello@kilt.org>

use pallet_did_lookup::linkable_account::LinkableAccountId;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

/// The kind of resources that can be linked to a DID, preventing its deletion.
#[derive(Debug, Encode, Decode, TypeInfo, PartialEq, Eq, PartialOrd, Ord)]
pub enum LinkedDidResource<Web3Name, DotName> {
	/// A Web3name.
	Web3Name(Web3Name),
	/// A Dotname.
	DotName(DotName),
	/// An account linked to the DID and resolvable by or to a Web3name.
	Web3NameAccount(LinkableAccountId),
	/// An account linked to the DID and resolvable by or to a Dotname.
	DotNameAccount(LinkableAccountId),
}
