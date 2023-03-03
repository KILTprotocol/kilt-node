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

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::EnsureOrigin, RuntimeDebug};
use scale_info::TypeInfo;
use sp_std::marker::PhantomData;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct KiltDidOrigin<DidIdentifier, AccountId> {
	pub did_subject: DidIdentifier,
	pub account_address: AccountId,
}

pub struct EnsureKiltDidOrigin<DidIdentifier, AccountId>(PhantomData<(DidIdentifier, AccountId)>);

impl<OuterOrigin, DidIdentifier, AccountId> EnsureOrigin<OuterOrigin> for EnsureKiltDidOrigin<DidIdentifier, AccountId>
where
	OuterOrigin: Into<Result<KiltDidOrigin<DidIdentifier, AccountId>, OuterOrigin>>,
{
	type Success = KiltDidOrigin<DidIdentifier, AccountId>;

	fn try_origin(o: OuterOrigin) -> Result<Self::Success, OuterOrigin> {
		o.into()
	}
}
