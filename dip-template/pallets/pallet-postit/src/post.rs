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

use frame_support::{traits::ConstU32, BoundedVec};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub struct Post<Id, Text, Username> {
	pub author: Username,
	pub text: Text,
	pub likes: BoundedVec<Username, ConstU32<1_000>>,
	pub comments: BoundedVec<Id, ConstU32<1_000>>,
}

impl<Id, Text, Username> Post<Id, Text, Username> {
	pub(crate) fn from_text_and_author(text: Text, author: Username) -> Self {
		Self {
			text,
			author,
			likes: BoundedVec::default(),
			comments: BoundedVec::default(),
		}
	}
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub struct Comment<Id, Text, Username> {
	pub details: Post<Id, Text, Username>,
	pub in_response_to: Id,
}

impl<Id, Text, Username> Comment<Id, Text, Username> {
	pub(crate) fn from_post_id_text_and_author(in_response_to: Id, text: Text, author: Username) -> Self {
		Self {
			in_response_to,
			details: Post::from_text_and_author(text, author),
		}
	}
}
